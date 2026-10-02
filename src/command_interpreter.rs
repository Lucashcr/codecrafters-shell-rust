use std::io::{self, Write};

#[derive(Debug, PartialEq)]
pub enum Command {
    Echo(String),
    Type(String, Box<Command>),
    Exit,
    Nothing,
    Unknown(String),
    Executable(String),
}

pub fn interpret(command: &Command, out: &mut impl Write) -> io::Result<bool> {
    match command {
        Command::Nothing => {
            writeln!(out)?;
        }
        Command::Exit => {
            return Ok(true);
        }
        Command::Echo(arg) => {
            writeln!(out, "{arg}")?;
        }
        Command::Type(name, command) => match &**command {
            Command::Unknown(_) => writeln!(out, "{name}: not found")?,
            Command::Executable(cmd) => writeln!(out, "{name} is {cmd}")?,
            _ => writeln!(out, "{name} is a shell builtin")?,
        },
        Command::Executable(path) => {
            writeln!(out, "executing {path}...")?;
        }
        Command::Unknown(cmd) => {
            writeln!(out, "{cmd}: command not found")?;
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt_parser::{make_echo, make_executable, make_type, make_unknown};
    use std::collections::HashMap;

    fn mock_executables() -> HashMap<String, String> {
        HashMap::from([
            (String::from("grep"), String::from("/usr/bin/grep")),
        ])
    }

    fn run(command: &Command) -> (String, bool) {
        let mut out = Vec::new();
        let should_stop = interpret(command, &mut out).unwrap();
        (String::from_utf8(out).unwrap(), should_stop)
    }

    #[test]
    fn exit_stops_without_output() {
        assert_eq!(run(&Command::Exit), (String::new(), true));
    }

    #[test]
    fn echo_prints_argument() {
        assert_eq!(run(&make_echo("hi")), (String::from("hi\n"), false));
    }

    #[test]
    fn echo_empty_prints_newline() {
        assert_eq!(run(&make_echo("")), (String::from("\n"), false));
    }

    #[test]
    fn type_echo_is_builtin() {
        let cmd = make_type("echo", &mock_executables());
        assert_eq!(
            run(&cmd),
            (String::from("echo is a shell builtin\n"), false)
        );
    }

    #[test]
    fn type_exit_is_builtin() {
        let cmd = make_type("exit", &mock_executables());
        assert_eq!(
            run(&cmd),
            (String::from("exit is a shell builtin\n"), false)
        );
    }

    #[test]
    fn type_type_is_builtin() {
        let cmd = make_type("type", &mock_executables());
        assert_eq!(
            run(&cmd),
            (String::from("type is a shell builtin\n"), false)
        );
    }

    #[test]
    fn type_grep_is_executable_file() {
        let cmd = make_type("grep", &mock_executables());
        assert_eq!(run(&cmd), (String::from("grep is /usr/bin/grep\n"), false));
    }

    #[test]
    fn type_ls_is_not_executable_file() {
        let cmd = make_type("ls", &mock_executables());
        assert_eq!(run(&cmd), (String::from("ls: not found\n"), false));
    }

    #[test]
    fn type_unknown_is_not_found() {
    let cmd = make_type("foo", &mock_executables());
        assert_eq!(run(&cmd), (String::from("foo: not found\n"), false));
    }

    #[test]
    fn grep_is_executable_file() {
        let cmd = make_executable("grep");
        assert_eq!(run(&cmd), (String::from("executing grep...\n"), false));
    }

    #[test]
    fn unknown_command_not_found() {
        let cmd = make_unknown("foo");
        assert_eq!(run(&cmd), (String::from("foo: command not found\n"), false));
    }
}
