//! Compile the brain fuck AST into

use crate::{
    parser::Instruction,
    tokenizer::{Branch, Op},
};
use core::slice;
use iced_x86::code_asm::*;

pub struct JIT {
    code_assembler: CodeAssembler,
    data_label: CodeLabel,
    code: Vec<u8>,
    // memory that will be handled by the code,
    // data pointer will point to half of this vector
    memory: Box<Vec<u8>>,
    //data_pointer: u64,
}

impl JIT {
    pub fn new() -> anyhow::Result<Self> {
        let memory = Box::new(vec![0u8; 1024]);
        let mut code_assembler = CodeAssembler::new(64).unwrap();
        let data_label = code_assembler.create_label();

        let jit = JIT {
            code_assembler,
            data_label,
            memory,
            code: vec![],
        };

        Ok(jit)
    }

    pub fn execute(&mut self, instructions: Vec<Instruction>) {
        // the following is super bad, super wrong and makes the compilation
        // 2n instead of n, it is also ugly code wise.... but I just want to see it working

        let mut labels: Vec<CodeLabel> = instructions
            .iter()
            .filter_map(|i| {
                i.get_name()
                    .and_then(|_| Some(self.code_assembler.create_label()))
            })
            .collect();

        for instruction in instructions {
            self.append_code(instruction, &mut labels);
        }

        unsafe {
            let max_code_size = 4 * 1024;
            let code_pointer = nix::sys::mman::mmap(
                None,
                (max_code_size).try_into().unwrap(),
                nix::sys::mman::ProtFlags::PROT_READ
                    | nix::sys::mman::ProtFlags::PROT_WRITE
                    | nix::sys::mman::ProtFlags::PROT_EXEC,
                nix::sys::mman::MapFlags::MAP_PRIVATE | nix::sys::mman::MapFlags::MAP_ANONYMOUS,
                -1,
                0,
            )
            .unwrap();

            let bytes = self.code_assembler.assemble(code_pointer as u64).unwrap();
            let code_size = bytes.len();

            Self::print_code(&bytes);

            let code = slice::from_raw_parts_mut(&mut *(code_pointer as *mut u8), code_size);
            code.copy_from_slice(&bytes);

            let fun: extern "C" fn() = core::mem::transmute(code_pointer);
            fun();
        }
    }

    fn op_code(&mut self, op: Op) {
        let c = &mut self.code_assembler;
        match op {
            Op::IncrementDataPointer => {
                c.mov(rax, qword_ptr(self.data_label)).unwrap();
                c.inc(rax).unwrap();
                c.mov(qword_ptr(self.data_label), rax).unwrap();
            }
            Op::DecrementDataPointer => {
                c.mov(rax, qword_ptr(self.data_label)).unwrap();
                c.dec(rax).unwrap();
                c.mov(qword_ptr(self.data_label), rax).unwrap();
            }
            Op::Increment => {
                c.mov(rdi, qword_ptr(self.data_label)).unwrap();
                c.mov(bh, byte_ptr(rdi)).unwrap();
                c.inc(bh).unwrap();
                c.mov(byte_ptr(rdi), bh).unwrap();
            }
            Op::Decrement => {
                c.mov(rdi, qword_ptr(self.data_label)).unwrap();
                c.mov(bh, byte_ptr(rdi)).unwrap();
                c.dec(bh).unwrap();
                c.mov(byte_ptr(rdi), bh).unwrap();
            }
            Op::Output => {
                c.mov(rax, 1u64).unwrap();
                c.mov(rdi, 1u64).unwrap();
                c.mov(rsi, qword_ptr(self.data_label)).unwrap();
                c.mov(rdx, 1u64).unwrap();
                c.syscall().unwrap();
            }
            Op::Input => {
                c.mov(rax, 0u64).unwrap();
                c.mov(rdi, 1u64).unwrap();
                c.mov(rsi, qword_ptr(self.data_label)).unwrap();
                c.mov(rdx, 1u64).unwrap();
                c.syscall().unwrap();
            }
        }
    }

    fn branch_code(&mut self, branch: Branch, name_to_jump: u32, labels: &mut Vec<CodeLabel>) {
        let c = &mut self.code_assembler;
        c.mov(rdi, qword_ptr(self.data_label)).unwrap();
        c.mov(bh, byte_ptr(rdi)).unwrap();
        // iced why you implementd cmp of 8 bit registers against u32 ?
        c.cmp(bh, 0u32).unwrap();
        match branch {
            Branch::JumpZero => c.je(labels[name_to_jump as usize]).unwrap(),
            Branch::JumpNotZero => c.jne(labels[name_to_jump as usize]).unwrap(),
        };
    }

    fn append_code(&mut self, instruction: Instruction, labels: &mut Vec<CodeLabel>) {
        // append name if specified
        if let Some(label_id) = instruction.get_name() {
            self.code_assembler
                .set_label(&mut labels[label_id as usize])
                .unwrap();
            println!("added label {}, at instruction {:?}", label_id, instruction);
        }

        match instruction {
            Instruction::Op { instruction, .. } => self.op_code(instruction),
            Instruction::Branch {
                instruction,
                name_jump_to,
                ..
            } => self.branch_code(instruction, name_jump_to, labels),
            Instruction::End { name: _name } => {
                let c = &mut self.code_assembler;
                c.ret().unwrap();
                c.set_label(&mut self.data_label).unwrap();
                c.dq(&[&self.memory[0] as *const u8 as u64]).unwrap();
                //c.dq(&[self.memory[self.memory.len() / 2] as *const u8 as u64]).unwrap();
            }
        }
    }

    fn print_code(bytes: &Vec<u8>) {
        use iced_x86::*;

        let mut decoder = Decoder::new(64, bytes, DecoderOptions::NONE);

        // Formatters: Masm*, Nasm*, Gas* (AT&T) and Intel* (XED).
        // For fastest code, see `SpecializedFormatter` which is ~3.3x faster. Use it if formatting
        // speed is more important than being able to re-assemble formatted instructions.
        let mut formatter = GasFormatter::new();

        // Change some options, there are many more
        formatter.options_mut().set_digit_separator(" ");
        formatter.options_mut().set_first_operand_char_index(10);

        // String implements FormatterOutput
        let mut output = String::new();

        // Initialize this outside the loop because decode_out() writes to every field
        let mut instruction = Instruction::default();

        // The decoder also implements Iterator/IntoIterator so you could use a for loop:
        //      for instruction in &mut decoder { /* ... */ }
        // or collect():
        //      let instructions: Vec<_> = decoder.into_iter().collect();
        // but can_decode()/decode_out() is a little faster:
        while decoder.can_decode() {
            // There's also a decode() method that returns an instruction but that also
            // means it copies an instruction (40 bytes):
            //     instruction = decoder.decode();
            decoder.decode_out(&mut instruction);

            // Format the instruction ("disassemble" it)
            output.clear();
            formatter.format(&instruction, &mut output);

            // Eg. "00007FFAC46ACDB2 488DAC2400FFFFFF     lea       rbp,[rsp-100h]"
            println!(" {}", output);
        }
    }
}
