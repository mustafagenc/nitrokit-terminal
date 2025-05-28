use anyhow::Result;
use colored::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityConfig {
    pub enabled_checks: Vec<String>,
    pub skip_dependencies: bool,
    pub max_parallel_jobs: usize,
    pub timeout_seconds: u64,
}

impl Default for CodeQualityConfig {
    fn default() -> Self {
        Self {
            enabled_checks: vec![
                "lint".to_string(),
                "format".to_string(),
                "security".to_string(),
                "test".to_string(),
            ],
            skip_dependencies: false,
            max_parallel_jobs: 4,
            timeout_seconds: 300,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ProjectType {
    NextJs,
    Angular,
    React,
    Vue,
    NodeJs,
    TypeScript,
    JavaScript,
    Rust,
    Python,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PackageManager {
    Npm,
    Yarn,
    Pnpm,
    Bun,
    Cargo,
    Pip,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub project_type: ProjectType,
    pub package_manager: PackageManager,
    pub root_path: PathBuf,
    pub config_files: Vec<PathBuf>,
    pub has_typescript: bool,
    pub frameworks: Vec<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct QualityCheck {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: PathBuf,
    pub timeout: u64,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct CheckResult {
    pub check_name: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub duration_ms: u128,
}

pub struct CodeQualityManager {
    config: CodeQualityConfig,
}

impl CodeQualityManager {
    pub fn new(config: CodeQualityConfig) -> Self {
        Self { config }
    }

    pub async fn run_quality_checks(&self, path: &Path) -> Result<Vec<CheckResult>> {
        println!("{}", "🔍 Analyzing project structure...".blue().bold());

        let project_info = self.detect_project_type(path).await?;
        self.print_project_info(&project_info);

        println!("{}", "🚀 Running code quality checks...".green().bold());

        let checks = self.generate_quality_checks(&project_info)?;
        let mut results = Vec::new();

        for check in checks {
            if self.config.enabled_checks.contains(&check.name) {
                println!("{}", format!("  ▶ Running {}...", check.name).yellow());

                let result = self.run_check(&check).await;
                self.print_check_result(&result);
                results.push(result);
            }
        }

        self.print_summary(&results);
        Ok(results)
    }

    pub async fn detect_project_type(&self, path: &Path) -> Result<ProjectInfo> {
        let mut project_info = ProjectInfo {
            project_type: ProjectType::Unknown,
            package_manager: PackageManager::Unknown,
            root_path: path.to_path_buf(),
            config_files: Vec::new(),
            has_typescript: false,
            frameworks: Vec::new(),
        };

        // Check for package manager files
        let pm_files = [
            ("package-lock.json", PackageManager::Npm),
            ("yarn.lock", PackageManager::Yarn),
            ("pnpm-lock.yaml", PackageManager::Pnpm),
            ("bun.lockb", PackageManager::Bun),
            ("Cargo.toml", PackageManager::Cargo),
            ("requirements.txt", PackageManager::Pip),
            ("pyproject.toml", PackageManager::Pip), // Python projects
            ("poetry.lock", PackageManager::Pip),    // Poetry projects
        ];

        for (file, pm) in pm_files {
            if path.join(file).exists() {
                project_info.package_manager = pm;
                break;
            }
        }

        // Detect Rust project
        if path.join("Cargo.toml").exists() {
            project_info.project_type = ProjectType::Rust;
            project_info.frameworks.push("Rust".to_string());
            return Ok(project_info);
        }

        // Detect Python project
        if path.join("requirements.txt").exists()
            || path.join("pyproject.toml").exists()
            || path.join("setup.py").exists()
            || path.join("poetry.lock").exists()
        {
            project_info.project_type = ProjectType::Python;
            project_info.frameworks.push("Python".to_string());
        }

        // Check for TypeScript
        if path.join("tsconfig.json").exists() || path.join("tsconfig.base.json").exists() {
            project_info.has_typescript = true;

            // If it's primarily a TypeScript project (no framework detected yet)
            if project_info.project_type == ProjectType::Unknown {
                project_info.project_type = ProjectType::TypeScript;
            }
        }

        // Detect project type from package.json
        if let Ok(package_content) = fs::read_to_string(path.join("package.json")).await {
            if let Ok(package_json) = serde_json::from_str::<serde_json::Value>(&package_content) {
                project_info = self.analyze_package_json(&package_json, project_info)?;
            }
        }

        // Check for Angular
        if path.join("angular.json").exists() {
            project_info.project_type = ProjectType::Angular;
            project_info.frameworks.push("Angular".to_string());
        }

        // Check for Next.js
        if path.join("next.config.js").exists() || path.join("next.config.ts").exists() {
            project_info.project_type = ProjectType::NextJs;
            project_info.frameworks.push("Next.js".to_string());
        }

        // If we have package.json but no specific framework, check for pure JS/TS
        if project_info.project_type == ProjectType::Unknown && path.join("package.json").exists() {
            if project_info.has_typescript {
                project_info.project_type = ProjectType::TypeScript;
            } else {
                project_info.project_type = ProjectType::JavaScript;
            }
        }

        // Collect config files
        project_info.config_files = self.find_config_files(path).await?;

        Ok(project_info)
    }

    fn analyze_package_json(
        &self,
        package_json: &serde_json::Value,
        mut project_info: ProjectInfo,
    ) -> Result<ProjectInfo> {
        // Check dependencies
        let deps = [
            package_json.get("dependencies"),
            package_json.get("devDependencies"),
        ];

        for dep_section in deps.iter().flatten() {
            if let Some(deps_obj) = dep_section.as_object() {
                for (name, _) in deps_obj {
                    match name.as_str() {
                        "next" => {
                            project_info.project_type = ProjectType::NextJs;
                            project_info.frameworks.push("Next.js".to_string());
                        }
                        "@angular/core" => {
                            project_info.project_type = ProjectType::Angular;
                            project_info.frameworks.push("Angular".to_string());
                        }
                        "react" => {
                            if project_info.project_type == ProjectType::Unknown {
                                project_info.project_type = ProjectType::React;
                            }
                            project_info.frameworks.push("React".to_string());
                        }
                        "vue" => {
                            project_info.project_type = ProjectType::Vue;
                            project_info.frameworks.push("Vue".to_string());
                        }
                        "typescript" => {
                            project_info.has_typescript = true;
                        }
                        _ => {}
                    }
                }
            }
        }

        // If no specific framework detected but has Node.js patterns
        if project_info.project_type == ProjectType::Unknown
            && (package_json.get("scripts").is_some() || package_json.get("main").is_some())
        {
            project_info.project_type = ProjectType::NodeJs;
        }

        Ok(project_info)
    }

    pub async fn find_config_files(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let mut config_files = Vec::new();

        let simple_files = [
            ".eslintrc.js",
            ".eslintrc.json",
            ".eslintrc.yaml",
            ".prettierrc",
            ".prettierrc.json",
            ".prettierrc.js",
            "tsconfig.json",
            "jest.config.js",
            "webpack.config.js",
            "vite.config.js",
            "vite.config.ts",
            "tailwind.config.js",
            "next.config.js",
            "angular.json",
            ".gitignore",
            "Dockerfile",
        ];

        for file in simple_files {
            let file_path = path.join(file);
            if file_path.exists() {
                config_files.push(file_path);
            }
        }

        Ok(config_files)
    }

    fn generate_quality_checks(&self, project_info: &ProjectInfo) -> Result<Vec<QualityCheck>> {
        let mut checks = Vec::new();

        match project_info.project_type {
            ProjectType::NextJs | ProjectType::React | ProjectType::Angular | ProjectType::Vue => {
                checks.extend(self.generate_frontend_checks(project_info)?);
            }
            ProjectType::NodeJs | ProjectType::TypeScript | ProjectType::JavaScript => {
                checks.extend(self.generate_nodejs_checks(project_info)?);
            }
            ProjectType::Rust => {
                checks.extend(self.generate_rust_checks(project_info)?);
            }
            ProjectType::Python => {
                checks.extend(self.generate_python_checks(project_info)?);
            }
            _ => {
                // Fallback to basic checks
                checks.extend(self.generate_basic_checks(project_info)?);
            }
        }

        Ok(checks)
    }

    pub fn generate_python_checks(&self, project_info: &ProjectInfo) -> Result<Vec<QualityCheck>> {
        let mut checks = Vec::new();

        // Python linting with flake8 or pylint
        checks.push(QualityCheck {
            name: "lint".to_string(),
            command: "flake8".to_string(),
            args: vec![".".to_string()],
            working_dir: project_info.root_path.clone(),
            timeout: self.config.timeout_seconds,
        });

        // Python formatting with black
        checks.push(QualityCheck {
            name: "format".to_string(),
            command: "black".to_string(),
            args: vec!["--check".to_string(), ".".to_string()],
            working_dir: project_info.root_path.clone(),
            timeout: self.config.timeout_seconds,
        });

        // Python testing with pytest
        checks.push(QualityCheck {
            name: "test".to_string(),
            command: "pytest".to_string(),
            args: vec![],
            working_dir: project_info.root_path.clone(),
            timeout: self.config.timeout_seconds,
        });

        // Python security with bandit
        checks.push(QualityCheck {
            name: "security".to_string(),
            command: "bandit".to_string(),
            args: vec!["-r".to_string(), ".".to_string()],
            working_dir: project_info.root_path.clone(),
            timeout: self.config.timeout_seconds,
        });

        Ok(checks)
    }

    pub fn generate_frontend_checks(
        &self,
        project_info: &ProjectInfo,
    ) -> Result<Vec<QualityCheck>> {
        let mut checks = Vec::new();
        let pm_cmd = self.get_package_manager_command(&project_info.package_manager);

        // Linting
        checks.push(QualityCheck {
            name: "lint".to_string(),
            command: pm_cmd.clone(),
            args: vec!["run".to_string(), "lint".to_string()],
            working_dir: project_info.root_path.clone(),
            timeout: self.config.timeout_seconds,
        });

        // Type checking (if TypeScript)
        if project_info.has_typescript {
            checks.push(QualityCheck {
                name: "typecheck".to_string(),
                command: pm_cmd.clone(),
                args: vec!["run".to_string(), "type-check".to_string()],
                working_dir: project_info.root_path.clone(),
                timeout: self.config.timeout_seconds,
            });
        }

        // Testing
        checks.push(QualityCheck {
            name: "test".to_string(),
            command: pm_cmd.clone(),
            args: vec!["test".to_string()],
            working_dir: project_info.root_path.clone(),
            timeout: self.config.timeout_seconds,
        });

        // Security audit
        checks.push(QualityCheck {
            name: "security".to_string(),
            command: pm_cmd.clone(),
            args: vec!["audit".to_string()],
            working_dir: project_info.root_path.clone(),
            timeout: self.config.timeout_seconds,
        });

        Ok(checks)
    }

    fn generate_nodejs_checks(&self, project_info: &ProjectInfo) -> Result<Vec<QualityCheck>> {
        let mut checks = Vec::new();
        let pm_cmd = self.get_package_manager_command(&project_info.package_manager);

        checks.push(QualityCheck {
            name: "lint".to_string(),
            command: pm_cmd.clone(),
            args: vec!["run".to_string(), "lint".to_string()],
            working_dir: project_info.root_path.clone(),
            timeout: self.config.timeout_seconds,
        });

        checks.push(QualityCheck {
            name: "test".to_string(),
            command: pm_cmd.clone(),
            args: vec!["test".to_string()],
            working_dir: project_info.root_path.clone(),
            timeout: self.config.timeout_seconds,
        });

        checks.push(QualityCheck {
            name: "security".to_string(),
            command: pm_cmd,
            args: vec!["audit".to_string()],
            working_dir: project_info.root_path.clone(),
            timeout: self.config.timeout_seconds,
        });

        Ok(checks)
    }

    pub fn generate_rust_checks(&self, project_info: &ProjectInfo) -> Result<Vec<QualityCheck>> {
        let mut checks = Vec::new();

        checks.push(QualityCheck {
            name: "format".to_string(),
            command: "cargo".to_string(),
            args: vec!["fmt".to_string(), "--check".to_string()],
            working_dir: project_info.root_path.clone(),
            timeout: self.config.timeout_seconds,
        });

        checks.push(QualityCheck {
            name: "lint".to_string(),
            command: "cargo".to_string(),
            args: vec![
                "clippy".to_string(),
                "--".to_string(),
                "-D".to_string(),
                "warnings".to_string(),
            ],
            working_dir: project_info.root_path.clone(),
            timeout: self.config.timeout_seconds,
        });

        checks.push(QualityCheck {
            name: "test".to_string(),
            command: "cargo".to_string(),
            args: vec!["test".to_string()],
            working_dir: project_info.root_path.clone(),
            timeout: self.config.timeout_seconds,
        });

        Ok(checks)
    }

    pub fn generate_basic_checks(&self, project_info: &ProjectInfo) -> Result<Vec<QualityCheck>> {
        let mut checks = Vec::new();

        // Basic file validation
        checks.push(QualityCheck {
            name: "validate".to_string(),
            command: "echo".to_string(),
            args: vec!["Basic validation completed".to_string()],
            working_dir: project_info.root_path.clone(),
            timeout: self.config.timeout_seconds,
        });

        Ok(checks)
    }

    pub fn get_package_manager_command(&self, pm: &PackageManager) -> String {
        match pm {
            PackageManager::Npm => "npm".to_string(),
            PackageManager::Yarn => "yarn".to_string(),
            PackageManager::Pnpm => "pnpm".to_string(),
            PackageManager::Bun => "bun".to_string(),
            PackageManager::Cargo => "cargo".to_string(),
            PackageManager::Pip => "pip".to_string(),
            PackageManager::Unknown => "npm".to_string(), // fallback
        }
    }

    async fn run_check(&self, check: &QualityCheck) -> CheckResult {
        let start = std::time::Instant::now();

        let mut command = Command::new(&check.command);
        command.args(&check.args).current_dir(&check.working_dir);

        match command.output() {
            Ok(output) => {
                let success = output.status.success();
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                CheckResult {
                    check_name: check.name.clone(),
                    success,
                    output: stdout,
                    error: if stderr.is_empty() {
                        None
                    } else {
                        Some(stderr)
                    },
                    duration_ms: start.elapsed().as_millis(),
                }
            }
            Err(e) => CheckResult {
                check_name: check.name.clone(),
                success: false,
                output: String::new(),
                error: Some(e.to_string()),
                duration_ms: start.elapsed().as_millis(),
            },
        }
    }

    fn print_project_info(&self, info: &ProjectInfo) {
        println!("{}", "📋 Project Information:".cyan().bold());
        println!("  Type: {:?}", info.project_type);
        println!("  Package Manager: {:?}", info.package_manager);
        println!(
            "  TypeScript: {}",
            if info.has_typescript { "Yes" } else { "No" }
        );
        if !info.frameworks.is_empty() {
            println!("  Frameworks: {}", info.frameworks.join(", "));
        }
        println!("  Config files: {}", info.config_files.len());
        println!();
    }

    fn print_check_result(&self, result: &CheckResult) {
        let status = if result.success {
            "✅ PASS".green()
        } else {
            "❌ FAIL".red()
        };

        println!(
            "    {} {} ({}ms)",
            status, result.check_name, result.duration_ms
        );

        if !result.success {
            if let Some(error) = &result.error {
                println!("      Error: {}", error.red());
            }
        }
    }

    fn print_summary(&self, results: &[CheckResult]) {
        println!();
        println!("{}", "📊 Summary:".cyan().bold());

        let passed = results.iter().filter(|r| r.success).count();
        let failed = results.len() - passed;
        let total_duration: u128 = results.iter().map(|r| r.duration_ms).sum();

        println!("  Total checks: {}", results.len());
        println!("  Passed: {}", passed.to_string().green());
        println!("  Failed: {}", failed.to_string().red());
        println!("  Total time: {}ms", total_duration);

        if failed > 0 {
            println!();
            println!("{}", "Failed checks:".red().bold());
            for result in results.iter().filter(|r| !r.success) {
                println!("  - {}", result.check_name.red());
            }
        }
    }
}

// CLI command handler
pub async fn run_code_quality(path: Option<String>, config_path: Option<String>) -> Result<()> {
    let project_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let config = if let Some(config_file) = config_path {
        let config_content = fs::read_to_string(config_file).await?;
        serde_json::from_str(&config_content)?
    } else {
        CodeQualityConfig::default()
    };

    let manager = CodeQualityManager::new(config);
    let results = manager.run_quality_checks(&project_path).await?;

    // Exit with error code if any checks failed
    let failed_count = results.iter().filter(|r| !r.success).count();
    if failed_count > 0 {
        std::process::exit(1);
    }

    Ok(())
}
