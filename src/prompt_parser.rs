use crate::command_interpreter::Command;

pub fn make_echo(s: &str) -> Command {
    Command::Echo(String::from(s))
}

pub fn make_type(s: &str) -> Command {
    Command::Type(String::from(s), Box::new(parse(s)))
}

pub fn make_unknown(s: &str) -> Command {
    Command::Unknown(String::from(s))
}

pub fn parse(prompt: &str) -> Command {
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

    if command == "type" {
        match (args.next(), args.next()) {
            (Some(arg), None) => return make_type(arg),
            (Some(_), Some(_)) => return make_echo("too many arguments"),
            _ => return make_echo("type requires an argument"),
        }
    }

    make_unknown(command)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_exit() {
        assert_eq!(parse("exit"), Command::Exit);
    }

    #[test]
    fn parses_echo_without_args() {
        assert_eq!(parse("echo"), make_echo(""));
    }

    #[test]
    fn parses_echo_single_arg() {
        assert_eq!(parse("echo hello"), make_echo("hello"));
    }

    #[test]
    fn parses_echo_multiple_args() {
        assert_eq!(parse("echo hello world"), make_echo("hello world"));
    }

    #[test]
    fn echo_normalizes_whitespace() {
        assert_eq!(parse("echo a    b\tc"), make_echo("a b c"));
    }

    #[test]
    fn echo_with_leading_whitespace() {
        assert_eq!(parse("   echo hi"), make_echo("hi"));
    }

    #[test]
    fn parses_type_of_builtins() {
        assert_eq!(
            parse("type echo"),
            Command::Type("echo".into(), Box::new(make_echo("")))
        );
        assert_eq!(
            parse("type exit"),
            Command::Type("exit".into(), Box::new(Command::Exit))
        );
        assert_eq!(
            parse("type type"),
            Command::Type("type".into(), Box::new(make_echo("type requires an argument")))
        );
    }

    #[test]
    fn parses_type_of_unknown() {
        assert_eq!(
            parse("type foo"),
            Command::Type("foo".into(), Box::new(make_unknown("foo")))
        );
    }

    #[test]
    fn type_without_argument() {
        assert_eq!(parse("type"), make_echo("type requires an argument"));
    }

    #[test]
    fn type_with_too_many_arguments() {
        assert_eq!(parse("type a b"), make_echo("too many arguments"));
    }

    #[test]
    fn parses_unknown_command() {
        assert_eq!(parse("foo"), make_unknown("foo"));
    }

    #[test]
    fn unknown_command_ignores_args() {
        assert_eq!(parse("foo bar baz"), make_unknown("foo"));
    }

    #[test]
    fn commands_are_case_sensitive() {
        assert_eq!(parse("ECHO hi"), make_unknown("ECHO"));
        assert_eq!(parse("Exit"), make_unknown("Exit"));
    }

    #[test]
    fn similar_names_are_not_builtins() {
        assert_eq!(parse("echoo"), make_unknown("echoo"));
        assert_eq!(parse("exits"), make_unknown("exits"));
    }

    // Known bug: parse unwraps the first token, so blank input panics.
    #[test]
    fn blank_input_does_not_panic() {
        for input in ["", "   ", "\t"] {
            assert!(
                std::panic::catch_unwind(|| parse(input)).is_ok(),
                "parse({input:?}) panicked"
            );
        }
    }
}
