mod interpreter;

use interpreter::interpreter::Interpreter;
fn main() {
    let mut interpreter = Interpreter::interpret_file("test/helloworld.bf").expect("Could not interpret file");
    interpreter.run();
}
