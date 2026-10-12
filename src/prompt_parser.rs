use std::path::PathBuf;

use crate::command::Command;
use crate::path_loader::Executables;

pub fn parse(prompt: &str, executables: &Executables) -> Command {
    let mut args = prompt.split_whitespace();
    let Some(command) = args.next() else {
        return Command::Nothing;
    };

    match command {
        "exit" => Command::Exit,
        "echo" => {
            let args: Vec<&str> = args.collect();
            Command::new_echo(&args.join(" "))
        }
        "pwd" => Command::Pwd,
        "cd" => match args.next() {
            Some(arg) => {
                if arg == "~" {
                    let Some(home_dir) = std::env::var_os("HOME") else {
                        return Command::Error("var HOME nor defined".into());
                    };
                    return Command::Cd(home_dir.into());
                }
                let path = PathBuf::from(arg);
                if !path.is_dir() {
                    return Command::Error(format!("cd: {arg}: No such file or directory"));
                }
                Command::Cd(path)
            }
            None => Command::new_error("cd requires an argument"),
        },
        "type" => match (args.next(), args.next()) {
            (Some(arg), None) => Command::new_type(arg, executables),
            (Some(_), Some(_)) => Command::new_error("too many arguments"),
            _ => Command::new_error("type requires an argument"),
        },
        _ => match executables.lookup(command) {
            Some(path) => {
                let args: Vec<String> = args.map(String::from).collect();
                Command::new_executable(command, path, args)
            }
            None => Command::new_unknown(command),
        },
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use super::*;
    use crate::command::CommandType;
    use crate::test_support::mock_executables;

    #[test]
    fn parses_exit() {
        assert_eq!(parse("exit", &mock_executables()), Command::Exit);
    }

    #[test]
    fn parses_echo_without_args() {
        assert_eq!(parse("echo", &mock_executables()), Command::new_echo(""));
    }

    #[test]
    fn parses_echo_single_arg() {
        assert_eq!(
            parse("echo hello", &mock_executables()),
            Command::new_echo("hello")
        );
    }

    #[test]
    fn parses_echo_multiple_args() {
        assert_eq!(
            parse("echo hello world", &mock_executables()),
            Command::new_echo("hello world")
        );
    }

    #[test]
    fn echo_normalizes_whitespace() {
        assert_eq!(
            parse("echo a    b\tc", &mock_executables()),
            Command::new_echo("a b c")
        );
    }

    #[test]
    fn echo_with_leading_whitespace() {
        assert_eq!(
            parse("   echo hi", &mock_executables()),
            Command::new_echo("hi")
        );
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
        assert_eq!(
            parse("type pwd", &mock_executables()),
            Command::Type("pwd".into(), CommandType::Builtin)
        );
    }

    #[test]
    fn parses_pwd() {
        assert_eq!(parse("pwd", &mock_executables()), Command::Pwd);
    }

    #[test]
    fn pwd_ignores_args() {
        assert_eq!(parse("pwd foo bar", &mock_executables()), Command::Pwd);
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
            Command::new_error("type requires an argument")
        );
    }

    #[test]
    fn type_with_too_many_arguments() {
        assert_eq!(
            parse("type a b", &mock_executables()),
            Command::new_error("too many arguments")
        );
    }

    #[test]
    fn grep_is_executable_file() {
        assert_eq!(
            parse("grep", &mock_executables()),
            Command::new_executable("grep", &PathBuf::from("/usr/bin/grep"), vec![]),
        );
    }

    #[test]
    fn ls_is_not_executable_file() {
        assert_eq!(parse("ls", &mock_executables()), Command::new_unknown("ls"));
    }

    #[test]
    fn parses_unknown_command() {
        assert_eq!(
            parse("foo", &mock_executables()),
            Command::new_unknown("foo")
        );
    }

    #[test]
    fn unknown_command_ignores_args() {
        assert_eq!(
            parse("foo bar baz", &mock_executables()),
            Command::new_unknown("foo")
        );
    }

    #[test]
    fn commands_are_case_sensitive() {
        assert_eq!(
            parse("ECHO hi", &mock_executables()),
            Command::new_unknown("ECHO")
        );
        assert_eq!(
            parse("Exit", &mock_executables()),
            Command::new_unknown("Exit")
        );
    }

    #[test]
    fn similar_names_are_not_builtins() {
        assert_eq!(
            parse("echoo", &mock_executables()),
            Command::new_unknown("echoo")
        );
        assert_eq!(
            parse("exits", &mock_executables()),
            Command::new_unknown("exits")
        );
    }

    fn executables_shadowing_builtins() -> Executables {
        Executables::from(HashMap::from([
            (String::from("echo"), PathBuf::from("/usr/bin/echo")),
            (String::from("exit"), PathBuf::from("/usr/bin/exit")),
            (String::from("type"), PathBuf::from("/usr/bin/type")),
            (String::from("pwd"), PathBuf::from("/usr/bin/pwd")),
        ]))
    }

    #[test]
    fn pwd_builtin_wins_over_executable_with_same_name() {
        assert_eq!(
            parse("pwd", &executables_shadowing_builtins()),
            Command::Pwd
        );
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
            Command::new_echo("hi")
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
