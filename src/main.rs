mod interpreter;
mod parse;

extern crate argparse;

use argparse::ArgumentParser;
use std::error::Error;

use interpreter::interpreter::Interpreter;
fn main() -> Result<(), Box<dyn Error>> {
    let mut fname = String::new();
    let mut memsize = 4096;
    {
        // Parse args
        let mut parser = ArgumentParser::new();
        parser.set_description("An over-engineered brainf*ck interpreter.");

        parser.refer(&mut fname)
            .add_argument("fname", argparse::Store, "brainf*ck file to run");

        parser.refer(&mut memsize)
            .add_option(&["--memsize"], argparse::Store, "amount of memory to allocate in bytes");

        if let Err(code) = parser.parse_args() {
            return Err(format!("Error while parsing arguments: code {}", code).into());
        }
    }
    // Run interpreter
    if fname.is_empty() {
        // CL mode
        todo!("Command line mode is not supported yet");
    } else {
        let mut interpreter = Interpreter::new();
        interpreter.load_file(&fname)?;
        interpreter.run()?;
    }
    Ok(())
}
