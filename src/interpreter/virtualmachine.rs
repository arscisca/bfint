use std::error::Error;
use std::io::{Read, Write};
use crate::parse::program::Instruction;


pub struct VirtualMachine {
    memory: Vec<u8>,
    mp: usize,
    pc: usize,
    status: Status,
    settings: Settings,
}

#[derive(PartialEq, Eq)]
pub enum Status {
    Idle,
    Running,
}

pub struct Settings {
    pub memory_size: usize,
    pub memory_overflow_behavior: MemoryOverflowBehavior,
    pub input: Box<dyn Read>,
    pub output: Box<dyn Write>,
}

pub enum MemoryOverflowBehavior {
    Unchecked,
    Saturate,
    Wrap,
}

/* Environment ********************************************************************************************************/
impl VirtualMachine {
    /// Create a VirtualMachine with the default settings
    pub fn new() -> VirtualMachine {
        VirtualMachine::with_settings(Settings {
            memory_size: 4096,
            memory_overflow_behavior: MemoryOverflowBehavior::Unchecked,
            input: Box::new(std::io::stdin()),
            output: Box::new(std::io::stdout()),
        })
    }

    /// Create a VirtualMachine with the specified settings
    pub fn with_settings(settings: Settings) -> VirtualMachine {
        VirtualMachine {
            memory: vec![0; settings.memory_size],
            mp: 0,
            pc: 0,
            status: Status::Idle,
            settings,
        }
    }

    /// Completely reset the VirtualMachine, including memory
    pub fn reset(&mut self) {
        self.reset_core();
        self.reset_memory();
    }

    /// Fill memory with 0
    pub fn reset_memory(&mut self) {
        self.memory.fill(0);
    }

    /// Reset the core of the machine. This resets the program counter, memory pointer and status. Note: this method
    /// does not clear the memory
    pub fn reset_core(&mut self) {
        self.pc = 0;
        self.mp = 0;
        self.status = Status::Idle;
    }

    /// Bring status from Idle to Running. Returns an error if status is not idle.
    pub fn wakeup(&mut self) -> Result<(), Box<dyn Error>>{
        match self.status {
            Status::Idle => self.status = Status::Running,
            _ => return Err("Virtual Machine status is not Idle".into()),
        }
        Ok(())
    }

    /// Get current status
    pub fn status(&self) -> &Status {
        &self.status
    }

    /// Return the current value of the program counter
    pub fn pc(&self) -> usize {
        self.pc
    }

    /// Execute requested instruction
    pub fn execute_instruction(&mut self, instruction: &Instruction) -> Result<&Status, Box<dyn Error>> {
        let mut next_pc = self.pc + 1;
        // Execute instruction
        match *instruction {
            Instruction::IncPtr => self.inc_mp(),
            Instruction::DecPtr => self.dec_mp(),
            Instruction::IncData => self.mem_inc(),
            Instruction::DecData => self.mem_dec(),
            Instruction::Output => self.write_byte()?,
            Instruction::Input => self.read_byte(true)?,
            Instruction::JNZ(addr) => {
                if self.mem_rd() != 0 {
                    next_pc = addr;
                }
            }
            Instruction::JZ(addr) => {
                if self.mem_rd() == 0 {
                    next_pc = addr;
                }
            }
            Instruction::Exit => self.status = Status::Idle,
        }
        // Update program counter
        self.pc = next_pc;
        Ok(&self.status)
    }

    /// Read memory location under current memory pointer
    pub fn mem_rd(&self) -> u8 {
        self.memory[self.mp]
    }

    /// Write to memory location under current memory pointer
    pub fn mem_wr(&mut self, val: u8) {
        self.memory[self.mp] = val
    }

    /// Increment data under current memory pointer
    pub fn mem_inc(&mut self) {
        self.memory[self.mp] += 1;
    }

    /// Decrement data under current memory pointer
    pub fn mem_dec(&mut self) {
        self.memory[self.mp] -= 1;
    }

    /// Read one byte from VirtualMachine's input source and store it under current memory pointer
    pub fn read_byte(&mut self, ignore_newlines: bool) -> Result<(), std::io::Error> {
        let mut buffer = [0u8];
        self.settings.input.read(&mut buffer)?;
        while ignore_newlines && buffer[0] == '\n' as u8 {
            self.settings.input.read(&mut buffer)?;
        }
        self.memory[self.mp] = buffer[0];
        Ok(())
    }

    /// Output one byte under current memory pointer to the VirtualMachine's output
    pub fn write_byte(&mut self) -> Result<(), std::io::Error> {
        write!(self.settings.output, "{}", self.memory[self.mp] as char)
    }

    fn inc_mp(&mut self) {
        use MemoryOverflowBehavior::*;
        match self.settings.memory_overflow_behavior {
            Unchecked => self.mp += 1,
            Saturate => {
                if self.mp < self.memory.len() - 1 {
                    self.mp += 1;
                }
            }
            Wrap => {
                self.mp += 1;
                if self.mp >= self.memory.len() {
                    self.mp = 0;
                }
            },
        };
    }

    fn dec_mp(&mut self) {
        use MemoryOverflowBehavior::*;
        match self.settings.memory_overflow_behavior {
            Unchecked => self.mp -= 1,
            Saturate => {
                if self.mp > 0 {
                    self.mp -= 1;
                }
            }
            Wrap => {
                if self.mp > 0 {
                    self.mp -= 1;
                } else {
                    self.mp = self.memory.len() - 1;
                }
            }
        }
    }
}
