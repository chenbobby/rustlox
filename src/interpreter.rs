use crate::parser::Parse;
use std::io::Write;

use crate::parser::recursive_descent::RecursiveDescentParser;
use crate::scanner::Scanner;

pub struct Interpreter {
    had_error: bool,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter { had_error: false }
    }

    pub fn run_file(&mut self, filename: &str) -> Result<(), std::io::Error> {
        let mut source_code = std::fs::read_to_string(filename)?;
        self.run(&mut source_code);
        Ok(())
    }

    pub fn run_prompt(&mut self) -> Result<(), std::io::Error> {
        loop {
            print!(">> ");
            std::io::stdout().flush()?;
            let mut source_code = String::new();
            match std::io::stdin().read_line(&mut source_code) {
                Err(error) => return Err(error),
                Ok(0) => return Ok(()),
                Ok(_) => {
                    self.run(&mut source_code);
                    if self.had_error {
                        return Ok(());
                    }
                }
            };
        }
    }

    fn run(&mut self, source_code: &mut str) {
        let mut scanner = Scanner::new();
        if let Err(scan_error) = scanner.scan(source_code) {
            self.error(scan_error.line_number, &scan_error.message)
        }
        let tokens = &scanner.tokens;
        println!("Token:\n{:#?}", tokens);
        let node = RecursiveDescentParser::new().parse(tokens);
        println!("AST:\n{:#?}", node);
    }

    fn error(&mut self, line_number: i32, message: &str) {
        self.report(line_number, message);
    }

    fn report(&mut self, line_number: i32, message: &str) {
        eprintln!("[line {}] Error: {}", line_number, message);
        self.had_error = true;
    }
}
