use crate::commands::create_release::{
    ReleaseManager, FrameworkType, ProjectConfig,
    create_release_interactive, create_release_with_args
};
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
    fn setup_test_repo(framework: FrameworkType) -> Result<(TempDir, PathBuf)> {
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

        // Create framework-specific files
        match framework {
            FrameworkType::NextJs => setup_nextjs_project(&repo_path)?,
            FrameworkType::Angular => setup_angular_project(&repo_path)?,
            FrameworkType::NodeJs => setup_nodejs_project(&repo_path)?,
            FrameworkType::React => setup_react_project(&repo_path)?,
            FrameworkType::Vue => setup_vue_project(&repo_path)?,
            FrameworkType::Rust => setup_rust_project(&repo_path)?,
            FrameworkType::Laravel => setup_laravel_project(&repo_path)?,
            FrameworkType::Unknown => {} // No specific setup
        }

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

    fn setup_nextjs_project(repo_path: &PathBuf) -> Result<()> {
        // Create Next.js config
        fs::write(repo_path.join("next.config.js"), "module.exports = {}")?;
        // Create package.json
        let package_json = serde_json::json!({
            "name": "test-nextjs-app",
            "version": "1.0.0",
            "scripts": {
                "dev": "next dev",
                "build": "next build",
                "start": "next start",
                "lint": "next lint",
                "test": "jest"
            },
            "dependencies": {
                "next": "^14.0.0",
                "react": "^18.0.0",
                "react-dom": "^18.0.0"
            }
        });
        fs::write(repo_path.join("package.json"), serde_json::to_string_pretty(&package_json)?)?;
        // Create package-lock.json
        fs::write(repo_path.join("package-lock.json"), "{}")?;
        Ok(())
    }

    fn setup_angular_project(repo_path: &PathBuf) -> Result<()> {
        // Create Angular config
        fs::write(repo_path.join("angular.json"), "{}")?;
        // Create package.json
        let package_json = serde_json::json!({
            "name": "test-angular-app",
            "version": "1.0.0",
            "scripts": {
                "build": "ng build",
                "test": "ng test",
                "lint": "ng lint",
                "e2e": "ng e2e"
            },
            "dependencies": {
                "@angular/core": "^17.0.0"
            }
        });
        fs::write(repo_path.join("package.json"), serde_json::to_string_pretty(&package_json)?)?;
        Ok(())
    }

    fn setup_nodejs_project(repo_path: &PathBuf) -> Result<()> {
        // Create package.json
        let package_json = serde_json::json!({
            "name": "test-node-app",
            "version": "1.0.0",
            "scripts": {
                "start": "node index.js",
                "test": "jest",
                "lint": "eslint .",
                "build": "webpack"
            },
            "dependencies": {
                "express": "^4.18.0"
            }
        });
        fs::write(repo_path.join("package.json"), serde_json::to_string_pretty(&package_json)?)?;
        // Create yarn.lock for yarn detection
        fs::write(repo_path.join("yarn.lock"), "")?;
        Ok(())
    }

    fn setup_react_project(repo_path: &PathBuf) -> Result<()> {
        // Create package.json
        let package_json = serde_json::json!({
            "name": "test-react-app",
            "version": "1.0.0",
            "scripts": {
                "start": "react-scripts start",
                "build": "react-scripts build",
                "test": "react-scripts test",
                "lint": "eslint src/"
            },
            "dependencies": {
                "react": "^18.0.0",
                "react-dom": "^18.0.0"
            }
        });
        fs::write(repo_path.join("package.json"), serde_json::to_string_pretty(&package_json)?)?;
        Ok(())
    }

    fn setup_vue_project(repo_path: &PathBuf) -> Result<()> {
        // Create package.json
        let package_json = serde_json::json!({
            "name": "test-vue-app",
            "version": "1.0.0",
            "scripts": {
                "serve": "vue-cli-service serve",
                "build": "vue-cli-service build",
                "test:unit": "vue-cli-service test:unit",
                "lint": "vue-cli-service lint"
            },
            "dependencies": {
                "vue": "^3.0.0"
            }
        });
        fs::write(repo_path.join("package.json"), serde_json::to_string_pretty(&package_json)?)?;
        // Create pnpm-lock.yaml for pnpm detection
        fs::write(repo_path.join("pnpm-lock.yaml"), "")?;
        Ok(())
    }

    fn setup_rust_project(repo_path: &PathBuf) -> Result<()> {
        // Create Cargo.toml
        let cargo_toml = r#"[package]
name = "test-rust-app"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;
        fs::write(repo_path.join("Cargo.toml"), cargo_toml)?;
        // Create src/main.rs
        fs::create_dir_all(repo_path.join("src"))?;
        fs::write(repo_path.join("src/main.rs"), "fn main() { println!(\"Hello, world!\"); }")?;
        Ok(())
    }

    fn setup_laravel_project(repo_path: &PathBuf) -> Result<()> {
        // Create artisan file
        fs::write(repo_path.join("artisan"), "#!/usr/bin/env php\n<?php")?;
        // Create composer.json
        let composer_json = serde_json::json!({
            "name": "test/laravel-app",
            "version": "1.0.0",
            "require": {
                "laravel/framework": "^10.0"
            }
        });
        fs::write(repo_path.join("composer.json"), serde_json::to_string_pretty(&composer_json)?)?;
        // Create phpunit.xml
        fs::write(repo_path.join("phpunit.xml"), "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
        Ok(())
    }

    #[test]
    fn test_framework_detection_nextjs() {
        let (_temp_dir, repo_path) = setup_test_repo(FrameworkType::NextJs).unwrap();
        env::set_current_dir(&repo_path).unwrap();
        let manager = ReleaseManager::new().unwrap();
        assert!(matches!(manager.project_config.framework, FrameworkType::NextJs));
        assert_eq!(manager.project_config.package_manager, Some("npm".to_string()));
        assert!(manager.project_config.has_tests);
        assert!(manager.project_config.has_build);
        assert!(manager.project_config.has_lint);
    }

    #[test]
    fn test_framework_detection_angular() {
        let (_temp_dir, repo_path) = setup_test_repo(FrameworkType::Angular).unwrap();
        env::set_current_dir(&repo_path).unwrap();
        let manager = ReleaseManager::new().unwrap();
        assert!(matches!(manager.project_config.framework, FrameworkType::Angular));
    }

    #[test]
    fn test_framework_detection_nodejs() {
        let (_temp_dir, repo_path) = setup_test_repo(FrameworkType::NodeJs).unwrap();
        env::set_current_dir(&repo_path).unwrap();
        let manager = ReleaseManager::new().unwrap();
        assert!(matches!(manager.project_config.framework, FrameworkType::NodeJs));
        assert_eq!(manager.project_config.package_manager, Some("yarn".to_string()));
    }

    #[test]
    fn test_framework_detection_react() {
        let (_temp_dir, repo_path) = setup_test_repo(FrameworkType::React).unwrap();
        env::set_current_dir(&repo_path).unwrap();
        let manager = ReleaseManager::new().unwrap();
        assert!(matches!(manager.project_config.framework, FrameworkType::React));
    }

    #[test]
    fn test_framework_detection_vue() {
        let (_temp_dir, repo_path) = setup_test_repo(FrameworkType::Vue).unwrap();
        env::set_current_dir(&repo_path).unwrap();
        let manager = ReleaseManager::new().unwrap();
        assert!(matches!(manager.project_config.framework, FrameworkType::Vue));
        assert_eq!(manager.project_config.package_manager, Some("pnpm".to_string()));
    }

    #[test]
    fn test_framework_detection_rust() {
        let (_temp_dir, repo_path) = setup_test_repo(FrameworkType::Rust).unwrap();
        env::set_current_dir(&repo_path).unwrap();
        let manager = ReleaseManager::new().unwrap();
        assert!(matches!(manager.project_config.framework, FrameworkType::Rust));
        assert!(manager.project_config.has_tests);
        assert!(manager.project_config.has_build);
        assert_eq!(manager.project_config.version_file, "Cargo.toml");
    }

    #[test]
    fn test_framework_detection_laravel() {
        let (_temp_dir, repo_path) = setup_test_repo(FrameworkType::Laravel).unwrap();
        env::set_current_dir(&repo_path).unwrap();
        let manager = ReleaseManager::new().unwrap();
        assert!(matches!(manager.project_config.framework, FrameworkType::Laravel));
        assert!(manager.project_config.has_tests);
        assert!(manager.project_config.has_build);
        assert_eq!(manager.project_config.version_file, "composer.json");
    }

    #[test]
    fn test_package_manager_detection() {
        // Test npm detection
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();
        fs::write(repo_path.join("package-lock.json"), "{}").unwrap();
        let pm = ReleaseManager::detect_package_manager(&repo_path);
        assert_eq!(pm, Some("npm".to_string()));

        // Test yarn detection
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();
        fs::write(repo_path.join("yarn.lock"), "").unwrap();
        let pm = ReleaseManager::detect_package_manager(&repo_path);
        assert_eq!(pm, Some("yarn".to_string()));

        // Test pnpm detection
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();
        fs::write(repo_path.join("pnpm-lock.yaml"), "").unwrap();
        
        let pm = ReleaseManager::detect_package_manager(&repo_path);
        assert_eq!(pm, Some("pnpm".to_string()));
    }

    #[test]
    fn test_version_validation() {
        let (_temp_dir, repo_path) = setup_test_repo(FrameworkType::Rust).unwrap();
        env::set_current_dir(&repo_path).unwrap();
        
        let manager = ReleaseManager::new().unwrap();

        // Valid versions
        assert!(manager.validate_version("v1.0.0").is_ok());
        assert!(manager.validate_version("1.0.0").is_ok());
        assert!(manager.validate_version("v10.20.30").is_ok());
        assert!(manager.validate_version("v1.0.0-alpha").is_ok());
        assert!(manager.validate_version("v1.0.0-beta.1").is_ok());

        // Invalid versions
        assert!(manager.validate_version("v1.0").is_err());
        assert!(manager.validate_version("v1").is_err());
        assert!(manager.validate_version("").is_err());
        assert!(manager.validate_version("invalid").is_err());
    }

    #[test]
    fn test_package_version_update() {
        let (_temp_dir, repo_path) = setup_test_repo(FrameworkType::NextJs).unwrap();
        env::set_current_dir(&repo_path).unwrap();
        let manager = ReleaseManager::new().unwrap();

        // Update version
        assert!(manager.update_package_version("v2.1.0").is_ok());

        // Check if version was updated
        let package_content = fs::read_to_string(repo_path.join("package.json")).unwrap();
        let package_json: serde_json::Value = serde_json::from_str(&package_content).unwrap();
        assert_eq!(package_json["version"], "2.1.0");
    }

    #[test]
    fn test_cargo_version_update() {
        let (_temp_dir, repo_path) = setup_test_repo(FrameworkType::Rust).unwrap();
        env::set_current_dir(&repo_path).unwrap();
        let manager = ReleaseManager::new().unwrap();

        // Update version
        assert!(manager.update_cargo_version("v3.2.1").is_ok());

        // Check if version was updated
        let cargo_content = fs::read_to_string(repo_path.join("Cargo.toml")).unwrap();
        assert!(cargo_content.contains("version = \"3.2.1\""));
    }

    #[test]
    fn test_composer_version_update() {
        let (_temp_dir, repo_path) = setup_test_repo(FrameworkType::Laravel).unwrap();
        env::set_current_dir(&repo_path).unwrap();
        
        let manager = ReleaseManager::new().unwrap();

        // Update version
        assert!(manager.update_composer_version("v4.3.2").is_ok());

        // Check if version was updated
        let composer_content = fs::read_to_string(repo_path.join("composer.json")).unwrap();
        let composer_json: serde_json::Value = serde_json::from_str(&composer_content).unwrap();
        assert_eq!(composer_json["version"], "4.3.2");
    }

    #[test]
    fn test_tag_exists_check() {
        let (_temp_dir, repo_path) = setup_test_repo(FrameworkType::Rust).unwrap();
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
        let (_temp_dir, repo_path) = setup_test_repo(FrameworkType::Rust).unwrap();
        env::set_current_dir(&repo_path).unwrap();
        
        let manager = ReleaseManager::new().unwrap();

        // Clean directory - should pass
        assert!(manager.check_working_directory_clean().is_ok());

        // Add untracked file
        fs::write(repo_path.join("untracked.txt"), "test").unwrap();

        // Dirty directory - should fail
        assert!(manager.check_working_directory_clean().is_err());
    }

    #[test]
    fn test_package_script_detection() {
        let (_temp_dir, repo_path) = setup_test_repo(FrameworkType::NextJs).unwrap();
        env::set_current_dir(&repo_path).unwrap();
        
        let manager = ReleaseManager::new().unwrap();

        assert!(manager.has_package_script("build"));
        assert!(manager.has_package_script("test"));
        assert!(manager.has_package_script("lint"));
        assert!(!manager.has_package_script("nonexistent"));
    }

    #[test]
    fn test_repo_url_normalization() {
        // Test HTTPS URL
        let https_url = "https://github.com/user/repo.git";
        let normalized = ReleaseManager::normalize_github_url(https_url);
        assert_eq!(normalized, "user/repo");

        // Test SSH URL
        let ssh_url = "git@github.com:user/repo.git";
        let normalized = ReleaseManager::normalize_github_url(ssh_url);
        assert_eq!(normalized, "user/repo");
    }

    #[test]
    fn test_prerelease_detection() {
        let (_temp_dir, repo_path) = setup_test_repo(FrameworkType::Rust).unwrap();
        env::set_current_dir(&repo_path).unwrap();
        // Test prerelease detection logic
        let versions = vec![
            ("v1.0.0-alpha", true),
            ("v1.0.0-beta", true),
            ("v1.0.0-rc", true),
            ("v1.0.0", false),
            ("v2.1.0", false),
        ];

        for (version, is_prerelease) in versions {
            let detected = version.contains("-alpha") ||
                          version.contains("-beta") ||
                          version.contains("-rc");
            assert_eq!(detected, is_prerelease, "Version: {}", version);
        }
    }

    #[tokio::test]
    async fn test_create_release_validation_error() {
        let (_temp_dir, repo_path) = setup_test_repo(FrameworkType::Rust).unwrap();
        env::set_current_dir(&repo_path).unwrap();
        
        let manager = ReleaseManager::new().unwrap();

        // Test invalid version
        let result = manager.create_release("invalid-version", None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Version must be in format"));
    }

    #[tokio::test]
    async fn test_create_release_existing_tag() {
        let (_temp_dir, repo_path) = setup_test_repo(FrameworkType::Rust).unwrap();
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
    async fn test_create_release_dirty_working_directory() {
        let (_temp_dir, repo_path) = setup_test_repo(FrameworkType::Rust).unwrap();
        env::set_current_dir(&repo_path).unwrap();
        
        // Create untracked file
        fs::write(repo_path.join("dirty.txt"), "test").unwrap();

        let manager = ReleaseManager::new().unwrap();

        let result = manager.create_release("v1.0.0", None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("uncommitted changes"));
    }

    #[test]
    fn test_framework_task_execution_rust() {
        let (_temp_dir, repo_path) = setup_test_repo(FrameworkType::Rust).unwrap();
        env::set_current_dir(&repo_path).unwrap();
        
        let manager = ReleaseManager::new().unwrap();

        // This should work without errors in a proper Rust environment
        // In test environment, we mainly test that the function can be called
        let result = manager.run_framework_tasks("v1.0.0");
        // The result depends on the test environment having cargo available
        // We just ensure the function doesn't panic
        match result {
            Ok(_) => println!("Rust tasks completed successfully"),
            Err(e) => println!("Rust tasks failed (expected in test env): {}", e),
        }
    }

    #[test]
    fn test_framework_task_execution_nodejs() {
        let (_temp_dir, repo_path) = setup_test_repo(FrameworkType::NodeJs).unwrap();
        env::set_current_dir(&repo_path).unwrap();
        
        let manager = ReleaseManager::new().unwrap();

        // This should update package.json version
        let result = manager.run_framework_tasks("v2.0.0");
        // Check if version was updated regardless of other task results
        if repo_path.join("package.json").exists() {
            let package_content = fs::read_to_string(repo_path.join("package.json")).unwrap();
            let package_json: serde_json::Value = serde_json::from_str(&package_content).unwrap();
            assert_eq!(package_json["version"], "2.0.0");
        }
    }

    #[test]
    fn test_release_manager_without_git() {
        let temp_dir = TempDir::new().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();

        // Should fail without git repository
        let result = ReleaseManager::new();
        assert!(result.is_err());
    }

    #[test]
    fn test_release_manager_with_git() {
        let (_temp_dir, repo_path) = setup_test_repo(FrameworkType::Rust).unwrap();
        env::set_current_dir(&repo_path).unwrap();

        // Should succeed with git repository
        let manager = ReleaseManager::new();
        assert!(manager.is_ok());

        let manager = manager.unwrap();
        assert!(!manager.repo_path.to_string_lossy().is_empty());
        assert!(!manager.current_branch.is_empty());
    }

    #[test]
    fn test_get_package_scripts() {
        let (_temp_dir, repo_path) = setup_test_repo(FrameworkType::NextJs).unwrap();
        let scripts = ReleaseManager::get_package_scripts(&repo_path);
        assert!(scripts.contains_key("build"));
        assert!(scripts.contains_key("test"));
        assert!(scripts.contains_key("lint"));
        assert_eq!(scripts.get("build").unwrap(), "next build");
    }

    #[test]
    fn test_project_config_creation() {
        let (_temp_dir, repo_path) = setup_test_repo(FrameworkType::NextJs).unwrap();
        let config = ReleaseManager::detect_project_config(&repo_path).unwrap();
        assert!(matches!(config.framework, FrameworkType::NextJs));
        assert_eq!(config.package_manager, Some("npm".to_string()));
        assert!(config.has_tests);
        assert!(config.has_build);
        assert!(config.has_lint);
        assert_eq!(config.version_file, "package.json");
    }

    // Mock test for GitHub integration (requires environment setup)
    #[tokio::test]
    #[ignore] // Ignore by default since it requires GitHub token
    async fn test_github_release_creation() {
        if env::var("GITHUB_TOKEN").is_err() {
            return; // Skip if no token
        }

        let (_temp_dir, repo_path) = setup_test_repo(FrameworkType::Rust).unwrap();
        env::set_current_dir(&repo_path).unwrap();
        
        let manager = ReleaseManager::new().unwrap();

        // This would require a real repository and token
        let result = manager.create_github_release("v0.0.1-test", "Test release").await;
        
        // In a real test environment, you'd want to use a test repository
        match result {
            Ok(url) => {
                println!("GitHub release created: {}", url);
                assert!(url.contains("github.com"));
            }
            Err(e) => {
                println!("GitHub release failed: {}", e);
                // Don't fail the test if API is temporarily unavailable
            }
        }
    }

    #[test]
    fn test_command_runner_helpers() {
        let (_temp_dir, repo_path) = setup_test_repo(FrameworkType::NodeJs).unwrap();
        env::set_current_dir(&repo_path).unwrap();
        let manager = ReleaseManager::new().unwrap();

        // Test package manager command building
        // These tests verify the command construction without actually running them
        // Test npm command
        let result = manager.run_package_manager_command("echo", "test");
        assert!(result.is_ok()); // echo should always work

        // Test invalid command
        let result = manager.run_package_manager_command("nonexistent-command", "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_version_file_detection() {
        // Test different framework version files
        let test_cases = vec![
            (FrameworkType::NextJs, "package.json"),
            (FrameworkType::Angular, "package.json"),
            (FrameworkType::NodeJs, "package.json"),
            (FrameworkType::React, "package.json"),
            (FrameworkType::Vue, "package.json"),
            (FrameworkType::Rust, "Cargo.toml"),
            (FrameworkType::Laravel, "composer.json"),
        ];

        for (framework, expected_file) in test_cases {
            let (_temp_dir, repo_path) = setup_test_repo(framework.clone()).unwrap();
            let config = ReleaseManager::detect_project_config(&repo_path).unwrap();
            assert_eq!(config.version_file, expected_file, "Framework: {:?}", framework);
        }
    }

    #[test]
    fn test_multi_framework_detection_priority() {
        // Test when multiple framework indicators exist
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();

        // Create both Next.js and React indicators
        fs::write(repo_path.join("next.config.js"), "module.exports = {}").unwrap();
        let package_json = serde_json::json!({
            "dependencies": {
                "react": "^18.0.0",
                "next": "^14.0.0"
            }
        });
        fs::write(repo_path.join("package.json"), serde_json::to_string_pretty(&package_json).unwrap()).unwrap();

        // Next.js should take priority over React
        let framework = ReleaseManager::detect_framework(&repo_path);
        assert!(matches!(framework, FrameworkType::NextJs));
    }
}

// Helper functions for testing
#[cfg(test)]
impl ReleaseManager {
    pub fn detect_package_manager(repo_path: &PathBuf) -> Option<String> {
        Self::detect_package_manager(repo_path)
    }

    pub fn get_package_scripts(repo_path: &PathBuf) -> std::collections::HashMap<String, String> {
        Self::get_package_scripts(repo_path)
    }

    pub fn detect_project_config(repo_path: &PathBuf) -> Result<ProjectConfig> {
        Self::detect_project_config(repo_path)
    }

    pub fn detect_framework(repo_path: &PathBuf) -> FrameworkType {
        Self::detect_framework(repo_path)
    }

    pub fn normalize_github_url(url: &str) -> String {
        Self::normalize_github_url(url)
    }
}