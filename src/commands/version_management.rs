use crate::commands::release_notes::generate_release_notes_for_version;
use anyhow::Result;
use std::process::Command;
use colored::*;

pub async fn bump_and_release(bump_type: &str, message: Option<&str>) -> Result<()> {
    // 1. Current version'u al
    let current_version = env!("CARGO_PKG_VERSION");
    let new_version = bump_version(bump_type, current_version)?;
    
    println!("🔄 Bumping version from {} to {}", current_version, new_version);
    
    // 2. Cargo.toml'u güncelle
    update_cargo_toml(&new_version)?;
    
    // 3. Release notes oluştur
    let latest_tag = get_latest_tag()?;
    let _release_notes = generate_release_notes_for_version(
        latest_tag.as_deref(), 
        Some(&format!("v{}", new_version))
    )?;
    
    // 4. Git commit ve tag
    create_git_tag(&new_version, message).await?;
    
    println!("🎉 Successfully released version {}", new_version.green());
    Ok(())
}

fn bump_version(bump_type: &str, current: &str) -> Result<String> {
    let parts: Vec<&str> = current.split('.').collect();
    if parts.len() != 3 {
        return Err(anyhow::anyhow!("Invalid version format"));
    }
    
    let major: u32 = parts[0].parse()?;
    let minor: u32 = parts[1].parse()?;
    let patch: u32 = parts[2].parse()?;
    
    let new_version = match bump_type {
        "major" => format!("{}.0.0", major + 1),
        "minor" => format!("{}.{}.0", major, minor + 1),
        "patch" => format!("{}.{}.{}", major, minor, patch + 1),
        _ => return Err(anyhow::anyhow!("Invalid bump type: {}", bump_type)),
    };
    
    Ok(new_version)
}

fn update_cargo_toml(new_version: &str) -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    let cargo_content = std::fs::read_to_string("Cargo.toml")?;
    let updated = cargo_content.replace(
        &format!("version = \"{}\"", current_version),
        &format!("version = \"{}\"", new_version)
    );
    std::fs::write("Cargo.toml", updated)?;
    println!("✅ Updated Cargo.toml");
    Ok(())
}

fn get_latest_tag() -> Result<Option<String>> {
    let output = Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .output()?;
    
    if output.status.success() {
        let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(Some(tag))
    } else {
        Ok(None)
    }
}

async fn create_git_tag(version: &str, message: Option<&str>) -> Result<()> {
    let tag_name = format!("v{}", version);
    
    // Commit changes
    Command::new("git")
        .args(["add", "Cargo.toml"])
        .output()?;
    
    Command::new("git")
        .args(["commit", "-m", &format!("bump: version {}", version)])
        .output()?;
    
    // Create tag with message
    let default_message = format!("Release {}", tag_name);
    let tag_message = message.unwrap_or(&default_message);
    
    Command::new("git")
        .args(["tag", "-a", &tag_name, "-m", tag_message])
        .output()?;
    
    // Push changes and tag
    Command::new("git")
        .args(["push", "origin", "main"])
        .output()?;
    
    Command::new("git")
        .args(["push", "origin", &tag_name])
        .output()?;
    
    println!("✅ Created and pushed tag: {}", tag_name.green());
    Ok(())
}

pub async fn show_version_history() -> Result<()> {
    println!("{}", "📋 Version History:".cyan().bold());
    println!("{}", "═".repeat(40).dimmed());
    
    let output = Command::new("git")
        .args(["tag", "--sort=-version:refname", "-l", "v*"])
        .output()?;
    
    if output.status.success() {
        let tags = String::from_utf8_lossy(&output.stdout);
        if tags.trim().is_empty() {
            println!("{}", "No version tags found.".dimmed());
        } else {
            for tag in tags.lines().take(10) {
                println!("  📌 {}", tag.trim().green());
            }
        }
    } else {
        println!("{}", "Failed to retrieve version history.".red());
    }
    
    Ok(())
}