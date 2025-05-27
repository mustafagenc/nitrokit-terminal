use crate::utils::{log_error, log_info, log_success, log_warning, read_file_to_string};
use chrono::{DateTime, Local};
use colored::*;
use serde_json::Value;
use std::fs;
use std::process::Command;
use std::time::Instant;

pub fn update_dependencies() {
    log_info("Scanning for dependency files...");

    let project_files = find_project_files();

    if project_files.is_empty() {
        log_warning("No dependency files found in current directory");
        return;
    }

    for file in project_files {
        log_info(&format!("Analyzing: {}", file));

        match file.as_str() {
            "package.json" => {
                analyze_package_json();
                update_node_dependencies();
            }
            "Cargo.toml" => {
                analyze_cargo_toml();
                update_cargo_dependencies();
            }
            "requirements.txt" => {
                analyze_requirements_txt();
                update_pip_dependencies();
            }
            "composer.json" => {
                analyze_composer_json();
                update_composer_dependencies();
            }
            _ => {
                log_warning(&format!("Unknown file type: {}", file));
            }
        }
    }

    println!();
    println!(
        "{}",
        "🎉 All dependency operations completed successfully!"
            .green()
            .bold()
    );
    log_success("Dependency analysis and update completed!");
}

fn find_project_files() -> Vec<String> {
    let mut files = Vec::new();

    // Package.json (Node.js/npm/yarn/pnpm)
    if crate::utils::file_exists("package.json") {
        files.push("package.json".to_string());
    }

    // Cargo.toml (Rust)
    if crate::utils::file_exists("Cargo.toml") {
        files.push("Cargo.toml".to_string());
    }

    // requirements.txt (Python)
    if crate::utils::file_exists("requirements.txt") {
        files.push("requirements.txt".to_string());
    }

    // composer.json (PHP)
    if crate::utils::file_exists("composer.json") {
        files.push("composer.json".to_string());
    }

    files
}

fn detect_node_package_manager() -> Option<String> {
    // Check for lock files to determine package manager
    if crate::utils::file_exists("pnpm-lock.yaml") {
        Some("pnpm".to_string())
    } else if crate::utils::file_exists("yarn.lock") {
        Some("yarn".to_string())
    } else if crate::utils::file_exists("package-lock.json") {
        Some("npm".to_string())
    } else {
        // Default to checking which package managers are available
        for pm in &["pnpm", "yarn", "npm"] {
            if is_command_available(pm) {
                return Some(pm.to_string());
            }
        }
        None
    }
}

fn is_command_available(command: &str) -> bool {
    // Windows için .cmd uzantısı da kontrol et
    if cfg!(target_os = "windows") {
        let cmd_version = format!("{}.cmd", command);
        Command::new(command).arg("--version").output().is_ok()
            || Command::new(&cmd_version).arg("--version").output().is_ok()
    } else {
        Command::new(command).arg("--version").output().is_ok()
    }
}

fn update_node_dependencies() {
    log_info("Detecting Node.js package manager...");

    match detect_node_package_manager() {
        Some(pm) => {
            log_info(&format!("Using package manager: {}", pm.cyan().bold()));
            // Backup lock files before updating
            backup_lock_files(&pm);
            match pm.as_str() {
                "pnpm" => update_pnpm_dependencies(),
                "yarn" => update_yarn_dependencies(),
                "npm" => update_npm_dependencies(),
                _ => log_warning("Unknown package manager detected"),
            }
        }
        None => {
            log_warning("No Node.js package manager found (npm, yarn, or pnpm)");
        }
    }
}

