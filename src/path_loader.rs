use std::collections::HashMap;
use std::io::{Stderr, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

fn is_executable(metadata: &std::fs::Metadata) -> bool {
    metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
}

fn read_entry(entry: &str) -> HashMap<String, PathBuf> {
    let Ok(dir) = std::fs::read_dir(entry) else {
        return HashMap::new();
    };

    let mut programs = HashMap::new();

    for file_result in dir {
        match file_result {
            Ok(entry) => {
                if let Ok(metadata) = std::fs::metadata(entry.path()) {
                    let filename = entry.file_name();
                    let Some(filename) = filename.to_str() else {
                        continue;
                    };
                    if is_executable(&metadata) && !programs.contains_key(filename) {
                        programs.insert(String::from(filename), entry.path());
                    }
                }
            },
            Err(_) => continue
        }
    }

    programs
}

pub fn load(err: &mut Stderr) -> HashMap<String, PathBuf> {
    let Ok(path) = std::env::var("PATH") else {
        let _ = writeln!(err, "couldn't load path");
        return HashMap::new();
    };

    let Some(result) = path
        .split(":")
        .map(read_entry)
        .reduce(|a, b| b.into_iter().chain(a).collect())
    else {
        let _ = writeln!(err, "failed to index commands from path");
        return HashMap::new();
    };

    result
}
