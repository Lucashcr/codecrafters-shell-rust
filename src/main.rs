use std::collections::HashMap;
use std::io;
use std::io::BufRead;
use std::io::Write;
use std::path::PathBuf;

mod command;
mod path_loader;
mod prompt_parser;

fn repl(
    stdin: &mut impl BufRead,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
    executables: &HashMap<String, PathBuf>,
) -> io::Result<bool> {
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

            let line = prompt.trim();
            if line.is_empty() {
                writeln!(stdout)?;
                return Ok(false);
            }

            let command = prompt_parser::parse(line, executables);
            let result = command.interp(stdout, stderr)?;
            Ok(result.should_finish)
        }
        Err(error) => {
            writeln!(stderr, "{error}")?;
            Ok(false)
        }
    }
}

fn main() -> io::Result<()> {
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();

    let executables = path_loader::load();

    loop {
        if repl(&mut stdin, &mut stdout, &mut stderr, &executables)? {
            break;
        }
    }

    Ok(())
}
