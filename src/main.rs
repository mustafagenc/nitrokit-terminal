use clap::Command;
use colored::*;
use std::io::{self, Write};

mod commands;
mod utils;

#[cfg(test)]
mod tests;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_banner() {
    let banner = r#"
    ███╗   ██╗██╗████████╗██████╗  ██████╗ ██╗  ██╗██╗████████╗
    ████╗  ██║██║╚══██╔══╝██╔══██╗██╔═══██╗██║ ██╔╝██║╚══██╔══╝
    ██╔██╗ ██║██║   ██║   ██████╔╝██║   ██║█████╔╝ ██║   ██║   
    ██║╚██╗██║██║   ██║   ██╔══██╗██║   ██║██╔═██╗ ██║   ██║   
    ██║ ╚████║██║   ██║   ██║  ██║╚██████╔╝██║  ██╗██║   ██║   
    ╚═╝  ╚═══╝╚═╝   ╚═╝   ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝   ╚═╝   
    "#;

    #[cfg(windows)]
    {
        extern "system" {
            fn SetConsoleTitleW(title: *const u16) -> i32;
        }
        
        // UTF-16 string oluştur
        let title = "🚀 Nitrokit Terminal Tool\0"
            .encode_utf16()
            .collect::<Vec<u16>>();
        
        unsafe {
            SetConsoleTitleW(title.as_ptr());
        }
    }

    println!("{}", banner.cyan().bold());
    println!(
        "{}",
        format!(
            "    A terminal tool for project management and automation {}",
            format!("v{}", VERSION).green().bold()
        )
        .dimmed()
    );
    println!(
        "{}",
        "    Developed by Mustafa Genc <eposta@mustafagenc.info>".dimmed()
    );
    println!(
        "{}",
        "    https://nitrokit.tr".dimmed()
    );
    
    println!();
}

fn show_menu() {
    println!("{}", "Available commands:".yellow().bold());
    println!(
        "  {} {}",
        "1. 🚀 create-release".green(),
        "Create a new release"
    );
    println!(
        "  {} {}",
        "2. 📦 release-notes".green(),
        "Generate release notes from git commits"
    );
    println!(
        "  {} {}",
        "3. 📝 update-dependencies".green(),
        "Analyze and update project dependencies"
    );
    println!(
        "  {} {}",
        "4. 🌍 sync-translations".green(),
        "Sync translations using Gemini AI"
    );
    println!(
        "  {} {}",
        "5. ⚙️ config".blue(),
        "Manage configuration settings"
    );
    println!(
        "  {} {}",
        "6. 🏷️ version".blue(),
        "Manage project versioning"
    );
    println!("  {} {}", "7. ❓ help".blue(), "Show this help menu");
    println!("  {} {}", "8. 🚪 exit".red(), "Exit Nitrokit");
    println!();
}

