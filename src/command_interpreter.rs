pub enum Command {
    Echo(String),
    Type(String, Box<Command>),
    Exit,
    Unknown(String),
}

impl Command {
    fn is_builtin(&self) -> bool {
        !matches!(self, Command::Unknown(_))
    }
}

pub fn interpret(command: &Command) -> bool {
    match command {
        Command::Exit => {
            return true;
        }
        Command::Echo(arg) => {
            println!("{arg}");
        }
        Command::Type(name, command) => {
            if command.is_builtin() {
                println!("{name} is a shell builtin");
            } else {
                println!("{name}: not found");
            }
        }
        Command::Unknown(cmd) => {
            println!("{cmd}: command not found");
        }
    }

    false
}
