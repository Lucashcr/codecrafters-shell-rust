use std::io::Write;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};

use crate::path_loader::Executables;

#[derive(Debug, PartialEq, Eq)]
pub enum CommandType {
    Builtin,
    Executable(PathBuf),
    NotFound,
}

const BUILTINS: &[&str] = &["echo", "exit", "type"];

impl CommandType {
    pub fn resolve(cmd: &str, executables: &Executables) -> Self {
        if BUILTINS.contains(&cmd) {
            return Self::Builtin;
        }
        if let Some(path) = executables.lookup(cmd) {
            return Self::Executable(path.into());
        }
        Self::NotFound
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Echo(String),
    Type(String, CommandType),
    Exit,
    UsageError(String),
    Nothing,
    Unknown(String),
    Executable(String, PathBuf, Vec<String>),
}

impl Command {
    pub fn new_echo(s: &str) -> Self {
        Self::Echo(s.into())
    }

    pub fn new_type(s: &str, executables: &Executables) -> Self {
        Command::Type(s.into(), CommandType::resolve(s, executables))
    }

    pub fn new_executable(name: &str, path: &Path, args: Vec<String>) -> Command {
        Command::Executable(name.into(), path.into(), args)
    }

    pub fn new_usage_error(message: &str) -> Command {
        Command::UsageError(message.into())
    }

    pub fn new_unknown(s: &str) -> Command {
        Command::Unknown(s.into())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct InterpResult {
    pub return_code: i32,
    pub should_finish: bool,
}

impl InterpResult {
    const fn ok() -> Self {
        Self {
            return_code: 0,
            should_finish: false,
        }
    }
    const fn exit() -> Self {
        Self {
            return_code: 0,
            should_finish: true,
        }
    }
    const fn with_code(code: i32) -> Self {
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
            Self::Nothing => InterpResult::ok(),
            Self::Exit => InterpResult::exit(),
            Self::Echo(arg) => {
                writeln!(stdout, "{arg}")?;
                InterpResult::ok()
            }
            Self::Type(name, command) => match command {
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
            Self::UsageError(message) => {
                writeln!(stderr, "{message}")?;
                InterpResult::with_code(1)
            }
            Self::Executable(name, path, args) => {
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
            Self::Unknown(cmd) => {
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
    use crate::test_support::mock_executables;

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
            (String::new(), String::new(), InterpResult::exit())
        );
    }

    #[test]
    fn echo_prints_argument() {
        assert_eq!(
            run(&Command::new_echo("hi")),
            (String::from("hi\n"), String::new(), InterpResult::ok())
        );
    }

    #[test]
    fn echo_empty_prints_newline() {
        assert_eq!(
            run(&Command::new_echo("")),
            (String::from("\n"), String::new(), InterpResult::ok())
        );
    }

    #[test]
    fn type_echo_is_builtin() {
        let cmd = Command::new_type("echo", &mock_executables());
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
        let cmd = Command::new_type("exit", &mock_executables());
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
        let cmd = Command::new_type("type", &mock_executables());
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
        let cmd = Command::new_type("grep", &mock_executables());
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
        let cmd = Command::new_type("ls", &mock_executables());
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
        let cmd = Command::new_type("foo", &mock_executables());
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
        let cmd = Command::new_unknown("foo");
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
