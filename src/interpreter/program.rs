use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::{Read, Write};

use super::token::{TokenKind, Tokenizer};

pub struct Program {
    instructions: Vec<Instruction>,
}

#[derive(Debug)]
pub enum Instruction {
    IncPtr,
    DecPtr,
    IncData,
    DecData,
    Input,
    Output,
    JZ(usize),
    JNZ(usize),
    Exit,
}

/* Program ************************************************************************************************************/
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
                }
                TokenKind::RightBracket => {
                    if let Some(open_bracket_pos) = open_bracket_stack.pop() {
                        instructions[open_bracket_pos] = Instruction::JZ(i + 1);
                        Instruction::JNZ(open_bracket_pos)
                    } else {
                        return Err("No matching '['".into());
                    }
                }
            };
            instructions.push(instruction);
        }
        if !open_bracket_stack.is_empty() {
            return Err("Unmatched '['".into());
        }
        // Always push exit instruction at the end
        instructions.push(Instruction::Exit);
        Ok(Program { instructions })
    }

    pub fn instruction(&self, addr: usize) -> &Instruction {
        &self.instructions[addr]
    }

    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    pub fn dump<W: Write>(&self, sink: &mut W) -> Result<(), std::io::Error> {
        for (i, instruction) in self.instructions.iter().enumerate() {
            writeln!(sink, "0x{:08x}: {}", i, instruction)?;
        }
        Ok(())
    }
}

/* Instruction ********************************************************************************************************/
impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match *self {
                Instruction::IncPtr => String::from("incp"),
                Instruction::DecPtr => String::from("decp"),
                Instruction::IncData => String::from("incd"),
                Instruction::DecData => String::from("decd"),
                Instruction::Input => String::from("rd"),
                Instruction::Output => String::from("wr"),
                Instruction::JZ(addr) => format!("jz 0x{:08x}", addr),
                Instruction::JNZ(addr) => format!("jnz 0x{:08x}", addr),
                Instruction::Exit => String::from("exit"),
            }
        )
    }
}
