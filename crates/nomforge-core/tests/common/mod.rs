use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

/// Create a temporary directory with test files.
///
/// Returns the TempDir (which auto-cleans on drop) and a sorted list of file paths.
pub fn create_test_dir(files: &[(&str, &str)]) -> (TempDir, Vec<PathBuf>) {
    let tmp = TempDir::new().expect("failed to create temp dir");
    let mut paths = Vec::new();

    for (name, content) in files {
        let path = tmp.path().join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, content).unwrap();
        paths.push(path);
    }

    paths.sort();
    (tmp, paths)
}

/// Create a temporary directory with a specific structure of subdirectories and files.
pub fn create_test_dir_nested(structure: &[(&str, &str)]) -> TempDir {
    let tmp = TempDir::new().expect("failed to create temp dir");

    for (name, content) in structure {
        let path = tmp.path().join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, content).unwrap();
    }

    tmp
}

/// Get sorted file names from a directory (non-recursive).
pub fn file_names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    names.sort();
    names
}

/// Get sorted file names recursively.
pub fn file_names_recursive(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    visit_dirs(dir, dir, &mut names);
    names.sort();
    names
}

fn visit_dirs(base: &Path, dir: &Path, names: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let rel = path.strip_prefix(base).unwrap_or(&path);
                names.push(rel.to_string_lossy().into_owned());
            } else if path.is_dir() {
                visit_dirs(base, &path, names);
            }
        }
    }
}

/// Read file content as string.
pub fn read_content(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}
