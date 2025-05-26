use std::fs;
use std::path::Path;
use std::io;

pub fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn read_file_to_string(path: &str) -> Result<String, io::Error> {
    fs::read_to_string(path)
}

pub fn write_string_to_file(path: &str, content: &str) -> Result<(), io::Error> {
    fs::write(path, content)
}

pub fn find_project_files() -> Vec<String> {
    let mut files = Vec::new();
    
    // Package.json (Node.js/npm/yarn/pnpm)
    if file_exists("package.json") {
        files.push("package.json".to_string());
    }
    
    // Cargo.toml (Rust)
    if file_exists("Cargo.toml") {
        files.push("Cargo.toml".to_string());
    }
    
    // requirements.txt (Python)
    if file_exists("requirements.txt") {
        files.push("requirements.txt".to_string());
    }
    
    // composer.json (PHP)
    if file_exists("composer.json") {
        files.push("composer.json".to_string());
    }
    
    files
}

pub fn detect_node_package_manager() -> Option<String> {
    // Check for lock files to determine package manager preference
    if file_exists("pnpm-lock.yaml") {
        Some("pnpm".to_string())
    } else if file_exists("yarn.lock") {
        Some("yarn".to_string())
    } else if file_exists("package-lock.json") {
        Some("npm".to_string())
    } else {
        None
    }
}