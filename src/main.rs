use std::io;

use codecrafters_shell::path_loader;
use codecrafters_shell::repl;

fn main() -> io::Result<()> {
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();

    let executables = path_loader::load();

    loop {
        if repl(&mut stdin, &mut stdout, &mut stderr, &executables)? {
            break;
        }
    }

    Ok(())
}

