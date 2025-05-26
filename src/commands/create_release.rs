use anyhow::{anyhow, Result};
use colored::*;
use git2::{Repository, Status};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub enum FrameworkType {
    NextJs,
    Angular,
    React,
    Vue,
    NodeJs,
    Rust,
    Laravel,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ProjectConfig {
    pub framework: FrameworkType,
    pub package_manager: Option<String>,
    pub has_tests: bool,
    pub has_build: bool,
    pub has_lint: bool,
    #[allow(dead_code)]
    pub version_file: String,
}

#[derive(Debug, Serialize)]
struct GitHubReleaseRequest {
    tag_name: String,
    target_commitish: String,
    name: String,
    body: String,
    draft: bool,
    prerelease: bool,
    generate_release_notes: bool,
}

#[derive(Debug, Deserialize)]
struct GitHubReleaseResponse {
    html_url: String,
}

pub struct ReleaseManager {
    repo_path: PathBuf,
    repo_url: String,
    current_branch: String,
    project_config: ProjectConfig,
}

impl ReleaseManager {
    pub fn new() -> Result<Self> {
        let repo_path = env::current_dir()?;
        let repo = Repository::open(&repo_path)?;

        let repo_url = Self::get_repo_url(&repo)?;

        let head = repo.head()?;
        let current_branch = head
            .shorthand()
            .ok_or_else(|| anyhow!("Could not get current branch name"))?
            .to_string();

        let project_config = Self::detect_project_config(&repo_path)?;

        Ok(Self {
            repo_path,
            repo_url,
            current_branch,
            project_config,
        })
    }

    fn detect_project_config(repo_path: &Path) -> Result<ProjectConfig> {
        let framework = Self::detect_framework(repo_path);
        let package_manager = Self::detect_package_manager(repo_path);

        let (has_tests, has_build, has_lint, version_file) = match &framework {
            FrameworkType::NextJs
            | FrameworkType::Angular
            | FrameworkType::NodeJs
            | FrameworkType::React
            | FrameworkType::Vue => {
                let scripts = Self::get_package_scripts(repo_path);
                (
                    scripts.contains_key("test"),
                    scripts.contains_key("build"),
                    scripts.contains_key("lint"),
                    "package.json".to_string(),
                )
            }
            FrameworkType::Rust => (
                repo_path.join("Cargo.toml").exists(),
                repo_path.join("Cargo.toml").exists(),
                false,
                "Cargo.toml".to_string(),
            ),
            FrameworkType::Laravel => (
                repo_path.join("phpunit.xml").exists() || repo_path.join("composer.json").exists(),
                true,
                false,
                "composer.json".to_string(),
            ),
            FrameworkType::Unknown => (false, false, false, "".to_string()),
        };

        Ok(ProjectConfig {
            framework,
            package_manager,
            has_tests,
            has_build,
            has_lint,
            version_file,
        })
    }

    fn detect_framework(repo_path: &Path) -> FrameworkType {
        if repo_path.join("next.config.js").exists()
            || repo_path.join("next.config.ts").exists()
            || repo_path.join("next.config.mjs").exists()
        {
            return FrameworkType::NextJs;
        }

        if repo_path.join("angular.json").exists() || repo_path.join("ng.json").exists() {
            return FrameworkType::Angular;
        }

        if repo_path.join("artisan").exists() || repo_path.join("composer.json").exists() {
            if let Ok(content) = fs::read_to_string(repo_path.join("composer.json")) {
                if content.contains("laravel/framework") {
                    return FrameworkType::Laravel;
                }
            }
        }

        if repo_path.join("Cargo.toml").exists() {
            return FrameworkType::Rust;
        }

        if let Ok(content) = fs::read_to_string(repo_path.join("package.json")) {
            if let Ok(package_json) = serde_json::from_str::<serde_json::Value>(&content) {
                let deps = package_json["dependencies"].as_object();
                let dev_deps = package_json["devDependencies"].as_object();

                if let Some(deps) = deps {
                    if deps.contains_key("react") {
                        return FrameworkType::React;
                    }
                    if deps.contains_key("vue") || deps.contains_key("@vue/cli") {
                        return FrameworkType::Vue;
                    }
                }

                if let Some(dev_deps) = dev_deps {
                    if dev_deps.contains_key("react") {
                        return FrameworkType::React;
                    }
                    if dev_deps.contains_key("vue") || dev_deps.contains_key("@vue/cli") {
                        return FrameworkType::Vue;
                    }
                }
            }
        }

        if repo_path.join("package.json").exists() {
            return FrameworkType::NodeJs;
        }

        FrameworkType::Unknown
    }

    fn detect_package_manager(repo_path: &Path) -> Option<String> {
        if repo_path.join("pnpm-lock.yaml").exists() {
            Some("pnpm".to_string())
        } else if repo_path.join("yarn.lock").exists() {
            Some("yarn".to_string())
        } else if repo_path.join("package-lock.json").exists() {
            Some("npm".to_string())
        } else if repo_path.join("composer.lock").exists() {
            Some("composer".to_string())
        } else {
            None
        }
    }

    fn get_package_scripts(repo_path: &Path) -> HashMap<String, String> {
        let mut scripts = HashMap::new();

        if let Ok(content) = fs::read_to_string(repo_path.join("package.json")) {
            if let Ok(package_json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(scripts_obj) = package_json["scripts"].as_object() {
                    for (key, value) in scripts_obj {
                        if let Some(script) = value.as_str() {
                            scripts.insert(key.clone(), script.to_string());
                        }
                    }
                }
            }
        }

        scripts
    }

    fn get_repo_url(repo: &Repository) -> Result<String> {
        let remotes = repo.remotes()?;

        for remote_name in remotes.iter().flatten() {
            if let Ok(remote) = repo.find_remote(remote_name) {
                if let Some(url) = remote.url() {
                    if url.contains("github.com") {
                        return Ok(Self::normalize_github_url(url));
                    }
                }
            }
        }

        Err(anyhow!("No GitHub remote found"))
    }

    fn normalize_github_url(url: &str) -> String {
        url.replace("git@github.com:", "https://github.com/")
            .replace(".git", "")
            .replace("https://github.com/", "")
    }

    fn run_framework_tasks(&self, version: &str) -> Result<()> {
        match &self.project_config.framework {
            FrameworkType::NextJs => self.run_nextjs_tasks(version),
            FrameworkType::Angular => self.run_angular_tasks(version),
            FrameworkType::NodeJs => self.run_nodejs_tasks(version),
            FrameworkType::React => self.run_react_tasks(version),
            FrameworkType::Vue => self.run_vue_tasks(version),
            FrameworkType::Rust => self.run_rust_tasks(version),
            FrameworkType::Laravel => self.run_laravel_tasks(version),
            FrameworkType::Unknown => {
                println!(
                    "{}   ⚠️  Unknown framework, skipping framework-specific tasks",
                    "  ".yellow()
                );
                Ok(())
            }
        }
    }

    fn run_nextjs_tasks(&self, version: &str) -> Result<()> {
        println!("\n{}", "⚡ Running Next.js tasks...".blue().bold());
        self.update_package_version(version)?;

        if let Some(pm) = &self.project_config.package_manager {
            self.run_package_manager_command(pm, "install")?;
            println!("{}   ✓ Dependencies installed", "  ".dimmed());

            if self.project_config.has_lint {
                self.run_package_command(pm, "lint")?;
                println!("{}   ✓ Code linted", "  ".dimmed());
            }

            if self.project_config.has_tests {
                self.run_package_command(pm, "test")?;
                println!("{}   ✓ Tests passed", "  ".dimmed());
            }

            if self.project_config.has_build {
                self.run_package_command(pm, "build")?;
                println!("{}   ✓ Project built", "  ".dimmed());
            }

            if self.has_package_script("build:analyze") {
                self.run_package_command(pm, "build:analyze")?;
                println!("{}   ✓ Build analyzed", "  ".dimmed());
            }
        }

        Ok(())
    }

    fn run_angular_tasks(&self, version: &str) -> Result<()> {
        println!("\n{}", "🅰️  Running Angular tasks...".blue().bold());
        self.update_package_version(version)?;

        if let Some(pm) = &self.project_config.package_manager {
            self.run_package_manager_command(pm, "install")?;
            println!("{}   ✓ Dependencies installed", "  ".dimmed());

            if self.has_package_script("lint") {
                self.run_package_command(pm, "lint")?;
                println!("{}   ✓ Code linted", "  ".dimmed());
            }

            if self.has_package_script("test") {
                self.run_package_command(pm, "test")?;
                println!("{}   ✓ Tests passed", "  ".dimmed());
            }

            if self.has_package_script("e2e") {
                self.run_package_command(pm, "e2e")?;
                println!("{}   ✓ E2E tests passed", "  ".dimmed());
            }

            if self.has_package_script("build") {
                self.run_package_command(pm, "build")?;
                println!("{}   ✓ Project built", "  ".dimmed());
            }

            if self.has_package_script("build:prod") {
                self.run_package_command(pm, "build:prod")?;
                println!("{}   ✓ Production build completed", "  ".dimmed());
            }
        }

        Ok(())
    }

    fn run_nodejs_tasks(&self, version: &str) -> Result<()> {
        println!("\n{}", "📦 Running Node.js tasks...".blue().bold());
        self.update_package_version(version)?;

        if let Some(pm) = &self.project_config.package_manager {
            self.run_package_manager_command(pm, "install")?;
            println!("{}   ✓ Dependencies installed", "  ".dimmed());

            if pm == "npm" && self.has_package_script("audit") {
                self.run_package_command(pm, "audit")?;
                println!("{}   ✓ Security audit completed", "  ".dimmed());
            }

            if self.project_config.has_lint {
                self.run_package_command(pm, "lint")?;
                println!("{}   ✓ Code linted", "  ".dimmed());
            }

            if self.has_package_script("format") {
                self.run_package_command(pm, "format")?;
                println!("{}   ✓ Code formatted", "  ".dimmed());
            }

            if self.project_config.has_tests {
                if self.has_package_script("test:coverage") {
                    self.run_package_command(pm, "test:coverage")?;
                    println!("{}   ✓ Tests passed with coverage", "  ".dimmed());
                } else {
                    self.run_package_command(pm, "test")?;
                    println!("{}   ✓ Tests passed", "  ".dimmed());
                }
            }

            if self.project_config.has_build {
                self.run_package_command(pm, "build")?;
                println!("{}   ✓ Project built", "  ".dimmed());
            }
        }

        Ok(())
    }

    fn run_react_tasks(&self, version: &str) -> Result<()> {
        println!("\n{}", "⚛️  Running React tasks...".blue().bold());
        self.update_package_version(version)?;

        if let Some(pm) = &self.project_config.package_manager {
            self.run_package_manager_command(pm, "install")?;
            println!("{}   ✓ Dependencies installed", "  ".dimmed());

            if self.project_config.has_lint {
                self.run_package_command(pm, "lint")?;
                println!("{}   ✓ Code linted", "  ".dimmed());
            }

            if self.project_config.has_tests {
                self.run_package_command(pm, "test")?;
                println!("{}   ✓ Tests passed", "  ".dimmed());
            }

            if self.project_config.has_build {
                self.run_package_command(pm, "build")?;
                println!("{}   ✓ Project built", "  ".dimmed());
            }

            if self.has_package_script("analyze") {
                self.run_package_command(pm, "analyze")?;
                println!("{}   ✓ Bundle analyzed", "  ".dimmed());
            }
        }

        Ok(())
    }

    fn run_vue_tasks(&self, version: &str) -> Result<()> {
        println!("\n{}", "💚 Running Vue.js tasks...".blue().bold());
        self.update_package_version(version)?;

        if let Some(pm) = &self.project_config.package_manager {
            self.run_package_manager_command(pm, "install")?;
            println!("{}   ✓ Dependencies installed", "  ".dimmed());

            if self.project_config.has_lint {
                self.run_package_command(pm, "lint")?;
                println!("{}   ✓ Code linted", "  ".dimmed());
            }

            if self.project_config.has_tests {
                if self.has_package_script("test:unit") {
                    self.run_package_command(pm, "test:unit")?;
                    println!("{}   ✓ Unit tests passed", "  ".dimmed());
                } else {
                    self.run_package_command(pm, "test")?;
                    println!("{}   ✓ Tests passed", "  ".dimmed());
                }
            }

            if self.project_config.has_build {
                self.run_package_command(pm, "build")?;
                println!("{}   ✓ Project built", "  ".dimmed());
            }
        }

        Ok(())
    }

    fn run_rust_tasks(&self, version: &str) -> Result<()> {
        println!("\n{}", "🦀 Running Rust tasks...".blue().bold());
        self.update_cargo_version(version)?;

        if Command::new("cargo")
            .arg("fmt")
            .arg("--version")
            .output()
            .is_ok()
        {
            self.run_cargo_command("fmt")?;
            println!("{}   ✓ Code formatted", "  ".dimmed());
        }

        if Command::new("cargo")
            .arg("clippy")
            .arg("--version")
            .output()
            .is_ok()
        {
            self.run_cargo_command("clippy -- -D warnings")?;
            println!("{}   ✓ Clippy linting passed", "  ".dimmed());
        }

        self.run_cargo_command("test")?;
        println!("{}   ✓ Tests passed", "  ".dimmed());

        self.run_cargo_command("build --release")?;
        println!("{}   ✓ Release build completed", "  ".dimmed());

        self.run_cargo_command("doc --no-deps")?;
        println!("{}   ✓ Documentation generated", "  ".dimmed());

        Ok(())
    }

    fn run_laravel_tasks(&self, version: &str) -> Result<()> {
        println!("\n{}", "🐘 Running Laravel tasks...".blue().bold());
        self.update_composer_version(version)?;

        if Command::new("composer").arg("--version").output().is_ok() {
            self.run_composer_command("install --optimize-autoloader")?;
            println!("{}   ✓ Composer dependencies installed", "  ".dimmed());
        }

        if self.repo_path.join(".env.example").exists() && !self.repo_path.join(".env").exists() {
            fs::copy(
                self.repo_path.join(".env.example"),
                self.repo_path.join(".env"),
            )?;
            self.run_artisan_command("key:generate")?;
            println!("{}   ✓ Application key generated", "  ".dimmed());
        }

        self.run_artisan_command("config:clear")?;
        self.run_artisan_command("cache:clear")?;
        self.run_artisan_command("route:clear")?;
        self.run_artisan_command("view:clear")?;
        println!("{}   ✓ Caches cleared", "  ".dimmed());

        if self.repo_path.join("phpunit.xml").exists() {
            self.run_php_command("vendor/bin/phpunit")?;
            println!("{}   ✓ PHPUnit tests passed", "  ".dimmed());
        }

        if self.repo_path.join("vendor/bin/phpstan").exists() {
            self.run_php_command("vendor/bin/phpstan analyse")?;
            println!("{}   ✓ Static analysis passed", "  ".dimmed());
        }

        self.run_artisan_command("config:cache")?;
        self.run_artisan_command("route:cache")?;
        self.run_artisan_command("view:cache")?;
        println!("{}   ✓ Production optimization completed", "  ".dimmed());

        Ok(())
    }

    fn run_package_manager_command(&self, pm: &str, command: &str) -> Result<()> {
        println!(
            "{}   → Running: {} {}",
            "  ".dimmed(),
            pm.cyan(),
            command.cyan()
        );

        let output = Command::new(pm)
            .args(command.split_whitespace())
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| anyhow!("Failed to run {} {}: {}", pm, command, e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("{} {} failed: {}", pm, command, stderr));
        }

        Ok(())
    }

    fn run_package_command(&self, pm: &str, command: &str) -> Result<()> {
        println!(
            "{}   → Running: {} {}",
            "  ".dimmed(),
            pm.cyan(),
            command.cyan()
        );

        let mut cmd = Command::new(pm);
        match pm {
            "npm" => {
                cmd.arg("run").arg(command);
            }
            "yarn" | "pnpm" => {
                cmd.arg(command);
            }
            _ => return Err(anyhow!("Unknown package manager: {}", pm)),
        }

        let output = cmd
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| anyhow!("Failed to run {} {}: {}", pm, command, e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("{} {} failed: {}", pm, command, stderr));
        }

        Ok(())
    }

    fn run_cargo_command(&self, command: &str) -> Result<()> {
        println!("{}   → Running: cargo {}", "  ".dimmed(), command.cyan());

        let mut cmd = Command::new("cargo");
        for arg in command.split_whitespace() {
            cmd.arg(arg);
        }

        let output = cmd
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| anyhow!("Failed to run cargo {}: {}", command, e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("cargo {} failed: {}", command, stderr));
        }

        Ok(())
    }

    fn run_composer_command(&self, command: &str) -> Result<()> {
        println!("{}   → Running: composer {}", "  ".dimmed(), command.cyan());

        let output = Command::new("composer")
            .args(command.split_whitespace())
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| anyhow!("Failed to run composer {}: {}", command, e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("composer {} failed: {}", command, stderr));
        }

        Ok(())
    }

    fn run_artisan_command(&self, command: &str) -> Result<()> {
        println!(
            "{}   → Running: php artisan {}",
            "  ".dimmed(),
            command.cyan()
        );

        let output = Command::new("php")
            .arg("artisan")
            .args(command.split_whitespace())
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| anyhow!("Failed to run php artisan {}: {}", command, e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("php artisan {} failed: {}", command, stderr));
        }

        Ok(())
    }

    fn run_php_command(&self, command: &str) -> Result<()> {
        println!("{}   → Running: php {}", "  ".dimmed(), command.cyan());

        let output = Command::new("php")
            .args(command.split_whitespace())
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| anyhow!("Failed to run php {}: {}", command, e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("php {} failed: {}", command, stderr));
        }

        Ok(())
    }

    fn has_package_script(&self, script: &str) -> bool {
        if let Ok(content) = fs::read_to_string(self.repo_path.join("package.json")) {
            if let Ok(package_json) = serde_json::from_str::<serde_json::Value>(&content) {
                return package_json["scripts"][script].is_string();
            }
        }
        false
    }

    fn update_package_version(&self, version: &str) -> Result<()> {
        let package_json_path = self.repo_path.join("package.json");
        if !package_json_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&package_json_path)?;
        let mut package_json: serde_json::Value = serde_json::from_str(&content)?;

        let version_without_v = version.strip_prefix('v').unwrap_or(version);
        package_json["version"] = serde_json::Value::String(version_without_v.to_string());

        let updated_content = serde_json::to_string_pretty(&package_json)?;
        fs::write(&package_json_path, updated_content)?;

        println!(
            "{}   ✓ Updated package.json version to {}",
            "  ".dimmed(),
            version_without_v.green()
        );

        Ok(())
    }

    fn update_cargo_version(&self, version: &str) -> Result<()> {
        let cargo_toml_path = self.repo_path.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&cargo_toml_path)?;
        let version_without_v = version.strip_prefix('v').unwrap_or(version);

        let updated_content = content
            .lines()
            .map(|line| {
                if line.trim_start().starts_with("version = ") && !line.contains('#') {
                    format!("version = \"{}\"", version_without_v)
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        fs::write(&cargo_toml_path, updated_content)?;
        println!(
            "{}   ✓ Updated Cargo.toml version to {}",
            "  ".dimmed(),
            version_without_v.green()
        );

        Ok(())
    }

    fn update_composer_version(&self, version: &str) -> Result<()> {
        let composer_json_path = self.repo_path.join("composer.json");
        if !composer_json_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&composer_json_path)?;
        let mut composer_json: serde_json::Value = serde_json::from_str(&content)?;

        let version_without_v = version.strip_prefix('v').unwrap_or(version);
        composer_json["version"] = serde_json::Value::String(version_without_v.to_string());

        let updated_content = serde_json::to_string_pretty(&composer_json)?;
        fs::write(&composer_json_path, updated_content)?;

        println!(
            "{}   ✓ Updated composer.json version to {}",
            "  ".dimmed(),
            version_without_v.green()
        );

        Ok(())
    }

    fn check_working_directory_clean(&self) -> Result<()> {
        let repo = Repository::open(&self.repo_path)?;
        let statuses = repo.statuses(None)?;

        let mut has_changes = false;
        for entry in statuses.iter() {
            let flags = entry.status();
            if flags.intersects(
                Status::INDEX_NEW
                    | Status::INDEX_MODIFIED
                    | Status::INDEX_DELETED
                    | Status::WT_NEW
                    | Status::WT_MODIFIED
                    | Status::WT_DELETED,
            ) {
                has_changes = true;
                break;
            }
        }

        if has_changes {
            return Err(anyhow!("Working directory has uncommitted changes. Please commit or stash changes before creating a release."));
        }

        Ok(())
    }

    fn validate_version(&self, version: &str) -> Result<()> {
        let version = version.strip_prefix('v').unwrap_or(version);
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() < 3 {
            return Err(anyhow!("Version must be in format x.y.z (e.g., 1.0.0)"));
        }

        for (i, part) in parts.iter().enumerate() {
            if i < 3 && part.parse::<u32>().is_err() {
                return Err(anyhow!("Version parts must be numbers"));
            }
        }

        Ok(())
    }

    fn check_tag_exists(&self, version: &str) -> Result<()> {
        let repo = Repository::open(&self.repo_path)?;
        let mut tag_exists = false;

        repo.tag_foreach(|_oid, name| {
            if let Ok(tag_name) = std::str::from_utf8(name) {
                if tag_name.ends_with(version) {
                    tag_exists = true;
                    return false;
                }
            }
            true
        })?;

        if tag_exists {
            return Err(anyhow!("Tag {} already exists", version));
        }

        Ok(())
    }

    async fn create_github_release(&self, version: &str, release_notes: &str) -> Result<String> {
        let github_token = env::var("GITHUB_TOKEN")
            .map_err(|_| anyhow!("GITHUB_TOKEN environment variable not set"))?;

        let is_prerelease =
            version.contains("-alpha") || version.contains("-beta") || version.contains("-rc");

        let release_request = GitHubReleaseRequest {
            tag_name: version.to_string(),
            target_commitish: self.current_branch.clone(),
            name: format!("Release {}", version),
            body: release_notes.to_string(),
            draft: false,
            prerelease: is_prerelease,
            generate_release_notes: release_notes.trim().is_empty(),
        };

        let client = reqwest::Client::new();
        let url = format!("https://api.github.com/repos/{}/releases", self.repo_url);

        let response = client
            .post(&url)
            .header("Authorization", format!("token {}", github_token))
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "nitrokit-release-tool")
            .json(&release_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("GitHub API error: {}", error_text));
        }

        let release: GitHubReleaseResponse = response.json().await?;
        Ok(release.html_url)
    }

    pub async fn create_release(&self, version: &str, message: Option<&str>) -> Result<()> {
        println!(
            "{}",
            format!(
                "🚀 Creating release {} for {:?}...",
                version, self.project_config.framework
            )
            .cyan()
            .bold()
        );
        println!("{}", "═".repeat(50).dimmed());

        self.validate_version(version)?;
        println!("{}   ✓ Version format validated", "  ".dimmed());

        self.check_tag_exists(version)?;
        println!("{}   ✓ Tag {} is available", "  ".dimmed(), version.green());

        self.check_working_directory_clean()?;
        println!("{}   ✓ Working directory is clean", "  ".dimmed());

        if self.current_branch != "main" && self.current_branch != "master" {
            return Err(anyhow!(
                "Not on main/master branch (currently on: {})",
                self.current_branch
            ));
        }
        println!(
            "{}   ✓ On {} branch",
            "  ".dimmed(),
            self.current_branch.green()
        );

        println!("\n{}", "📥 Pulling latest changes...".blue().bold());
        let output = Command::new("git")
            .args(["pull", "origin", &self.current_branch])
            .current_dir(&self.repo_path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Git pull failed: {}", stderr));
        }
        println!("{}   ✓ Latest changes pulled", "  ".dimmed());

        self.run_framework_tasks(version)?;

        let release_notes = message
            .unwrap_or(&format!("Release {}", version))
            .to_string();

        println!("\n{}", "📝 Committing changes...".blue().bold());

        let mut files_to_add = Vec::new();
        if self.repo_path.join("package.json").exists() {
            files_to_add.push("package.json");
        }
        if self.repo_path.join("Cargo.toml").exists() {
            files_to_add.push("Cargo.toml");
        }
        if self.repo_path.join("composer.json").exists() {
            files_to_add.push("composer.json");
        }

        if !files_to_add.is_empty() {
            Command::new("git")
                .arg("add")
                .args(&files_to_add)
                .current_dir(&self.repo_path)
                .output()?;

            Command::new("git")
                .args(["commit", "-m", &format!("Bump version to {}", version)])
                .current_dir(&self.repo_path)
                .output()?;

            println!("{}   ✓ Version commit created", "  ".dimmed());
        }

        println!("\n{}", "🏷️  Creating and pushing tag...".blue().bold());

        Command::new("git")
            .args(["tag", "-a", version, "-m", &format!("Release {}", version)])
            .current_dir(&self.repo_path)
            .output()?;

        Command::new("git")
            .args(["push", "origin", &self.current_branch])
            .current_dir(&self.repo_path)
            .output()?;

        Command::new("git")
            .args(["push", "origin", version])
            .current_dir(&self.repo_path)
            .output()?;

        println!(
            "{}   ✓ Tag {} pushed to GitHub",
            "  ".dimmed(),
            version.green()
        );

        if env::var("GITHUB_TOKEN").is_ok() {
            println!("\n{}", "🎉 Creating GitHub release...".blue().bold());

            match self.create_github_release(version, &release_notes).await {
                Ok(release_url) => {
                    println!("{}   ✓ GitHub release created", "  ".dimmed());
                    println!("\n{}", "🎉 Release completed successfully!".green().bold());
                    println!("{}", "═".repeat(50).dimmed());
                    println!();
                    println!("{}   📋 Release: {}", "  ".dimmed(), release_url.cyan());
                    println!(
                        "{}   🔗 Actions: https://github.com/{}/actions",
                        "  ".dimmed(),
                        self.repo_url
                    );
                    println!();
                }
                Err(e) => {
                    println!(
                        "{}   ⚠️  GitHub release creation failed: {}",
                        "  ".dimmed(),
                        e.to_string().yellow()
                    );
                    println!(
                        "{}   📝 You can create it manually at: https://github.com/{}/releases",
                        "  ".dimmed(),
                        self.repo_url
                    );
                }
            }
        } else {
            println!(
                "\n{}",
                "⚠️  GITHUB_TOKEN not set, skipping GitHub release creation".yellow()
            );
            println!(
                "{}   📝 Create release manually at: https://github.com/{}/releases",
                "  ".dimmed(),
                self.repo_url
            );
        }

        Ok(())
    }
}

