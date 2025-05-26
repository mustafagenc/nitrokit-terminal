use anyhow::{anyhow, Result};
use colored::*;
use git2::{Repository, Status};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
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
}

impl ReleaseManager {
    pub fn new() -> Result<Self> {
        let repo_path = env::current_dir()?;
        let repo = Repository::open(&repo_path)?;

        // Get repository URL
        let repo_url = Self::get_repo_url(&repo)?;

        // Get current branch
        let head = repo.head()?;
        let current_branch = head
            .shorthand()
            .ok_or_else(|| anyhow!("Could not get current branch name"))?
            .to_string();

        Ok(Self {
            repo_path,
            repo_url,
            current_branch,
        })
    }

    fn get_repo_url(repo: &Repository) -> Result<String> {
        let remotes = repo.remotes()?;

        for remote_name in remotes.iter() {
            if let Some(name) = remote_name {
                if let Ok(remote) = repo.find_remote(name) {
                    if let Some(url) = remote.url() {
                        if url.contains("github.com") {
                            return Ok(Self::normalize_github_url(url));
                        }
                    }
                }
            }
        }

        Err(anyhow!("No GitHub remote found"))
    }

    fn normalize_github_url(url: &str) -> String {
        // Convert SSH/HTTPS URLs to API format
        url.replace("git@github.com:", "https://github.com/")
            .replace(".git", "")
            .replace("https://github.com/", "")
    }

    fn detect_package_manager(&self) -> Option<String> {
        if self.repo_path.join("pnpm-lock.yaml").exists() {
            Some("pnpm".to_string())
        } else if self.repo_path.join("yarn.lock").exists() {
            Some("yarn".to_string())
        } else if self.repo_path.join("package-lock.json").exists() {
            Some("npm".to_string())
        } else {
            None
        }
    }

    fn has_package_script(&self, script: &str) -> bool {
        if let Ok(content) = fs::read_to_string(self.repo_path.join("package.json")) {
            if let Ok(package_json) = serde_json::from_str::<serde_json::Value>(&content) {
                return package_json["scripts"][script].is_string();
            }
        }
        false
    }

    fn run_package_command(&self, pm: &str, command: &str) -> Result<()> {
        println!(
            "{}   → Running: {} {}",
            "  ".dimmed(),
            pm.cyan(),
            command.cyan()
        );

        let mut cmd = Command::new(pm);

        // Handle different package managers
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

        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.trim().is_empty() {
            println!("{}", stdout);
        }

        Ok(())
    }

    fn run_cargo_command(&self, command: &str) -> Result<()> {
        println!("{}   → Running: cargo {}", "  ".dimmed(), command.cyan());

        let mut cmd = Command::new("cargo");

        // Handle multi-word commands like "build --release"
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

        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.trim().is_empty() {
            println!("{}", stdout);
        }

        Ok(())
    }

