use crate::commands::code_quality::{
    CheckResult, CodeQualityConfig, CodeQualityManager, PackageManager, ProjectInfo, ProjectType,
    QualityCheck,
};
use serde_json::json;
use std::fs;
use tempfile::tempdir;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_quality_config_default() {
        let config = CodeQualityConfig::default();

        assert_eq!(config.enabled_checks.len(), 4);
        assert!(config.enabled_checks.contains(&"lint".to_string()));
        assert!(config.enabled_checks.contains(&"format".to_string()));
        assert!(config.enabled_checks.contains(&"security".to_string()));
        assert!(config.enabled_checks.contains(&"test".to_string()));
        assert_eq!(config.skip_dependencies, false);
        assert_eq!(config.max_parallel_jobs, 4);
        assert_eq!(config.timeout_seconds, 300);
    }

    #[test]
    fn test_code_quality_config_custom() {
        let config = CodeQualityConfig {
            enabled_checks: vec!["lint".to_string(), "test".to_string()],
            skip_dependencies: true,
            max_parallel_jobs: 8,
            timeout_seconds: 600,
        };

        assert_eq!(config.enabled_checks.len(), 2);
        assert!(config.skip_dependencies);
        assert_eq!(config.max_parallel_jobs, 8);
        assert_eq!(config.timeout_seconds, 600);
    }

    #[test]
    fn test_project_type_enum() {
        let project_types = vec![
            ProjectType::NextJs,
            ProjectType::Angular,
            ProjectType::React,
            ProjectType::Vue,
            ProjectType::NodeJs,
            ProjectType::TypeScript,
            ProjectType::JavaScript,
            ProjectType::Rust,
            ProjectType::Python,
            ProjectType::Unknown,
        ];

        // Test that all variants can be created and compared
        for project_type in project_types {
            assert!(project_type == project_type.clone());
        }
    }

    #[test]
    fn test_package_manager_enum() {
        let package_managers = vec![
            PackageManager::Npm,
            PackageManager::Yarn,
            PackageManager::Pnpm,
            PackageManager::Bun,
            PackageManager::Cargo,
            PackageManager::Pip,
            PackageManager::Unknown,
        ];

        for pm in package_managers {
            assert!(pm == pm.clone());
        }
    }

    #[tokio::test]
    async fn test_detect_nextjs_project() {
        let temp_dir = tempdir().unwrap();
        let project_path = temp_dir.path();

        // Create Next.js project files
        let package_json = json!({
            "name": "test-nextjs",
            "dependencies": {
                "next": "13.0.0",
                "react": "18.0.0"
            },
            "scripts": {
                "dev": "next dev",
                "build": "next build"
            }
        });

        fs::write(project_path.join("package.json"), package_json.to_string()).unwrap();
        fs::write(project_path.join("next.config.js"), "module.exports = {}").unwrap();
        fs::write(project_path.join("package-lock.json"), "{}").unwrap();

        let config = CodeQualityConfig::default();
        let manager = CodeQualityManager::new(config);

        let project_info = manager.detect_project_type(project_path).await.unwrap();

        assert_eq!(project_info.project_type, ProjectType::NextJs);
        assert_eq!(project_info.package_manager, PackageManager::Npm);
        assert!(project_info.frameworks.contains(&"Next.js".to_string()));
    }

    #[tokio::test]
    async fn test_detect_angular_project() {
        let temp_dir = tempdir().unwrap();
        let project_path = temp_dir.path();

        // Create Angular project files
        let package_json = json!({
            "name": "test-angular",
            "dependencies": {
                "@angular/core": "15.0.0",
                "@angular/common": "15.0.0"
            },
            "scripts": {
                "ng": "ng",
                "start": "ng serve"
            }
        });

        fs::write(project_path.join("package.json"), package_json.to_string()).unwrap();
        fs::write(project_path.join("angular.json"), "{}").unwrap();
        fs::write(project_path.join("yarn.lock"), "").unwrap();

        let config = CodeQualityConfig::default();
        let manager = CodeQualityManager::new(config);

        let project_info = manager.detect_project_type(project_path).await.unwrap();

        assert_eq!(project_info.project_type, ProjectType::Angular);
        assert_eq!(project_info.package_manager, PackageManager::Yarn);
        assert!(project_info.frameworks.contains(&"Angular".to_string()));
    }

    #[tokio::test]
    async fn test_detect_rust_project() {
        let temp_dir = tempdir().unwrap();
        let project_path = temp_dir.path();

        // Create Rust project files
        let cargo_toml = r#"
[package]
name = "test-rust"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = "1.0"
serde = "1.0"
"#;

        fs::write(project_path.join("Cargo.toml"), cargo_toml).unwrap();
        fs::create_dir_all(project_path.join("src")).unwrap();
        fs::write(project_path.join("src/main.rs"), "fn main() {}").unwrap();

        let config = CodeQualityConfig::default();
        let manager = CodeQualityManager::new(config);

        let project_info = manager.detect_project_type(project_path).await.unwrap();

        assert_eq!(project_info.project_type, ProjectType::Rust);
        assert_eq!(project_info.package_manager, PackageManager::Cargo);
        assert!(project_info.frameworks.contains(&"Rust".to_string()));
    }

    #[tokio::test]
    async fn test_detect_python_project() {
        let temp_dir = tempdir().unwrap();
        let project_path = temp_dir.path();

        // Create Python project files
        fs::write(
            project_path.join("requirements.txt"),
            "flask==2.0.0\nrequests==2.25.0",
        )
        .unwrap();
        fs::write(project_path.join("main.py"), "print('Hello World')").unwrap();
        fs::write(
            project_path.join("setup.py"),
            "from setuptools import setup",
        )
        .unwrap();

        let config = CodeQualityConfig::default();
        let manager = CodeQualityManager::new(config);

        let project_info = manager.detect_project_type(project_path).await.unwrap();

        assert_eq!(project_info.project_type, ProjectType::Python);
        assert_eq!(project_info.package_manager, PackageManager::Pip);
        assert!(project_info.frameworks.contains(&"Python".to_string()));
    }

    #[tokio::test]
    async fn test_detect_typescript_project() {
        let temp_dir = tempdir().unwrap();
        let project_path = temp_dir.path();

        // Create TypeScript project files
        let package_json = json!({
            "name": "test-typescript",
            "devDependencies": {
                "typescript": "4.9.0",
                "@types/node": "18.0.0"
            },
            "scripts": {
                "build": "tsc",
                "dev": "ts-node src/index.ts"
            }
        });

        fs::write(project_path.join("package.json"), package_json.to_string()).unwrap();
        fs::write(project_path.join("tsconfig.json"), "{}").unwrap();
        fs::write(project_path.join("pnpm-lock.yaml"), "").unwrap();

        let config = CodeQualityConfig::default();
        let manager = CodeQualityManager::new(config);

        let project_info = manager.detect_project_type(project_path).await.unwrap();

        assert_eq!(project_info.project_type, ProjectType::TypeScript);
        assert_eq!(project_info.package_manager, PackageManager::Pnpm);
        assert!(project_info.has_typescript);
    }

    #[tokio::test]
    async fn test_detect_react_project() {
        let temp_dir = tempdir().unwrap();
        let project_path = temp_dir.path();

        // Create React project files
        let package_json = json!({
            "name": "test-react",
            "dependencies": {
                "react": "18.2.0",
                "react-dom": "18.2.0"
            },
            "scripts": {
                "start": "react-scripts start",
                "build": "react-scripts build"
            }
        });

        fs::write(project_path.join("package.json"), package_json.to_string()).unwrap();
        fs::write(project_path.join("bun.lockb"), "").unwrap();

        let config = CodeQualityConfig::default();
        let manager = CodeQualityManager::new(config);

        let project_info = manager.detect_project_type(project_path).await.unwrap();

        assert_eq!(project_info.project_type, ProjectType::React);
        assert_eq!(project_info.package_manager, PackageManager::Bun);
        assert!(project_info.frameworks.contains(&"React".to_string()));
    }

    #[tokio::test]
    async fn test_detect_vue_project() {
        let temp_dir = tempdir().unwrap();
        let project_path = temp_dir.path();

        // Create Vue project files
        let package_json = json!({
            "name": "test-vue",
            "dependencies": {
                "vue": "3.3.0"
            },
            "devDependencies": {
                "@vue/cli": "5.0.0"
            },
            "scripts": {
                "serve": "vue-cli-service serve",
                "build": "vue-cli-service build"
            }
        });

        fs::write(project_path.join("package.json"), package_json.to_string()).unwrap();

        let config = CodeQualityConfig::default();
        let manager = CodeQualityManager::new(config);

        let project_info = manager.detect_project_type(project_path).await.unwrap();

        assert_eq!(project_info.project_type, ProjectType::Vue);
        assert!(project_info.frameworks.contains(&"Vue".to_string()));
    }

    #[test]
    fn test_package_manager_command_generation() {
        let config = CodeQualityConfig::default();
        let manager = CodeQualityManager::new(config);

        assert_eq!(
            manager.get_package_manager_command(&PackageManager::Npm),
            "npm"
        );
        assert_eq!(
            manager.get_package_manager_command(&PackageManager::Yarn),
            "yarn"
        );
        assert_eq!(
            manager.get_package_manager_command(&PackageManager::Pnpm),
            "pnpm"
        );
        assert_eq!(
            manager.get_package_manager_command(&PackageManager::Bun),
            "bun"
        );
        assert_eq!(
            manager.get_package_manager_command(&PackageManager::Cargo),
            "cargo"
        );
        assert_eq!(
            manager.get_package_manager_command(&PackageManager::Pip),
            "pip"
        );
        assert_eq!(
            manager.get_package_manager_command(&PackageManager::Unknown),
            "npm"
        );
    }

    #[test]
    fn test_quality_check_creation() {
        let temp_dir = tempdir().unwrap();
        let check = QualityCheck {
            name: "test-lint".to_string(),
            command: "eslint".to_string(),
            args: vec![
                "--ext".to_string(),
                ".js,.ts".to_string(),
                "src/".to_string(),
            ],
            working_dir: temp_dir.path().to_path_buf(),
            timeout: 300,
        };

        assert_eq!(check.name, "test-lint");
        assert_eq!(check.command, "eslint");
        assert_eq!(check.args.len(), 3);
        assert_eq!(check.timeout, 300);
    }

    #[test]
    fn test_check_result_creation() {
        let result = CheckResult {
            check_name: "lint".to_string(),
            success: true,
            output: "All checks passed".to_string(),
            error: None,
            duration_ms: 1500,
        };

        assert_eq!(result.check_name, "lint");
        assert!(result.success);
        assert_eq!(result.output, "All checks passed");
        assert!(result.error.is_none());
        assert_eq!(result.duration_ms, 1500);

        let failed_result = CheckResult {
            check_name: "format".to_string(),
            success: false,
            output: "".to_string(),
            error: Some("Formatting issues found".to_string()),
            duration_ms: 800,
        };

        assert!(!failed_result.success);
        assert!(failed_result.error.is_some());
    }

    #[tokio::test]
    async fn test_config_file_detection() {
        let temp_dir = tempdir().unwrap();
        let project_path = temp_dir.path();

        // Create various config files
        let config_files = [
            ".eslintrc.js",
            ".prettierrc.json",
            "tsconfig.json",
            "jest.config.js",
            "webpack.config.js",
            "vite.config.ts", // Bu dosyayı değiştirdik
            "tailwind.config.js",
            ".gitignore",
            "Dockerfile",
        ];

        for file in &config_files {
            fs::write(project_path.join(file), "{}").unwrap();
        }

        let config = CodeQualityConfig::default();
        let manager = CodeQualityManager::new(config);

        let found_configs = manager.find_config_files(project_path).await.unwrap();

        // Debug output
        println!("Expected files: {:?}", config_files);
        println!(
            "Found files: {:?}",
            found_configs
                .iter()
                .map(|p| p.file_name().unwrap().to_string_lossy())
                .collect::<Vec<_>>()
        );

        assert_eq!(
            found_configs.len(),
            config_files.len(),
            "Expected {} config files, found {}",
            config_files.len(),
            found_configs.len()
        );

        for expected_file in &config_files {
            assert!(
                found_configs
                    .iter()
                    .any(|path| path.file_name().unwrap().to_string_lossy() == *expected_file),
                "Config file {} not found in {:?}",
                expected_file,
                found_configs
                    .iter()
                    .map(|p| p.file_name().unwrap().to_string_lossy())
                    .collect::<Vec<_>>()
            );
        }
    }

    #[tokio::test]
    async fn test_frontend_checks_generation() {
        let temp_dir = tempdir().unwrap();
        let project_info = ProjectInfo {
            project_type: ProjectType::NextJs,
            package_manager: PackageManager::Npm,
            root_path: temp_dir.path().to_path_buf(),
            config_files: Vec::new(),
            has_typescript: true,
            frameworks: vec!["Next.js".to_string()],
        };

        let config = CodeQualityConfig::default();
        let manager = CodeQualityManager::new(config);

        let checks = manager.generate_frontend_checks(&project_info).unwrap();

        assert!(!checks.is_empty());
        assert!(checks.iter().any(|c| c.name == "lint"));
        assert!(checks.iter().any(|c| c.name == "test"));
        assert!(checks.iter().any(|c| c.name == "security"));

        // Should have typecheck for TypeScript projects
        assert!(checks.iter().any(|c| c.name == "typecheck"));
    }

    #[tokio::test]
    async fn test_rust_checks_generation() {
        let temp_dir = tempdir().unwrap();
        let project_info = ProjectInfo {
            project_type: ProjectType::Rust,
            package_manager: PackageManager::Cargo,
            root_path: temp_dir.path().to_path_buf(),
            config_files: Vec::new(),
            has_typescript: false,
            frameworks: vec!["Rust".to_string()],
        };

        let config = CodeQualityConfig::default();
        let manager = CodeQualityManager::new(config);

        let checks = manager.generate_rust_checks(&project_info).unwrap();

        assert!(!checks.is_empty());
        assert!(checks.iter().any(|c| c.name == "format"));
        assert!(checks.iter().any(|c| c.name == "lint"));
        assert!(checks.iter().any(|c| c.name == "test"));

        // Check that cargo commands are used
        for check in &checks {
            assert_eq!(check.command, "cargo");
        }
    }

    #[tokio::test]
    async fn test_python_checks_generation() {
        let temp_dir = tempdir().unwrap();
        let project_info = ProjectInfo {
            project_type: ProjectType::Python,
            package_manager: PackageManager::Pip,
            root_path: temp_dir.path().to_path_buf(),
            config_files: Vec::new(),
            has_typescript: false,
            frameworks: vec!["Python".to_string()],
        };

        let config = CodeQualityConfig::default();
        let manager = CodeQualityManager::new(config);

        let checks = manager.generate_python_checks(&project_info).unwrap();

        assert!(!checks.is_empty());
        assert!(checks.iter().any(|c| c.name == "lint"));
        assert!(checks.iter().any(|c| c.name == "format"));
        assert!(checks.iter().any(|c| c.name == "test"));
        assert!(checks.iter().any(|c| c.name == "security"));
    }

    #[tokio::test]
    async fn test_basic_checks_generation() {
        let temp_dir = tempdir().unwrap();
        let project_info = ProjectInfo {
            project_type: ProjectType::Unknown,
            package_manager: PackageManager::Unknown,
            root_path: temp_dir.path().to_path_buf(),
            config_files: Vec::new(),
            has_typescript: false,
            frameworks: Vec::new(),
        };

        let config = CodeQualityConfig::default();
        let manager = CodeQualityManager::new(config);

        let checks = manager.generate_basic_checks(&project_info).unwrap();

        assert!(!checks.is_empty());
        assert!(checks.iter().any(|c| c.name == "validate"));
    }

    #[test]
    fn test_enabled_checks_filtering() {
        let mut config = CodeQualityConfig::default();
        config.enabled_checks = vec!["lint".to_string(), "test".to_string()];

        assert!(config.enabled_checks.contains(&"lint".to_string()));
        assert!(config.enabled_checks.contains(&"test".to_string()));
        assert!(!config.enabled_checks.contains(&"format".to_string()));
        assert!(!config.enabled_checks.contains(&"security".to_string()));
    }

    #[tokio::test]
    async fn test_run_quality_checks_mock() {
        let temp_dir = tempdir().unwrap();
        let project_path = temp_dir.path();

        // Create a simple project structure
        fs::write(
            project_path.join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )
        .unwrap();

        let config = CodeQualityConfig {
            enabled_checks: vec!["validate".to_string()], // Only basic validation
            skip_dependencies: true,
            max_parallel_jobs: 1,
            timeout_seconds: 10,
        };

        let manager = CodeQualityManager::new(config);

        // This should not fail for basic validation
        let result = manager.run_quality_checks(project_path).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_project_info_creation() {
        let temp_dir = tempdir().unwrap();
        let project_info = ProjectInfo {
            project_type: ProjectType::NextJs,
            package_manager: PackageManager::Yarn,
            root_path: temp_dir.path().to_path_buf(),
            config_files: vec![temp_dir.path().join("tsconfig.json")],
            has_typescript: true,
            frameworks: vec!["Next.js".to_string(), "React".to_string()],
        };

        assert_eq!(project_info.project_type, ProjectType::NextJs);
        assert_eq!(project_info.package_manager, PackageManager::Yarn);
        assert!(project_info.has_typescript);
        assert_eq!(project_info.frameworks.len(), 2);
        assert_eq!(project_info.config_files.len(), 1);
    }

    #[test]
    fn test_timeout_configuration() {
        let config = CodeQualityConfig {
            enabled_checks: vec!["lint".to_string()],
            skip_dependencies: false,
            max_parallel_jobs: 2,
            timeout_seconds: 60,
        };

        assert_eq!(config.timeout_seconds, 60);

        let check = QualityCheck {
            name: "lint".to_string(),
            command: "eslint".to_string(),
            args: vec![".".to_string()],
            working_dir: std::env::temp_dir(),
            timeout: config.timeout_seconds,
        };

        assert_eq!(check.timeout, 60);
    }

    #[test]
    fn test_config_serialization() {
        let config = CodeQualityConfig::default();

        // Test that config can be serialized to JSON
        let json_result = serde_json::to_string(&config);
        assert!(json_result.is_ok());

        let json_str = json_result.unwrap();
        assert!(json_str.contains("enabled_checks"));
        assert!(json_str.contains("skip_dependencies"));
        assert!(json_str.contains("max_parallel_jobs"));
        assert!(json_str.contains("timeout_seconds"));

        // Test deserialization
        let deserialized: Result<CodeQualityConfig, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());

        let config_back = deserialized.unwrap();
        assert_eq!(config_back.enabled_checks, config.enabled_checks);
        assert_eq!(config_back.skip_dependencies, config.skip_dependencies);
    }
}
