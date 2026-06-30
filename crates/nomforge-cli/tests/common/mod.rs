use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

/// Path to the built nomforge binary.
fn nomforge_bin() -> PathBuf {
    // When running `cargo test` from workspace root, the binary is in target/debug/
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // Go up to workspace root
    path.pop(); // nomforge-cli
    path.pop(); // crates
    path.join("target/debug/nomforge")
}

/// Run the nomforge CLI with the given arguments.
///
/// Returns (exit code, stdout, stderr).
pub fn run_nomforge(args: &[&str]) -> (i32, String, String) {
    let output = Command::new(nomforge_bin())
        .args(args)
        .output()
        .expect("failed to execute nomforge");

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    (exit_code, stdout, stderr)
}

/// Create a temporary directory with test files.
pub fn create_test_dir(files: &[(&str, &str)]) -> TempDir {
    let tmp = TempDir::new().expect("failed to create temp dir");

    for (name, content) in files {
        let path = tmp.path().join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, content).unwrap();
    }

    tmp
}

/// Get sorted file names from a directory.
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

/// Read file content as string.
pub fn read_content(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}
