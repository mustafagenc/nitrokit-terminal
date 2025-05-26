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
        "5. ⚙️  config".blue(),
        "Manage configuration settings"
    );
    println!("  {} {}", "6. ❓ help".blue(), "Show this help menu");
    println!("  {} {}", "7. 🚪 exit".red(), "Exit Nitrokit");
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
        );

    let matches = app.try_get_matches();

    match matches {
        Ok(matches) => match matches.subcommand() {
            Some(("create-release", sub_matches)) => {
                if let Some(version) = sub_matches.get_one::<String>("version") {
                    let message = sub_matches.get_one::<String>("message").map(|s| s.as_str());
                    if let Err(e) = commands::create_release_with_args(version, message).await {
                        eprintln!("{}", format!("❌ Release creation failed: {}", e).red());
                        std::process::exit(1);
                    }
                } else {
                    if let Err(e) = commands::create_release_interactive().await {
                        eprintln!("{}", format!("❌ Release creation failed: {}", e).red());
                        std::process::exit(1);
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
                if let Err(e) = commands::create_release_interactive().await {
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
            "6" | "help" => {
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
            "7" | "exit" | "quit" | "q" => {
                println!(
                    "{}",
                    format!("\n👋 Thank you for using Nitrokit v{}!", VERSION).green()
                );
                break;
            }
            "version" | "--version" | "-v" => {
                println!("\n{}", format!("Nitrokit v{}", VERSION).cyan().bold());
                println!(
                    "{}",
                    "A terminal tool for project management and automation".dimmed()
                );
                println!("{}", "Built with Rust 🦀".dimmed());
                println!("\n{}", "Press Enter to continue...".dimmed());
                let _ = get_user_input();
            }
            "check-updates" | "update" => {
                println!("{}", "\n🔍 Checking for updates...".yellow());
                if let Err(e) = utils::check_for_updates(VERSION, true).await {
                    println!("{}", format!("❌ Failed to check for updates: {}", e).red());
                }
                println!("\n{}", "Press Enter to continue...".dimmed());
                let _ = get_user_input();
            }
            "" => {
                // Empty input, just continue
                continue;
            }
            _ => {
                println!("{} {}", "❌ Unknown command:".red(), input.yellow());
                println!(
                    "{}",
                    "Please choose a valid option (1-7) or type the command name.".dimmed()
                );
                println!("{}", "Type 'help' for more information.".dimmed());
                println!();
            }
        }
    }
}