pub async fn create_release_interactive() -> Result<()> {
    println!("{}", "🚀 NitroKit Release Creator".cyan().bold());
    println!("{}", "═".repeat(40).dimmed());
    println!();

    let manager = ReleaseManager::new()?;

    println!(
        "{}   📂 Repository: {}",
        "  ".dimmed(),
        manager.repo_url.cyan()
    );
    println!(
        "{}   🌿 Branch: {}",
        "  ".dimmed(),
        manager.current_branch.green()
    );
    println!(
        "{}   🔧 Framework: {:?}",
        "  ".dimmed(),
        manager.project_config.framework
    );
    if let Some(pm) = &manager.project_config.package_manager {
        println!("{}   📦 Package Manager: {}", "  ".dimmed(), pm.cyan());
    }
    println!();

    print!("{}", "Enter release version (e.g., v1.0.0): ".cyan());
    io::stdout().flush()?;
    let mut version = String::new();
    io::stdin().read_line(&mut version)?;
    let version = version.trim();

    if version.is_empty() {
        return Err(anyhow!("Version is required"));
    }

    print!(
        "{}",
        format!("Enter release message [Release {}]: ", version).cyan()
    );
    io::stdout().flush()?;
    let mut message = String::new();
    io::stdin().read_line(&mut message)?;
    let message = message.trim();

    let message = if message.is_empty() {
        None
    } else {
        Some(message)
    };

    println!();
    println!("{}", "📋 Release Summary:".yellow().bold());
    println!("{}   Version: {}", "  ".dimmed(), version.green());
    println!(
        "{}   Repository: {}",
        "  ".dimmed(),
        manager.repo_url.cyan()
    );
    println!(
        "{}   Branch: {}",
        "  ".dimmed(),
        manager.current_branch.cyan()
    );
    println!(
        "{}   Framework: {:?}",
        "  ".dimmed(),
        manager.project_config.framework
    );
    if let Some(msg) = &message {
        println!("{}   Message: {}", "  ".dimmed(), msg.cyan());
    }
    println!();

    print!("{}", "Continue with release? (y/N): ".yellow());
    io::stdout().flush()?;
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;

    if confirm.trim().to_lowercase() != "y" {
        println!("{}", "❌ Release cancelled".red());
        return Ok(());
    }

    manager.create_release(version, message).await?;

    Ok(())
}

