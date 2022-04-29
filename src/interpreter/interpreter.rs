use super::program::{Instruction, Program};
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use crate::interpreter::interpreter::InterpreterStatus::Running;

pub struct Interpreter {
    program: Program,
    env: Environment,
    status: InterpreterStatus,
}

struct Environment {
    memory: Vec<u8>,
    mp: usize,
    pc: usize,
    input: Box<dyn Read>,
    output: Box<dyn Write>,
}

#[derive(PartialEq, Eq)]
pub enum InterpreterStatus {
    Idle,
    Running,
    Waiting(WaitReason),
}

#[derive(PartialEq, Eq)]
pub enum WaitReason {
    Input,
    Output,
}

/* Interpreter *******************************************************************************************************/
impl Interpreter {
    pub fn new<R: Read>(source: R) -> Result<Interpreter, Box<dyn Error>> {
        Ok(Interpreter {
            program: Program::compile(source)?,
            env: Environment::new(4096),
            status: InterpreterStatus::Idle,
        })
    }

    pub fn interpret_file(fname: &str) -> Result<Interpreter, Box<dyn Error>> {
        Interpreter::new(File::open(fname)?)
    }

    pub fn with_memory(mut self, mem_size: usize) -> Interpreter {
        self.env.memory = vec![0; mem_size];
        self
    }

    pub fn with_input_source(mut self, source: Box<dyn Read>) -> Interpreter {
        self.env.input = source;
        self
    }

    pub fn with_output_sink(mut self, sink: Box<dyn Write>) -> Interpreter {
        self.env.output = sink;
        self
    }

    pub fn dump_program<W: Write>(&self, sink: &mut W) -> Result<(), std::io::Error>{
        self.program.dump(sink)
    }

    pub fn startup(&mut self) {
        self.status = InterpreterStatus::Running;
    }

    pub fn step(&mut self) -> Result<(), Box<dyn Error>>{
        // Check if instruction should be running
        if self.status != InterpreterStatus::Running {
            return Err("Interpreter is not running".into());
        }
        let instruction = self.program.instruction(self.env.pc);
        let mut next_pc = self.env.pc + 1;
        // Execute instruction
        match *instruction {
            Instruction::IncPtr => self.env.mp += 1,
            Instruction::DecPtr => self.env.mp -= 1,
            Instruction::IncData => self.env.mem_inc(),
            Instruction::DecData => self.env.mem_dec(),
            Instruction::Output => {
                self.status = InterpreterStatus::Waiting(WaitReason::Output);
            }
            Instruction::Input => {
                self.status = InterpreterStatus::Waiting(WaitReason::Input);
            }
            Instruction::JNZ(addr) => {
                if self.env.mem_rd() != 0 {
                    next_pc = addr;
                }
            }
            Instruction::JZ(addr) => {
                if self.env.mem_rd() == 0 {
                    next_pc = addr;
                }
            }
            Instruction::Exit => self.status = InterpreterStatus::Idle,
        }
        // Update program counter
        self.env.pc = next_pc;
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.startup();
        loop {
            match self.status() {
                InterpreterStatus::Running => self.step()?,
                InterpreterStatus::Idle => break,
                InterpreterStatus::Waiting(reason) => match reason {
                    WaitReason::Input => {
                        let input = self.env.read_byte(true)?;
                        self.env.mem_wr(input);
                        self.status = Running;
                    },
                    WaitReason::Output => {
                        self.env.write_byte(self.env.mem_rd())?;
                        self.status = Running;
                    }
                },
            }
        }
        Ok(())
    }

    pub fn status(&self) -> &InterpreterStatus {
        &self.status
    }
}

/* Environment ********************************************************************************************************/
impl Environment {
    pub fn new(mem_size: usize) -> Environment {
        Environment {
            memory: vec![0; mem_size],
            mp: 0,
            pc: 0,
            input: Box::new(std::io::stdin()),
            output: Box::new(std::io::stdout()),
        }
    }

    pub fn mem_rd(&self) -> u8 {
        self.memory[self.mp]
    }

    pub fn mem_wr(&mut self, val: u8) {
        self.memory[self.mp] = val
    }

    pub fn mem_inc(&mut self) {
        self.memory[self.mp] += 1;
    }

    pub fn mem_dec(&mut self) {
        self.memory[self.mp] -= 1;
    }

    pub fn read_byte(&mut self, ignore_newlines: bool) -> Result<u8, std::io::Error> {
        let mut buffer = [0u8];
        self.input.read(&mut buffer)?;
        while ignore_newlines && buffer[0] == '\n' as u8 {
            self.input.read(&mut buffer)?;
        }
        Ok(buffer[0])
    }

    pub fn write_byte(&mut self, data: u8) -> Result<(), std::io::Error> {
        write!(self.output, "{}", data as char)
    }
}


#[cfg(test)]
mod test {
    use super::*;

    /// Execute helloworld.bf as an overall sanity check
    #[test]
    fn run_hello_world() {
        let sink = Vec::new();
        Interpreter::interpret_file("test/helloworld.bf")
            .expect("Could not open file")
            .with_memory(128)
            .with_output_sink(Box::new(sink))
            .run()
            .expect("Error while running");
    }
}