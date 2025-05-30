use clap::Command;
use colored::*;
use std::io::{self, Write};

mod commands;
mod utils;

#[cfg(test)]
mod tests;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_banner() {
    let banner_lines = vec![
        "      ███╗   ██╗██╗████████╗██████╗  ██████╗ ██╗  ██╗██╗████████╗     ",
        "      ████╗  ██║██║╚══██╔══╝██╔══██╗██╔═══██╗██║ ██╔╝██║╚══██╔══╝     ",
        "      ██╔██╗ ██║██║   ██║   ██████╔╝██║   ██║█████╔╝ ██║   ██║        ",
        "      ██║╚██╗██║██║   ██║   ██╔══██╗██║   ██║██╔═██╗ ██║   ██║        ",
        "      ██║ ╚████║██║   ██║   ██║  ██║╚██████╔╝██║  ██╗██║   ██║        ",
        "      ╚═╝  ╚═══╝╚═╝   ╚═╝   ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝   ╚═╝        ",
    ];

    #[cfg(windows)]
    {
        extern "system" {
            fn SetConsoleTitleW(title: *const u16) -> i32;
        }

        let title = "🚀 Nitroterm Terminal Tool\0"
            .encode_utf16()
            .collect::<Vec<u16>>();

        unsafe {
            SetConsoleTitleW(title.as_ptr());
        }
    }

    println!(
        "{}",
        "╔══════════════════════════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "║                                                                      ║".cyan()
    );
    println!(
        "{}",
        "║       A terminal tool for project management and automation for      ║".cyan()
    );
    println!(
        "{}",
        "║                                                                      ║".cyan()
    );

        for (line_idx, line) in banner_lines.iter().enumerate() {
        print!("{}", "║".cyan());

        // Her karakteri farklı renkle boyayalım
        for (char_idx, ch) in line.chars().enumerate() {
            let progress = (char_idx as f32 + line_idx as f32 * 20.0) / (line.len() as f32 + banner_lines.len() as f32 * 20.0);

            // Blue to Green gradient
            let r = (65.0 + progress * (0.0 - 65.0)) as u8;
            let g = (105.0 + progress * (255.0 - 105.0)) as u8;
            let b = (225.0 + progress * (127.0 - 225.0)) as u8;

            print!("{}", ch.to_string().truecolor(r, g, b).bold());
        }

        println!("{}", "║".cyan());
    }

    println!(
        "{}",
        "║                                                                      ║".cyan()
    );
    println!(
        "{}",
        "║                                                                      ║".cyan()
    );
    println!(
        "{}",
        "║           🌐 https://nitrokit.tr  •  📧 hello@nitrokit.tr            ║".cyan()
    );
    println!(
        "{}",
        "║                                                                      ║".cyan()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════════════════╝".cyan()
    );
    println!();
}

fn show_menu() {
    println!(
        "{}",
        format!(" Nitroterm v{} - Built with Rust 🦀", VERSION).dimmed().bold().blue()
    );
    println!();
    println!("{}", " 🚀 Tools".yellow().bold());
    println!();
    println!("  {} Create a new release", "1. 🎁 create-release".green());
    println!(
        "  {} Generate release notes from git commits",
        "2. 📦 release-notes".green()
    );
    println!(
        "  {} Analyze and update project dependencies",
        "3. 📝 update-dependencies".green()
    );
    println!(
        "  {} Sync translations using Gemini AI",
        "4. 🌍 sync-translations".green()
    );
    println!(
        "  {} Run code quality checks (lint, format, security)",
        "5. 🔍 code-quality".green()
    );

    println!();
    println!("{}", " 🤝 Collaboration".cyan().bold());
    println!();
    println!("  {} Manage GitHub repository labels", "6. 🏷️ github-labels".green());

    println!();
    println!("{}", " ⚙️ Settings".cyan().bold());
    println!();
    println!("  {} Manage configuration settings", "7. ⚙️ config".blue());
    println!("  {} Manage project versioning", "8. 🏷️ version".blue());
    println!("  {} Show this help menu", "9. ❓ help".blue());
    println!();
    println!("  {}", "0  🚪 exit".red());
    println!();
}

