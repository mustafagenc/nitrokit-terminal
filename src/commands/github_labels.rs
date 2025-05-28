use anyhow::{anyhow, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubLabel {
    pub name: String,
    pub description: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelUpdate {
    pub old_name: String,
    pub new_name: String,
    pub description: String,
    pub color: String,
}

#[derive(Debug, Clone)]
pub struct GitHubLabelsConfig {
    pub skip_auth: bool,
    pub skip_install: bool,
    pub dry_run: bool,
    pub list_only: bool,
    pub delete_all: bool,
    pub update_only: bool,
}

impl Default for GitHubLabelsConfig {
    fn default() -> Self {
        Self {
            skip_auth: false,
            skip_install: false,
            dry_run: false,
            list_only: false,
            delete_all: false,
            update_only: false,
        }
    }
}

pub struct GitHubLabelsManager {
    pub config: GitHubLabelsConfig,
}

impl GitHubLabelsManager {
    pub fn new(config: GitHubLabelsConfig) -> Self {
        Self { config }
    }

    pub async fn run(&self) -> Result<()> {
        self.print_banner();
        self.show_configuration();

        if !self.config.skip_install {
            self.check_and_install_gh_cli().await?;
        }

        if !self.config.skip_auth {
            self.check_authentication().await?;
        }

        if self.config.list_only {
            return self.list_labels().await;
        }

        if self.config.delete_all {
            self.delete_all_labels().await?;
        }

        if !self.config.update_only {
            self.update_existing_labels().await?;
        }

        self.create_new_labels().await?;
        self.show_completion_info().await?;

        Ok(())
    }

    pub fn print_banner(&self) {
        println!(
            "{}",
            "╔══════════════════════════════════════════════════════════════════════╗"
                .bright_blue()
        );
        println!(
            "{}",
            "║                                                                      ║"
                .bright_blue()
        );
        println!(
            "{}",
            "║                    🏷️  NITROTERM GITHUB LABELS                       ║"
                .bright_cyan()
                .bold()
        );
        println!(
            "{}",
            "║              GitHub Repository Label Management Tool                 ║"
                .bright_green()
        );
        println!(
            "{}",
            "║                                                                      ║"
                .bright_blue()
        );
        println!(
            "{}",
            "╚══════════════════════════════════════════════════════════════════════╝"
                .bright_blue()
        );
        println!();
    }

    pub fn show_configuration(&self) {
        println!(
            "{}",
            "🏷️  Managing GitHub labels for Nitroterm...".cyan().bold()
        );

        if self.config.dry_run {
            println!(
                "{}",
                "🔍 DRY RUN MODE - No changes will be made".yellow().bold()
            );
        }

        if self.config.list_only {
            println!(
                "{}",
                "📋 LIST ONLY MODE - Just showing current labels"
                    .blue()
                    .bold()
            );
        }

        if self.config.delete_all {
            println!(
                "{}",
                "🗑️  DELETE ALL MODE - Will remove existing labels first"
                    .red()
                    .bold()
            );
        }

        if self.config.update_only {
            println!(
                "{}",
                "🔄 UPDATE ONLY MODE - Only updating existing labels"
                    .green()
                    .bold()
            );
        }

        println!();
    }

    pub async fn check_and_install_gh_cli(&self) -> Result<()> {
        println!("{}", "🔧 Checking GitHub CLI installation...".yellow());

        if self.is_gh_cli_installed().await {
            let version = self.get_gh_version().await?;
            println!("{}", format!("✅ GitHub CLI found: {}", version).green());
            return Ok(());
        }

        println!("{}", "❌ GitHub CLI (gh) is not installed.".red());

        if self.config.dry_run {
            println!("{}", "🔍 DRY RUN: Would install GitHub CLI".yellow());
            return Ok(());
        }

        let install = self
            .prompt_user("🤔 Would you like to install it automatically? (y/N): ")
            .await?;

        if install.to_lowercase() == "y" || install.to_lowercase() == "yes" {
            self.install_gh_cli().await?;

            if self.is_gh_cli_installed().await {
                let version = self.get_gh_version().await?;
                println!(
                    "{}",
                    format!("✅ GitHub CLI installed successfully! {}", version).green()
                );
            } else {
                return Err(anyhow!("❌ Installation failed. Please install manually."));
            }
        } else {
            return Err(anyhow!("❌ GitHub CLI is required. Please install it manually:\n\n🍎 macOS: brew install gh\n🐧 Ubuntu/Debian: apt install gh\n🟦 Windows: choco install gh\n📦 Or download from: https://cli.github.com/"));
        }

        Ok(())
    }

    pub async fn is_gh_cli_installed(&self) -> bool {
        Command::new("gh")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    pub async fn get_gh_version(&self) -> Result<String> {
        let output = Command::new("gh").arg("--version").output()?;

        let version = String::from_utf8(output.stdout)?;
        Ok(version
            .lines()
            .next()
            .unwrap_or("unknown version")
            .to_string())
    }

    pub async fn install_gh_cli(&self) -> Result<()> {
        let os = self.detect_os();
        println!(
            "{}",
            format!("🔧 Installing GitHub CLI for {}...", os).yellow()
        );

        match os.as_str() {
            "macos" => self.install_macos().await,
            "ubuntu" | "debian" => self.install_ubuntu().await,
            "centos" | "rhel" => self.install_centos().await,
            "fedora" => self.install_fedora().await,
            "windows" => {
                println!(
                    "{}",
                    "❌ Windows detected. Please install GitHub CLI manually:".red()
                );
                println!("1. Download from: https://github.com/cli/cli/releases");
                println!("2. Or use Chocolatey: choco install gh");
                println!("3. Or use Scoop: scoop install gh");
                println!("4. Or use Winget: winget install --id GitHub.cli");
                Err(anyhow!("Manual installation required for Windows"))
            }
            _ => {
                println!(
                    "{}",
                    format!("❌ Unsupported operating system: {}", os).red()
                );
                println!("Please install GitHub CLI manually from: https://cli.github.com/");
                Err(anyhow!("Unsupported OS"))
            }
        }
    }

    pub fn detect_os(&self) -> String {
        if cfg!(target_os = "macos") {
            "macos".to_string()
        } else if cfg!(target_os = "linux") {
            // Try to detect Linux distribution
            if std::path::Path::new("/etc/debian_version").exists() {
                "ubuntu".to_string()
            } else if std::path::Path::new("/etc/fedora-release").exists() {
                "fedora".to_string()
            } else if std::path::Path::new("/etc/centos-release").exists() {
                "centos".to_string()
            } else {
                "linux".to_string()
            }
        } else if cfg!(target_os = "windows") {
            "windows".to_string()
        } else {
            "unknown".to_string()
        }
    }

    pub async fn install_macos(&self) -> Result<()> {
        if Command::new("brew").arg("--version").output().is_ok() {
            println!("{}", "Installing via Homebrew...".blue());
            let status = Command::new("brew").args(&["install", "gh"]).status()?;

            if status.success() {
                Ok(())
            } else {
                Err(anyhow!("Failed to install via Homebrew"))
            }
        } else {
            Err(anyhow!("❌ Homebrew not found. Please install Homebrew first:\n/bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"\nThen run: brew install gh"))
        }
    }

    pub async fn install_ubuntu(&self) -> Result<()> {
        println!("{}", "Installing via apt-get...".blue());

        // Add GitHub CLI repository
        let commands = vec![
            vec![
                "curl",
                "-fsSL",
                "https://cli.github.com/packages/githubcli-archive-keyring.gpg",
            ],
            vec!["sudo", "apt", "update"],
            vec!["sudo", "apt", "install", "gh", "-y"],
        ];

        for cmd in commands {
            let status = Command::new(&cmd[0]).args(&cmd[1..]).status()?;

            if !status.success() {
                return Err(anyhow!("Failed to execute: {}", cmd.join(" ")));
            }
        }

        Ok(())
    }

    pub async fn install_centos(&self) -> Result<()> {
        println!("{}", "Installing via yum...".blue());

        let commands = vec![
            vec!["sudo", "yum", "install", "-y", "dnf-plugins-core"],
            vec![
                "sudo",
                "yum",
                "config-manager",
                "--add-repo",
                "https://cli.github.com/packages/rpm/gh-cli.repo",
            ],
            vec!["sudo", "yum", "install", "gh", "-y"],
        ];

        for cmd in commands {
            let status = Command::new(&cmd[0]).args(&cmd[1..]).status()?;

            if !status.success() {
                return Err(anyhow!("Failed to execute: {}", cmd.join(" ")));
            }
        }

        Ok(())
    }

    pub async fn install_fedora(&self) -> Result<()> {
        println!("{}", "Installing via dnf...".blue());

        let status = Command::new("sudo")
            .args(&["dnf", "install", "gh", "-y"])
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(anyhow!("Failed to install via dnf"))
        }
    }

    pub async fn check_authentication(&self) -> Result<()> {
        println!("{}", "🔐 Checking GitHub authentication...".yellow());

        let status = Command::new("gh").args(&["auth", "status"]).output()?;

        if status.status.success() {
            println!("{}", "✅ Already authenticated with GitHub".green());
            let auth_info = String::from_utf8_lossy(&status.stderr);
            println!("{}", auth_info.trim().dimmed());
        } else {
            println!("{}", "❌ Not authenticated with GitHub.".red());

            if self.config.dry_run {
                println!("{}", "🔍 DRY RUN: Would authenticate with GitHub".yellow());
                return Ok(());
            }

            let authenticate = self
                .prompt_user("🔑 Would you like to authenticate now? (y/N): ")
                .await?;

            if authenticate.to_lowercase() == "y" || authenticate.to_lowercase() == "yes" {
                println!("{}", "🌐 Opening browser for authentication...".blue());

                let status = Command::new("gh")
                    .args(&["auth", "login", "--web"])
                    .status()?;

                if status.success() {
                    println!("{}", "✅ Authentication successful!".green());
                } else {
                    return Err(anyhow!("❌ Authentication failed."));
                }
            } else {
                return Err(anyhow!(
                    "❌ GitHub authentication is required.\nRun: gh auth login"
                ));
            }
        }

        Ok(())
    }

    pub async fn prompt_user(&self, message: &str) -> Result<String> {
        print!("{}", message.cyan());
        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        Ok(input.trim().to_string())
    }

    pub async fn list_labels(&self) -> Result<()> {
        println!("{}", "📋 Current labels:".cyan().bold());

        let output = Command::new("gh")
            .args(&["label", "list", "--limit", "50"])
            .output()?;

        if output.status.success() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        } else {
            return Err(anyhow!("Failed to list labels"));
        }

        Ok(())
    }

    pub async fn delete_all_labels(&self) -> Result<()> {
        println!("{}", "🗑️  Deleting all existing labels...".red().bold());

        let output = Command::new("gh")
            .args(&["label", "list", "--limit", "100"])
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Failed to list labels for deletion"));
        }

        let labels_output = String::from_utf8_lossy(&output.stdout);

        for line in labels_output.lines() {
            if let Some(label_name) = line.split_whitespace().next() {
                if self.config.dry_run {
                    println!("{}", format!("🔍 Would delete: {}", label_name).yellow());
                } else {
                    println!("Deleting: {}", label_name);
                    let status = Command::new("gh")
                        .args(&["label", "delete", label_name, "--yes"])
                        .status();

                    match status {
                        Ok(status) if status.success() => {
                            println!("  ✅ Deleted successfully");
                        }
                        _ => {
                            println!("  ⚠️  Could not delete {}", label_name);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn update_existing_labels(&self) -> Result<()> {
        println!(
            "{}",
            "🔄 Updating existing labels with emojis...".blue().bold()
        );

        let existing_labels_to_update = self.get_existing_labels_to_update();

        for label_update in existing_labels_to_update {
            println!(
                "Updating: {} → {}",
                label_update.old_name.yellow(),
                label_update.new_name.green()
            );

            if self.config.dry_run {
                println!(
                    "{}",
                    format!(
                        "🔍 DRY RUN: Would update {} to {}",
                        label_update.old_name, label_update.new_name
                    )
                    .yellow()
                );
            } else {
                let status = Command::new("gh")
                    .args(&[
                        "label",
                        "edit",
                        &label_update.old_name,
                        "--name",
                        &label_update.new_name,
                        "--description",
                        &label_update.description,
                        "--color",
                        &label_update.color,
                    ])
                    .status();

                match status {
                    Ok(status) if status.success() => {
                        println!("  ✅ Updated successfully");
                    }
                    _ => {
                        println!("  ⚠️  Error updating or label not found");
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn create_new_labels(&self) -> Result<()> {
        if self.config.update_only {
            return Ok(());
        }

        println!("{}", "🎨 Creating new Nitroterm labels...".green().bold());

        let new_labels = self.get_new_labels_to_create();

        for label in new_labels {
            println!("Creating: {}", label.name.bright_green());

            if self.config.dry_run {
                println!(
                    "{}",
                    format!("🔍 DRY RUN: Would create label '{}'", label.name).yellow()
                );
            } else {
                let status = Command::new("gh")
                    .args(&[
                        "label",
                        "create",
                        &label.name,
                        "--description",
                        &label.description,
                        "--color",
                        &label.color,
                    ])
                    .status();

                match status {
                    Ok(status) if status.success() => {
                        println!("  ✅ Created successfully");
                    }
                    _ => {
                        println!("  ⚠️  Error creating label or already exists");
                    }
                }
            }
        }

        Ok(())
    }

    pub fn get_existing_labels_to_update(&self) -> Vec<LabelUpdate> {
        vec![
            LabelUpdate {
                old_name: "bug".to_string(),
                new_name: "🐛 bug".to_string(),
                description: "Software bugs and defects".to_string(),
                color: "D73A49".to_string(),
            },
            LabelUpdate {
                old_name: "dependencies".to_string(),
                new_name: "📦 dependencies".to_string(),
                description: "Dependency updates and package management".to_string(),
                color: "0366D6".to_string(),
            },
            LabelUpdate {
                old_name: "documentation".to_string(),
                new_name: "📚 documentation".to_string(),
                description: "Documentation improvements and updates".to_string(),
                color: "0075CA".to_string(),
            },
            LabelUpdate {
                old_name: "duplicate".to_string(),
                new_name: "🔄 duplicate".to_string(),
                description: "Duplicate issues already reported".to_string(),
                color: "CFD3D7".to_string(),
            },
            LabelUpdate {
                old_name: "enhancement".to_string(),
                new_name: "✨ enhancement".to_string(),
                description: "New features and improvements".to_string(),
                color: "A2EEEF".to_string(),
            },
            LabelUpdate {
                old_name: "github_actions".to_string(),
                new_name: "⚙️ github_actions".to_string(),
                description: "CI/CD and GitHub Actions workflow".to_string(),
                color: "000000".to_string(),
            },
            LabelUpdate {
                old_name: "good first issue".to_string(),
                new_name: "🌟 good first issue".to_string(),
                description: "Beginner-friendly issues for new contributors".to_string(),
                color: "7057FF".to_string(),
            },
            LabelUpdate {
                old_name: "help wanted".to_string(),
                new_name: "🙏 help wanted".to_string(),
                description: "Issues where community help is needed".to_string(),
                color: "008672".to_string(),
            },
            LabelUpdate {
                old_name: "invalid".to_string(),
                new_name: "❌ invalid".to_string(),
                description: "Invalid or incorrectly reported issues".to_string(),
                color: "E4E669".to_string(),
            },
            LabelUpdate {
                old_name: "question".to_string(),
                new_name: "❓ question".to_string(),
                description: "Questions about usage or implementation".to_string(),
                color: "CC317C".to_string(),
            },
            LabelUpdate {
                old_name: "wontfix".to_string(),
                new_name: "🚫 wontfix".to_string(),
                description: "Issues that won't be addressed".to_string(),
                color: "FFFFFF".to_string(),
            },
        ]
    }

    pub fn get_new_labels_to_create(&self) -> Vec<GitHubLabel> {
        vec![
            // Priority Labels
            GitHubLabel {
                name: "🔴 priority: critical".to_string(),
                description: "Critical issues that need immediate attention".to_string(),
                color: "B60205".to_string(),
            },
            GitHubLabel {
                name: "🟠 priority: high".to_string(),
                description: "High priority issues".to_string(),
                color: "D93F0B".to_string(),
            },
            GitHubLabel {
                name: "🟡 priority: medium".to_string(),
                description: "Medium priority issues".to_string(),
                color: "FBCA04".to_string(),
            },
            GitHubLabel {
                name: "🟢 priority: low".to_string(),
                description: "Low priority issues".to_string(),
                color: "0E8A16".to_string(),
            },
            // Status Labels
            GitHubLabel {
                name: "🔄 status: in progress".to_string(),
                description: "Currently being worked on".to_string(),
                color: "0052CC".to_string(),
            },
            GitHubLabel {
                name: "👀 status: needs review".to_string(),
                description: "Waiting for code review".to_string(),
                color: "006B75".to_string(),
            },
            GitHubLabel {
                name: "🚧 status: blocked".to_string(),
                description: "Blocked by external dependencies".to_string(),
                color: "D4C5F9".to_string(),
            },
            GitHubLabel {
                name: "✅ status: ready".to_string(),
                description: "Ready to be implemented".to_string(),
                color: "0E8A16".to_string(),
            },
            // Component Labels
            GitHubLabel {
                name: "🎨 ui/ux".to_string(),
                description: "User interface and experience".to_string(),
                color: "F9D0C4".to_string(),
            },
            GitHubLabel {
                name: "🌍 translation".to_string(),
                description: "Translation and internationalization".to_string(),
                color: "1D76DB".to_string(),
            },
            GitHubLabel {
                name: "🔧 cli".to_string(),
                description: "Command line interface".to_string(),
                color: "5319E7".to_string(),
            },
            GitHubLabel {
                name: "📦 release".to_string(),
                description: "Release management and versioning".to_string(),
                color: "0366D6".to_string(),
            },
            GitHubLabel {
                name: "🔍 code-quality".to_string(),
                description: "Code quality checks and linting".to_string(),
                color: "D93F0B".to_string(),
            },
            // Difficulty Labels
            GitHubLabel {
                name: "🌱 easy".to_string(),
                description: "Easy to implement, good for beginners".to_string(),
                color: "C2E0C6".to_string(),
            },
            GitHubLabel {
                name: "🌿 medium".to_string(),
                description: "Moderate complexity".to_string(),
                color: "FEF2C0".to_string(),
            },
            GitHubLabel {
                name: "🌳 hard".to_string(),
                description: "Complex implementation required".to_string(),
                color: "F9D0C4".to_string(),
            },
            // Type Labels
            GitHubLabel {
                name: "🔒 security".to_string(),
                description: "Security related issues".to_string(),
                color: "D73A49".to_string(),
            },
            GitHubLabel {
                name: "⚡ performance".to_string(),
                description: "Performance optimization".to_string(),
                color: "FBCA04".to_string(),
            },
            GitHubLabel {
                name: "♿ accessibility".to_string(),
                description: "Accessibility improvements".to_string(),
                color: "0052CC".to_string(),
            },
            GitHubLabel {
                name: "🧪 testing".to_string(),
                description: "Testing related issues".to_string(),
                color: "BFD4F2".to_string(),
            },
            GitHubLabel {
                name: "🐞 fix".to_string(),
                description: "Bug fixes".to_string(),
                color: "D73A49".to_string(),
            },
            GitHubLabel {
                name: "✨ feature".to_string(),
                description: "New feature implementation".to_string(),
                color: "A2EEEF".to_string(),
            },
        ]
    }

    pub async fn show_completion_info(&self) -> Result<()> {
        println!();
        println!("{}", "🎉 Label management completed!".green().bold());
        println!();

        if !self.config.dry_run {
            println!("{}", "📋 Current labels:".cyan().bold());
            let _ = self.list_labels().await;
        }

        println!();
        println!("{}", "🔧 Useful commands:".blue().bold());
        println!("📋 To view all labels: {}", "gh label list".green());
        println!(
            "🗑️  To delete a label: {}",
            "gh label delete 'label-name' --yes".green()
        );
        println!(
            "✏️  To edit a label: {}",
            "gh label edit 'label-name' --description 'new desc' --color 'FFFFFF'".green()
        );
        println!("❓ For help: {}", "nitroterm github-labels --help".green());

        Ok(())
    }
}

// CLI command handlers
pub async fn run_github_labels(
    skip_auth: bool,
    skip_install: bool,
    dry_run: bool,
    list_only: bool,
    delete_all: bool,
    update_only: bool,
) -> Result<()> {
    let config = GitHubLabelsConfig {
        skip_auth,
        skip_install,
        dry_run,
        list_only,
        delete_all,
        update_only,
    };

    let manager = GitHubLabelsManager::new(config);
    manager.run().await
}

pub async fn run_github_labels_interactive() -> Result<()> {
    let manager = GitHubLabelsManager::new(GitHubLabelsConfig::default());
    manager.run().await
}
