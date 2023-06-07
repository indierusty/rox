use rox::interpreter::Interpreter;

fn main() {
    let file_paths: Vec<_> = std::env::args().collect();
    Interpreter::new().interpret(&std::fs::read_to_string(&file_paths[1]).unwrap());
}
