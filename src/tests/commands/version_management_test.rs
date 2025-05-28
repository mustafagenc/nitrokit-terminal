use std::fs;
use tempfile::tempdir;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_parsing() {
        let valid_versions = vec![
            ("1.0.0", (1, 0, 0, None)),
            ("2.5.10", (2, 5, 10, None)),
            ("0.1.0", (0, 1, 0, None)),
            ("10.20.30", (10, 20, 30, None)),
            ("1.0.0-alpha", (1, 0, 0, Some("alpha"))),
            ("2.1.0-beta.1", (2, 1, 0, Some("beta.1"))),
            ("1.0.0-rc.2", (1, 0, 0, Some("rc.2"))),
        ];

        for (version_str, expected) in valid_versions {
            let clean_version = version_str.trim_start_matches('v');
            let parts: Vec<&str> = clean_version.split('-').collect();
            let version_part = parts[0];
            let prerelease = if parts.len() > 1 {
                Some(parts[1])
            } else {
                None
            };

            let version_numbers: Vec<u32> = version_part
                .split('.')
                .map(|v| v.parse().unwrap())
                .collect();

            assert_eq!(version_numbers.len(), 3);
            assert_eq!(
                (
                    version_numbers[0],
                    version_numbers[1],
                    version_numbers[2],
                    prerelease
                ),
                expected
            );
        }
    }

    #[test]
    fn test_invalid_semver_parsing() {
        let invalid_versions = vec![
            "1.0",     // Missing patch
            "1",       // Missing minor and patch
            "1.0.0.0", // Too many parts
            "a.b.c",   // Non-numeric
            "1.0.a",   // Non-numeric patch
            "",        // Empty
            "v",       // Just prefix
            "1.-1.0",  // Negative numbers
        ];

        for version_str in invalid_versions {
            let clean_version = version_str.trim_start_matches('v');
            let parts: Vec<&str> = clean_version.split('-').collect();
            let version_part = parts[0];

            if version_part.is_empty() {
                continue; // Skip empty versions
            }

            let version_numbers: Result<Vec<u32>, _> =
                version_part.split('.').map(|v| v.parse::<u32>()).collect();

            let is_valid = version_numbers.is_ok() && version_numbers.as_ref().unwrap().len() == 3;

            assert!(!is_valid, "Version '{}' should be invalid", version_str);
        }
    }

    #[test]
    fn test_version_comparison() {
        let version_pairs = vec![
            ("1.0.0", "1.0.1", std::cmp::Ordering::Less),
            ("1.1.0", "1.0.0", std::cmp::Ordering::Greater),
            ("2.0.0", "1.9.9", std::cmp::Ordering::Greater),
            ("1.0.0", "1.0.0", std::cmp::Ordering::Equal),
            ("0.1.0", "0.2.0", std::cmp::Ordering::Less),
            ("10.0.0", "9.0.0", std::cmp::Ordering::Greater),
        ];

        for (v1_str, v2_str, expected) in version_pairs {
            let v1_parts: Vec<u32> = v1_str.split('.').map(|v| v.parse().unwrap()).collect();
            let v2_parts: Vec<u32> = v2_str.split('.').map(|v| v.parse().unwrap()).collect();

            let comparison = v1_parts.cmp(&v2_parts);
            assert_eq!(comparison, expected, "Comparing {} with {}", v1_str, v2_str);
        }
    }

    #[test]
    fn test_version_increment() {
        let test_cases = vec![
            ("1.0.0", "major", "2.0.0"),
            ("1.2.3", "minor", "1.3.0"),
            ("1.2.3", "patch", "1.2.4"),
            ("0.1.0", "major", "1.0.0"),
            ("9.9.9", "major", "10.0.0"),
            ("1.9.9", "minor", "1.10.0"),
            ("1.2.9", "patch", "1.2.10"),
        ];

        for (current, bump_type, expected) in test_cases {
            let parts: Vec<u32> = current.split('.').map(|v| v.parse().unwrap()).collect();
            let (major, minor, patch) = (parts[0], parts[1], parts[2]);

            let new_version = match bump_type {
                "major" => format!("{}.0.0", major + 1),
                "minor" => format!("{}.{}.0", major, minor + 1),
                "patch" => format!("{}.{}.{}", major, minor, patch + 1),
                _ => current.to_string(),
            };

            assert_eq!(new_version, expected);
        }
    }

    #[test]
    fn test_cargo_toml_version_extraction() {
        let temp_dir = tempdir().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        let cargo_content = r#"
[package]
name = "test-package"
version = "1.2.3"
edition = "2021"
description = "A test package"

[dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }
"#;

        fs::write(&cargo_toml_path, cargo_content).unwrap();

        let content = fs::read_to_string(&cargo_toml_path).unwrap();

        // Extract version using regex-like pattern
        let lines: Vec<&str> = content.lines().collect();
        let mut version = None;

        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with("version") && trimmed.contains("=") {
                let parts: Vec<&str> = trimmed.split('=').collect();
                if parts.len() == 2 {
                    let version_part = parts[1].trim().trim_matches('"');
                    version = Some(version_part.to_string());
                    break;
                }
            }
        }

        assert_eq!(version, Some("1.2.3".to_string()));
    }

    #[test]
    fn test_cargo_toml_version_update() {
        let temp_dir = tempdir().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        let original_content = r#"
[package]
name = "test-package"
version = "1.0.0"
edition = "2021"

[dependencies]
serde = "1.0"
"#;

        fs::write(&cargo_toml_path, original_content).unwrap();

        // Update version from 1.0.0 to 1.1.0
        let content = fs::read_to_string(&cargo_toml_path).unwrap();
        let new_content = content.replace(r#"version = "1.0.0""#, r#"version = "1.1.0""#);
        fs::write(&cargo_toml_path, new_content).unwrap();

        // Verify update
        let updated_content = fs::read_to_string(&cargo_toml_path).unwrap();
        assert!(updated_content.contains(r#"version = "1.1.0""#));
        assert!(!updated_content.contains(r#"version = "1.0.0""#));
    }

    #[test]
    fn test_package_json_version_handling() {
        let temp_dir = tempdir().unwrap();
        let package_json_path = temp_dir.path().join("package.json");

        let package_content = r#"{
  "name": "test-package",
  "version": "1.0.0",
  "description": "A test package",
  "main": "index.js",
  "scripts": {
    "test": "echo \"Error: no test specified\" && exit 1"
  }
}"#;

        fs::write(&package_json_path, package_content).unwrap();

        let content = fs::read_to_string(&package_json_path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();

        let version = json["version"].as_str().unwrap();
        assert_eq!(version, "1.0.0");

        // Update version
        let mut updated_json = json;
        updated_json["version"] = serde_json::Value::String("1.1.0".to_string());

        let updated_content = serde_json::to_string_pretty(&updated_json).unwrap();
        fs::write(&package_json_path, updated_content).unwrap();

        // Verify update
        let final_content = fs::read_to_string(&package_json_path).unwrap();
        let final_json: serde_json::Value = serde_json::from_str(&final_content).unwrap();
        assert_eq!(final_json["version"].as_str().unwrap(), "1.1.0");
    }

    #[test]
    fn test_prerelease_version_handling() {
        let versions = vec![
            "1.0.0-alpha",
            "1.0.0-alpha.1",
            "1.0.0-beta",
            "1.0.0-beta.2",
            "1.0.0-rc.1",
            "2.0.0-dev",
        ];

        for version in versions {
            let parts: Vec<&str> = version.split('-').collect();
            assert_eq!(parts.len(), 2);

            let version_part = parts[0];
            let prerelease_part = parts[1];

            // Validate version part
            let version_numbers: Vec<u32> = version_part
                .split('.')
                .map(|v| v.parse().unwrap())
                .collect();
            assert_eq!(version_numbers.len(), 3);

            // Validate prerelease part
            assert!(!prerelease_part.is_empty());
            assert!(prerelease_part
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.'));
        }
    }

    #[test]
    fn test_version_tag_formatting() {
        let versions = vec![
            ("1.0.0", "v1.0.0"),
            ("2.1.3", "v2.1.3"),
            ("0.1.0-alpha", "v0.1.0-alpha"),
            ("1.0.0-rc.1", "v1.0.0-rc.1"),
        ];

        for (version, expected_tag) in versions {
            let tag = format!("v{}", version);
            assert_eq!(tag, expected_tag);
            assert!(tag.starts_with('v'));
        }
    }

    #[test]
    fn test_version_validation_rules() {
        let validation_cases = vec![
            ("1.0.0", true, "Valid semantic version"),
            ("0.0.0", true, "Valid zero version"),
            ("999.999.999", true, "Valid large numbers"),
            ("1.0.0-alpha", true, "Valid prerelease"),
            ("1.0", false, "Missing patch version"),
            ("1.0.0.0", false, "Too many version parts"),
            ("v1.0.0", false, "Contains 'v' prefix"),
            ("-1.0.0", false, "Negative major version"),
            ("1.-1.0", false, "Negative minor version"),
            ("1.0.-1", false, "Negative patch version"),
        ];

        for (version, should_be_valid, description) in validation_cases {
            let is_valid = validate_semver(version);
            assert_eq!(is_valid, should_be_valid, "{}: {}", description, version);
        }
    }

    fn validate_semver(version: &str) -> bool {
        if version.starts_with('v') || version.is_empty() {
            return false;
        }

        let parts: Vec<&str> = version.split('-').collect();
        let version_part = parts[0];

        let version_numbers: Result<Vec<u32>, _> =
            version_part.split('.').map(|v| v.parse::<u32>()).collect();

        match version_numbers {
            Ok(numbers) => numbers.len() == 3,
            Err(_) => false,
        }
    }

    #[test]
    fn test_version_history_tracking() {
        #[derive(Debug)]
        struct VersionEntry {
            version: String,
            timestamp: String,
            changes: Vec<String>,
        }

        let version_history = vec![
            VersionEntry {
                version: "1.0.0".to_string(),
                timestamp: "2025-01-01".to_string(),
                changes: vec!["Initial release".to_string()],
            },
            VersionEntry {
                version: "1.1.0".to_string(),
                timestamp: "2025-02-01".to_string(),
                changes: vec!["Added new features".to_string(), "Bug fixes".to_string()],
            },
            VersionEntry {
                version: "2.0.0".to_string(),
                timestamp: "2025-03-01".to_string(),
                changes: vec!["Breaking changes".to_string(), "Major refactor".to_string()],
            },
        ];

        assert_eq!(version_history.len(), 3);
        assert_eq!(version_history[0].version, "1.0.0");
        assert_eq!(version_history[0].timestamp, "2025-01-01");
        assert_eq!(version_history[2].changes.len(), 2);
    }

    #[test]
    fn test_version_rollback_scenario() {
        let current_version = "2.1.0";
        let previous_version = "2.0.0";

        // Simulate rollback decision
        let rollback_needed = true;

        if rollback_needed {
            let rollback_version = previous_version;
            assert_eq!(rollback_version, "2.0.0");

            // Validate rollback version is lower than current
            let current_parts: Vec<u32> = current_version
                .split('.')
                .map(|v| v.parse().unwrap())
                .collect();
            let rollback_parts: Vec<u32> = rollback_version
                .split('.')
                .map(|v| v.parse().unwrap())
                .collect();

            assert!(rollback_parts < current_parts);
        }
    }

    #[test]
    fn test_multiple_file_version_sync() {
        let temp_dir = tempdir().unwrap();

        // Create multiple files with versions
        let files = vec![
            ("Cargo.toml", r#"version = "1.0.0""#),
            ("package.json", r#""version": "1.0.0""#),
            ("version.txt", "1.0.0"),
        ];

        for (filename, content) in &files {
            let file_path = temp_dir.path().join(filename);
            fs::write(&file_path, content).unwrap();
        }

        // Update all files to new version
        let new_version = "1.1.0";
        for (filename, _) in files {
            let file_path = temp_dir.path().join(filename);
            let content = fs::read_to_string(&file_path).unwrap();

            let updated_content = content.replace("1.0.0", new_version);
            fs::write(&file_path, updated_content).unwrap();

            // Verify update
            let final_content = fs::read_to_string(&file_path).unwrap();
            assert!(final_content.contains(new_version));
            assert!(!final_content.contains("1.0.0"));
        }
    }

    #[test]
    fn test_version_constraint_checking() {
        let constraints = vec![
            ("^1.0.0", "1.2.3", true),   // Compatible
            ("^1.0.0", "2.0.0", false),  // Major version change
            ("~1.2.0", "1.2.5", true),   // Patch compatible
            ("~1.2.0", "1.3.0", false),  // Minor version change
            (">=1.0.0", "1.5.0", true),  // Greater than minimum
            (">=1.0.0", "0.9.0", false), // Less than minimum
        ];

        for (constraint, version, should_match) in constraints {
            // Simple constraint checking logic
            let matches = match constraint.chars().next().unwrap() {
                '^' => {
                    let constraint_version = &constraint[1..];
                    let constraint_major: u32 = constraint_version
                        .split('.')
                        .next()
                        .unwrap()
                        .parse()
                        .unwrap();
                    let version_major: u32 = version.split('.').next().unwrap().parse().unwrap();
                    constraint_major == version_major
                }
                '~' => {
                    let constraint_version = &constraint[1..];
                    let constraint_parts: Vec<u32> = constraint_version
                        .split('.')
                        .map(|v| v.parse().unwrap())
                        .collect();
                    let version_parts: Vec<u32> =
                        version.split('.').map(|v| v.parse().unwrap()).collect();
                    constraint_parts[0] == version_parts[0]
                        && constraint_parts[1] == version_parts[1]
                }
                '>' => {
                    let constraint_version = &constraint[2..]; // Skip ">="
                    let constraint_parts: Vec<u32> = constraint_version
                        .split('.')
                        .map(|v| v.parse().unwrap())
                        .collect();
                    let version_parts: Vec<u32> =
                        version.split('.').map(|v| v.parse().unwrap()).collect();
                    version_parts >= constraint_parts
                }
                _ => false,
            };

            assert_eq!(
                matches, should_match,
                "Constraint {} with version {}",
                constraint, version
            );
        }
    }
}
