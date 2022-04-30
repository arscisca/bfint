use std::error::Error;
use std::fs::File;
use std::io::Write;
use crate::interpreter::virtualmachine;

use crate::parse::program::Program;
use super::virtualmachine::{VirtualMachine, Settings};

pub struct Interpreter {
    program: Program,
    vm: VirtualMachine,
}

/* Interpreter *******************************************************************************************************/
impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            program: Program::new(),
            vm: VirtualMachine::new(),
        }
    }

    pub fn with_vm_settings(settings: Settings) -> Interpreter {
        Interpreter {
            program: Program::new(),
            vm: VirtualMachine::with_settings(settings),
        }
    }

    pub fn load_file(&mut self, fname: &str) -> Result<(), Box<dyn Error>> {
        let program = Program::compile(File::open(fname)?)?;
        self.program = program;
        self.vm.reset();
        Ok(())
    }

    pub fn dump_program<W: Write>(&self, sink: &mut W) -> Result<(), std::io::Error> {
        self.program.dump(sink)
    }

    pub fn startup(&mut self) -> Result<(), Box<dyn Error>> {
        self.vm.wakeup()
    }

    pub fn step(&mut self) -> Result<(), Box<dyn Error>> {
        // Check if instruction should be running
        if *self.vm.status() != virtualmachine::Status::Running {
            return Err("Interpreter is not running".into());
        }
        let instruction = self.program.instruction(self.vm.pc());
        self.vm.execute_instruction(instruction)?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.startup()?;
        loop {
            match self.vm.status() {
                virtualmachine::Status::Running => self.step()?,
                virtualmachine::Status::Idle => break,
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Execute helloworld.bf as an overall sanity check
    #[test]
    fn run_hello_world() {
        let sink = Vec::new();
        let settings = virtualmachine::Settings {
            memory_size: 128,
            memory_overflow_behavior: virtualmachine::MemoryOverflowBehavior::Unchecked,
            input: Box::new(std::io::stdin()),
            output: Box::new(sink),
        };
        let mut interpreter = Interpreter::new();
        interpreter.load_file("test/helloworld.bf");
        interpreter.run()
            .expect("Error while running");
    }
}