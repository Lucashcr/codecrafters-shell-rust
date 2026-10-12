use std::{collections::HashMap, path::PathBuf};

use crate::path_loader::Executables;

pub fn mock_executables() -> Executables {
    Executables::from(HashMap::from([(
        String::from("grep"),
        PathBuf::from("/usr/bin/grep"),
    )]))
}
