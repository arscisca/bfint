use std::error::Error;
use std::io::Read;

use super::token::{Tokenizer, Token, TokenKind};

pub struct Program {
    instructions: Vec<Instruction>,
}

impl Program {
    pub fn compile<R: Read>(source: R) -> Result<Program, Box<dyn Error>> {
        let mut instructions = Vec::new();
        let mut open_bracket_stack = Vec::new();
        for (i, token) in Tokenizer::read(source).enumerate() {
            let token = token?;
            let instruction = match token.kind() {
                TokenKind::RightBrace => Instruction::IncPtr,
                TokenKind::LeftBrace => Instruction::DecPtr,
                TokenKind::Plus => Instruction::IncData,
                TokenKind::Minus => Instruction::DecData,
                TokenKind::Dot => Instruction::Output,
                TokenKind::Comma => Instruction::Input,
                TokenKind::LeftBracket => {
                    open_bracket_stack.push(i);
                    Instruction::JZ(0)
                },
                TokenKind::RightBracket => {
                    if let Some(open_bracket_pos) = open_bracket_stack.pop() {
                        instructions[open_bracket_pos] = Instruction::JZ(i);
                        Instruction::JNZ(open_bracket_pos)
                    } else {
                        return Err("No matching '['".into());
                    }
                },
            };
            instructions.push(instruction);
        }
        if !open_bracket_stack.is_empty() {
            return Err("Unmatched '['".into());
        }
        Ok(Program { instructions })
    }

    pub fn instruction(&self, addr: usize) -> &Instruction {
        &self.instructions[addr]
    }

    pub fn len(&self) -> usize {
        self.instructions.len()
    }
}


pub enum Instruction {
    IncPtr,
    DecPtr,
    IncData,
    DecData,
    Input,
    Output,
    JZ(usize),
    JNZ(usize),
}