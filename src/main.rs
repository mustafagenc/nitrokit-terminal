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
    println!("{}", format!("    A terminal tool for project management and automation {}", format!("v{}", VERSION).green().bold()).dimmed());
    println!("{}", "    Developed by Mustafa Genc <eposta@mustafagenc.info>".dimmed());
    println!();
}

fn show_menu() {
    println!("{}", "Available commands:".yellow().bold());
    println!("  {} {}", "1. 📦 release-notes".green(), "Generate release notes from git commits");
    println!("  {} {}", "2. 📝 update-dependencies".green(), "Analyze and update project dependencies");
    println!("  {} {}", "3. ❓ help".blue(), "Show this help menu");
    println!("  {} {}", "4. 🚪 exit".red(), "Exit Nitrokit");
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
    let app = Command::new("nitrokit")
        .version(VERSION)
        .about("A terminal tool for project management and automation")
        .author("Mustafa Genc <eposta@mustafagenc.info>")
        .subcommand(
            Command::new("release-notes")
                .about("Generate release notes from git commits")
        )
        .subcommand(
            Command::new("update-dependencies")
                .about("Analyze and update project dependencies")
        )
        .subcommand(
            Command::new("interactive")
                .about("Run in interactive mode")
        )
        .subcommand(
            Command::new("check-updates")
                .about("Check for available updates")
        );

    let matches = app.try_get_matches();

    match matches {
        Ok(matches) => {
            match matches.subcommand() {
                Some(("release-notes", _)) => {
                    commands::release_notes::generate_release_notes();
                }
                Some(("update-dependencies", _)) => {
                    commands::dependency_update::update_dependencies();
                }
                Some(("interactive", _)) => {
                    run_interactive_mode().await;
                }
                Some(("check-updates", _)) => {
                    if let Err(e) = utils::check_for_updates(VERSION, true).await {
                        println!("{}", format!("❌ Failed to check for updates: {}", e).red());
                    }
                }
                _ => {
                    // No subcommand, run interactive mode by default
                    run_interactive_mode().await;
                }
            }
        }
        Err(_) => {
            // Invalid command, run interactive mode
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
            "1" | "release-notes" => {
                println!("{}", "\n🔄 Generating release notes...".yellow());
                commands::release_notes::generate_release_notes();
                println!("\n{}", "Press Enter to continue...".dimmed());
                let _ = get_user_input();
            }
            "2" | "update-dependencies" => {
                println!("{}", "\n🔄 Analyzing and updating dependencies...".yellow());
                commands::dependency_update::update_dependencies();
                println!("\n{}", "Press Enter to continue...".dimmed());
                let _ = get_user_input();
            }
            "3" | "help" => {
                println!("\n{}", format!("❓ NITROKIT {} - Project Management Tool", format!("v{}", VERSION).green().bold()).cyan().bold());
                println!("{}", "═".repeat(50).dimmed());
                println!();
                println!("{}", "Available Commands:".yellow().bold());
                println!("  {} - Generate comprehensive release notes from git history", "📦 release-notes".green());
                println!("  {} - Scan and update project dependencies", "📝 update-dependencies".green());
                println!("  {} - Show this help information", "❓ help".blue());
                println!("  {} - Exit the application", "🚪 exit".red());
                println!();
                println!("{}", "Usage Examples:".yellow().bold());
                println!("  {} {}", "Direct command:".dimmed(), "nitrokit release-notes");
                println!("  {} {}", "Interactive mode:".dimmed(), "nitrokit (then select option)");
                println!("  {} {}", "Version info:".dimmed(), "nitrokit --version");
                println!("  {} {}", "Check updates:".dimmed(), "nitrokit check-updates");
                println!();
                println!("{}", format!("NitroKit v{} - Built with Rust 🦀", VERSION).dimmed());
                println!("{}", "Press Enter to continue...".dimmed());
                let _ = get_user_input();
            }
            "4" | "exit" | "quit" | "q" => {
                println!("{}", format!("\n👋 Thank you for using Nitrokit v{}!", VERSION).green());
                break;
            }
            "version" | "--version" | "-v" => {
                utils::show_version_info(VERSION);
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
                println!("{}", "Please choose a valid option (1-4) or type the command name.".dimmed());
                println!("{}", "Type 'version' or 'check-updates' for more options.".dimmed());
                println!();
            }
        }
    }
}