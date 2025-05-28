#[cfg(test)]
mod tests {
    use crate::utils::version_check::{
        GitHubRelease, VersionCache, CACHE_FILE, CHECK_INTERVAL_HOURS, GITHUB_API_URL,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tempfile::tempdir;

    // Helper function to clean version string
    fn clean_version_string(version: &str) -> String {
        if version.starts_with('v') {
            version[1..].to_string()
        } else {
            version.to_string()
        }
    }

    // Helper function to compare versions
    fn compare_versions(
        current: &str,
        latest: &str,
    ) -> Result<std::cmp::Ordering, Box<dyn std::error::Error>> {
        let clean_current = clean_version_string(current);
        let clean_latest = clean_version_string(latest);

        let current_parts: Vec<&str> = clean_current.split('.').collect();
        let latest_parts: Vec<&str> = clean_latest.split('.').collect();

        if current_parts.len() != 3 || latest_parts.len() != 3 {
            return Err("Invalid version format".into());
        }

        for i in 0..3 {
            let current_num: u32 = current_parts[i].split('-').next().unwrap().parse()?;
            let latest_num: u32 = latest_parts[i].split('-').next().unwrap().parse()?;

            match current_num.cmp(&latest_num) {
                std::cmp::Ordering::Less => return Ok(std::cmp::Ordering::Less),
                std::cmp::Ordering::Greater => return Ok(std::cmp::Ordering::Greater),
                std::cmp::Ordering::Equal => continue,
            }
        }

        Ok(std::cmp::Ordering::Equal)
    }

    // Helper functions with custom cache file path
    fn save_version_cache_to_path(version: &str, cache_path: &PathBuf) {
        let cache = VersionCache {
            last_check: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            latest_version: version.to_string(),
            check_interval_hours: CHECK_INTERVAL_HOURS,
        };

        if let Ok(json) = serde_json::to_string(&cache) {
            let _ = fs::write(cache_path, json);
        }
    }

    fn load_version_cache_from_path(cache_path: &PathBuf) -> Option<VersionCache> {
        if let Ok(content) = fs::read_to_string(cache_path) {
            serde_json::from_str(&content).ok()
        } else {
            None
        }
    }

    fn should_check_for_updates_with_path(cache_path: &PathBuf) -> bool {
        if let Some(cache) = load_version_cache_from_path(cache_path) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let hours_since_check = (now - cache.last_check) / 3600;
            hours_since_check >= cache.check_interval_hours
        } else {
            true
        }
    }

    #[test]
    fn test_github_release_struct() {
        let release = GitHubRelease {
            tag_name: "v1.0.0".to_string(),
            name: "Release v1.0.0".to_string(),
            published_at: "2023-01-01T00:00:00Z".to_string(),
            html_url: "https://github.com/user/repo/releases/tag/v1.0.0".to_string(),
            prerelease: false,
        };

        assert_eq!(release.tag_name, "v1.0.0");
        assert_eq!(release.name, "Release v1.0.0");
        assert!(!release.prerelease);
    }

    #[test]
    fn test_version_cache_struct() {
        let cache = VersionCache {
            last_check: 1640995200, // 2022-01-01 00:00:00 UTC
            latest_version: "v1.0.0".to_string(),
            check_interval_hours: 24,
        };

        assert_eq!(cache.last_check, 1640995200);
        assert_eq!(cache.latest_version, "v1.0.0");
        assert_eq!(cache.check_interval_hours, 24);
    }

    #[test]
    fn test_clean_version_string() {
        assert_eq!(clean_version_string("v1.0.0"), "1.0.0");
        assert_eq!(clean_version_string("v2.1.3"), "2.1.3");
        assert_eq!(clean_version_string("1.0.0"), "1.0.0");
        assert_eq!(clean_version_string("0.1.0"), "0.1.0");
        assert_eq!(clean_version_string(""), "");
        assert_eq!(clean_version_string("vv1.0.0"), "v1.0.0");
    }