fn backup_lock_files(package_manager: &str) {
    log_info("Creating backup of lock files...");
    let now: DateTime<Local> = Local::now();
    let backup_dir = format!("./backup/{}", now.format("%Y%m%d%H%M%S"));
    // Create backup directory
    if let Err(e) = fs::create_dir_all(&backup_dir) {
        log_error(&format!("Failed to create backup directory: {}", e));
        return;
    }
    let mut backed_up_files = Vec::new();
    // Backup package.json first
    if crate::utils::file_exists("package.json") {
        if let Err(e) = fs::copy("package.json", format!("{}/package.json", backup_dir)) {
            log_warning(&format!("Failed to backup package.json: {}", e));
        } else {
            backed_up_files.push("package.json".to_string());
        }
    }
    // Backup lock files based on package manager
    let lock_files = match package_manager {
        "pnpm" => vec!["pnpm-lock.yaml"],
        "yarn" => vec!["yarn.lock"],
        "npm" => vec!["package-lock.json"],
        _ => vec!["package-lock.json", "yarn.lock", "pnpm-lock.yaml"], // Backup all if unknown
    };
    for lock_file in lock_files {
        if crate::utils::file_exists(lock_file) {
            match fs::copy(lock_file, format!("{}/{}", backup_dir, lock_file)) {
                Ok(_) => {
                    backed_up_files.push(lock_file.to_string());
                }
                Err(e) => {
                    log_warning(&format!("Failed to backup {}: {}", lock_file, e));
                }
            }
        }
    }
    if !backed_up_files.is_empty() {
        log_success(&format!(
            "Backed up {} files to: {}",
            backed_up_files.len(),
            backup_dir.cyan()
        ));
        println!("  📁 Backed up files:");
        for file in backed_up_files {
            println!("    ✓ {}", file.green());
        }
        println!();
    } else {
        log_warning("No files were backed up");
        // Remove empty backup directory
        let _ = fs::remove_dir(&backup_dir);
    }
}

fn analyze_package_json() {
    match read_file_to_string("package.json") {
        Ok(content) => match serde_json::from_str::<Value>(&content) {
            Ok(package_json) => {
                println!("{}", "📦 Node.js Dependencies:".blue().bold());

                if let Some(deps) = package_json["dependencies"].as_object() {
                    for (name, version) in deps {
                        println!(
                            "  {} -> {}",
                            name.green(),
                            version.as_str().unwrap_or("unknown")
                        );
                    }
                }

                if let Some(dev_deps) = package_json["devDependencies"].as_object() {
                    println!("{}", "🔧 Dev Dependencies:".yellow().bold());
                    for (name, version) in dev_deps {
                        println!(
                            "  {} -> {}",
                            name.green(),
                            version.as_str().unwrap_or("unknown")
                        );
                    }
                }
            }
            Err(e) => {
                log_error(&format!("Failed to parse package.json: {}", e));
            }
        },
        Err(e) => {
            log_error(&format!("Failed to read package.json: {}", e));
        }
    }
}