fn get_user_input() -> String {
    print!("{}", "🚀 nitroterm > ".cyan().bold());
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let app = Command::new("nitroterm")
        .version(VERSION)
        .about("A terminal tool for project management and automation")
        .author("Mustafa Genc <eposta@mustafagenc.info>")
        .subcommand(Command::new("release-notes").about("Generate release notes from git commits"))
        .subcommand(
            Command::new("update-dependencies").about("Analyze and update project dependencies"),
        )
        .subcommand(Command::new("sync-translations").about("Sync translations using Gemini AI"))
        .subcommand(
            Command::new("create-release")
                .about("Create a new release")
                .arg(
                    clap::Arg::new("version")
                        .help("Release version (e.g., v1.0.0)")
                        .required(false)
                        .index(1),
                )
                .arg(
                    clap::Arg::new("message")
                        .help("Release message")
                        .required(false)
                        .index(2),
                ),
        )
        .subcommand(
            Command::new("code-quality")
                .about("Run code quality checks (linting, formatting, security)")
                .arg(
                    clap::Arg::new("path")
                        .short('p')
                        .long("path")
                        .value_name("PATH")
                        .help("Project path to analyze")
                        .required(false),
                )
                .arg(
                    clap::Arg::new("config")
                        .short('c')
                        .long("config")
                        .value_name("FILE")
                        .help("Custom config file path")
                        .required(false),
                )
                .arg(
                    clap::Arg::new("skip-deps")
                        .long("skip-deps")
                        .help("Skip dependency installation")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    clap::Arg::new("checks")
                        .long("checks")
                        .value_name("LIST")
                        .help("Enable specific checks only (comma-separated)")
                        .value_delimiter(',')
                        .required(false),
                ),
        )
        .subcommand(
            Command::new("github-labels")
                .about("Manage GitHub repository labels with emojis and categorization")
                .arg(
                    clap::Arg::new("skip-auth")
                        .long("skip-auth")
                        .help("Skip GitHub authentication check")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    clap::Arg::new("skip-install")
                        .long("skip-install")
                        .help("Skip GitHub CLI installation check")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    clap::Arg::new("dry-run")
                        .long("dry-run")
                        .help("Show what would be done without making changes")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    clap::Arg::new("list-only")
                        .long("list-only")
                        .help("Only list current labels, don't make changes")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    clap::Arg::new("delete-all")
                        .long("delete-all")
                        .help("Delete all existing labels before creating new ones")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    clap::Arg::new("update-only")
                        .long("update-only")
                        .help("Only update existing labels, don't create new ones")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("version")
                .about("Manage project versioning")
                .subcommand(Command::new("patch").about("Bump patch version"))
                .subcommand(Command::new("minor").about("Bump minor version"))
                .subcommand(Command::new("major").about("Bump major version"))
                .subcommand(Command::new("show").about("Show current version"))
                .subcommand(Command::new("history").about("Show version history")),
        )
        .subcommand(
            Command::new("config")
                .about("Manage configuration settings")
                .subcommand(Command::new("show").about("Show current configuration"))
                .subcommand(Command::new("setup").about("Setup configuration"))
                .subcommand(Command::new("reset").about("Reset configuration")),
        );

    let matches = app.try_get_matches();

    match matches {
        Ok(matches) => match matches.subcommand() {
            Some(("create-release", sub_matches)) => {
                if let Some(version) = sub_matches.get_one::<String>("version") {
                    let message = sub_matches.get_one::<String>("message").map(|s| s.as_str());
                    if let Err(e) =
                        commands::create_release::create_release_with_args(version, message).await
                    {
                        eprintln!("{}", format!("❌ Release creation failed: {}", e).red());
                        std::process::exit(1);
                    }
                } else if let Err(e) = commands::create_release::create_release_interactive().await
                {
                    eprintln!("{}", format!("❌ Release creation failed: {}", e).red());
                    std::process::exit(1);
                }
            }
            Some(("release-notes", _)) => {
                println!("{}", "🔄 Generating release notes...".yellow());
                commands::release_notes::generate_release_notes();
            }
            Some(("update-dependencies", _)) => {
                println!("{}", "🔄 Analyzing and updating dependencies...".yellow());
                commands::dependency_update::update_dependencies();
            }
            Some(("sync-translations", _)) => {
                println!("{}", "🌍 Syncing translations...".yellow());
                if let Err(e) = commands::translation_sync::sync_translations_interactive().await {
                    eprintln!("{}", format!("❌ Translation sync failed: {}", e).red());
                    std::process::exit(1);
                }
            }
            Some(("code-quality", sub_matches)) => {
                let path = sub_matches.get_one::<String>("path").cloned();
                let config_path = sub_matches.get_one::<String>("config").cloned();
                let skip_deps = sub_matches.get_flag("skip-deps");
                let checks: Option<Vec<String>> = sub_matches
                    .get_many::<String>("checks")
                    .map(|vals| vals.cloned().collect());

                println!("{}", "🔍 Running code quality checks...".yellow());
                let mut quality_config = if let Some(config_file) = &config_path {
                    match tokio::fs::read_to_string(config_file).await {
                        Ok(content) => match serde_json::from_str(&content) {
                            Ok(config) => config,
                            Err(e) => {
                                eprintln!("{}", format!("❌ Failed to parse config file: {}", e).red());
                                std::process::exit(1);
                            }
                        },
                        Err(e) => {
                            eprintln!("{}", format!("❌ Failed to read config file: {}", e).red());
                            std::process::exit(1);
                        }
                    }
                } else {
                    commands::code_quality::CodeQualityConfig::default()
                };

                if skip_deps {
                    quality_config.skip_dependencies = true;
                }

                if let Some(check_list) = checks {
                    quality_config.enabled_checks = check_list;
                }

                if let Err(e) = commands::code_quality::run_code_quality(path, config_path).await {
                    eprintln!("{}", format!("❌ Code quality checks failed: {}", e).red());
                    std::process::exit(1);
                }
            }
            Some(("github-labels", sub_matches)) => {
                let skip_auth = sub_matches.get_flag("skip-auth");
                let skip_install = sub_matches.get_flag("skip-install");
                let dry_run = sub_matches.get_flag("dry-run");
                let list_only = sub_matches.get_flag("list-only");
                let delete_all = sub_matches.get_flag("delete-all");
                let update_only = sub_matches.get_flag("update-only");

                if let Err(e) = commands::github_labels::run_github_labels(
                    skip_auth, skip_install, dry_run, list_only, delete_all, update_only
                ).await {
                    eprintln!("{}", format!("❌ GitHub labels management failed: {}", e).red());
                    std::process::exit(1);
                }
            }
            Some(("version", sub_matches)) => match sub_matches.subcommand() {
                Some(("patch", _)) => {
                    println!("{}", "🔄 Bumping patch version...".yellow());
                    if let Err(e) =
                        commands::version_management::bump_and_release("patch", None).await
                    {
                        eprintln!(
                            "{}",
                            format!("❌ Failed to bump patch version: {}", e).red()
                        );
                        std::process::exit(1);
                    }
                }
                Some(("minor", _)) => {
                    println!("{}", "🔄 Bumping minor version...".yellow());
                    if let Err(e) =
                        commands::version_management::bump_and_release("minor", None).await
                    {
                        eprintln!(
                            "{}",
                            format!("❌ Failed to bump minor version: {}", e).red()
                        );
                        std::process::exit(1);
                    }
                }
                Some(("major", _)) => {
                    println!("{}", "🔄 Bumping major version...".yellow());
                    if let Err(e) =
                        commands::version_management::bump_and_release("major", None).await
                    {
                        eprintln!(
                            "{}",
                            format!("❌ Failed to bump major version: {}", e).red()
                        );
                        std::process::exit(1);
                    }
                }
                Some(("show", _)) => {
                    println!("{}", format!("Current version: v{}", VERSION).cyan().bold());
                }
                Some(("history", _)) => {
                    if let Err(e) = commands::version_management::show_version_history().await {
                        eprintln!(
                            "{}",
                            format!("❌ Failed to show version history: {}", e).red()
                        );
                        std::process::exit(1);
                    }
                }
                _ => {
                    println!("{}", format!("Current version: v{}", VERSION).cyan().bold());
                }
            },
            Some(("config", sub_matches)) => match sub_matches.subcommand() {
                Some(("show", _)) => {
                    if let Err(e) = commands::translation_sync::show_config().await {
                        eprintln!("{}", format!("❌ Failed to show config: {}", e).red());
                        std::process::exit(1);
                    }
                }
                Some(("setup", _)) => {
                    if let Err(e) = commands::translation_sync::setup_config().await {
                        eprintln!("{}", format!("❌ Failed to setup config: {}", e).red());
                        std::process::exit(1);
                    }
                }
                Some(("reset", _)) => {
                    if let Err(e) = commands::translation_sync::reset_config().await {
                        eprintln!("{}", format!("❌ Failed to reset config: {}", e).red());
                        std::process::exit(1);
                    }
                }
                _ => {
                    if let Err(e) = commands::translation_sync::show_config().await {
                        eprintln!("{}", format!("❌ Failed to show config: {}", e).red());
                        std::process::exit(1);
                    }
                }
            },
            _ => {
                run_interactive_mode().await;
            }
        },
        Err(_) => {
            run_interactive_mode().await;
        }
    }
}

async fn run_interactive_mode() {
    print_banner();
    let _ = utils::check_for_updates(VERSION, false).await;
    loop {
        show_menu();
        let input = get_user_input();
        match input.as_str() {
            "1" | "create-release" => {
                println!("{}", "\n🚀 Creating release...".yellow());
                if let Err(e) = commands::create_release::create_release_interactive().await {
                    println!("{}", format!("❌ Release creation failed: {}", e).red());
                }
                println!("\n{}", "Press Enter to continue...".dimmed());
                let _ = get_user_input();
            }
            "2" | "release-notes" => {
                println!("{}", "\n🔄 Generating release notes...".yellow());
                commands::release_notes::generate_release_notes();
                println!("\n{}", "Press Enter to continue...".dimmed());
                let _ = get_user_input();
            }
            "3" | "update-dependencies" => {
                println!("{}", "\n🔄 Analyzing and updating dependencies...".yellow());
                commands::dependency_update::update_dependencies();
                println!("\n{}", "Press Enter to continue...".dimmed());
                let _ = get_user_input();
            }
            "4" | "sync-translations" => {
                println!("{}", "\n🌍 Syncing translations...".yellow());
                if let Err(e) = commands::translation_sync::sync_translations_interactive().await {
                    println!("{}", format!("❌ Translation sync failed: {}", e).red());
                }
                println!("\n{}", "Press Enter to continue...".dimmed());
                let _ = get_user_input();
            }
            "5" | "code-quality" => {
                println!("{}", "\n🔍 Running code quality checks...".yellow());
                if let Err(e) = commands::code_quality::run_code_quality(None, None).await {
                    println!("{}", format!("❌ Code quality checks failed: {}", e).red());
                }
                println!("\n{}", "Press Enter to continue...".dimmed());
                let _ = get_user_input();
            }
            "6" | "github-labels" => {
                println!("{}", "\n🏷️ Managing GitHub labels...".yellow());
                if let Err(e) = commands::github_labels::run_github_labels_interactive().await {
                    println!("{}", format!("❌ GitHub labels management failed: {}", e).red());
                }
                println!("\n{}", "Press Enter to continue...".dimmed());
                let _ = get_user_input();
            }
            "7" | "config" => {
                println!("\n{}", "⚙️  Configuration Management".cyan().bold());
                println!("{}", "═".repeat(30).dimmed());
                println!("  {} Show current configuration", "1.".dimmed());
                println!("  {} Setup configuration", "2.".dimmed());
                println!("  {} Reset configuration", "3.".dimmed());
                print!("\n{}", "Select option (1-3): ".cyan());
                let config_input = get_user_input();
                match config_input.as_str() {
                    "1" | "show" => {
                        if let Err(e) = commands::translation_sync::show_config().await {
                            println!("{}", format!("❌ Failed to show config: {}", e).red());
                        }
                    }
                    "2" | "setup" => {
                        if let Err(e) = commands::translation_sync::setup_config().await {
                            println!("{}", format!("❌ Failed to setup config: {}", e).red());
                        }
                    }
                    "3" | "reset" => {
                        if let Err(e) = commands::translation_sync::reset_config().await {
                            println!("{}", format!("❌ Failed to reset config: {}", e).red());
                        }
                    }
                    _ => {
                        if let Err(e) = commands::translation_sync::show_config().await {
                            println!("{}", format!("❌ Failed to show config: {}", e).red());
                        }
                    }
                }
                println!("\n{}", "Press Enter to continue...".dimmed());
                let _ = get_user_input();
            }
            "8" | "version" => {
                println!("\n{}", "🏷️  Version Management".cyan().bold());
                println!("{}", "═".repeat(30).dimmed());
                println!("  {} Bump patch version (x.x.X)", "1.".dimmed());
                println!("  {} Bump minor version (x.X.0)", "2.".dimmed());
                println!("  {} Bump major version (X.0.0)", "3.".dimmed());
                println!("  {} Show current version", "4.".dimmed());
                println!("  {} Show version history", "5.".dimmed());
                print!("\n{}", "Select option (1-5): ".cyan());
                let version_input = get_user_input();
                match version_input.as_str() {
                    "1" | "patch" => {
                        if let Err(e) =
                            commands::version_management::bump_and_release("patch", None).await
                        {
                            println!(
                                "{}",
                                format!("❌ Failed to bump patch version: {}", e).red()
                            );
                        }
                    }
                    "2" | "minor" => {
                        if let Err(e) =
                            commands::version_management::bump_and_release("minor", None).await
                        {
                            println!(
                                "{}",
                                format!("❌ Failed to bump minor version: {}", e).red()
                            );
                        }
                    }
                    "3" | "major" => {
                        if let Err(e) =
                            commands::version_management::bump_and_release("major", None).await
                        {
                            println!(
                                "{}",
                                format!("❌ Failed to bump major version: {}", e).red()
                            );
                        }
                    }
                    "4" | "show" => {
                        println!(
                            "\n{}",
                            format!("Current version: v{}", VERSION).cyan().bold()
                        );
                    }
                    "5" | "history" => {
                        if let Err(e) = commands::version_management::show_version_history().await {
                            println!(
                                "{}",
                                format!("❌ Failed to show version history: {}", e).red()
                            );
                        }
                    }
                    _ => {
                        println!(
                            "\n{}",
                            format!("Current version: v{}", VERSION).cyan().bold()
                        );
                    }
                }
                println!("\n{}", "Press Enter to continue...".dimmed());
                let _ = get_user_input();
            }
            "9" | "help" => {
                println!(
                    "\n{}",
                    format!(
                        "❓ NITROKIT {} - Project Management Tool",
                        format!("v{}", VERSION).green().bold()
                    )
                    .cyan()
                    .bold()
                );
                println!("{}", "═".repeat(50).dimmed());
                println!();
                println!("{}", "Available Commands:".yellow().bold());
                println!(
                    "  {} - Create a comprehensive release",
                    "🚀 create-release".green()
                );
                println!(
                    "  {} - Generate comprehensive release notes from git history",
                    "📦 release-notes".green()
                );
                println!(
                    "  {} - Scan and update project dependencies",
                    "📝 update-dependencies".green()
                );
                println!(
                    "  {} - Sync translations using Gemini AI",
                    "🌍 sync-translations".green()
                );
                println!(
                    "  {} - Run code quality checks (lint, format, security)",
                    "🔍 code-quality".green()
                );
                println!("  {} - Manage GitHub repository labels", "🏷️ github-labels".green());
                println!("  {} - Manage configuration settings", "⚙️  config".blue());
                println!("  {} - Manage project versioning", "🏷️  version".blue());
                println!("  {} - Show this help information", "❓ help".blue());
                println!("  {} - Exit the application", "🚪 exit".red());
                println!();
                println!("{}", "Usage Examples:".yellow().bold());
                println!(
                    "  {} nitroterm create-release v1.0.0",
                    "Create release:".dimmed()
                );
                println!("  {} nitroterm release-notes", "Direct command:".dimmed());
                println!(
                    "  {} nitroterm sync-translations",
                    "Sync translations:".dimmed()
                );
                println!(
                    "  {} nitroterm code-quality --path ./my-project",
                    "Code quality:".dimmed()
                );
                println!(
                    "  {} nitroterm github-labels --dry-run",
                    "GitHub labels:".dimmed()
                );
                println!("  {} nitroterm config show", "Config management:".dimmed());
                println!("  {} nitroterm version patch", "Version bump:".dimmed());
                println!(
                    "  {} nitroterm (then select option)",
                    "Interactive mode:".dimmed()
                );
                println!();
                println!(
                    "{}",
                    format!("Nitroterm v{} - Built with Rust 🦀", VERSION).dimmed()
                );
                println!("\n{}", "Press Enter to continue...".dimmed());
                let _ = get_user_input();
            }
            "0" | "exit" | "quit" | "q" => {
                println!(
                    "{}",
                    format!("\n👋 Thank you for using Nitroterm v{}!", VERSION).green()
                );
                break;
            }
            _ => {
                println!("{} {}", "❌ Unknown command:".red(), input.yellow());
                println!(
                    "{}",
                    "Please choose a valid option (1-9) or type the command name.".dimmed()
                );
                println!("{}", "Type 'help' for more information.".dimmed());
                println!();
            }
        }
    }
}
