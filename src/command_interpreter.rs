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

    fn run(command: &Command) -> (String, bool) {
        let mut out = Vec::new();
        let should_stop = interpret(command, &mut out).unwrap();
        (String::from_utf8(out).unwrap(), should_stop)
    }

    fn type_of(name: &str, inner: Command) -> Command {
        Command::Type(name.into(), Box::new(inner))
    }

    #[test]
    fn exit_stops_without_output() {
        assert_eq!(run(&Command::Exit), (String::new(), true));
    }

    #[test]
    fn echo_prints_argument() {
        assert_eq!(run(&Command::Echo("hi".into())), ("hi\n".into(), false));
    }

    #[test]
    fn echo_empty_prints_newline() {
        assert_eq!(run(&Command::Echo(String::new())), ("\n".into(), false));
    }

    #[test]
    fn type_echo_is_builtin() {
        let cmd = type_of("echo", Command::Echo(String::new()));
        assert_eq!(run(&cmd), ("echo is a shell builtin\n".into(), false));
    }

    #[test]
    fn type_exit_is_builtin() {
        let cmd = type_of("exit", Command::Exit);
        assert_eq!(run(&cmd), ("exit is a shell builtin\n".into(), false));
    }

    #[test]
    fn type_type_is_builtin() {
        let cmd = type_of("type", type_of("x", Command::Exit));
        assert_eq!(run(&cmd), ("type is a shell builtin\n".into(), false));
    }

    #[test]
    fn type_unknown_is_not_found() {
        let cmd = type_of("foo", Command::Unknown("foo".into()));
        assert_eq!(run(&cmd), ("foo: not found\n".into(), false));
    }

    #[test]
    fn unknown_command_not_found() {
        let cmd = Command::Unknown("foo".into());
        assert_eq!(run(&cmd), ("foo: command not found\n".into(), false));
    }

    #[test]
    fn is_builtin_classification() {
        assert!(Command::Echo(String::new()).is_builtin());
        assert!(Command::Exit.is_builtin());
        assert!(type_of("x", Command::Exit).is_builtin());
        assert!(!Command::Unknown("x".into()).is_builtin());
    }
}
