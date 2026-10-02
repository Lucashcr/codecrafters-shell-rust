use std::collections::HashMap;

use crate::command_interpreter::Command;

pub fn make_echo(s: &str) -> Command {
    Command::Echo(String::from(s))
}

pub fn make_type(s: &str, executables: &HashMap<String, String>) -> Command {
    Command::Type(String::from(s), Box::new(parse(s, executables)))
}

pub fn make_executable(s: &str) -> Command {
    Command::Executable(s.to_string())
}

pub fn make_unknown(s: &str) -> Command {
    Command::Unknown(String::from(s))
}

pub fn parse(prompt: &str, executables: &HashMap<String, String>) -> Command {
    let mut args = prompt.split_whitespace();
    let Some(command) = args.next() else {
        return Command::Nothing;
    };

    if command == "exit" {
        return Command::Exit;
    }

    if command == "echo" {
        let x: Vec<&str> = args.collect();
        return Command::Echo(x.join(" "));
    }

    if let Some(cmd) = executables.get(command) {
        // return Command::Executable(cmd.clone());
        return make_executable(cmd);
    }

    if command == "type" {
        match (args.next(), args.next()) {
            (Some(arg), None) => {
                return make_type(arg, executables);
            }
            (Some(_), Some(_)) => return make_echo("too many arguments"),
            _ => return make_echo("type requires an argument"),
        }
    }

    make_unknown(command)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_executables() -> HashMap<String, String> {
        HashMap::from([
            (String::from("grep"), String::from("/usr/bin/grep")),
        ])
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
            Command::Type("echo".into(), Box::new(make_echo("")))
        );
        assert_eq!(
            parse("type exit", &mock_executables()),
            Command::Type("exit".into(), Box::new(Command::Exit))
        );
        assert_eq!(
            parse("type type", &mock_executables()),
            Command::Type(
                "type".into(),
                Box::new(make_echo("type requires an argument"))
            )
        );
    }

    #[test]
    fn parses_type_of_unknown() {
        assert_eq!(
            parse("type foo", &mock_executables()),
            Command::Type("foo".into(), Box::new(make_unknown("foo")))
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
            make_executable("/usr/bin/grep"),
        );
    }

    #[test]
    fn ls_is_not_executable_file() {
        assert_ne!(
            parse("ls", &mock_executables()),
            make_executable("/usr/bin/ls"),
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

    #[test]
    fn blank_input_does_not_panic() {
        for input in ["", "   ", "\t"] {
            assert!(
                std::panic::catch_unwind(|| parse(input, &mock_executables())).is_ok(),
                "parse({input:?}) panicked"
            );
        }
    }
}
