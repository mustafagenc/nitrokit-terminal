use anyhow::Result;
use colored::*;
use std::io::{self, Write};
use std::process::Command;

pub async fn create_release_with_args(version: &str, message: Option<&str>) -> Result<()> {
    println!("{}", format!("🚀 Creating release with version: {}", version).cyan());
    
    // Version string'ini analiz et ve bump type belirle
    let bump_type = determine_bump_type(version)?;
    
    // Version management'ı kullanarak release oluştur
    bump_and_release(bump_type, message).await?;
    
    println!("{}", "✅ Release created successfully!".green());
    Ok(())
}

pub async fn create_release_interactive() -> Result<()> {
    println!("{}", "\n🚀 Interactive Release Creation".cyan().bold());
    println!("{}", "═".repeat(35).dimmed());
    
    // Release type seçimi
    println!("{}", "Select release type:".yellow().bold());
    println!("  {} Patch (bug fixes, backward compatible)", "1.".dimmed());
    println!("  {} Minor (new features, backward compatible)", "2.".dimmed());
    println!("  {} Major (breaking changes)", "3.".dimmed());
    print!("\n{}", "Select option (1-3): ".cyan());
    
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    let bump_type = match input.trim() {
        "1" | "patch" => "patch",
        "2" | "minor" => "minor", 
        "3" | "major" => "major",
        _ => {
            println!("{}", "Invalid selection, defaulting to patch.".yellow());
            "patch"
        }
    };
    
    // Release mesajı al
    print!("{}", "Enter release message (optional): ".cyan());
    io::stdout().flush().unwrap();
    let mut message = String::new();
    io::stdin().read_line(&mut message).unwrap();
    let message = message.trim();
    
    let release_message = if message.is_empty() { 
        None 
    } else { 
        Some(message) 
    };
    
    // Onay
    println!("\n{}", "Review Release Details:".yellow().bold());
    println!("{}", "═".repeat(25).dimmed());
    println!("  {} {}", "Type:".dimmed(), bump_type.green());
    if let Some(msg) = release_message {
        println!("  {} {}", "Message:".dimmed(), msg.green());
    }
    print!("\n{}", "Proceed with release? (y/N): ".cyan());
    
    io::stdout().flush().unwrap();
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm).unwrap();
    
    if confirm.trim().to_lowercase() != "y" && confirm.trim().to_lowercase() != "yes" {
        println!("{}", "Release cancelled.".yellow());
        return Ok(());
    }
    
    // Release oluştur
    bump_and_release(bump_type, release_message).await?;
    
    println!("{}", "✅ Release created successfully!".green());
    Ok(())
}

fn determine_bump_type(version: &str) -> Result<&'static str> {
    // v prefix'ini kaldır
    let version = version.strip_prefix('v').unwrap_or(version);
    
    // Eğer tam versiyon numarası verilmişse (1.2.3), mevcut versiyonla karşılaştır
    if version.matches('.').count() == 2 {
        let current_version = env!("CARGO_PKG_VERSION");
        let current_parts: Vec<u32> = current_version
            .split('.')
            .map(|s| s.parse().unwrap_or(0))
            .collect();
        
        let new_parts: Vec<u32> = version
            .split('.')
            .map(|s| s.parse().unwrap_or(0))
            .collect();
        
        if new_parts.len() != 3 || current_parts.len() != 3 {
            return Err(anyhow::anyhow!("Invalid version format"));
        }
        
        // Hangi tip bump olduğunu belirle
        if new_parts[0] > current_parts[0] {
            Ok("major")
        } else if new_parts[1] > current_parts[1] {
            Ok("minor")
        } else if new_parts[2] > current_parts[2] {
            Ok("patch")
        } else {
            Err(anyhow::anyhow!("New version must be higher than current version"))
        }
    } else {
        // Eğer sadece bump type verilmişse
        match version.to_lowercase().as_str() {
            "major" => Ok("major"),
            "minor" => Ok("minor"), 
            "patch" => Ok("patch"),
            _ => Err(anyhow::anyhow!("Invalid version format. Use 'major', 'minor', 'patch' or semantic version like '1.2.3'"))
        }
    }
}

pub async fn bump_and_release(bump_type: &str, message: Option<&str>) -> Result<()> {
    // 1. Current version'u al
    let current_version = env!("CARGO_PKG_VERSION");
    let new_version = bump_version(bump_type, current_version)?;
    
    println!("🔄 Bumping version from {} to {}", current_version, new_version);
    
    // 2. Cargo.toml'u güncelle
    update_cargo_toml(&new_version)?;
    
    // 3. Git repository kontrolü
    check_git_repository()?;
    
    // 4. Release notes oluştur (opsiyonel, hata verirse devam et)
    let _release_notes = match generate_release_notes_safely().await {
        Ok(notes) => Some(notes),
        Err(e) => {
            println!("{}", format!("⚠️  Could not generate release notes: {}", e).yellow());
            None
        }
    };
    
    // 5. Git commit ve tag
    create_git_tag(&new_version, message).await?;
    
    println!("🎉 Successfully released version {}", new_version.green());
    Ok(())
}

fn check_git_repository() -> Result<()> {
    // Git repository olup olmadığını kontrol et
    let output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("Not a git repository. Please initialize git first with 'git init'"));
    }
    
    // Git'te herhangi bir commit olup olmadığını kontrol et
    let output = Command::new("git")
        .args(["log", "--oneline", "-1"])
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("No commits found. Please make an initial commit first"));
    }
    
    Ok(())
}

