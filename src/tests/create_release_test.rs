use crate::commands::create_release::{ReleaseManager, GitHubReleaseRequest, GitHubReleaseResponse};
use anyhow::Result;
use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio;

#[cfg(test)]
mod tests {
    use super::*;

    // Test helper to create a temporary git repository
    fn setup_test_repo() -> Result<(TempDir, PathBuf)> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().to_path_buf();

        // Initialize git repo
        std::process::Command::new("git")
            .args(&["init"])
            .current_dir(&repo_path)
            .output()?;

        // Set git config
        std::process::Command::new("git")
            .args(&["config", "user.email", "test@example.com"])
            .current_dir(&repo_path)
            .output()?;

        std::process::Command::new("git")
            .args(&["config", "user.name", "Test User"])
            .current_dir(&repo_path)
            .output()?;

        // Create a basic Cargo.toml
        let cargo_toml = r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;
        fs::write(repo_path.join("Cargo.toml"), cargo_toml)?;

        // Create basic src/main.rs
        fs::create_dir_all(repo_path.join("src"))?;
        fs::write(repo_path.join("src/main.rs"), "fn main() { println!(\"Hello\"); }")?;

        // Initial commit
        std::process::Command::new("git")
            .args(&["add", "."])
            .current_dir(&repo_path)
            .output()?;

        std::process::Command::new("git")
            .args(&["commit", "-m", "Initial commit"])
            .current_dir(&repo_path)
            .output()?;

