use clap::{Arg, Command};
use nitrokit::commands::{generate_release_notes, update_dependencies};
use colored::*;
use std::io::{self, Write};

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

fn wait_for_continue() {
    println!();
    println!("{}", "─".repeat(60).dimmed());
    print!("{}", "Press Enter to continue...".dimmed());
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
}

fn interactive_mode() {
    loop {
        print_banner();
        show_menu();
        
        print!("{}", "nitrokit> ".cyan().bold());
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim().to_lowercase();
                
                match input.as_str() {
                    "1" | "release-notes" => {
                        println!();
                        generate_release_notes();
                        wait_for_continue();
                    }
                    "2" | "update-dependencies" => {
                        println!();
                        update_dependencies();
                        wait_for_continue();
                    }
                    "3" | "help" => {
                        // Menu zaten gösteriliyor, sadece devam et
                        continue;
                    }
                    "4" | "exit" | "quit" | "q" => {
                        println!("{}", "👋 Thank you for using Nitrokit!".green());
                        break;
                    }
                    "" => {
                        // Enter tuşuna basıldı, menüyü tekrar göster
                        continue;
                    }
                    _ => {
                        println!("{}", "❌ Unknown command. Please try again.".red());
                        println!();
                        wait_for_continue();
                    }
                }
            }
            Err(e) => {
                println!("{}", format!("❌ Error reading input: {}", e).red());
                break;
            }
        }
        
        // Ekranı temizle (Windows ve Unix uyumlu)
        if cfg!(target_os = "windows") {
            std::process::Command::new("cls").status().ok();
        } else {
            std::process::Command::new("clear").status().ok();
        }
    }
}

fn main() {
    let matches = Command::new("nitrokit")
        .version("0.1.0")
        .author("Nitrokit Team")
        .about("A terminal tool for project management and automation")
        .arg(
            Arg::new("interactive")
                .short('i')
                .long("interactive")
                .action(clap::ArgAction::SetTrue)
                .help("Run in interactive mode")
        )
        .subcommand(
            Command::new("release-notes")
                .about("Generate release notes from git commits")
                .arg(
                    Arg::new("version")
                        .short('v')
                        .long("version")
                        .value_name("VERSION")
                        .help("Specify the version for release notes")
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("FILE")
                        .help("Output file path (default: RELEASE_NOTES.md)")
                )
                .arg(
                    Arg::new("commits")
                        .short('c')
                        .long("commits")
                        .value_name("COUNT")
                        .help("Number of commits to include (default: 20)")
                )
        )
        .subcommand(
            Command::new("update-dependencies")
                .about("Analyze and update project dependencies")
                .arg(
                    Arg::new("check-only")
                        .long("check-only")
                        .action(clap::ArgAction::SetTrue)
                        .help("Only check dependencies, don't update")
                )
                .arg(
                    Arg::new("force")
                        .short('f')
                        .long("force")
                        .action(clap::ArgAction::SetTrue)
                        .help("Force update all dependencies")
                )
        )
        .get_matches();

    // Interactive mode kontrolü
    if matches.get_flag("interactive") {
        interactive_mode();
        return;
    }

    match matches.subcommand() {
        Some(("release-notes", sub_matches)) => {
            print_banner();
            
            // Gelecekte argümanları kullanabilmek için
            let _version = sub_matches.get_one::<String>("version");
            let _output = sub_matches.get_one::<String>("output");
            let _commits = sub_matches.get_one::<String>("commits");
            
            generate_release_notes();
        }
        Some(("update-dependencies", sub_matches)) => {
            print_banner();
            
            let check_only = sub_matches.get_flag("check-only");
            let _force = sub_matches.get_flag("force");
            
            if check_only {
                println!("{}", "🔍 Check-only mode: analyzing dependencies without updating...".yellow());
            }
            
            update_dependencies();
        }
        _ => {
            // Hiçbir argüman verilmemişse interactive mode'a geç
            interactive_mode();
        }
    }
}