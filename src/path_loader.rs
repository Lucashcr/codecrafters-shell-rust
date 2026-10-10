use std::collections::HashMap;
use std::io::{Stderr, Write};
use std::os::unix::fs::PermissionsExt;

fn is_executable(metadata: &std::fs::Metadata) -> bool {
    metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
}

fn read_entry(entry: &str) -> HashMap<String, String> {
    let Ok(dir) = std::fs::read_dir(entry) else {
        return HashMap::new();
    };

    let mut programs: HashMap<String, String> = HashMap::new();

    for file_result in dir {
        match file_result {
            Ok(entry) => {
                if let Ok(metadata) = std::fs::metadata(entry.path()) {
                    let Ok(filename) = entry.file_name().into_string() else {
                        continue;
                    };
                    let Ok(filepath) = entry.path().into_string() else {
                        continue;
                    };
                    if is_executable(&metadata) && !programs.contains_key(&filename) {
                        programs.insert(filename, filepath);
                    }
                }
            },
            Err(_) => continue
        }
    }

    programs
}

pub fn load(err: &mut Stderr) -> HashMap<String, String> {
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
