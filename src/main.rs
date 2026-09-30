use std::io;
use std::io::Write;

fn repl(stdin: &io::Stdin) -> bool {
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut prompt = String::new();
    match stdin.read_line(&mut prompt) {
        Ok(_) => {
            prompt = String::from(prompt.trim());
            if prompt == "exit" {
                return true;
            }
            println!("{prompt}: command not found");
        }
        Err(error) => {
            println!("{error}");
        }
    }

    io::stdout().flush().unwrap();
    false
}

fn main() {
    let stdin = io::stdin();

    let mut finish_loop = false;
    while !finish_loop {
        finish_loop = repl(&stdin);
    }
}
