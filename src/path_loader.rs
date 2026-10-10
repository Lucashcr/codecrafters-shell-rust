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

    for entry_result in dir {
        let Ok(entry) = entry_result else {
            continue;
        };
        let Ok(metadata) = std::fs::metadata(entry.path()) else {
            continue;
        };
        let Ok(filename) = entry.file_name().into_string() else {
            continue;
        };
        if is_executable(&metadata) {
            programs.insert(filename, entry.path());
        }
    }

    programs
}

pub fn load() -> HashMap<String, PathBuf> {
    let Some(path_var) = std::env::var_os("PATH") else {
        return HashMap::new();
    };

    let mut result = HashMap::new();

    for path in std::env::split_paths(&path_var) {
        for (filename, filepath) in read_entry(path) {
            result.entry(filename).or_insert(filepath);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::path::Path;

    /// Directory under the system temp dir, removed on drop.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir()
                .join(format!("codecrafters-shell-{}-{name}", std::process::id()));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }

        fn file(&self, name: &str, mode: u32) -> PathBuf {
            let path = self.0.join(name);
            fs::write(&path, "#!/bin/sh\n").unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(mode)).unwrap();
            path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn indexes_only_executable_files() {
        let dir = TempDir::new("exec-only");
        let tool = dir.file("tool", 0o755);
        dir.file("notes.txt", 0o644);

        let programs = read_entry(dir.path().to_path_buf());

        assert_eq!(programs.get("tool"), Some(&tool));
        assert!(!programs.contains_key("notes.txt"));
    }

    #[test]
    fn indexes_symlink_to_executable() {
        let targets = TempDir::new("symlink-targets");
        let dir = TempDir::new("symlink-exec");
        let target = targets.file("real", 0o755);
        symlink(&target, dir.path().join("alias")).unwrap();

        let programs = read_entry(dir.path().to_path_buf());

        assert_eq!(programs.get("alias"), Some(&dir.path().join("alias")));
    }

    #[test]
    fn skips_symlink_to_non_executable() {
        let targets = TempDir::new("symlink-plain-targets");
        let dir = TempDir::new("symlink-plain");
        let target = targets.file("data", 0o644);
        symlink(&target, dir.path().join("alias")).unwrap();

        let programs = read_entry(dir.path().to_path_buf());

        assert!(!programs.contains_key("alias"));
    }

    #[test]
    fn does_not_scan_subdirectories() {
        let dir = TempDir::new("no-recursion");
        dir.file("top", 0o755);
        let sub = dir.path().join("sub");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("nested"), "").unwrap();
        fs::set_permissions(sub.join("nested"), fs::Permissions::from_mode(0o755)).unwrap();

        let programs = read_entry(dir.path().to_path_buf());

        assert!(programs.contains_key("top"));
        assert!(!programs.contains_key("nested"));
        assert!(!programs.contains_key("sub"));
    }

    #[test]
    fn missing_directory_yields_no_programs() {
        let dir = TempDir::new("missing");

        assert!(read_entry(dir.path().join("does-not-exist")).is_empty());
    }
}