fn get_user_input() -> String {
    print!("{}", "nitrokit> ".cyan().bold());
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let app = Command::new("nitrokit")
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
            Command::new("version")
                .about("Manage project versioning")
                .subcommand(Command::new("patch").about("Bump patch version"))
                .subcommand(Command::new("minor").about("Bump minor version"))
                .subcommand(Command::new("major").about("Bump major version"))
                .subcommand(Command::new("show").about("Show current version"))
                .subcommand(Command::new("history").about("Show version history"))
        )
        .subcommand(
            Command::new("config")
                .about("Manage configuration settings")
                .subcommand(Command::new("show").about("Show current configuration"))
                .subcommand(Command::new("setup").about("Setup configuration"))
                .subcommand(Command::new("reset").about("Reset configuration"))
        );

    let matches = app.try_get_matches();

    match matches {
        Ok(matches) => match matches.subcommand() {
            Some(("create-release", sub_matches)) => {
                if let Some(version) = sub_matches.get_one::<String>("version") {
                    let message = sub_matches.get_one::<String>("message").map(|s| s.as_str());
                    if let Err(e) = commands::create_release::create_release_with_args(version, message).await {
                        eprintln!("{}", format!("❌ Release creation failed: {}", e).red());
                        std::process::exit(1);
                    }
                } else {
                    if let Err(e) = commands::create_release::create_release_interactive().await {
                        eprintln!("{}", format!("❌ Release creation failed: {}", e).red());
                        std::process::exit(1);
                    }
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
            Some(("version", sub_matches)) => {
                match sub_matches.subcommand() {
                    Some(("patch", _)) => {
                        println!("{}", "🔄 Bumping patch version...".yellow());
                        if let Err(e) = commands::version_management::bump_and_release("patch", None).await {
                            eprintln!("{}", format!("❌ Failed to bump patch version: {}", e).red());
                            std::process::exit(1);
                        }
                    }
                    Some(("minor", _)) => {
                        println!("{}", "🔄 Bumping minor version...".yellow());
                        if let Err(e) = commands::version_management::bump_and_release("minor", None).await {
                            eprintln!("{}", format!("❌ Failed to bump minor version: {}", e).red());
                            std::process::exit(1);
                        }
                    }
                    Some(("major", _)) => {
                        println!("{}", "🔄 Bumping major version...".yellow());
                        if let Err(e) = commands::version_management::bump_and_release("major", None).await {
                            eprintln!("{}", format!("❌ Failed to bump major version: {}", e).red());
                            std::process::exit(1);
                        }
                    }
                    Some(("show", _)) => {
                        println!("{}", format!("Current version: v{}", VERSION).cyan().bold());
                    }
                    Some(("history", _)) => {
                        if let Err(e) = commands::version_management::show_version_history().await {
                            eprintln!("{}", format!("❌ Failed to show version history: {}", e).red());
                            std::process::exit(1);
                        }
                    }
                    _ => {
                        println!("{}", format!("Current version: v{}", VERSION).cyan().bold());
                    }
                }
            }
            Some(("config", sub_matches)) => {
                match sub_matches.subcommand() {
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
                }
            }
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
            "5" | "config" => {
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
            "6" | "version" => {
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
                        if let Err(e) = commands::version_management::bump_and_release("patch", None).await {
                            println!("{}", format!("❌ Failed to bump patch version: {}", e).red());
                        }
                    }
                    "2" | "minor" => {
                        if let Err(e) = commands::version_management::bump_and_release("minor", None).await {
                            println!("{}", format!("❌ Failed to bump minor version: {}", e).red());
                        }
                    }
                    "3" | "major" => {
                        if let Err(e) = commands::version_management::bump_and_release("major", None).await {
                            println!("{}", format!("❌ Failed to bump major version: {}", e).red());
                        }
                    }
                    "4" | "show" => {
                        println!("\n{}", format!("Current version: v{}", VERSION).cyan().bold());
                    }
                    "5" | "history" => {
                        if let Err(e) = commands::version_management::show_version_history().await {
                            println!("{}", format!("❌ Failed to show version history: {}", e).red());
                        }
                    }
                    _ => {
                        println!("\n{}", format!("Current version: v{}", VERSION).cyan().bold());
                    }
                }
                println!("\n{}", "Press Enter to continue...".dimmed());
                let _ = get_user_input();
            }
            "7" | "help" => {
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
                println!("  {} - Manage configuration settings", "⚙️  config".blue());
                println!("  {} - Manage project versioning", "🏷️  version".blue());
                println!("  {} - Show this help information", "❓ help".blue());
                println!("  {} - Exit the application", "🚪 exit".red());
                println!();
                println!("{}", "Usage Examples:".yellow().bold());
                println!(
                    "  {} {}",
                    "Create release:".dimmed(),
                    "nitrokit create-release v1.0.0"
                );
                println!(
                    "  {} {}",
                    "Direct command:".dimmed(),
                    "nitrokit release-notes"
                );
                println!(
                    "  {} {}",
                    "Sync translations:".dimmed(),
                    "nitrokit sync-translations"
                );
                println!(
                    "  {} {}",
                    "Config management:".dimmed(),
                    "nitrokit config show"
                );
                println!(
                    "  {} {}",
                    "Version bump:".dimmed(),
                    "nitrokit version patch"
                );
                println!(
                    "  {} {}",
                    "Interactive mode:".dimmed(),
                    "nitrokit (then select option)"
                );
                println!("  {} {}", "Version info:".dimmed(), "nitrokit --version");
                println!(
                    "  {} {}",
                    "Check updates:".dimmed(),
                    "nitrokit check-updates"
                );
                println!();
                println!(
                    "{}",
                    format!("NitroKit v{} - Built with Rust 🦀", VERSION).dimmed()
                );
                println!("\n{}", "Press Enter to continue...".dimmed());
                let _ = get_user_input();
            }
            "8" | "exit" | "quit" | "q" => {
                println!(
                    "{}",
                    format!("\n👋 Thank you for using Nitrokit v{}!", VERSION).green()
                );
                break;
            }
            // ...existing code...
            _ => {
                println!("{} {}", "❌ Unknown command:".red(), input.yellow());
                println!(
                    "{}",
                    "Please choose a valid option (1-8) or type the command name.".dimmed()
                );
                println!("{}", "Type 'help' for more information.".dimmed());
                println!();
            }
        }
    }
}