        Ok((temp_dir, repo_path))
    }

    #[test]
    fn test_version_validation() {
        let (_temp_dir, repo_path) = setup_test_repo().unwrap();
        
        // Change to the test repo directory
        env::set_current_dir(&repo_path).unwrap();
        
        let manager = ReleaseManager::new().unwrap();

        // Valid versions
        assert!(manager.validate_version("v1.0.0").is_ok());
        assert!(manager.validate_version("v0.1.0").is_ok());
        assert!(manager.validate_version("v10.20.30").is_ok());
        assert!(manager.validate_version("v1.0.0-alpha").is_ok());
        assert!(manager.validate_version("v1.0.0-beta.1").is_ok());

        // Invalid versions
        assert!(manager.validate_version("1.0.0").is_err()); // Missing 'v'
        assert!(manager.validate_version("v1.0").is_err());  // Missing patch version
        assert!(manager.validate_version("v1").is_err());    // Too short
        assert!(manager.validate_version("").is_err());      // Empty
        assert!(manager.validate_version("invalid").is_err()); // Invalid format
    }

    #[test]
    fn test_cargo_version_update() {
        let (_temp_dir, repo_path) = setup_test_repo().unwrap();
        env::set_current_dir(&repo_path).unwrap();
        
        let manager = ReleaseManager::new().unwrap();

        // Update version
        assert!(manager.update_cargo_version("v1.2.3").is_ok());

        // Check if version was updated
        let cargo_content = fs::read_to_string("Cargo.toml").unwrap();
        assert!(cargo_content.contains("version = \"1.2.3\""));
    }

    #[test]
    fn test_tag_exists_check() {
        let (_temp_dir, repo_path) = setup_test_repo().unwrap();
        env::set_current_dir(&repo_path).unwrap();
        
        let manager = ReleaseManager::new().unwrap();

        // Tag doesn't exist - should pass
        assert!(manager.check_tag_exists("v1.0.0").is_ok());

        // Create a tag
        std::process::Command::new("git")
            .args(&["tag", "v1.0.0"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        // Tag exists - should fail
        assert!(manager.check_tag_exists("v1.0.0").is_err());
    }

    #[test]
    fn test_working_directory_clean() {
        let (_temp_dir, repo_path) = setup_test_repo().unwrap();
        env::set_current_dir(&repo_path).unwrap();
        
        let manager = ReleaseManager::new().unwrap();

        // Clean directory - should pass
        assert!(manager.check_working_directory_clean().is_ok());

        // Add untracked file
        fs::write("untracked.txt", "test").unwrap();

        // Dirty directory - should fail
        assert!(manager.check_working_directory_clean().is_err());
    }

    #[test]
    fn test_package_manager_detection() {
        let (_temp_dir, repo_path) = setup_test_repo().unwrap();
        env::set_current_dir(&repo_path).unwrap();
        
        let manager = ReleaseManager::new().unwrap();

        // No package.json - should return None
        assert!(manager.detect_package_manager().is_none());

        // Create package.json
        let package_json = r#"{"name": "test", "version": "1.0.0"}"#;
        fs::write("package.json", package_json).unwrap();

        // Should detect npm (default)
        assert_eq!(manager.detect_package_manager(), Some("npm".to_string()));

        // Create yarn.lock
        fs::write("yarn.lock", "").unwrap();
        assert_eq!(manager.detect_package_manager(), Some("yarn".to_string()));

        // Create pnpm-lock.yaml
        fs::write("pnpm-lock.yaml", "").unwrap();
        assert_eq!(manager.detect_package_manager(), Some("pnpm".to_string()));
    }

    #[test]
    fn test_package_script_detection() {
        let (_temp_dir, repo_path) = setup_test_repo().unwrap();
        env::set_current_dir(&repo_path).unwrap();
        
        let manager = ReleaseManager::new().unwrap();

        // No package.json
        assert!(!manager.has_package_script("test"));

        // Create package.json with scripts
        let package_json = r#"{
            "name": "test",
            "version": "1.0.0",
            "scripts": {
                "test": "jest",
                "build": "webpack",
                "lint": "eslint"
            }
        }"#;
        fs::write("package.json", package_json).unwrap();

        assert!(manager.has_package_script("test"));
        assert!(manager.has_package_script("build"));
        assert!(manager.has_package_script("lint"));
        assert!(!manager.has_package_script("nonexistent"));
    }

    #[test]
    fn test_github_release_request_serialization() {
        let request = GitHubReleaseRequest {
            tag_name: "v1.0.0".to_string(),
            target_commitish: "main".to_string(),
            name: "Release v1.0.0".to_string(),
            body: "Release notes".to_string(),
            draft: false,
            prerelease: false,
            generate_release_notes: true,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("v1.0.0"));
        assert!(json.contains("main"));
        assert!(json.contains("Release notes"));
    }

    #[test]
    fn test_github_release_response_deserialization() {
        let json = r#"{
            "id": 123,
            "html_url": "https://github.com/owner/repo/releases/tag/v1.0.0",
            "upload_url": "https://uploads.github.com/repos/owner/repo/releases/123/assets{?name,label}"
        }"#;

        let response: GitHubReleaseResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.id, 123);
        assert_eq!(response.html_url, "https://github.com/owner/repo/releases/tag/v1.0.0");
    }

    #[tokio::test]
    async fn test_create_release_with_invalid_version() {
        let (_temp_dir, repo_path) = setup_test_repo().unwrap();
        env::set_current_dir(&repo_path).unwrap();
        
        let manager = ReleaseManager::new().unwrap();

        let result = manager.create_release("invalid-version", None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid version format"));
    }

    #[tokio::test]
    async fn test_create_release_with_existing_tag() {
        let (_temp_dir, repo_path) = setup_test_repo().unwrap();
        env::set_current_dir(&repo_path).unwrap();
        
        // Create existing tag
        std::process::Command::new("git")
            .args(&["tag", "v1.0.0"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        let manager = ReleaseManager::new().unwrap();

        let result = manager.create_release("v1.0.0", None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[tokio::test]
    async fn test_create_release_with_dirty_working_directory() {
        let (_temp_dir, repo_path) = setup_test_repo().unwrap();
        env::set_current_dir(&repo_path).unwrap();
        
        // Create untracked file
        fs::write("dirty.txt", "test").unwrap();

        let manager = ReleaseManager::new().unwrap();

        let result = manager.create_release("v1.0.0", None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Working directory is not clean"));
    }

    #[test]
    fn test_run_cargo_command() {
        let (_temp_dir, repo_path) = setup_test_repo().unwrap();
        env::set_current_dir(&repo_path).unwrap();
        
        let manager = ReleaseManager::new().unwrap();

        // Valid command
        assert!(manager.run_cargo_command("check").is_ok());

        // Invalid command
        assert!(manager.run_cargo_command("invalid-command").is_err());
    }

    #[test]
    fn test_prerelease_detection() {
        let (_temp_dir, repo_path) = setup_test_repo().unwrap();
        env::set_current_dir(&repo_path).unwrap();
        
        let manager = ReleaseManager::new().unwrap();

        // Simulate the prerelease detection logic
        let version = "v1.0.0-alpha";
        let is_prerelease = version.contains("-alpha") || 
                           version.contains("-beta") || 
                           version.contains("-rc");
        assert!(is_prerelease);

        let version = "v1.0.0";
        let is_prerelease = version.contains("-alpha") || 
                           version.contains("-beta") || 
                           version.contains("-rc");
        assert!(!is_prerelease);
    }

    #[test]
    fn test_release_manager_creation_without_git() {
        let temp_dir = TempDir::new().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();

        // Should fail without git repository
        assert!(ReleaseManager::new().is_err());
    }

    #[test]
    fn test_release_manager_creation_with_git() {
        let (_temp_dir, repo_path) = setup_test_repo().unwrap();
        env::set_current_dir(&repo_path).unwrap();

        // Should succeed with git repository
        let manager = ReleaseManager::new();
        assert!(manager.is_ok());

        let manager = manager.unwrap();
        assert!(!manager.repo_path.to_string_lossy().is_empty());
        assert!(!manager.current_branch.is_empty());
    }

    // Integration test that requires GITHUB_TOKEN
    #[tokio::test]
    #[ignore] // Ignore by default since it requires real GitHub token
    async fn test_github_release_creation() {
        if env::var("GITHUB_TOKEN").is_err() {
            return; // Skip if no token
        }

        let (_temp_dir, repo_path) = setup_test_repo().unwrap();
        env::set_current_dir(&repo_path).unwrap();
        
        let manager = ReleaseManager::new().unwrap();

        // This would require a real repository and token
        // Only run manually with proper setup
        let result = manager.create_github_release("v0.0.1-test", "Test release").await;
        
        // In a real test environment, you'd want to use a test repository
        // For now, we just check that the function can be called
        println!("GitHub release test result: {:?}", result);
    }

    #[test]
    fn test_cargo_toml_version_regex() {
        let cargo_content = r#"[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#;

        let version_regex = regex::Regex::new(r#"version\s*=\s*"([^"]+)""#).unwrap();
        let captures = version_regex.captures(cargo_content).unwrap();
        assert_eq!(&captures[1], "0.1.0");
    }

    #[test]
    fn test_version_format_parsing() {
        // Test version parsing logic
        let version = "v1.2.3";
        let clean_version = version.strip_prefix('v').unwrap_or(version);
        assert_eq!(clean_version, "1.2.3");

        let version = "1.2.3";
        let clean_version = version.strip_prefix('v').unwrap_or(version);
        assert_eq!(clean_version, "1.2.3");
    }
}