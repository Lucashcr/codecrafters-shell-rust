use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::path::PathBuf;

mod command_interpreter;
mod path_loader;
mod prompt_parser;

fn repl(
    stdin: &io::Stdin,
    stdout: &mut io::Stdout,
    stderr: &mut io::Stderr,
    executables: &HashMap<String, PathBuf>,
) -> io::Result<bool> {
    let mut result = command_interpreter::InterpreterResult {
        return_code: 0,
        should_finish: false,
    };

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
            result = command_interpreter::interpret(&command, stdout, stderr)?;
        }
        Err(error) => {
            writeln!(stderr, "{error}")?;
            result.return_code = 1;
            result.should_finish = false;
        }
    }

    stdout.flush()?;
    Ok(result.should_finish)
}

fn main() -> std::result::Result<(), std::io::Error> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    let executables = path_loader::load();
    // println!("{executables:?}");

    let mut finish_loop = false;
    while !finish_loop {
        finish_loop = repl(&stdin, &mut stdout, &mut stderr, &executables)?;
    }

    Ok(())
}
