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

#[derive(fmt::Debug)]
enum Op {
    IncrementDataPointer,
    DecrementDataPointer,
    Increment,
    Decrement,
    Output,
    Input,
}

#[derive(fmt::Debug)]
enum Branch {
    // TODO: probably those two will need to carry the next information
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

pub enum Node {
    Op {
        instruction: Op,
        next: Option<Rc<RefCell<Node>>>,
    },
    Branch {
        instruction: Branch,
        next: Option<Rc<RefCell<Node>>>,
        jump: Option<Rc<RefCell<Node>>>,
    },
}

impl Node {
    fn partial_node_from_token(token: Token) -> Node {
        match token {
            Token::Op(op) => Node::Op {
                instruction: op,
                next: Default::default(),
            },
            Token::Branch(branch) => Node::Branch {
                instruction: branch,
                next: Default::default(),
                jump: Default::default(),
            },
        }
    }

    fn set_next(&mut self, new_next: Option<Rc<RefCell<Node>>>) {
        let next = match self {
            Node::Op { next, .. } => next,
            Node::Branch { next, .. } => next,
        };
        *next = new_next;
    }
}

pub fn get_ast(tokens: Vec<Token>) -> anyhow::Result<Rc<RefCell<Node>>> {
    let mut waiting: Vec<Rc<RefCell<Node>>> = Vec::new();
    // going backward through the tokens, the prev_node is the next of the current
    let mut prev_node: Option<Rc<RefCell<Node>>> = Default::default();

    for token in tokens.into_iter().rev() {
        let current_node = Rc::new(RefCell::new(Node::partial_node_from_token(dbg!(token))));
        current_node.borrow_mut().set_next(prev_node);

        prev_node = Some(Rc::clone(&current_node));

        match &mut *current_node.borrow_mut() {
            // add JumpNotZero instruction to the list of waiting ]
            Node::Branch {
                instruction: Branch::JumpNotZero,
                ..
            } => {
                println!("adding closing");
                waiting.push(Rc::clone(&current_node))
            }
            // connect two []
            Node::Branch {
                instruction: Branch::JumpZero,
                jump: ref mut opening_jump,
                next: ref mut opening_next,
            } => {
                println!("openign");
                let closing = waiting
                    .pop()
                    .ok_or(anyhow!("closing ] withouth opening ["))?;

                let Node::Branch {
                    instruction: Branch::JumpNotZero,
                    jump: closing_jump,
                    next: closing_next,
                } = &mut *closing.borrow_mut()
                else {
                    return Err(anyhow!("lol"));
                };

                // the current opening [ should jump to the closing next instruction
                // and vice versa
                *opening_jump = closing_next.clone();
                *closing_jump = opening_next.clone();
            }
            _ => (),
        };
    }

    // assert on waiting empty
    if waiting.len() != 0 {
        return Err(anyhow!("not matching []"));
    }

    prev_node.ok_or(anyhow!("lol pt2"))
}
