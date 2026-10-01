use std::io;
use std::io::Write;

mod prompt_parser;
mod command_interpreter;

fn repl(stdin: &io::Stdin) -> bool {
    let mut should_stop = false;

    print!("$ ");
    io::stdout().flush().unwrap();

    let mut prompt = String::new();
    match stdin.read_line(&mut prompt) {
        Ok(_) => {
            prompt = String::from(prompt.trim());
            if prompt.is_empty() {
                return false;
            }

            let command = prompt_parser::parse(&prompt);
            should_stop = command_interpreter::interpret(&command);
        }
        Err(error) => {
            println!("{error}");
        }
    }

    io::stdout().flush().unwrap();
    should_stop
}

fn main() {
    let stdin = io::stdin();

    let mut finish_loop = false;
    while !finish_loop {
        finish_loop = repl(&stdin);
    }
}
