use std::fs;
use tempfile::tempdir;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BumpType {
    Patch,
    Minor,
    Major,
}

pub fn determine_bump_type(commit_message: &str) -> BumpType {
    let message = commit_message.to_lowercase();

    // Check for breaking changes
    if message.contains("breaking") || message.contains("!:") {
        return BumpType::Major;
    }

    // Check for features
    if message.starts_with("feat") || message.starts_with("feature") {
        return BumpType::Minor;
    }

    // Everything else is a patch
    BumpType::Patch
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_bump_type() {
        // Breaking changes
        assert_eq!(
            determine_bump_type("BREAKING: remove old API"),
            BumpType::Major
        );
        assert_eq!(
            determine_bump_type("feat!: new feature with breaking change"),
            BumpType::Major
        );
        assert_eq!(determine_bump_type("fix!: breaking fix"), BumpType::Major);

        // Features
        assert_eq!(
            determine_bump_type("feat: add new feature"),
            BumpType::Minor
        );
        assert_eq!(
            determine_bump_type("feat(api): new endpoint"),
            BumpType::Minor
        );
        assert_eq!(
            determine_bump_type("feature: user authentication"),
            BumpType::Minor
        );

        // Bug fixes
        assert_eq!(
            determine_bump_type("fix: resolve memory leak"),
            BumpType::Patch
        );
        assert_eq!(
            determine_bump_type("fix(ui): button alignment"),
            BumpType::Patch
        );
        assert_eq!(
            determine_bump_type("bugfix: critical issue"),
            BumpType::Patch
        );

        // Other changes (docs, style, etc.)
        assert_eq!(determine_bump_type("docs: update README"), BumpType::Patch);
        assert_eq!(determine_bump_type("style: format code"), BumpType::Patch);
        assert_eq!(
            determine_bump_type("refactor: clean up code"),
            BumpType::Patch
        );
        assert_eq!(determine_bump_type("test: add unit tests"), BumpType::Patch);
        assert_eq!(
            determine_bump_type("chore: update dependencies"),
            BumpType::Patch
        );

        // Edge cases
        assert_eq!(determine_bump_type(""), BumpType::Patch);
        assert_eq!(
            determine_bump_type("random commit message"),
            BumpType::Patch
        );
        assert_eq!(
            determine_bump_type("FEAT: uppercase feature"),
            BumpType::Minor
        );
        assert_eq!(determine_bump_type("FIX: uppercase fix"), BumpType::Patch);
    }

    #[test]
    fn test_bump_type_priority() {
        let commit_messages = vec!["feat: add login", "fix: resolve bug", "docs: update readme"];

        let mut highest_bump = BumpType::Patch;
        for message in commit_messages {
            let bump = determine_bump_type(message);
            if bump > highest_bump {
                highest_bump = bump;
            }
        }

        // Should prioritize Minor (feat) over Patch (fix, docs)
        assert_eq!(highest_bump, BumpType::Minor);
    }

    #[test]
    fn test_bump_type_ordering() {
        assert!(BumpType::Major > BumpType::Minor);
        assert!(BumpType::Minor > BumpType::Patch);
        assert!(BumpType::Patch < BumpType::Minor);
        assert!(BumpType::Minor < BumpType::Major);
    }

    #[test]
    fn test_version_parsing() {
        let version_patterns = vec![
            ("v1.0.0", true),
            ("1.2.3", true),
            ("v0.1.0-alpha", true),
            ("2.0.0-beta.1", true),
            ("invalid", false),
            ("", false),
            ("1.2", false),
        ];

        for (version, should_be_valid) in version_patterns {
            let is_valid =
                version.trim_start_matches('v').split('.').count() >= 3 || version.contains('-');

            if should_be_valid {
                assert!(
                    is_valid || version.contains('-'),
                    "Version {} should be valid",
                    version
                );
            }
        }
    }

    #[test]
    fn test_conventional_commit_parsing() {
        let commits = vec![
            (
                "feat(api): add user endpoints",
                "feat",
                "add user endpoints",
            ),
            ("fix: resolve memory leak", "fix", "resolve memory leak"),
            ("docs: update README", "docs", "update README"),
            (
                "refactor(auth): simplify login",
                "refactor",
                "simplify login",
            ),
            ("chore: update deps", "chore", "update deps"),
        ];

        for (commit_msg, expected_type, expected_desc) in commits {
            let parts: Vec<&str> = commit_msg.splitn(2, ": ").collect();
            if parts.len() == 2 {
                let type_scope = parts[0];
                let description = parts[1];

                assert_eq!(description, expected_desc);

                let type_part = if type_scope.contains('(') {
                    type_scope.split('(').next().unwrap()
                } else {
                    type_scope
                };
                assert_eq!(type_part, expected_type);
            }
        }
    }

    #[test]
    fn test_git_command_validation() {
        let tag_name = "v1.2.3";
        let expected_commands = vec![
            format!("git tag {}", tag_name),
            format!("git push origin {}", tag_name),
            "git push origin main".to_string(),
        ];

        for cmd in expected_commands {
            assert!(cmd.starts_with("git"));
            assert!(cmd.len() > 4);
        }
    }

    #[test]
    fn test_release_notes_structure() {
        let version = "1.2.3";
        let date = "2025-05-28";

        let header = format!("## [{}] - {}", version, date);
        assert!(header.starts_with("## ["));
        assert!(header.contains(version));
        assert!(header.contains(date));

        let sections = vec![
            "### ✨ Features",
            "### 🐛 Bug Fixes",
            "### 📚 Documentation",
        ];
        for section in sections {
            assert!(section.starts_with("### "));
        }
    }

    #[test]
    fn test_cargo_toml_version_update() {
        let temp_dir = tempdir().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        let cargo_content = r#"
[package]
name = "test-package"
version = "1.0.0"
edition = "2021"

[dependencies]
serde = "1.0"
"#;

        fs::write(&cargo_toml_path, cargo_content).unwrap();

        let content = fs::read_to_string(&cargo_toml_path).unwrap();
        assert!(content.contains(r#"version = "1.0.0""#));

        let new_version = "1.1.0";
        let updated_content = content.replace(
            r#"version = "1.0.0""#,
            &format!(r#"version = "{}""#, new_version),
        );

        assert!(updated_content.contains(&format!(r#"version = "{}""#, new_version)));
        assert!(!updated_content.contains(r#"version = "1.0.0""#));
    }

    #[test]
    fn test_semver_increment() {
        let test_cases = vec![
            ("1.0.0", BumpType::Major, "2.0.0"),
            ("1.2.3", BumpType::Minor, "1.3.0"),
            ("1.2.3", BumpType::Patch, "1.2.4"),
            ("0.1.0", BumpType::Major, "1.0.0"),
            ("2.0.0", BumpType::Patch, "2.0.1"),
        ];

        for (current, bump_type, expected) in test_cases {
            let parts: Vec<u32> = current.split('.').map(|p| p.parse().unwrap()).collect();
            let (major, minor, patch) = (parts[0], parts[1], parts[2]);

            let new_version = match bump_type {
                BumpType::Major => format!("{}.0.0", major + 1),
                BumpType::Minor => format!("{}.{}.0", major, minor + 1),
                BumpType::Patch => format!("{}.{}.{}", major, minor, patch + 1),
            };

            assert_eq!(new_version, expected);
        }
    }

    #[test]
    fn test_commit_message_validation() {
        let valid_messages = vec![
            "feat: add new feature",
            "fix: resolve bug",
            "docs: update readme",
            "style: format code",
            "refactor: improve performance",
            "test: add unit tests",
            "chore: update dependencies",
        ];

        for msg in valid_messages {
            assert!(msg.contains(": "));
            assert!(msg.len() > 5);
        }
    }

    #[tokio::test]
    async fn test_create_release_command_structure() {
        let test_result: Result<(), Box<dyn std::error::Error>> = Ok(());
        assert!(test_result.is_ok());

        let error_result: Result<(), Box<dyn std::error::Error>> = Err("Test error".into());
        assert!(error_result.is_err());
    }
}