fn update_yarn_dependencies() {
    log_info("Updating yarn dependencies...");

    // Check yarn availability
    let yarn_available = if cfg!(target_os = "windows") {
        Command::new("yarn").arg("--version").output().is_ok()
            || Command::new("yarn.cmd").arg("--version").output().is_ok()
    } else {
        Command::new("yarn").arg("--version").output().is_ok()
    };

    if !yarn_available {
        log_warning("yarn not found. Trying alternative methods...");

        // npm üzerinden yarn kontrol et
        if Command::new("npx")
            .arg("yarn")
            .arg("--version")
            .output()
            .is_ok()
        {
            log_info("Found yarn via npx, using npx yarn...");
            update_yarn_via_npx();
            return;
        }

        log_error("yarn not found in PATH. Please ensure yarn is installed and available in PATH.");
        log_info("You can install yarn via:");
        println!("  {} npm install -g yarn", "npm:".cyan());
        println!("  {} choco install yarn", "chocolatey:".cyan());
        println!("  {} scoop install yarn", "scoop:".cyan());
        return;
    }

    let yarn_cmd = if cfg!(target_os = "windows")
        && Command::new("yarn.cmd").arg("--version").output().is_ok()
    {
        "yarn.cmd"
    } else {
        "yarn"
    };

    log_info(&format!("Using yarn command: {}", yarn_cmd.green()));

    if let Ok(output) = Command::new(yarn_cmd).arg("--version").output() {
        let version = String::from_utf8_lossy(&output.stdout);
        log_info(&format!("Yarn version: {}", version.trim().cyan()));
    }

    log_info("Running yarn upgrade...");
    print!("{}", "⏳ Upgrading packages...".yellow());
    std::io::Write::flush(&mut std::io::stdout()).unwrap();

    let start_time = Instant::now();
    match Command::new(yarn_cmd).arg("upgrade").output() {
        Ok(output) => {
            let duration = start_time.elapsed();
            println!(
                " {}",
                format!("✅ Completed in {:.2}s", duration.as_secs_f64()).green()
            );

            if output.status.success() {
                log_success("yarn dependencies updated successfully!");

                let stdout_str = String::from_utf8_lossy(&output.stdout);
                if !stdout_str.trim().is_empty() {
                    println!("{}", "📊 Update output:".cyan().bold());
                    println!("{}", stdout_str);
                }

                // Check for outdated packages
                log_info("Checking for outdated packages...");
                print!("{}", "⏳ Scanning for outdated packages...".yellow());
                std::io::Write::flush(&mut std::io::stdout()).unwrap();

                let start_time = Instant::now();
                match Command::new(yarn_cmd).arg("outdated").output() {
                    Ok(outdated_output) => {
                        let duration = start_time.elapsed();
                        println!(
                            " {}",
                            format!("✅ Completed in {:.2}s", duration.as_secs_f64()).green()
                        );

                        let outdated_str = String::from_utf8_lossy(&outdated_output.stdout);
                        if !outdated_str.trim().is_empty() {
                            println!("{}", "📊 Outdated packages:".yellow().bold());
                            println!("{}", outdated_str);
                        } else {
                            log_success("All yarn packages are up to date!");
                        }
                    }
                    Err(e) => {
                        println!(" {}", "❌ Failed".red());
                        log_warning(&format!("Could not check outdated packages: {}", e));
                    }
                }

                // Show yarn audit for security
                log_info("Running security audit...");
                print!("{}", "⏳ Running security audit...".yellow());
                std::io::Write::flush(&mut std::io::stdout()).unwrap();

                let start_time = Instant::now();
                match Command::new(yarn_cmd).arg("audit").output() {
                    Ok(audit_output) => {
                        let duration = start_time.elapsed();
                        println!(
                            " {}",
                            format!("✅ Completed in {:.2}s", duration.as_secs_f64()).green()
                        );

                        let audit_str = String::from_utf8_lossy(&audit_output.stdout);
                        if audit_str.contains("vulnerabilities") {
                            println!("{}", "🔒 Security audit:".red().bold());
                            println!("{}", audit_str);
                        } else {
                            log_success("No security vulnerabilities found!");
                        }
                    }
                    Err(e) => {
                        println!(" {}", "❌ Failed".red());
                        log_warning(&format!("Could not run security audit: {}", e));
                    }
                }
            } else {
                println!(" {}", "❌ Failed".red());
                let error_msg = String::from_utf8_lossy(&output.stderr);
                log_error(&format!("yarn upgrade failed: {}", error_msg));
            }
        }
        Err(e) => {
            println!(" {}", "❌ Failed".red());
            log_error(&format!("Failed to run yarn upgrade: {}", e));
        }
    }
}

fn update_yarn_via_npx() {
    log_info("Running yarn upgrade via npx...");
    match Command::new("npx").arg("yarn").arg("upgrade").output() {
        Ok(output) => {
            if output.status.success() {
                log_success("yarn dependencies updated successfully via npx!");

                let stdout_str = String::from_utf8_lossy(&output.stdout);
                if !stdout_str.trim().is_empty() {
                    println!("{}", "📊 Update output:".cyan().bold());
                    println!("{}", stdout_str);
                }
            } else {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                log_error(&format!("yarn upgrade via npx failed: {}", error_msg));
            }
        }
        Err(e) => {
            log_error(&format!("Failed to run yarn via npx: {}", e));
        }
    }
}

