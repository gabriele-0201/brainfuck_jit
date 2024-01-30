use crate::tokenizer::{Branch, Op, Token};
use anyhow::anyhow;
use core::fmt;

#[derive(fmt::Debug, PartialEq)]
pub enum Instruction {
    Op {
        name: Option<u32>,
        instruction: Op,
    },
    Branch {
        name: Option<u32>,
        instruction: Branch,
        // zero will be used as intermediate value,
        // until the target is not found
        name_jump_to: u32,
    },
    // Special instruction just to represent the last instruction
    End {
        name: Option<u32>,
    },
}

impl Instruction {
    fn from_token(token: Token) -> Instruction {
        match token {
            Token::Op(op) => Instruction::Op {
                name: None,
                instruction: op,
            },
            Token::Branch(branch) => Instruction::Branch {
                name: None,
                instruction: branch,
                name_jump_to: 0,
            },
        }
    }

    fn set_name(&mut self, new_name: u32) {
        let name = match self {
            Instruction::Op { name, .. } => name,
            Instruction::Branch { name, .. } => name,
            Instruction::End { name } => name,
        };
        *name = Some(new_name);
    }

    pub fn get_name(&self) -> Option<u32> {
        match self {
            Instruction::Op { name, .. } => name.clone(),
            Instruction::Branch { name, .. } => name.clone(),
            Instruction::End { name } => name.clone(),
        }
    }

    fn set_jump_to_name(&mut self, name: u32) -> anyhow::Result<()> {
        match self {
            Instruction::Branch { name_jump_to, .. } => {
                *name_jump_to = name;
                Ok(())
            }
            _ => Err(anyhow!("impossible set jump name to instruction")),
        }
    }
}

pub fn parse(tokens: Vec<Token>) -> anyhow::Result<Vec<Instruction>> {
    // waiting opening [
    let mut waiting: Vec<usize> = Vec::new();
    let mut couples: Vec<(usize, usize)> = Vec::new();

    let mut instructions: Vec<Instruction> = Vec::new();

    for (index, token) in tokens.into_iter().enumerate() {
        let ins = Instruction::from_token(token);

        match ins {
            Instruction::Branch {
                instruction: Branch::JumpZero,
                ..
            } => waiting.push(index),
            Instruction::Branch {
                instruction: Branch::JumpNotZero,
                ..
            } => couples.push((
                waiting.pop().ok_or(anyhow!("] does not match any ["))?,
                index,
            )),
            _ => (),
        };

        instructions.push(ins);
    }

    instructions.push(Instruction::End { name: None });

    // assert on waiting empty
    if waiting.len() != 0 {
        return Err(anyhow!("not matching []"));
    }

    let mut label = 0;
    for (open, close) in couples.into_iter() {
        instructions[open + 1].set_name(label);
        instructions[close].set_jump_to_name(label)?;

        label += 1;

        instructions[close + 1].set_name(label);
        instructions[open].set_jump_to_name(label)?;

        label += 1;
    }

    Ok(instructions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::tokenize;

    #[test]
    fn test_ok_parser() {
        let code = "[ + - ]";
        let tokens = tokenize(&code).unwrap();

        let instructions = parse(tokens).unwrap();

        let expected = vec![
            Instruction::Branch {
                name: None,
                instruction: Branch::JumpZero,
                name_jump_to: 1,
            },
            Instruction::Op {
                name: Some(0),
                instruction: Op::Increment,
            },
            Instruction::Op {
                name: None,
                instruction: Op::Decrement,
            },
            Instruction::Branch {
                name: None,
                instruction: Branch::JumpNotZero,
                name_jump_to: 0,
            },
            Instruction::End { name: Some(1) },
        ];

        assert_eq!(instructions, expected);

        let code = "[ + [ - ] [ , ] > ]";
        let tokens = tokenize(&code).unwrap();

        let instructions = parse(tokens).unwrap();

        let expected = vec![
            Instruction::Branch {
                name: None,
                instruction: Branch::JumpZero,
                name_jump_to: 5,
            },
            Instruction::Op {
                name: Some(4),
                instruction: Op::Increment,
            },
            Instruction::Branch {
                name: None,
                instruction: Branch::JumpZero,
                name_jump_to: 1,
            },
            Instruction::Op {
                name: Some(0),
                instruction: Op::Decrement,
            },
            Instruction::Branch {
                name: None,
                instruction: Branch::JumpNotZero,
                name_jump_to: 0,
            },
            Instruction::Branch {
                name: Some(1),
                instruction: Branch::JumpZero,
                name_jump_to: 3,
            },
            Instruction::Op {
                name: Some(2),
                instruction: Op::Input,
            },
            Instruction::Branch {
                name: None,
                instruction: Branch::JumpNotZero,
                name_jump_to: 2,
            },
            Instruction::Op {
                name: Some(3),
                instruction: Op::IncrementDataPointer,
            },
            Instruction::Branch {
                name: None,
                instruction: Branch::JumpNotZero,
                name_jump_to: 4,
            },
            Instruction::End { name: Some(5) },
        ];

        assert_eq!(instructions, expected);
    }
}
