use std::collections::HashMap;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

fn is_executable(metadata: &std::fs::Metadata) -> bool {
    metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
}

fn read_entry(entry: PathBuf) -> HashMap<String, PathBuf> {
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
            }
            Err(_) => continue,
        }
    }

    programs
}

pub fn load() -> HashMap<String, PathBuf> {
    match std::env::var_os("PATH") {
        Some(path_var) => {
            let mut result = HashMap::new();

            for path in std::env::split_paths(&path_var) {
                for (filename, filepath) in read_entry(path) {
                    result.entry(filename).or_insert(filepath);
                }
            }

            result
        }
        None => HashMap::new(),
    }
}
