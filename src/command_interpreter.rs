use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub enum Command {
    Echo(String),
    Type(String, Box<Command>),
    Exit,
    Nothing,
    Unknown(String),
    Executable(String, PathBuf, Vec<String>),
}

pub struct InterpreterResult {
    pub return_code: i32,
    pub should_finish: bool,
}

pub fn interpret(
    command: &Command,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> std::io::Result<InterpreterResult> {
    match command {
        Command::Nothing => {
            writeln!(stdout)?;
            return Ok(InterpreterResult {
                return_code: 0,
                should_finish: false,
            });
        }
        Command::Exit => {
            writeln!(stdout)?;
            return Ok(InterpreterResult {
                return_code: 0,
                should_finish: true,
            });
        }
        Command::Echo(arg) => {
            writeln!(stdout, "{arg}")?;
        }
        Command::Type(name, command) => match &**command {
            Command::Unknown(_) => {
                writeln!(stdout, "{name}: not found")?;
            }
            Command::Executable(cmd, path, _) => {
                writeln!(stdout, "{cmd} is {}", path.to_string_lossy())?;
            }
            _ => {
                writeln!(stdout, "{name} is a shell builtin")?;
            }
        },
        Command::Executable(name, _, args) => {
            let status = std::process::Command::new(name).args(args).status();
            match status {
                Ok(status) => {
                    return Ok(InterpreterResult {
                        return_code: status.code().unwrap_or(1),
                        should_finish: false,
                    });
                }
                Err(error) => {
                    writeln!(stderr, "failed to execute {name}: {error}")?;
                    return Ok(InterpreterResult {
                        return_code: 1,
                        should_finish: false,
                    });
                }
            };
        }
        Command::Unknown(cmd) => {
            writeln!(stdout, "{cmd}: command not found")?;
            return Ok(InterpreterResult {
                return_code: 127,
                should_finish: false,
            });
        }
    }

    Ok(InterpreterResult {
        return_code: 0,
        should_finish: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt_parser::{make_echo, make_type, make_unknown};
    use std::collections::HashMap;

    fn mock_executables() -> HashMap<String, PathBuf> {
        HashMap::from([(String::from("grep"), PathBuf::from("/usr/bin/grep"))])
    }

    fn run(command: &Command) -> (String, i32, bool) {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let result = interpret(command, &mut stdout, &mut stderr);
        let result_ref = result.as_ref();
        (
            String::from_utf8(stdout).unwrap(),
            result_ref.unwrap().return_code,
            result_ref.unwrap().should_finish,
        )
    }

    #[test]
    fn exit_stops_without_output() {
        assert_eq!(run(&Command::Exit), (String::from("\n"), 0, true));
    }

    #[test]
    fn echo_prints_argument() {
        assert_eq!(run(&make_echo("hi")), (String::from("hi\n"), 0, false));
    }

    #[test]
    fn echo_empty_prints_newline() {
        assert_eq!(run(&make_echo("")), (String::from("\n"), 0, false));
    }

    #[test]
    fn type_echo_is_builtin() {
        let cmd = make_type("echo", &mock_executables());
        assert_eq!(
            run(&cmd),
            (String::from("echo is a shell builtin\n"), 0, false)
        );
    }

    #[test]
    fn type_exit_is_builtin() {
        let cmd = make_type("exit", &mock_executables());
        assert_eq!(
            run(&cmd),
            (String::from("exit is a shell builtin\n"), 0, false)
        );
    }

    #[test]
    fn type_type_is_builtin() {
        let cmd = make_type("type", &mock_executables());
        assert_eq!(
            run(&cmd),
            (String::from("type is a shell builtin\n"), 0, false)
        );
    }

    #[test]
    fn type_grep_is_executable_file() {
        let cmd = make_type("grep", &mock_executables());
        assert_eq!(
            run(&cmd),
            (String::from("grep is /usr/bin/grep\n"), 0, false)
        );
    }

    #[test]
    fn type_ls_is_not_executable_file() {
        let cmd = make_type("ls", &mock_executables());
        assert_eq!(run(&cmd), (String::from("ls: not found\n"), 0, false));
    }

    #[test]
    fn type_unknown_is_not_found() {
        let cmd = make_type("foo", &mock_executables());
        assert_eq!(run(&cmd), (String::from("foo: not found\n"), 0, false));
    }

    #[test]
    fn unknown_command_not_found() {
        let cmd = make_unknown("foo");
        assert_eq!(
            run(&cmd),
            (String::from("foo: command not found\n"), 127, false)
        );
    }
}
