// src/utils/version.rs oluştur
use std::process::Command;
use anyhow::Result;

pub fn get_current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

pub fn bump_version(bump_type: &str) -> Result<String> {
    let current = get_current_version();
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
        _ => return Err(anyhow::anyhow!("Invalid bump type")),
    };
    
    Ok(new_version)
}

pub fn update_cargo_toml(new_version: &str) -> Result<()> {
    let cargo_content = std::fs::read_to_string("Cargo.toml")?;
    let updated = cargo_content.replace(
        &format!("version = \"{}\"", get_current_version()),
        &format!("version = \"{}\"", new_version)
    );
    std::fs::write("Cargo.toml", updated)?;
    Ok(())
}