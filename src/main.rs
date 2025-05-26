use clap::Command;
use colored::*;
use std::io::{self, Write};

mod commands;
mod utils;

#[cfg(test)]
mod tests;

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
    println!("{}", "    A terminal tool for project management and automation".dimmed());
    println!("{}", "    Developed by Mustafa Genc <eposta@mustafagenc.info>".dimmed());
    println!();
}

fn show_menu() {
    println!("{}", "Available commands:".yellow().bold());
    println!("  {} {}", "1. release-notes".green(), "Generate release notes from git commits");
    println!("  {} {}", "2. update-dependencies".green(), "Analyze and update project dependencies");
    println!("  {} {}", "3. help".blue(), "Show this help menu");
    println!("  {} {}", "4. exit".red(), "Exit Nitrokit");
    println!();
}

fn get_user_input() -> String {
    print!("{}", "nitrokit> ".cyan().bold());
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn main() {
    let app = Command::new("nitrokit")
        .version("0.1.0")
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
                    run_interactive_mode();
                }
                _ => {
                    // No subcommand, run interactive mode by default
                    run_interactive_mode();
                }
            }
        }
        Err(_) => {
            // Invalid command, run interactive mode
            run_interactive_mode();
        }
    }
}

fn run_interactive_mode() {
    print_banner();
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
                println!("\n{}", "NITROKIT - Project Management Tool".cyan().bold());
                println!("{}", "═".repeat(40).dimmed());
                println!();
                println!("{}", "Available Commands:".yellow().bold());
                println!("  {} - Generate comprehensive release notes from git history", "release-notes".green());
                println!("  {} - Scan and update project dependencies", "update-dependencies".green());
                println!("  {} - Show this help information", "help".blue());
                println!("  {} - Exit the application", "exit".red());
                println!();
                println!("{}", "Usage Examples:".yellow().bold());
                println!("  {} {}", "Direct command:".dimmed(), "nitrokit release-notes");
                println!("  {} {}", "Interactive mode:".dimmed(), "nitrokit (then select option)");
                println!();
                println!("{}", "Press Enter to continue...".dimmed());
                let _ = get_user_input();
            }
            "4" | "exit" | "quit" | "q" => {
                println!("{}", "\n👋 Thank you for using Nitrokit!".green());
                break;
            }
            "" => {
                // Empty input, just continue
                continue;
            }
            _ => {
                println!("{} {}", "❌ Unknown command:".red(), input.yellow());
                println!("{}", "Please choose a valid option (1-4) or type the command name.".dimmed());
                println!();
            }
        }
    }
}