mod interpreter;
mod parser;
mod scanner;

use crate::interpreter::Interpreter;

const EXIT_CODE_USAGE: i32 = 32;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 2 {
        eprintln!("Usage: rustlox [script]");
        std::process::exit(EXIT_CODE_USAGE);
    } else if args.len() == 2 {
        let filename = &args[1];
        let mut interpreter = Interpreter::new();
        if let Err(error) = interpreter.run_file(filename) {
            eprintln!("{}", error);
            let code = error.raw_os_error().unwrap_or(1);
            std::process::exit(code);
        }
    } else {
        let mut interpreter = Interpreter::new();
        if let Err(error) = interpreter.run_prompt() {
            eprintln!("{}", error);
            let code = error.raw_os_error().unwrap_or(1);
            std::process::exit(code);
        }
    }
}