fn update_npm_dependencies() {
    log_info("Updating npm dependencies...");

    let npm_cmd = if cfg!(target_os = "windows")
        && Command::new("npm.cmd").arg("--version").output().is_ok()
    {
        "npm.cmd"
    } else {
        "npm"
    };

    if Command::new(npm_cmd).arg("--version").output().is_err() {
        log_warning("npm not found. Skipping npm update.");
        return;
    }

    log_info(&format!("Using npm command: {}", npm_cmd.green()));

    log_info("Running npm update...");
    print!("{}", "⏳ Updating packages...".yellow());
    std::io::Write::flush(&mut std::io::stdout()).unwrap();

    let start_time = Instant::now();
    match Command::new(npm_cmd).arg("update").output() {
        Ok(output) => {
            let duration = start_time.elapsed();
            println!(
                " {}",
                format!("✅ Completed in {:.2}s", duration.as_secs_f64()).green()
            );

            if output.status.success() {
                log_success("npm dependencies updated successfully!");

                // Check for outdated packages
                log_info("Checking for outdated packages...");
                print!("{}", "⏳ Scanning for outdated packages...".yellow());
                std::io::Write::flush(&mut std::io::stdout()).unwrap();

                let start_time = Instant::now();
                match Command::new(npm_cmd).arg("outdated").output() {
                    Ok(outdated_output) => {
                        let duration = start_time.elapsed();
                        println!(
                            " {}",
                            format!("✅ Completed in {:.2}s", duration.as_secs_f64()).green()
                        );

                        let outdated_str = String::from_utf8_lossy(&outdated_output.stdout);
                        if !outdated_str.trim().is_empty() {
                            println!("{}", "📊 Outdated packages:".yellow().bold());
                            println!("{}", outdated_str);
                        } else {
                            log_success("All npm packages are up to date!");
                        }
                    }
                    Err(e) => {
                        println!(" {}", "❌ Failed".red());
                        log_warning(&format!("Could not check outdated packages: {}", e));
                    }
                }
            } else {
                println!(" {}", "❌ Failed".red());
                let error_msg = String::from_utf8_lossy(&output.stderr);
                log_error(&format!("npm update failed: {}", error_msg));
            }
        }
        Err(e) => {
            println!(" {}", "❌ Failed".red());
            log_error(&format!("Failed to run npm update: {}", e));
        }
    }
}

fn update_pnpm_dependencies() {
    log_info("Updating pnpm dependencies...");

    let pnpm_cmd = if cfg!(target_os = "windows")
        && Command::new("pnpm.cmd").arg("--version").output().is_ok()
    {
        "pnpm.cmd"
    } else {
        "pnpm"
    };

    if Command::new(pnpm_cmd).arg("--version").output().is_err() {
        log_warning("pnpm not found. Skipping pnpm update.");
        return;
    }

    log_info(&format!("Using pnpm command: {}", pnpm_cmd.green()));

    log_info("Running pnpm update...");
    print!("{}", "⏳ Updating packages...".yellow());
    std::io::Write::flush(&mut std::io::stdout()).unwrap();

    let start_time = Instant::now();
    match Command::new(pnpm_cmd).arg("update").output() {
        Ok(output) => {
            let duration = start_time.elapsed();
            println!(
                " {}",
                format!("✅ Completed in {:.2}s", duration.as_secs_f64()).green()
            );

            if output.status.success() {
                log_success("pnpm dependencies updated successfully!");

                // Check for outdated packages
                log_info("Checking for outdated packages...");
                print!("{}", "⏳ Scanning for outdated packages...".yellow());
                std::io::Write::flush(&mut std::io::stdout()).unwrap();

                let start_time = Instant::now();
                match Command::new(pnpm_cmd).arg("outdated").output() {
                    Ok(outdated_output) => {
                        let duration = start_time.elapsed();
                        println!(
                            " {}",
                            format!("✅ Completed in {:.2}s", duration.as_secs_f64()).green()
                        );

                        let outdated_str = String::from_utf8_lossy(&outdated_output.stdout);
                        if !outdated_str.trim().is_empty() {
                            println!("{}", "📊 Outdated packages:".yellow().bold());
                            println!("{}", outdated_str);
                        } else {
                            log_success("All pnpm packages are up to date!");
                        }
                    }
                    Err(e) => {
                        println!(" {}", "❌ Failed".red());
                        log_warning(&format!("Could not check outdated packages: {}", e));
                    }
                }

                // Show pnpm audit for security
                log_info("Running security audit...");
                print!("{}", "⏳ Running security audit...".yellow());
                std::io::Write::flush(&mut std::io::stdout()).unwrap();

                let start_time = Instant::now();
                match Command::new(pnpm_cmd).arg("audit").output() {
                    Ok(audit_output) => {
                        let duration = start_time.elapsed();
                        println!(
                            " {}",
                            format!("✅ Completed in {:.2}s", duration.as_secs_f64()).green()
                        );

                        let audit_str = String::from_utf8_lossy(&audit_output.stdout);
                        if !audit_str.trim().is_empty() && audit_str.contains("vulnerabilities") {
                            println!("{}", "🔒 Security audit:".red().bold());
                            println!("{}", audit_str);
                        } else {
                            log_success("No security vulnerabilities found!");
                        }
                    }
                    Err(e) => {
                        println!(" {}", "❌ Failed".red());
                        log_warning(&format!("Could not run security audit: {}", e));
                    }
                }
            } else {
                println!(" {}", "❌ Failed".red());
                let error_msg = String::from_utf8_lossy(&output.stderr);
                log_error(&format!("pnpm update failed: {}", error_msg));
            }
        }
        Err(e) => {
            println!(" {}", "❌ Failed".red());
            log_error(&format!("Failed to run pnpm update: {}", e));
        }
    }
}

