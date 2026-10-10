use std::collections::HashMap;
use std::io::Write;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub enum CommandType {
    Builtin,
    Executable(PathBuf),
    NotFound,
}

const BUILTINS: &[&str] = &["echo", "exit", "type"];

impl CommandType {
    pub fn resolve(cmd: &str, executables: &HashMap<String, PathBuf>) -> CommandType {
        if BUILTINS.contains(&cmd) {
            return CommandType::Builtin;
        }
        if let Some(path) = executables.get(cmd) {
            return CommandType::Executable(path.clone());
        }
        CommandType::NotFound
    }
}

#[derive(Debug, PartialEq)]
pub enum Command {
    Echo(String),
    Type(String, CommandType),
    Exit,
    UsageError(String),
    Nothing,
    Unknown(String),
    Executable(String, PathBuf, Vec<String>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct InterpResult {
    pub return_code: i32,
    pub should_finish: bool,
}

impl InterpResult {
    fn ok() -> Self {
        Self {
            return_code: 0,
            should_finish: false,
        }
    }
    fn exit() -> Self {
        Self {
            return_code: 0,
            should_finish: true,
        }
    }
    fn with_code(code: i32) -> Self {
        Self {
            return_code: code,
            should_finish: false,
        }
    }
}

impl Command {
    pub fn interp(
        &self,
        stdout: &mut impl Write,
        stderr: &mut impl Write,
    ) -> std::io::Result<InterpResult> {
        let result = match self {
            Command::Nothing => InterpResult::ok(),
            Command::Exit => InterpResult::exit(),
            Command::Echo(arg) => {
                writeln!(stdout, "{arg}")?;
                InterpResult::ok()
            }
            Command::Type(name, command) => match command {
                CommandType::Builtin => {
                    writeln!(stdout, "{name} is a shell builtin")?;
                    InterpResult::ok()
                }
                CommandType::Executable(path) => {
                    writeln!(stdout, "{name} is {}", path.display())?;
                    InterpResult::ok()
                }
                CommandType::NotFound => {
                    writeln!(stdout, "{name}: not found")?;
                    InterpResult::ok()
                }
            },
            Command::UsageError(message) => {
                writeln!(stderr, "{message}")?;
                InterpResult::with_code(1)
            }
            Command::Executable(name, path, args) => {
                let status = std::process::Command::new(path)
                    .arg0(name)
                    .args(args)
                    .status();
                match status {
                    Ok(s) => InterpResult::with_code(s.code().unwrap_or(1)),
                    Err(error) => {
                        writeln!(stderr, "failed to execute {name}: {error}")?;
                        InterpResult::with_code(1)
                    }
                }
            }
            Command::Unknown(cmd) => {
                writeln!(stderr, "{cmd}: command not found")?;
                InterpResult::with_code(127)
            }
        };

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt_parser::{make_echo, make_type, make_unknown};
    use crate::tests_support::mock_executables;

    /// Returns `(stdout, stderr, result)` of interpreting `command`.
    fn run(command: &Command) -> (String, String, InterpResult) {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let result = command.interp(&mut stdout, &mut stderr).unwrap();
        (
            String::from_utf8(stdout).unwrap(),
            String::from_utf8(stderr).unwrap(),
            result,
        )
    }

    #[test]
    fn exit_stops_without_output() {
        assert_eq!(
            run(&Command::Exit),
            (String::from(""), String::new(), InterpResult::exit())
        );
    }

    #[test]
    fn echo_prints_argument() {
        assert_eq!(
            run(&make_echo("hi")),
            (String::from("hi\n"), String::new(), InterpResult::ok())
        );
    }

    #[test]
    fn echo_empty_prints_newline() {
        assert_eq!(
            run(&make_echo("")),
            (String::from("\n"), String::new(), InterpResult::ok())
        );
    }

    #[test]
    fn type_echo_is_builtin() {
        let cmd = make_type("echo", &mock_executables());
        assert_eq!(
            run(&cmd),
            (
                String::from("echo is a shell builtin\n"),
                String::new(),
                InterpResult::ok()
            )
        );
    }

    #[test]
    fn type_exit_is_builtin() {
        let cmd = make_type("exit", &mock_executables());
        assert_eq!(
            run(&cmd),
            (
                String::from("exit is a shell builtin\n"),
                String::new(),
                InterpResult::ok()
            )
        );
    }

    #[test]
    fn type_type_is_builtin() {
        let cmd = make_type("type", &mock_executables());
        assert_eq!(
            run(&cmd),
            (
                String::from("type is a shell builtin\n"),
                String::new(),
                InterpResult::ok()
            )
        );
    }

    #[test]
    fn type_grep_is_executable_file() {
        let cmd = make_type("grep", &mock_executables());
        assert_eq!(
            run(&cmd),
            (
                String::from("grep is /usr/bin/grep\n"),
                String::new(),
                InterpResult::ok()
            )
        );
    }

    #[test]
    fn type_ls_is_not_executable_file() {
        let cmd = make_type("ls", &mock_executables());
        assert_eq!(
            run(&cmd),
            (
                String::from("ls: not found\n"),
                String::new(),
                InterpResult::ok()
            )
        );
    }

    #[test]
    fn type_unknown_is_not_found() {
        let cmd = make_type("foo", &mock_executables());
        assert_eq!(
            run(&cmd),
            (
                String::from("foo: not found\n"),
                String::new(),
                InterpResult::ok()
            )
        );
    }

    #[test]
    fn unknown_command_not_found() {
        let cmd = make_unknown("foo");
        assert_eq!(
            run(&cmd),
            (
                String::new(),
                String::from("foo: command not found\n"),
                InterpResult::with_code(127)
            )
        );
    }
}
