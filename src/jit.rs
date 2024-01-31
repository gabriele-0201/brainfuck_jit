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
    code_pointer: u64,
    // memory that will be handled by the code,
    // data label will point to half of this vector
    // at the start of the bf code
    memory: Box<Vec<u8>>,
}

impl JIT {
    pub fn new(instructions: Vec<Instruction>) -> anyhow::Result<Self> {
        let memory = Box::new(vec![0u8; 1024]);
        let mut code_assembler = CodeAssembler::new(64).unwrap();
        let data_label = code_assembler.create_label();

        let max_code_size = 4 * 1024;
        let mut jit = JIT {
            code_assembler,
            data_label,
            memory,
            code: vec![],
            code_pointer: unsafe {
                nix::sys::mman::mmap(
                    None,
                    (max_code_size).try_into().unwrap(),
                    nix::sys::mman::ProtFlags::PROT_READ
                        | nix::sys::mman::ProtFlags::PROT_WRITE
                        | nix::sys::mman::ProtFlags::PROT_EXEC,
                    nix::sys::mman::MapFlags::MAP_PRIVATE | nix::sys::mman::MapFlags::MAP_ANONYMOUS,
                    -1,
                    0,
                )
                .unwrap() as u64
            },
        };

        // the following is super bad, super wrong and makes the compilation
        // 2n instead of n, it is also ugly code wise.... but I just want to see it working
        let mut labels: Vec<CodeLabel> = instructions
            .iter()
            .filter_map(|i| {
                i.get_name()
                    .and_then(|_| Some(jit.code_assembler.create_label()))
            })
            .collect();

        for instruction in instructions {
            jit.append_code(instruction, &mut labels);
        }

        jit.code = jit
            .code_assembler
            .assemble(jit.code_pointer as u64)
            .unwrap();

        Ok(jit)
    }

    pub fn execute(&mut self) {
        unsafe {
            let code =
                slice::from_raw_parts_mut(&mut *(self.code_pointer as *mut u8), self.code.len());
            code.copy_from_slice(&self.code);

            let fun: extern "C" fn() = core::mem::transmute(self.code_pointer);
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

    pub fn print_code(&self) {
        use iced_x86::*;

        let mut decoder = Decoder::new(64, &self.code, DecoderOptions::NONE);

        let mut formatter = GasFormatter::new();

        formatter.options_mut().set_digit_separator(" ");
        formatter.options_mut().set_first_operand_char_index(10);

        // String implements FormatterOutput
        let mut output = String::new();

        // Initialize this outside the loop because decode_out() writes to every field
        let mut instruction = Instruction::default();

        while decoder.can_decode() {
            decoder.decode_out(&mut instruction);

            // Format the instruction ("disassemble" it)
            output.clear();
            formatter.format(&instruction, &mut output);
        }
    }
}