fn analyze_cargo_toml() {
    match read_file_to_string("Cargo.toml") {
        Ok(content) => {
            println!("{}", "🦀 Rust Dependencies:".red().bold());

            let lines: Vec<&str> = content.lines().collect();
            let mut in_dependencies = false;

            for line in lines {
                if line.trim() == "[dependencies]" {
                    in_dependencies = true;
                    continue;
                }

                if in_dependencies {
                    if line.starts_with("[") {
                        break;
                    }

                    if line.contains("=") && !line.trim().is_empty() {
                        println!("  {}", line.trim().green());
                    }
                }
            }
        }
        Err(e) => {
            log_error(&format!("Failed to read Cargo.toml: {}", e));
        }
    }
}

fn update_cargo_dependencies() {
    log_info("Updating Cargo dependencies...");
    // Backup Cargo files before updating
    backup_cargo_files();
    // Check if cargo is available
    match Command::new("cargo").arg("--version").output() {
        Ok(_) => {
            log_info("Running cargo update...");
            print!("{}", "⏳ Updating packages...".yellow());
            std::io::Write::flush(&mut std::io::stdout()).unwrap();

            let start_time = Instant::now();
            match Command::new("cargo").arg("update").output() {
                Ok(output) => {
                    let duration = start_time.elapsed();
                    println!(
                        " {}",
                        format!("✅ Completed in {:.2}s", duration.as_secs_f64()).green()
                    );

                    if output.status.success() {
                        log_success("Cargo dependencies updated successfully!");

                        // Show updated dependencies
                        let output_str = String::from_utf8_lossy(&output.stderr);
                        if !output_str.trim().is_empty() {
                            println!("{}", "📊 Update details:".cyan().bold());
                            println!("{}", output_str);
                        }
                    } else {
                        println!(" {}", "❌ Failed".red());
                        let error_msg = String::from_utf8_lossy(&output.stderr);
                        log_error(&format!("cargo update failed: {}", error_msg));
                    }
                }
                Err(e) => {
                    println!(" {}", "❌ Failed".red());
                    log_error(&format!("Failed to run cargo update: {}", e));
                }
            }
        }
        Err(_) => {
            log_warning("cargo not found. Skipping cargo update.");
        }
    }
}

fn backup_cargo_files() {
    log_info("Creating backup of Cargo files...");
    let now: DateTime<Local> = Local::now();
    let backup_dir = format!("./backup/{}", now.format("%Y%m%d%H%M%S"));
    // Create backup directory
    if let Err(e) = fs::create_dir_all(&backup_dir) {
        log_error(&format!("Failed to create backup directory: {}", e));
        return;
    }
    let mut backed_up_files = Vec::new();
    // Backup Cargo.toml
    if crate::utils::file_exists("Cargo.toml") {
        if let Err(e) = fs::copy("Cargo.toml", format!("{}/Cargo.toml", backup_dir)) {
            log_warning(&format!("Failed to backup Cargo.toml: {}", e));
        } else {
            backed_up_files.push("Cargo.toml".to_string());
        }
    }
    // Backup Cargo.lock
    if crate::utils::file_exists("Cargo.lock") {
        if let Err(e) = fs::copy("Cargo.lock", format!("{}/Cargo.lock", backup_dir)) {
            log_warning(&format!("Failed to backup Cargo.lock: {}", e));
        } else {
            backed_up_files.push("Cargo.lock".to_string());
        }
    }
    if !backed_up_files.is_empty() {
        log_success(&format!(
            "Backed up {} Cargo files to: {}",
            backed_up_files.len(),
            backup_dir.cyan()
        ));
        println!("  📁 Backed up files:");
        for file in backed_up_files {
            println!("    ✓ {}", file.green());
        }
        println!();
    } else {
        log_warning("No Cargo files were backed up");
        // Remove empty backup directory
        let _ = fs::remove_dir(&backup_dir);
    }
}

