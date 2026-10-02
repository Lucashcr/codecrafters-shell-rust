use std::collections::HashMap;
use std::io::{Stderr, Write};
use std::os::unix::fs::PermissionsExt;

fn read_entry(entry: &str) -> HashMap<String, String> {
    let Ok(dir) = std::fs::read_dir(entry) else {
        println!("failed to read dir: {entry}");
        return HashMap::new();
    };

    let mut programs: HashMap<String, String> = HashMap::new();
    for file_result in dir {
        let Ok(file) = file_result else {
            continue;
        };

        if let Ok(metadata) = file.metadata() {
            let filepath = file.path();
            let Some(p) = filepath.to_str() else {
                continue;
            };

            if p.ends_with("/X11") {
                continue;
            }

            if metadata.is_dir() {
                programs.extend(read_entry(p));
                continue;
            } else if metadata.is_file() && metadata.permissions().mode() & 0o111 != 0 {
                let Ok(name) = file.file_name().into_string() else {
                    continue;
                };
                programs.insert(name, String::from(p));
                continue;
            } else if metadata.is_symlink() {
                if let Ok(target_metadata) = std::fs::metadata(p) {
                    if target_metadata.is_dir() {
                        programs.extend(read_entry(p));
                        continue;
                    } else if target_metadata.is_file() && metadata.permissions().mode() & 0o111 != 0 {
                        let Ok(name) = file.file_name().into_string() else {
                            continue;
                        };
                        programs.insert(name, String::from(p));
                        continue;
                    }
                }

                continue;
            }
        }
    }

    programs
}

pub fn load(err: &mut Stderr) -> HashMap<String, String> {
    let Ok(path) = std::env::var("PATH") else {
        writeln!(err, "couldn't load path").unwrap();
        return HashMap::new();
    };

    let Some(result) = path
        .split(":")
        .map(read_entry)
        .reduce(|a, b| a.into_iter().chain(b).collect())
    else {
        writeln!(err, "failed to index commands from path").unwrap();
        return HashMap::new();
    };

    result
}