async fn generate_release_notes_safely() -> Result<String> {
    // Son tag'i bul (yoksa None)
    let latest_tag = get_latest_tag_safe()?;
    
    // Release notes oluştur
    if let Some(tag) = latest_tag {
        crate::commands::release_notes::generate_release_notes_for_version(Some(&tag), None)
    } else {
        // Eğer hiç tag yoksa, tüm commit'leri al
        crate::commands::release_notes::generate_release_notes_for_version(None, None)
    }
}

fn get_latest_tag_safe() -> Result<Option<String>> {
    let output = Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .output()?;
    
    if output.status.success() {
        let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if tag.is_empty() {
            Ok(None)
        } else {
            Ok(Some(tag))
        }
    } else {
        // Tag yoksa hata vermek yerine None döndür
        Ok(None)
    }
}

fn bump_version(bump_type: &str, current: &str) -> Result<String> {
    let parts: Vec<&str> = current.split('.').collect();
    if parts.len() != 3 {
        return Err(anyhow::anyhow!("Invalid version format: {}", current));
    }
    
    let major: u32 = parts[0].parse()
        .map_err(|_| anyhow::anyhow!("Invalid major version: {}", parts[0]))?;
    let minor: u32 = parts[1].parse()
        .map_err(|_| anyhow::anyhow!("Invalid minor version: {}", parts[1]))?;
    let patch: u32 = parts[2].parse()
        .map_err(|_| anyhow::anyhow!("Invalid patch version: {}", parts[2]))?;
    
    let new_version = match bump_type {
        "major" => format!("{}.0.0", major + 1),
        "minor" => format!("{}.{}.0", major, minor + 1),
        "patch" => format!("{}.{}.{}", major, minor, patch + 1),
        _ => return Err(anyhow::anyhow!("Invalid bump type: {}. Use 'major', 'minor', or 'patch'", bump_type)),
    };
    
    Ok(new_version)
}

fn update_cargo_toml(new_version: &str) -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    let cargo_content = std::fs::read_to_string("Cargo.toml")
        .map_err(|e| anyhow::anyhow!("Failed to read Cargo.toml: {}", e))?;
    
    let updated = cargo_content.replace(
        &format!("version = \"{}\"", current_version),
        &format!("version = \"{}\"", new_version)
    );
    
    std::fs::write("Cargo.toml", updated)
        .map_err(|e| anyhow::anyhow!("Failed to write Cargo.toml: {}", e))?;
    
    println!("✅ Updated Cargo.toml");
    Ok(())
}

async fn create_git_tag(version: &str, message: Option<&str>) -> Result<()> {
    let tag_name = format!("v{}", version);
    
    // Working directory'de değişiklik olup olmadığını kontrol et
    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()?;
    
    if !status_output.stdout.is_empty() {
        // Değişiklikleri commit et
        println!("📝 Committing changes...");
        
        let add_output = Command::new("git")
            .args(["add", "Cargo.toml"])
            .output()?;
        
        if !add_output.status.success() {
            return Err(anyhow::anyhow!("Failed to stage Cargo.toml changes"));
        }
        
        let commit_output = Command::new("git")
            .args(["commit", "-m", &format!("bump: version {}", version)])
            .output()?;
        
        if !commit_output.status.success() {
            let error = String::from_utf8_lossy(&commit_output.stderr);
            return Err(anyhow::anyhow!("Failed to commit changes: {}", error));
        }
        
        println!("✅ Changes committed");
    }
    
    // Tag oluştur
    println!("🏷️  Creating git tag...");
    let default_message = format!("Release {}", tag_name);
    let tag_message = message.unwrap_or(&default_message);
    
    let tag_output = Command::new("git")
        .args(["tag", "-a", &tag_name, "-m", tag_message])
        .output()?;
    
    if !tag_output.status.success() {
        let error = String::from_utf8_lossy(&tag_output.stderr);
        return Err(anyhow::anyhow!("Failed to create tag: {}", error));
    }
    
    // Remote'a push et (opsiyonel)
    println!("🚀 Pushing changes to remote...");
    
    // Önce commit'leri push et
    let push_output = Command::new("git")
        .args(["push"])
        .output();
    
    match push_output {
        Ok(output) if output.status.success() => {
            println!("✅ Pushed commits to remote");
        }
        Ok(output) => {
            let error = String::from_utf8_lossy(&output.stderr);
            println!("{}", format!("⚠️  Could not push commits: {}", error).yellow());
        }
        Err(e) => {
            println!("{}", format!("⚠️  Could not push commits: {}", e).yellow());
        }
    }
    
    // Tag'i push et
    let push_tag_output = Command::new("git")
        .args(["push", "origin", &tag_name])
        .output();
    
    match push_tag_output {
        Ok(output) if output.status.success() => {
            println!("✅ Pushed tag to remote: {}", tag_name.green());
        }
        Ok(output) => {
            let error = String::from_utf8_lossy(&output.stderr);
            println!("{}", format!("⚠️  Could not push tag: {} (Tag created locally)", error).yellow());
        }
        Err(e) => {
            println!("{}", format!("⚠️  Could not push tag: {} (Tag created locally)", e).yellow());
        }
    }
    
    println!("✅ Created tag: {}", tag_name.green());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_bump_type() {
        assert_eq!(determine_bump_type("major").unwrap(), "major");
        assert_eq!(determine_bump_type("minor").unwrap(), "minor");
        assert_eq!(determine_bump_type("patch").unwrap(), "patch");
    }
}