fn analyze_requirements_txt() {
    match read_file_to_string("requirements.txt") {
        Ok(content) => {
            println!("{}", "🐍 Python Dependencies:".yellow().bold());

            for line in content.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with("#") {
                    println!("  {}", line.green());
                }
            }
        }
        Err(e) => {
            log_error(&format!("Failed to read requirements.txt: {}", e));
        }
    }
}

fn update_pip_dependencies() {
    log_info("Updating pip dependencies...");

    // Check if pip is available
    match Command::new("pip").arg("--version").output() {
        Ok(_) => {
            // Try to update packages from requirements.txt
            if crate::utils::file_exists("requirements.txt") {
                log_info("Upgrading packages from requirements.txt...");
                match Command::new("pip")
                    .arg("install")
                    .arg("--upgrade")
                    .arg("-r")
                    .arg("requirements.txt")
                    .output()
                {
                    Ok(output) => {
                        if output.status.success() {
                            log_success("pip dependencies updated successfully!");
                        } else {
                            let error_msg = String::from_utf8_lossy(&output.stderr);
                            log_error(&format!("pip update failed: {}", error_msg));
                        }
                    }
                    Err(e) => {
                        log_error(&format!("Failed to run pip update: {}", e));
                    }
                }
            }

            // Check for outdated packages
            log_info("Checking for outdated packages...");
            match Command::new("pip").arg("list").arg("--outdated").output() {
                Ok(output) => {
                    let outdated_str = String::from_utf8_lossy(&output.stdout);
                    if !outdated_str.trim().is_empty() {
                        println!("{}", "📊 Outdated packages:".yellow().bold());
                        println!("{}", outdated_str);
                    } else {
                        log_success("All pip packages are up to date!");
                    }
                }
                Err(e) => {
                    log_warning(&format!("Could not check outdated packages: {}", e));
                }
            }
        }
        Err(_) => {
            log_warning("pip not found. Skipping pip update.");
        }
    }
}

fn analyze_composer_json() {
    match read_file_to_string("composer.json") {
        Ok(content) => match serde_json::from_str::<Value>(&content) {
            Ok(composer_json) => {
                println!("{}", "🐘 PHP Dependencies:".purple().bold());

                if let Some(deps) = composer_json["require"].as_object() {
                    for (name, version) in deps {
                        println!(
                            "  {} -> {}",
                            name.green(),
                            version.as_str().unwrap_or("unknown")
                        );
                    }
                }

                if let Some(dev_deps) = composer_json["require-dev"].as_object() {
                    println!("{}", "🔧 Dev Dependencies:".yellow().bold());
                    for (name, version) in dev_deps {
                        println!(
                            "  {} -> {}",
                            name.green(),
                            version.as_str().unwrap_or("unknown")
                        );
                    }
                }
            }
            Err(e) => {
                log_error(&format!("Failed to parse composer.json: {}", e));
            }
        },
        Err(e) => {
            log_error(&format!("Failed to read composer.json: {}", e));
        }
    }
}

fn update_composer_dependencies() {
    log_info("Updating Composer dependencies...");

    // Check if composer is available
    match Command::new("composer").arg("--version").output() {
        Ok(_) => {
            log_info("Running composer update...");
            match Command::new("composer").arg("update").output() {
                Ok(output) => {
                    if output.status.success() {
                        log_success("Composer dependencies updated successfully!");

                        // Show outdated packages
                        log_info("Checking for outdated packages...");
                        match Command::new("composer").arg("outdated").output() {
                            Ok(outdated_output) => {
                                let outdated_str = String::from_utf8_lossy(&outdated_output.stdout);
                                if !outdated_str.trim().is_empty() {
                                    println!("{}", "📊 Outdated packages:".yellow().bold());
                                    println!("{}", outdated_str);
                                } else {
                                    log_success("All Composer packages are up to date!");
                                }
                            }
                            Err(e) => {
                                log_warning(&format!("Could not check outdated packages: {}", e));
                            }
                        }
                    } else {
                        let error_msg = String::from_utf8_lossy(&output.stderr);
                        log_error(&format!("composer update failed: {}", error_msg));
                    }
                }
                Err(e) => {
                    log_error(&format!("Failed to run composer update: {}", e));
                }
            }
        }
        Err(_) => {
            log_warning("composer not found. Skipping composer update.");
        }
    }
}
