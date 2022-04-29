use std::error::Error;
use std::fs::File;
use std::io::Read;
use crate::interpreter::interpreter::InterpreterStatus::{Done, Running};
use super::program::{ Program, Instruction };

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum InterpreterStatus {
    Running,
    Done
}


pub struct Interpreter {
    program: Program,
    env: Environment,
    status: InterpreterStatus,
}


struct Environment {
    memory: Vec<u8>,
    mp: usize,
    pc: usize,
}


impl Interpreter {
    pub fn new<R: Read>(source: R) -> Result<Interpreter, Box<dyn Error>> {
        Ok(Interpreter{
            program: Program::compile(source)?,
            env: Environment::new(4096),
            status: InterpreterStatus::Running,
        })
    }

    pub fn interpret_file(fname: &str) -> Result<Interpreter, Box<dyn Error>> {
        Interpreter::new(File::open(fname)?)
    }

    pub fn step(&mut self) {
        let instruction = self.program.instruction(self.env.pc);
        self.env.pc += 1;
        match *instruction {
            Instruction::IncPtr => self.env.mp += 1,
            Instruction::DecPtr => self.env.mp -= 1,
            Instruction::IncData => self.env.memory[self.env.mp] += 1,
            Instruction::DecData => self.env.memory[self.env.mp] -= 1,
            Instruction::Output => print!("{}", self.env.memory[self.env.mp] as char),
            Instruction::Input => todo!(),
            Instruction::JNZ(addr) => {
                if self.env.memory[self.env.mp] != 0 {
                    self.env.pc = addr;
                }
            }
            Instruction::JZ(addr) => {
                if self.env.memory[self.env.mp] == 0 {
                    self.env.pc = addr;
                }
            }
        }
        if self.env.pc < self.program.len() {
            self.status = Running;
        } else {
            self.status = Done;
        }
    }

    pub fn run(&mut self) {
        while self.status != InterpreterStatus::Done {
            self.step();
        }
    }

    pub fn status(&self) -> InterpreterStatus {
        self.status
    }
}

impl Environment {
    pub fn new(mem_size: usize) -> Environment {
        Environment {
            memory: vec![0; mem_size],
            mp: 0,
            pc: 0,
        }
    }
}