use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::command::Command;

pub fn make_echo(s: &str) -> Command {
    Command::Echo(String::from(s))
}

pub fn make_type(s: &str, executables: &HashMap<String, PathBuf>) -> Command {
    let command = parse(s, executables);
    Command::Type(String::from(s), command.resolve_type())
}

pub fn make_executable(name: &str, path: &Path, args: Vec<String>) -> Command {
    Command::Executable(name.to_string(), path.to_path_buf(), args)
}

pub fn make_unknown(s: &str) -> Command {
    Command::Unknown(String::from(s))
}

pub fn parse(prompt: &str, executables: &HashMap<String, PathBuf>) -> Command {
    let mut args = prompt.split_whitespace();
    let Some(command) = args.next() else {
        return Command::Nothing;
    };

    match command {
        "exit" => Command::Exit,
        "echo" => {
            let args: Vec<&str> = args.collect();
            make_echo(&args.join(" "))
        }
        "type" => match (args.next(), args.next()) {
            (Some(arg), None) => make_type(arg, executables),
            (Some(_), Some(_)) => make_echo("too many arguments"),
            _ => make_echo("type requires an argument"),
        },
        _ => match executables.get(command) {
            Some(path) => {
                let args: Vec<String> = args.map(String::from).collect();
                make_executable(command, path, args)
                // Command::Executable(String::from(command), path.to_path_buf(), args)
            }
            None => make_unknown(command),
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::command::CommandType;

    use super::*;

    fn mock_executables() -> HashMap<String, PathBuf> {
        HashMap::from([(String::from("grep"), PathBuf::from("/usr/bin/grep"))])
    }

    #[test]
    fn parses_exit() {
        assert_eq!(parse("exit", &mock_executables()), Command::Exit);
    }

    #[test]
    fn parses_echo_without_args() {
        assert_eq!(parse("echo", &mock_executables()), make_echo(""));
    }

    #[test]
    fn parses_echo_single_arg() {
        assert_eq!(parse("echo hello", &mock_executables()), make_echo("hello"));
    }

    #[test]
    fn parses_echo_multiple_args() {
        assert_eq!(
            parse("echo hello world", &mock_executables()),
            make_echo("hello world")
        );
    }

    #[test]
    fn echo_normalizes_whitespace() {
        assert_eq!(
            parse("echo a    b\tc", &mock_executables()),
            make_echo("a b c")
        );
    }

    #[test]
    fn echo_with_leading_whitespace() {
        assert_eq!(parse("   echo hi", &mock_executables()), make_echo("hi"));
    }

    #[test]
    fn parses_type_of_builtins() {
        assert_eq!(
            parse("type echo", &mock_executables()),
            Command::Type("echo".into(), CommandType::Builtin)
        );
        assert_eq!(
            parse("type exit", &mock_executables()),
            Command::Type("exit".into(), CommandType::Builtin)
        );
        assert_eq!(
            parse("type type", &mock_executables()),
            Command::Type("type".into(), CommandType::Builtin)
        );
    }

    #[test]
    fn parses_type_of_unknown() {
        assert_eq!(
            parse("type foo", &mock_executables()),
            Command::Type("foo".into(), CommandType::NotFound)
        );
    }

    #[test]
    fn type_without_argument() {
        assert_eq!(
            parse("type", &mock_executables()),
            make_echo("type requires an argument")
        );
    }

    #[test]
    fn type_with_too_many_arguments() {
        assert_eq!(
            parse("type a b", &mock_executables()),
            make_echo("too many arguments")
        );
    }

    #[test]
    fn grep_is_executable_file() {
        assert_eq!(
            parse("grep", &mock_executables()),
            make_executable("grep", &PathBuf::from("/usr/bin/grep"), vec![]),
        );
    }

    #[test]
    fn ls_is_not_executable_file() {
        assert_ne!(
            parse("ls", &mock_executables()),
            make_executable("ls", &PathBuf::from("/usr/bin/ls"), vec![]),
        );
    }

    #[test]
    fn parses_unknown_command() {
        assert_eq!(parse("foo", &mock_executables()), make_unknown("foo"));
    }

    #[test]
    fn unknown_command_ignores_args() {
        assert_eq!(
            parse("foo bar baz", &mock_executables()),
            make_unknown("foo")
        );
    }

    #[test]
    fn commands_are_case_sensitive() {
        assert_eq!(parse("ECHO hi", &mock_executables()), make_unknown("ECHO"));
        assert_eq!(parse("Exit", &mock_executables()), make_unknown("Exit"));
    }

    #[test]
    fn similar_names_are_not_builtins() {
        assert_eq!(parse("echoo", &mock_executables()), make_unknown("echoo"));
        assert_eq!(parse("exits", &mock_executables()), make_unknown("exits"));
    }

    fn executables_shadowing_builtins() -> HashMap<String, PathBuf> {
        HashMap::from([
            (String::from("echo"), PathBuf::from("/usr/bin/echo")),
            (String::from("exit"), PathBuf::from("/usr/bin/exit")),
            (String::from("type"), PathBuf::from("/usr/bin/type")),
        ])
    }

    #[test]
    fn type_of_executable_reports_its_path() {
        assert_eq!(
            parse("type grep", &mock_executables()),
            Command::Type(
                "grep".into(),
                CommandType::Executable(PathBuf::from("/usr/bin/grep"))
            )
        );
    }

    #[test]
    fn echo_builtin_wins_over_executable_with_same_name() {
        assert_eq!(
            parse("echo hi", &executables_shadowing_builtins()),
            make_echo("hi")
        );
    }

    #[test]
    fn exit_builtin_wins_over_executable_with_same_name() {
        assert_eq!(
            parse("exit", &executables_shadowing_builtins()),
            Command::Exit
        );
    }

    #[test]
    fn type_builtin_wins_over_executable_with_same_name() {
        assert_eq!(
            parse("type echo", &executables_shadowing_builtins()),
            Command::Type("echo".into(), CommandType::Builtin)
        );
    }

    #[test]
    fn blank_input_does_not_panic() {
        for input in ["", "   ", "\t"] {
            assert_eq!(parse(input, &mock_executables()), Command::Nothing);
        }
    }
}
