use std::io::Write; // Brings std::io::Stdout.flush into scope.

const EXIT_CODE_USAGE: i32 = 64;

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

struct Interpreter {
    had_error: bool,
}

impl Interpreter {
    fn new() -> Interpreter {
        Interpreter { had_error: false }
    }

    fn run_file(&mut self, filename: &str) -> Result<(), std::io::Error> {
        let mut source_code = std::fs::read_to_string(filename)?;
        self.run(&mut source_code);
        Ok(())
    }

    fn run_prompt(&mut self) -> Result<(), std::io::Error> {
        loop {
            print!(">> ");
            std::io::stdout().flush()?;
            let mut source_code = String::new();
            match std::io::stdin().read_line(&mut source_code) {
                Err(error) => return Err(error),
                Ok(0) => return Ok(()),
                Ok(_) => self.run(&mut source_code),
            };
        }
    }

    fn run(&mut self, source_code: &mut str) {}
}
