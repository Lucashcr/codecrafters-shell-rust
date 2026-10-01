use std::io::{self, Write};

#[derive(Debug, PartialEq)]
pub enum Command {
    Echo(String),
    Type(String, Box<Command>),
    Exit,
    Nothing,
    Unknown(String),
}

impl Command {
    fn is_builtin(&self) -> bool {
        !matches!(self, Command::Unknown(_))
    }
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
        Command::Type(name, command) => {
            if command.is_builtin() {
                writeln!(out, "{name} is a shell builtin")?;
            } else {
                writeln!(out, "{name}: not found")?;
            }
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
    use crate::prompt_parser::{make_echo, make_type, make_unknown};

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
        let cmd = make_type("echo");
        assert_eq!(run(&cmd), (String::from("echo is a shell builtin\n"), false));
    }

    #[test]
    fn type_exit_is_builtin() {
        let cmd = make_type("exit");
        assert_eq!(run(&cmd), (String::from("exit is a shell builtin\n"), false));
    }

    #[test]
    fn type_type_is_builtin() {
        let cmd = make_type("type");
        assert_eq!(run(&cmd), (String::from("type is a shell builtin\n"), false));
    }

    #[test]
    fn type_ls_is_executable_file() {
        let cmd = make_type("ls");
        assert_eq!(run(&cmd), (String::from("ls is /usr/bin/ls\n"), false));
    }

    #[test]
    fn type_unknown_is_not_found() {
        let cmd = make_type("foo");
        assert_eq!(run(&cmd), (String::from("foo: not found\n"), false));
    }

    #[test]
    fn unknown_command_not_found() {
        let cmd = make_unknown("foo");
        assert_eq!(run(&cmd), (String::from("foo: command not found\n"), false));
    }

    #[test]
    fn is_builtin_classification() {
        assert!(make_echo("").is_builtin());
        assert!(Command::Exit.is_builtin());
        assert!(make_type("x").is_builtin());
        assert!(!make_unknown("x").is_builtin());
    }
}