    fn check_working_directory_clean(&self) -> Result<()> {
        let repo = Repository::open(&self.repo_path)?;
        let statuses = repo.statuses(None)?;

        let mut has_changes = false;
        for entry in statuses.iter() {
            let flags = entry.status();
            // Check if file has any modifications
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
        // Remove 'v' prefix if present
        let version = version.strip_prefix('v').unwrap_or(version);

        // Check semantic version format
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() < 3 {
            return Err(anyhow!("Version must be in format x.y.z (e.g., 1.0.0)"));
        }

        for (i, part) in parts.iter().enumerate() {
            if i < 3 {
                // Check major.minor.patch parts
                if part.parse::<u32>().is_err() {
                    return Err(anyhow!("Version parts must be numbers"));
                }
            }
            // Allow pre-release suffixes like -beta.1
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
                    return false; // Stop iteration
                }
            }
            true // Continue iteration
        })?;

        if tag_exists {
            return Err(anyhow!("Tag {} already exists", version));
        }

        Ok(())
    }

    fn update_cargo_version(&self, version: &str) -> Result<()> {
        let cargo_toml_path = self.repo_path.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Ok(()); // No Cargo.toml, skip
        }

        let content = fs::read_to_string(&cargo_toml_path)?;
        let version_without_v = version.strip_prefix('v').unwrap_or(version);

        // Simple regex replacement for version
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
            format!("🚀 Creating release {}...", version).cyan().bold()
        );
        println!("{}", "═".repeat(50).dimmed());

        // Validate version format
        self.validate_version(version)?;
        println!("{}   ✓ Version format validated", "  ".dimmed());

        // Check if tag already exists
        self.check_tag_exists(version)?;
        println!("{}   ✓ Tag {} is available", "  ".dimmed(), version.green());

        // Check working directory is clean
        self.check_working_directory_clean()?;
        println!("{}   ✓ Working directory is clean", "  ".dimmed());

        // Ensure on main branch
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

        // Pull latest changes
        println!("\n{}", "📥 Pulling latest changes...".blue().bold());
        let output = Command::new("git")
            .args(&["pull", "origin", &self.current_branch])
            .current_dir(&self.repo_path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Git pull failed: {}", stderr));
        }
        println!("{}   ✓ Latest changes pulled", "  ".dimmed());

        // Run Node.js tasks if package.json exists
        if let Some(pm) = self.detect_package_manager() {
            println!("\n{}", format!("📦 Running {} tasks...", pm).blue().bold());

            // Format code
            if self.has_package_script("format:write") {
                self.run_package_command(&pm, "format:write")?;
                println!("{}   ✓ Code formatted", "  ".dimmed());
            }

            // Lint code
            if self.has_package_script("lint") {
                self.run_package_command(&pm, "lint")?;
                println!("{}   ✓ Code linted", "  ".dimmed());
            }

            // Run tests with coverage
            if self.has_package_script("test:coverage") {
                self.run_package_command(&pm, "test:coverage")?;
                println!("{}   ✓ Tests passed with coverage", "  ".dimmed());
            } else if self.has_package_script("test") {
                self.run_package_command(&pm, "test")?;
                println!("{}   ✓ Tests passed", "  ".dimmed());
            }

            // Build project
            if self.has_package_script("build") {
                self.run_package_command(&pm, "build")?;
                println!("{}   ✓ Project built", "  ".dimmed());
            }
        }

        // Run Rust tasks
        println!("\n{}", "🦀 Running Rust tasks...".blue().bold());

        // Update Cargo.toml version
        self.update_cargo_version(version)?;

        // Run tests
        self.run_cargo_command("test")?;
        println!("{}   ✓ Rust tests passed", "  ".dimmed());

        // Build release
        self.run_cargo_command("build --release")?;
        println!("{}   ✓ Release build completed", "  ".dimmed());

        // Update dependencies if nitrokit is available
        println!("\n{}", "🔄 Running NitroKit tasks...".blue().bold());

        if let Ok(_) = Command::new("./target/release/nitrokit")
            .arg("--version")
            .output()
        {
            // Update dependencies
            if let Ok(_) = Command::new("./target/release/nitrokit")
                .arg("update-dependencies")
                .current_dir(&self.repo_path)
                .output()
            {
                println!("{}   ✓ Dependencies updated", "  ".dimmed());
            }

            // Sync translations
            if let Ok(_) = Command::new("./target/release/nitrokit")
                .arg("sync-translations")
                .current_dir(&self.repo_path)
                .output()
            {
                println!("{}   ✓ Translations synced", "  ".dimmed());
            }

            // Generate release notes
            let release_notes = if let Ok(output) = Command::new("./target/release/nitrokit")
                .arg("release-notes")
                .current_dir(&self.repo_path)
                .output()
            {
                String::from_utf8_lossy(&output.stdout).to_string()
            } else {
                message
                    .unwrap_or(&format!("Release {}", version))
                    .to_string()
            };

            println!("{}   ✓ Release notes generated", "  ".dimmed());

            // Commit version update
            println!("\n{}", "📝 Committing changes...".blue().bold());

            Command::new("git")
                .args(&["add", "Cargo.toml"])
                .current_dir(&self.repo_path)
                .output()?;

            Command::new("git")
                .args(&["commit", "-m", &format!("Bump version to {}", version)])
                .current_dir(&self.repo_path)
                .output()?;

            println!("{}   ✓ Version commit created", "  ".dimmed());

            // Create and push tag
            println!("\n{}", "🏷️  Creating and pushing tag...".blue().bold());

            Command::new("git")
                .args(&["tag", "-a", version, "-m", &format!("Release {}", version)])
                .current_dir(&self.repo_path)
                .output()?;

            Command::new("git")
                .args(&["push", "origin", &self.current_branch])
                .current_dir(&self.repo_path)
                .output()?;

            Command::new("git")
                .args(&["push", "origin", version])
                .current_dir(&self.repo_path)
                .output()?;

            println!(
                "{}   ✓ Tag {} pushed to GitHub",
                "  ".dimmed(),
                version.green()
            );

            // Create GitHub release
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
        } else {
            return Err(anyhow!(
                "NitroKit binary not found. Run 'cargo build --release' first."
            ));
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
    println!();

    // Get version
    print!("{}", "Enter release version (e.g., v1.0.0): ".cyan());
    io::stdout().flush()?;
    let mut version = String::new();
    io::stdin().read_line(&mut version)?;
    let version = version.trim();

    if version.is_empty() {
        return Err(anyhow!("Version is required"));
    }

    // Get optional message
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

    // Confirm
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

    // Run async release - NO block_on needed!
    manager.create_release(version, message.as_deref()).await?;

    Ok(())
}

pub async fn create_release_with_args(version: &str, message: Option<&str>) -> Result<()> {
    let manager = ReleaseManager::new()?;
    manager.create_release(version, message).await
}
