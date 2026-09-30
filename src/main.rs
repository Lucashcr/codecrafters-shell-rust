#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    let stdin = io::stdin();

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
