use crate::commands::dependency_update::update_dependencies;
use crate::commands::release_notes::generate_release_notes;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_dependency_update_function() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a package.json file
    fs::write(
        temp_path.join("package.json"),
        r#"{"name": "test", "dependencies": {"lodash": "^4.17.21"}}"#,
    )
    .unwrap();

    // Change to temp directory
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_path).unwrap();

    // Test the function directly instead of command
    update_dependencies();

    // Restore original directory - this must succeed
    if let Err(e) = std::env::set_current_dir(&original_dir) {
        eprintln!("Failed to restore directory: {}", e);
        // Try to change to a known good directory
        let _ = std::env::set_current_dir("/tmp");
    }

    // Test passed if no panic occurred during update_dependencies()
}

#[test]
fn test_release_notes_function() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Initialize a git repo
    Command::new("git")
        .args(&["init"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to init git repo");

    // Configure git user for this repo
    Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to set git user name");

    Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to set git user email");

    // Create a file and commit
    fs::write(temp_path.join("README.md"), "# Test Project").unwrap();

    Command::new("git")
        .args(&["add", "."])
        .current_dir(temp_path)
        .output()
        .expect("Failed to add files");

    Command::new("git")
        .args(&["commit", "-m", "feat: initial commit"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to commit");

    // Change to temp directory
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_path).unwrap();

    // Test the function directly
    generate_release_notes();

    // Restore original directory
    if let Err(e) = std::env::set_current_dir(&original_dir) {
        eprintln!("Failed to restore directory: {}", e);
        let _ = std::env::set_current_dir("/tmp");
    }
}

#[tokio::test]
async fn test_cargo_build_succeeds() {
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();
    
    // Create a minimal Cargo.toml for testing
    let cargo_toml_content = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;
    
    let cargo_toml_path = temp_path.join("Cargo.toml");
    std::fs::write(&cargo_toml_path, cargo_toml_content).unwrap();
    
    // Create src/lib.rs
    let src_dir = temp_path.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("lib.rs"), "// Empty lib file").unwrap();
    
    // Now test cargo check
    let output = std::process::Command::new("cargo")
        .arg("check")
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute cargo check");
    
    assert!(output.status.success(), 
        "Cargo check failed:\nstdout: {}\nstderr: {}", 
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

// #[test]
// fn test_project_structure() {
//     // Test that important files exist
//     assert!(std::path::Path::new("Cargo.toml").exists());
//     assert!(std::path::Path::new("src/main.rs").exists());
//     assert!(std::path::Path::new("src/commands/mod.rs").exists());
//     assert!(std::path::Path::new("src/utils/mod.rs").exists());
// }

#[test]
fn test_temp_directory_operations() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Test file creation
    let test_file = temp_path.join("test.txt");
    fs::write(&test_file, "Hello, World!").unwrap();
    assert!(test_file.exists());

    // Test file reading
    let content = fs::read_to_string(&test_file).unwrap();
    assert_eq!(content, "Hello, World!");

    // Temp directory will be automatically cleaned up
}

#[test]
fn test_multiple_project_files() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create src directory for Cargo.toml
    fs::create_dir_all(temp_path.join("src")).unwrap();

    // Create multiple project files with valid content
    fs::write(
        temp_path.join("package.json"),
        r#"{"name": "test", "version": "1.0.0"}"#,
    )
    .unwrap();

    // Create a valid Cargo.toml with proper structure
    fs::write(
        temp_path.join("Cargo.toml"),
        r#"[package]
name = "test"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "test"
path = "src/main.rs""#,
    )
    .unwrap();

    // Create a simple main.rs for the Cargo project
    fs::write(
        temp_path.join("src/main.rs"),
        r#"fn main() {
    println!("Hello, world!");
}"#,
    )
    .unwrap();

    fs::write(
        temp_path.join("requirements.txt"),
        "requests==2.28.0\npandas==1.5.0",
    )
    .unwrap();

    // Change to temp directory and test dependency update
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_path).unwrap();

    // This should not panic with multiple project types
    update_dependencies();

    // Safe restore
    if let Err(e) = std::env::set_current_dir(&original_dir) {
        eprintln!("Failed to restore directory: {}", e);
        let _ = std::env::set_current_dir("/tmp");
    }
}

// #[test]
// fn test_git_repo_with_multiple_commits() {
//     let temp_dir = TempDir::new().unwrap();
//     let temp_path = temp_dir.path();

//     // Initialize git repo
//     Command::new("git")
//         .args(&["init"])
//         .current_dir(temp_path)
//         .output()
//         .expect("Failed to init git repo");

//     // Configure git
//     Command::new("git")
//         .args(&["config", "user.name", "Test User"])
//         .current_dir(temp_path)
//         .output()
//         .unwrap();

//     Command::new("git")
//         .args(&["config", "user.email", "test@example.com"])
//         .current_dir(temp_path)
//         .output()
//         .unwrap();

//     // Create multiple commits
//     let commits = vec![
//         ("README.md", "# Project", "docs: add README"),
//         ("src/main.rs", "fn main() {}", "feat: add main function"),
//         (
//             "Cargo.toml",
//             "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
//             "chore: add Cargo.toml",
//         ),
//         ("src/lib.rs", "// Library code", "feat: add library"),
//     ];

//     for (filename, content, message) in commits {
//         let file_path = temp_path.join(filename);
//         if filename.contains('/') {
//             let parent_path = file_path.parent().unwrap();
//             fs::create_dir_all(parent_path).unwrap();
//         }

//         fs::write(&file_path, content).unwrap();

//         Command::new("git")
//             .args(&["add", filename])
//             .current_dir(temp_path)
//             .output()
//             .unwrap();

//         Command::new("git")
//             .args(&["commit", "-m", message])
//             .current_dir(temp_path)
//             .output()
//             .unwrap();
//     }

//     // Change to temp directory and test release notes
//     let original_dir = std::env::current_dir().unwrap();
//     std::env::set_current_dir(temp_path).unwrap();

//     generate_release_notes();

//     // Safe restore
//     if let Err(e) = std::env::set_current_dir(&original_dir) {
//         eprintln!("Failed to restore directory: {}", e);
//         let _ = std::env::set_current_dir("/tmp");
//     }

//     // Check if release notes file was created
//     let release_files: Vec<_> = fs::read_dir(temp_path)
//         .unwrap()
//         .filter_map(|entry| {
//             let entry = entry.ok()?;
//             let filename = entry.file_name().to_string_lossy().to_string();
//             if filename.starts_with("ReleaseNotes_") && filename.ends_with(".md") {
//                 Some(filename)
//             } else {
//                 None
//             }
//         })
//         .collect();

//     assert!(
//         !release_files.is_empty(),
//         "Release notes file should be created"
//     );
// }

#[test]
fn test_package_json_only() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create only package.json
    fs::write(
        temp_path.join("package.json"),
        r#"{"name": "test-project", "version": "1.0.0", "dependencies": {"express": "^4.18.0"}}"#,
    )
    .unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_path).unwrap();

    // Should handle package.json only without errors
    update_dependencies();

    // Safe restore
    if let Err(e) = std::env::set_current_dir(&original_dir) {
        eprintln!("Failed to restore directory: {}", e);
        let _ = std::env::set_current_dir("/tmp");
    }
}

#[test]
fn test_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_path).unwrap();

    // Should handle empty directory gracefully
    update_dependencies();

    // Safe restore
    if let Err(e) = std::env::set_current_dir(&original_dir) {
        eprintln!("Failed to restore directory: {}", e);
        let _ = std::env::set_current_dir("/tmp");
    }
}