    #[test]
    fn test_compare_versions_equal() {
        let result = compare_versions("1.0.0", "v1.0.0");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), std::cmp::Ordering::Equal);

        let result = compare_versions("v2.1.3", "2.1.3");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_compare_versions_current_less() {
        let result = compare_versions("1.0.0", "1.0.1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), std::cmp::Ordering::Less);

        let result = compare_versions("1.0.0", "1.1.0");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), std::cmp::Ordering::Less);

        let result = compare_versions("1.0.0", "2.0.0");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), std::cmp::Ordering::Less);
    }

    #[test]
    fn test_compare_versions_current_greater() {
        let result = compare_versions("1.0.1", "1.0.0");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), std::cmp::Ordering::Greater);

        let result = compare_versions("1.1.0", "1.0.0");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), std::cmp::Ordering::Greater);

        let result = compare_versions("2.0.0", "1.0.0");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), std::cmp::Ordering::Greater);
    }

    #[test]
    fn test_compare_versions_invalid() {
        let result = compare_versions("invalid", "1.0.0");
        assert!(result.is_err());

        let result = compare_versions("1.0.0", "invalid");
        assert!(result.is_err());

        let result = compare_versions("1.0", "1.0.0");
        assert!(result.is_err());

        let result = compare_versions("", "1.0.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_version_cache_serialization() {
        let cache = VersionCache {
            last_check: 1640995200,
            latest_version: "v1.0.0".to_string(),
            check_interval_hours: 24,
        };

        let json = serde_json::to_string(&cache);
        assert!(json.is_ok());

        let deserialized: Result<VersionCache, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());

        let deserialized_cache = deserialized.unwrap();
        assert_eq!(deserialized_cache.last_check, cache.last_check);
        assert_eq!(deserialized_cache.latest_version, cache.latest_version);
        assert_eq!(
            deserialized_cache.check_interval_hours,
            cache.check_interval_hours
        );
    }

    #[test]
    fn test_save_and_load_version_cache() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let cache_file_path = temp_dir.path().join(CACHE_FILE);

        // Test saving cache with custom path
        save_version_cache_to_path("v1.2.3", &cache_file_path);

        // Test loading cache
        let loaded_cache = load_version_cache_from_path(&cache_file_path);
        assert!(
            loaded_cache.is_some(),
            "Cache should be loaded successfully"
        );

        let cache = loaded_cache.unwrap();
        assert_eq!(cache.latest_version, "v1.2.3");
        assert_eq!(cache.check_interval_hours, CHECK_INTERVAL_HOURS);
        assert!(cache.last_check > 0);
    }

    #[test]
    fn test_load_version_cache_no_file() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let non_existent_cache = temp_dir.path().join("non_existent_cache.json");

        // Test loading non-existent cache
        let cache = load_version_cache_from_path(&non_existent_cache);
        assert!(cache.is_none());
    }

    #[test]
    fn test_load_version_cache_invalid_json() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let invalid_cache_path = temp_dir.path().join("invalid_cache.json");

        // Write invalid JSON to cache file
        fs::write(&invalid_cache_path, "invalid json content")
            .expect("Failed to write invalid cache");

        // Test loading invalid cache
        let cache = load_version_cache_from_path(&invalid_cache_path);
        assert!(cache.is_none());
    }

    #[test]
    fn test_should_check_for_updates_no_cache() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let non_existent_cache = temp_dir.path().join("no_cache.json");

        // Should check when no cache exists
        assert!(should_check_for_updates_with_path(&non_existent_cache));
    }

    #[test]
    fn test_should_check_for_updates_recent_cache() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let cache_path = temp_dir.path().join("recent_cache.json");

        // Create recent cache (current timestamp)
        let recent_cache = VersionCache {
            last_check: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            latest_version: "v1.0.0".to_string(),
            check_interval_hours: 24,
        };

        let json = serde_json::to_string(&recent_cache).unwrap();
        fs::write(&cache_path, json).expect("Failed to write cache");

        // Should not check when cache is recent
        assert!(!should_check_for_updates_with_path(&cache_path));
    }

    #[test]
    fn test_should_check_for_updates_old_cache() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let cache_path = temp_dir.path().join("old_cache.json");

        // Create old cache (25 hours ago)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let old_cache = VersionCache {
            last_check: now - (25 * 3600), // 25 hours ago
            latest_version: "v1.0.0".to_string(),
            check_interval_hours: 24,
        };

        let json = serde_json::to_string(&old_cache).unwrap();
        fs::write(&cache_path, json).expect("Failed to write cache");

        // Should check when cache is old
        assert!(should_check_for_updates_with_path(&cache_path));
    }

    #[test]
    fn test_constants() {
        assert!(!GITHUB_API_URL.is_empty());
        assert!(GITHUB_API_URL.starts_with("https://api.github.com/"));
        assert!(GITHUB_API_URL.contains("releases/latest"));

        assert!(!CACHE_FILE.is_empty());
        assert!(CACHE_FILE.contains("nitroterm"));

        assert!(CHECK_INTERVAL_HOURS > 0);
        assert_eq!(CHECK_INTERVAL_HOURS, 24);
    }

    #[test]
    fn test_github_release_json_compatibility() {
        let json_response = r#"
        {
            "tag_name": "v1.0.0",
            "name": "Release 1.0.0",
            "published_at": "2023-01-01T00:00:00Z",
            "html_url": "https://github.com/user/repo/releases/tag/v1.0.0",
            "prerelease": false
        }
        "#;

        let release: Result<GitHubRelease, _> = serde_json::from_str(json_response);
        assert!(release.is_ok());

        let release = release.unwrap();
        assert_eq!(release.tag_name, "v1.0.0");
        assert_eq!(release.name, "Release 1.0.0");
        assert!(!release.prerelease);
    }

    #[test]
    fn test_version_comparison_basic() {
        let result = compare_versions("1.0.0", "1.0.1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), std::cmp::Ordering::Less);

        let result = compare_versions("2.0.0", "1.9.9");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), std::cmp::Ordering::Greater);

        let result = compare_versions("1.5.0", "1.5.0");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_cache_file_operations() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let cache_path_1 = temp_dir.path().join("cache1.json");
        let cache_path_2 = temp_dir.path().join("cache2.json");

        // Test cache doesn't exist initially
        assert!(load_version_cache_from_path(&cache_path_1).is_none());

        // Save cache
        save_version_cache_to_path("v2.0.0", &cache_path_1);

        // Verify cache file exists
        assert!(cache_path_1.exists());

        // Load and verify cache
        let cache = load_version_cache_from_path(&cache_path_1);
        assert!(cache.is_some(), "Cache should exist after saving");
        assert_eq!(cache.unwrap().latest_version, "v2.0.0");

        // Save different version to different file
        save_version_cache_to_path("v2.1.0", &cache_path_2);

        // Verify second cache was created
        let updated_cache = load_version_cache_from_path(&cache_path_2);
        assert!(updated_cache.is_some());
        assert_eq!(updated_cache.unwrap().latest_version, "v2.1.0");
    }

    #[test]
    fn test_version_string_variations() {
        let test_cases = vec![
            ("1.0.0", "v1.0.0", std::cmp::Ordering::Equal),
            ("v1.0.0", "1.0.0", std::cmp::Ordering::Equal),
            ("v1.0.0", "v1.0.0", std::cmp::Ordering::Equal),
            ("1.0.0", "1.0.0", std::cmp::Ordering::Equal),
            ("v2.0.0", "v1.9.9", std::cmp::Ordering::Greater),
            ("v1.0.0", "v1.0.1", std::cmp::Ordering::Less),
        ];

        for (current, latest, expected) in test_cases {
            let result = compare_versions(current, latest);
            assert!(
                result.is_ok(),
                "Failed to compare '{}' vs '{}'",
                current,
                latest
            );
            assert_eq!(
                result.unwrap(),
                expected,
                "Wrong comparison result for '{}' vs '{}'",
                current,
                latest
            );
        }
    }

    #[test]
    fn test_time_calculations() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let one_hour_ago = now - 3600;
        let hours_diff = (now - one_hour_ago) / 3600;
        assert_eq!(hours_diff, 1);

        let twenty_five_hours_ago = now - (25 * 3600);
        let hours_diff = (now - twenty_five_hours_ago) / 3600;
        assert_eq!(hours_diff, 25);
    }

    #[test]
    fn test_cache_interval_customization() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let cache_path = temp_dir.path().join("custom_interval_cache.json");

        // Create cache with custom interval and old timestamp
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let custom_cache = VersionCache {
            last_check: now - (12 * 3600), // 12 hours ago
            latest_version: "v1.0.0".to_string(),
            check_interval_hours: 6, // 6 hour interval
        };

        let json = serde_json::to_string(&custom_cache).unwrap();
        fs::write(&cache_path, json).expect("Failed to write cache");

        // Should check because 12 hours > 6 hour interval
        assert!(should_check_for_updates_with_path(&cache_path));
    }
}
