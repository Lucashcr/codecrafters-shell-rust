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
        Err(error) => match error.kind() {
            io::ErrorKind::Interrupted => Ok(false),
            io::ErrorKind::InvalidData => {
                writeln!(stderr, "{error}")?;
                Ok(false)
            }
            _ => Err(error),
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

#[cfg(test)]
pub(crate) mod tests_support {
    use std::collections::HashMap;
    use std::path::PathBuf;

    pub fn mock_executables() -> HashMap<String, PathBuf> {
        HashMap::from([(String::from("grep"), PathBuf::from("/usr/bin/grep"))])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Runs a single `repl` iteration. Returns `(stdout, stderr, should_finish)`.
    fn step(input: &str) -> (String, String, bool) {
        let mut stdin = Cursor::new(input.as_bytes());
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let finished = repl(&mut stdin, &mut stdout, &mut stderr, &HashMap::new()).unwrap();
        (
            String::from_utf8(stdout).unwrap(),
            String::from_utf8(stderr).unwrap(),
            finished,
        )
    }

    /// Runs `repl` until it asks to finish. Returns `(stdout, stderr)`.
    fn session(input: &str) -> (String, String) {
        let mut stdin = Cursor::new(input.as_bytes());
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        while !repl(&mut stdin, &mut stdout, &mut stderr, &HashMap::new()).unwrap() {}
        (
            String::from_utf8(stdout).unwrap(),
            String::from_utf8(stderr).unwrap(),
        )
    }

    #[test]
    fn eof_prints_newline_and_finishes() {
        assert_eq!(step(""), (String::from("$ \n"), String::new(), true));
    }

    #[test]
    fn blank_line_reprints_prompt_and_continues() {
        assert_eq!(step("   \n"), (String::from("$ \n"), String::new(), false));
    }

    #[test]
    fn echo_prints_and_continues() {
        assert_eq!(
            step("echo hi\n"),
            (String::from("$ hi\n"), String::new(), false)
        );
    }

    #[test]
    fn exit_finishes() {
        assert_eq!(step("exit\n"), (String::from("$ "), String::new(), true));
    }

    #[test]
    fn unknown_command_reports_on_stderr_and_continues() {
        assert_eq!(
            step("foo\n"),
            (
                String::from("$ "),
                String::from("foo: command not found\n"),
                false
            )
        );
    }

    #[test]
    fn session_stops_at_exit_and_ignores_the_rest() {
        let (stdout, stderr) = session("echo one\nexit\necho nope\n");
        assert_eq!(stdout, "$ one\n$ ");
        assert_eq!(stderr, "");
    }

    #[test]
    fn session_stops_at_eof() {
        let (stdout, _) = session("echo one\n");
        assert_eq!(stdout, "$ one\n$ \n");
    }
}
