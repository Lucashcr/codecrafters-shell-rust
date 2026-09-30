use std::io;
use std::io::Write;

fn repl(stdin: &io::Stdin) {
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut prompt = String::new();
    match stdin.read_line(&mut prompt) {
        Ok(_) => {
            prompt = String::from(prompt.trim());
            println!("{prompt}: command not found");
        }
        Err(error) => {
            println!("{error}");
        }
    }

    io::stdout().flush().unwrap();
}

fn main() {
    let stdin = io::stdin();

    loop {
        repl(&stdin);
    }
}