pub async fn create_release_with_args(version: &str, message: Option<&str>) -> Result<()> {
    let manager = ReleaseManager::new()?;
    manager.create_release(version, message).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_framework_detection_nextjs() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        fs::write(path.join("next.config.js"), "module.exports = {}").unwrap();

        let framework = ReleaseManager::detect_framework(path);
        assert!(matches!(framework, FrameworkType::NextJs));
    }

    #[test]
    fn test_framework_detection_rust() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        fs::write(path.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

        let framework = ReleaseManager::detect_framework(path);
        assert!(matches!(framework, FrameworkType::Rust));
    }

    #[test]
    fn test_package_manager_detection_npm() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        fs::write(path.join("package-lock.json"), "{}").unwrap();

        let pm = ReleaseManager::detect_package_manager(path);
        assert_eq!(pm, Some("npm".to_string()));
    }

    #[test]
    fn test_normalize_github_url_ssh() {
        let ssh_url = "git@github.com:user/repo.git";
        let normalized = ReleaseManager::normalize_github_url(ssh_url);
        assert_eq!(normalized, "user/repo");
    }

    #[test]
    fn test_normalize_github_url_https() {
        let https_url = "https://github.com/user/repo.git";
        let normalized = ReleaseManager::normalize_github_url(https_url);
        assert_eq!(normalized, "user/repo");
    }
}
