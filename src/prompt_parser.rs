use crate::command_interpreter::Command;

pub fn parse(prompt: &str) -> Command {
    let mut args = prompt.split_whitespace();
    let command = args.next().unwrap();

    if command == "exit" {
        return Command::Exit;
    }

    if command == "echo" {
        let x: Vec<&str> = args.collect();
        return Command::Echo(x.join(" "));
    }

    if command == "type" {
        match (args.next(), args.next()) {
            (Some(arg), None) => return Command::Type(arg.into(), Box::new(parse(arg))),
            (Some(_), Some(_)) => return Command::Echo("too many arguments".into()),
            _ => return Command::Echo("type requires an argument".into()),
        }
    }

    Command::Unknown(command.into())
}
