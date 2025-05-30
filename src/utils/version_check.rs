use colored::*;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Deserialize, Serialize, Debug)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub published_at: String,
    pub html_url: String,
    pub prerelease: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct VersionCache {
    pub last_check: u64,
    pub latest_version: String,
    pub check_interval_hours: u64,
}

pub const GITHUB_API_URL: &str =
    "https://api.github.com/repos/mustafagenc/nitroterm/releases/latest";
pub const CACHE_FILE: &str = ".nitroterm_version_cache.json";
pub const CHECK_INTERVAL_HOURS: u64 = 24; // Check once per day

pub async fn check_for_updates(
    current_version: &str,
    force_check: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if we should perform update check
    if !force_check && !should_check_for_updates() {
        return Ok(());
    }

    match fetch_latest_version().await {
        Ok(latest_release) => {
            // Save to cache
            save_version_cache(&latest_release.tag_name);

            // Compare versions
            if let Ok(comparison) = compare_versions(current_version, &latest_release.tag_name) {
                match comparison {
                    std::cmp::Ordering::Less => {
                        show_update_available(&latest_release, current_version);
                    }
                    std::cmp::Ordering::Equal => {
                        if force_check {
                            println!("{}", "✅ You're using the latest version!".green());
                        }
                    }
                    std::cmp::Ordering::Greater => {
                        if force_check {
                            println!("{}", "🚀 You're using a development version!".yellow());
                        }
                    }
                }
            }
        }
        Err(e) => {
            if force_check {
                println!(
                    "{}",
                    format!("⚠️  Could not check for updates: {}", e).yellow()
                );
            }
            // Silently fail for automatic checks
        }
    }

    Ok(())
}

pub async fn fetch_latest_version() -> Result<GitHubRelease, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("nitroterm")
        .build()?;

    let response = client.get(GITHUB_API_URL).send().await?;

    if !response.status().is_success() {
        return Err(format!("GitHub API returned status: {}", response.status()).into());
    }

    let release: GitHubRelease = response.json().await?;
    Ok(release)
}

pub fn should_check_for_updates() -> bool {
    match load_version_cache() {
        Some(cache) => {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let hours_since_check = (now - cache.last_check) / 3600;
            hours_since_check >= cache.check_interval_hours
        }
        None => true, // No cache, should check
    }
}

pub fn load_version_cache() -> Option<VersionCache> {
    if let Ok(content) = std::fs::read_to_string(CACHE_FILE) {
        serde_json::from_str(&content).ok()
    } else {
        None
    }
}

pub fn save_version_cache(latest_version: &str) {
    let cache = VersionCache {
        last_check: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        latest_version: latest_version.to_string(),
        check_interval_hours: CHECK_INTERVAL_HOURS,
    };

    if let Ok(json) = serde_json::to_string_pretty(&cache) {
        let _ = std::fs::write(CACHE_FILE, json);
    }
}

pub fn compare_versions(
    current: &str,
    latest: &str,
) -> Result<std::cmp::Ordering, Box<dyn std::error::Error>> {
    let current_clean = clean_version_string(current);
    let latest_clean = clean_version_string(latest);

    let current_ver = semver::Version::parse(&current_clean)?;
    let latest_ver = semver::Version::parse(&latest_clean)?;

    Ok(current_ver.cmp(&latest_ver))
}

pub fn clean_version_string(version: &str) -> String {
    version.trim_start_matches('v').to_string()
}

pub fn show_update_available(release: &GitHubRelease, current_version: &str) {
    println!();
    println!("{}", "🎉 NEW VERSION AVAILABLE!".green().bold());
    println!("{}", "═".repeat(50).dimmed());
    println!(
        "{} {} → {}",
        "Current version:".dimmed(),
        current_version.yellow(),
        release.tag_name.green().bold()
    );
    println!("{} {}", "Release:".dimmed(), release.name);
    println!(
        "{} {}",
        "Download:".dimmed(),
        release.html_url.blue().underline()
    );
    println!();
    println!("{}", "📦 Update options:".yellow().bold());
    println!("  {} Download from GitHub releases", "•".dimmed());
    println!(
        "  {}   {{}} Build from source: git pull && cargo build --release",
        "•".dimmed()
    );
    println!("  {} Use package manager (if available)", "•".dimmed());
    println!();
    println!(
        "{}",
        "💡 Tip: Run 'nitroterm --check-updates' to check again".dimmed()
    );
    println!("{}", "═".repeat(50).dimmed());
    println!();
}
