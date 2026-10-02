use std::collections::HashMap;
use std::io;
use std::io::Write;

mod prompt_parser;
mod command_interpreter;
mod path_loader;

fn repl(stdin: &io::Stdin, stdout: &mut io::Stdout, executables: &HashMap<String, String>) -> io::Result<bool> {
    let mut should_stop = false;

    write!(stdout, "$ ")?;
    stdout.flush()?;

    let mut prompt = String::new();
    match stdin.read_line(&mut prompt) {
        Ok(n) => {
            // EOF
            if n == 0 {
                writeln!(stdout)?;
                return Ok(true);
            }

            prompt = String::from(prompt.trim());
            if prompt.is_empty() {
                writeln!(stdout)?;
                return Ok(false);
            }

            let command = prompt_parser::parse(&prompt, executables);
            should_stop = command_interpreter::interpret(&command, stdout)?;
        }
        Err(error) => {
            println!("{error}");
        }
    }

    stdout.flush()?;
    Ok(should_stop)
}

fn main() -> std::result::Result<(), std::io::Error> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    let executables = path_loader::load(&mut stderr);
    // println!("{executables:?}");

    let mut finish_loop = false;
    while !finish_loop {
        finish_loop = repl(&stdin, &mut stdout, &executables)?;
    }

    Ok(())
}
