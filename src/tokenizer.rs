use anyhow::anyhow;
use core::fmt;
use std::{cell::RefCell, rc::Rc};

/// Brainfuck is composed by only 8 instructions and they are:
/// > :: Increment the data pointer by one (to point to the next cell to the right)
/// < :: Decrement the data pointer by one (to point to the next cell to the left)
/// + :: Increment the byte at the data pointer by one - Ib
/// - :: Decrement the byte at the data pointer by one - Db
/// . :: Output the byte at the data pointer -
/// , :: Accept one byte of input, storing its value in the byte at the data pointer
/// [ :: If the byte at the data pointer is zero, jump it forward to the command after the matching ] command
/// ] :: If the byte at the data pointer is nonzero, jump it back to the command after the matching [ command

/// enum Instructuion Jump
#[derive(fmt::Debug)]
pub enum Token {
    Op(Op),
    Branch(Branch),
}

#[derive(fmt::Debug, PartialEq)]
pub enum Op {
    IncrementDataPointer,
    DecrementDataPointer,
    Increment,
    Decrement,
    Output,
    Input,
}

#[derive(fmt::Debug, PartialEq)]
pub enum Branch {
    JumpZero,
    JumpNotZero,
}

impl Token {
    fn new(c: char) -> anyhow::Result<Self> {
        Ok(match c {
            '>' => Token::Op(Op::IncrementDataPointer),
            '<' => Token::Op(Op::DecrementDataPointer),
            '+' => Token::Op(Op::Increment),
            '-' => Token::Op(Op::Decrement),
            '.' => Token::Op(Op::Output),
            ',' => Token::Op(Op::Input),
            '[' => Token::Branch(Branch::JumpZero),
            ']' => Token::Branch(Branch::JumpNotZero),
            n => return Err(anyhow!("Not valid instruction {}", n)),
        })
    }
}

/// Create a vector of tokens, whitespaces and new lines will be skipped,
/// all others not valid characters will not be accepted
pub fn tokenize(code: &str) -> anyhow::Result<Vec<Token>> {
    // TODO: handle number of line and number of character to return an usefull error message
    // if an invalid character is found
    code.chars()
        .filter(|c| !matches!(c, ' ' | '\n'))
        .map(Token::new)
        .collect()
}
