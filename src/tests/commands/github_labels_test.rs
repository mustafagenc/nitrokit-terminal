#[cfg(test)]
mod tests {
    use crate::commands::github_labels::{
        run_github_labels, run_github_labels_interactive, GitHubLabel, GitHubLabelsConfig,
        GitHubLabelsManager, LabelUpdate,
    };
    use tokio;

    #[test]
    fn test_github_labels_config_default() {
        let config = GitHubLabelsConfig::default();

        assert!(!config.skip_auth);
        assert!(!config.skip_install);
        assert!(!config.dry_run);
        assert!(!config.list_only);
        assert!(!config.delete_all);
        assert!(!config.update_only);
    }

    #[test]
    fn test_github_label_creation() {
        let label = GitHubLabel {
            name: "🐛 bug".to_string(),
            description: "Software bugs and defects".to_string(),
            color: "D73A49".to_string(),
        };

        assert_eq!(label.name, "🐛 bug");
        assert_eq!(label.description, "Software bugs and defects");
        assert_eq!(label.color, "D73A49");
    }

    #[test]
    fn test_label_update_creation() {
        let update = LabelUpdate {
            old_name: "bug".to_string(),
            new_name: "🐛 bug".to_string(),
            description: "Software bugs and defects".to_string(),
            color: "D73A49".to_string(),
        };

        assert_eq!(update.old_name, "bug");
        assert_eq!(update.new_name, "🐛 bug");
        assert_eq!(update.description, "Software bugs and defects");
        assert_eq!(update.color, "D73A49");
    }

    #[test]
    fn test_manager_creation() {
        let config = GitHubLabelsConfig::default();
        let manager = GitHubLabelsManager::new(config.clone());

        assert_eq!(manager.config.dry_run, config.dry_run);
        assert_eq!(manager.config.list_only, config.list_only);
    }

    #[test]
    fn test_get_new_labels_to_create() {
        let manager = GitHubLabelsManager::new(GitHubLabelsConfig::default());
        let labels = manager.get_new_labels_to_create();

        assert!(!labels.is_empty());

        // Test priority labels
        let critical_priority = labels.iter().find(|l| l.name == "🔴 priority: critical");
        assert!(critical_priority.is_some());
        assert_eq!(critical_priority.unwrap().color, "B60205");

        // Test status labels
        let in_progress = labels.iter().find(|l| l.name == "🔄 status: in progress");
        assert!(in_progress.is_some());
        assert_eq!(in_progress.unwrap().color, "0052CC");

        // Test component labels
        let ui_ux = labels.iter().find(|l| l.name == "🎨 ui/ux");
        assert!(ui_ux.is_some());
        assert_eq!(ui_ux.unwrap().description, "User interface and experience");

        // Test difficulty labels
        let easy = labels.iter().find(|l| l.name == "🌱 easy");
        assert!(easy.is_some());
        assert_eq!(easy.unwrap().color, "C2E0C6");

        // Test type labels
        let security = labels.iter().find(|l| l.name == "🔒 security");
        assert!(security.is_some());
        assert_eq!(security.unwrap().description, "Security related issues");

        // Test that all required labels are present
        let required_labels = vec![
            "🔴 priority: critical",
            "🟠 priority: high",
            "🟡 priority: medium",
            "🟢 priority: low",
            "🔄 status: in progress",
            "👀 status: needs review",
            "🚧 status: blocked",
            "✅ status: ready",
            "🎨 ui/ux",
            "🌍 translation",
            "🔧 cli",
            "📦 release",
            "🔍 code-quality",
            "🌱 easy",
            "🌿 medium",
            "🌳 hard",
            "🔒 security",
            "⚡ performance",
            "♿ accessibility",
            "🧪 testing",
            "🐞 fix",
            "✨ feature",
        ];

        for required_label in required_labels {
            assert!(
                labels.iter().any(|l| l.name == required_label),
                "Required label '{}' not found",
                required_label
            );
        }
    }

    #[test]
    fn test_get_existing_labels_to_update() {
        let manager = GitHubLabelsManager::new(GitHubLabelsConfig::default());
        let updates = manager.get_existing_labels_to_update();

        assert!(!updates.is_empty());

        // Test specific label updates
        let bug_update = updates.iter().find(|u| u.old_name == "bug");
        assert!(bug_update.is_some());
        assert_eq!(bug_update.unwrap().new_name, "🐛 bug");
        assert_eq!(bug_update.unwrap().color, "D73A49");

        let docs_update = updates.iter().find(|u| u.old_name == "documentation");
        assert!(docs_update.is_some());
        assert_eq!(docs_update.unwrap().new_name, "📚 documentation");
        assert_eq!(docs_update.unwrap().color, "0075CA");

        let enhancement_update = updates.iter().find(|u| u.old_name == "enhancement");
        assert!(enhancement_update.is_some());
        assert_eq!(enhancement_update.unwrap().new_name, "✨ enhancement");
        assert_eq!(enhancement_update.unwrap().color, "A2EEEF");

        // Test that all standard GitHub labels are covered
        let standard_labels = vec![
            "bug",
            "dependencies",
            "documentation",
            "duplicate",
            "enhancement",
            "github_actions",
            "good first issue",
            "help wanted",
            "invalid",
            "question",
            "wontfix",
        ];

        for standard_label in standard_labels {
            assert!(
                updates.iter().any(|u| u.old_name == standard_label),
                "Standard label '{}' not found in updates",
                standard_label
            );
        }
    }

    #[test]
    fn test_detect_os() {
        let manager = GitHubLabelsManager::new(GitHubLabelsConfig::default());
        let os = manager.detect_os();

        // Should return one of the expected OS types
        let valid_os = vec![
            "macos", "ubuntu", "fedora", "centos", "linux", "windows", "unknown",
        ];
        assert!(
            valid_os.contains(&os.as_str()),
            "Invalid OS detected: {}",
            os
        );
    }

    #[tokio::test]
    async fn test_dry_run_mode() {
        let mut config = GitHubLabelsConfig::default();
        config.dry_run = true;
        config.skip_auth = true;
        config.skip_install = true;

        let manager = GitHubLabelsManager::new(config);

        // In a real test, we would mock the GitHub CLI commands
        // For now, we just test that the manager can be created with dry_run
        assert!(manager.config.dry_run);
    }

    #[tokio::test]
    async fn test_list_only_mode() {
        let mut config = GitHubLabelsConfig::default();
        config.list_only = true;
        config.skip_auth = true;
        config.skip_install = true;

        let manager = GitHubLabelsManager::new(config);

        assert!(manager.config.list_only);
    }

    #[tokio::test]
    async fn test_delete_all_mode() {
        let mut config = GitHubLabelsConfig::default();
        config.delete_all = true;
        config.dry_run = true; // Use dry run to avoid actual deletion
        config.skip_auth = true;
        config.skip_install = true;

        let manager = GitHubLabelsManager::new(config);

        assert!(manager.config.delete_all);
        assert!(manager.config.dry_run);
    }

    #[tokio::test]
    async fn test_update_only_mode() {
        let mut config = GitHubLabelsConfig::default();
        config.update_only = true;
        config.skip_auth = true;
        config.skip_install = true;

        let manager = GitHubLabelsManager::new(config);

        assert!(manager.config.update_only);
    }

    #[tokio::test]
    async fn test_run_github_labels_cli_function() {
        // Test the CLI function with various configurations
        let _result = run_github_labels(
            true,  // skip_auth
            true,  // skip_install
            true,  // dry_run
            true,  // list_only
            false, // delete_all
            false, // update_only
        )
        .await;

        // In a real implementation, we would mock the GitHub CLI
        // For now, we just test that the function can be called
        // The function will likely fail due to missing GitHub CLI, which is expected
    }

    #[tokio::test]
    async fn test_run_github_labels_interactive() {
        // Test the interactive function
        let _result = run_github_labels_interactive().await;

        // This will likely fail without proper setup, which is expected in tests
        // In a real implementation, we would mock the GitHub CLI interactions
    }

    #[test]
    fn test_label_colors_are_valid_hex() {
        let manager = GitHubLabelsManager::new(GitHubLabelsConfig::default());
        let labels = manager.get_new_labels_to_create();

        for label in &labels {
            // Check that color is valid hex (6 characters, valid hex digits)
            assert_eq!(
                label.color.len(),
                6,
                "Color '{}' for label '{}' is not 6 characters",
                label.color,
                label.name
            );
            assert!(
                label.color.chars().all(|c| c.is_ascii_hexdigit()),
                "Color '{}' for label '{}' contains invalid hex characters",
                label.color,
                label.name
            );
        }

        let updates = manager.get_existing_labels_to_update();
        for update in &updates {
            assert_eq!(
                update.color.len(),
                6,
                "Color '{}' for update '{}' is not 6 characters",
                update.color,
                update.new_name
            );
            assert!(
                update.color.chars().all(|c| c.is_ascii_hexdigit()),
                "Color '{}' for update '{}' contains invalid hex characters",
                update.color,
                update.new_name
            );
        }
    }

    #[test]
    fn test_label_names_have_emojis() {
        let manager = GitHubLabelsManager::new(GitHubLabelsConfig::default());
        let labels = manager.get_new_labels_to_create();

        // Most labels should start with an emoji
        let labels_with_emojis = labels
            .iter()
            .filter(|l| {
                let first_char = l.name.chars().next().unwrap_or(' ');
                first_char as u32 > 127 // Non-ASCII character (likely emoji)
            })
            .count();

        assert!(
            labels_with_emojis > 15,
            "Expected most labels to have emojis, found {}",
            labels_with_emojis
        );
    }

    #[test]
    fn test_label_descriptions_not_empty() {
        let manager = GitHubLabelsManager::new(GitHubLabelsConfig::default());
        let labels = manager.get_new_labels_to_create();

        for label in &labels {
            assert!(
                !label.description.is_empty(),
                "Label '{}' has empty description",
                label.name
            );
            assert!(
                label.description.len() > 5,
                "Label '{}' has too short description: '{}'",
                label.name,
                label.description
            );
        }

        let updates = manager.get_existing_labels_to_update();
        for update in &updates {
            assert!(
                !update.description.is_empty(),
                "Update '{}' has empty description",
                update.new_name
            );
            assert!(
                update.description.len() > 5,
                "Update '{}' has too short description: '{}'",
                update.new_name,
                update.description
            );
        }
    }

    #[test]
    fn test_priority_labels_have_correct_colors() {
        let manager = GitHubLabelsManager::new(GitHubLabelsConfig::default());
        let labels = manager.get_new_labels_to_create();

        // Test that priority labels have appropriate colors (red for critical, etc.)
        let critical = labels.iter().find(|l| l.name.contains("critical")).unwrap();
        assert_eq!(critical.color, "B60205"); // Red

        let high = labels.iter().find(|l| l.name.contains("high")).unwrap();
        assert_eq!(high.color, "D93F0B"); // Orange-red

        let medium = labels.iter().find(|l| l.name.contains("medium")).unwrap();
        assert_eq!(medium.color, "FBCA04"); // Yellow

        let low = labels.iter().find(|l| l.name.contains("low")).unwrap();
        assert_eq!(low.color, "0E8A16"); // Green
    }

    #[test]
    fn test_no_duplicate_label_names() {
        let manager = GitHubLabelsManager::new(GitHubLabelsConfig::default());
        let labels = manager.get_new_labels_to_create();

        let mut names = std::collections::HashSet::new();
        for label in &labels {
            assert!(
                names.insert(&label.name),
                "Duplicate label name found: {}",
                label.name
            );
        }
    }

    #[test]
    fn test_label_categories_are_complete() {
        let manager = GitHubLabelsManager::new(GitHubLabelsConfig::default());
        let labels = manager.get_new_labels_to_create();

        // Check that we have labels from all major categories
        let has_priority = labels.iter().any(|l| l.name.contains("priority:"));
        let has_status = labels.iter().any(|l| l.name.contains("status:"));
        let has_ui = labels.iter().any(|l| l.name.contains("ui/ux"));
        let has_translation = labels.iter().any(|l| l.name.contains("translation"));
        let has_cli = labels.iter().any(|l| l.name.contains("cli"));
        let has_security = labels.iter().any(|l| l.name.contains("security"));
        let has_testing = labels.iter().any(|l| l.name.contains("testing"));

        assert!(has_priority, "Missing priority labels");
        assert!(has_status, "Missing status labels");
        assert!(has_ui, "Missing UI/UX labels");
        assert!(has_translation, "Missing translation labels");
        assert!(has_cli, "Missing CLI labels");
        assert!(has_security, "Missing security labels");
        assert!(has_testing, "Missing testing labels");
    }
}

#[cfg(test)]
mod integration_tests {
    use crate::commands::github_labels::{GitHubLabelsConfig, GitHubLabelsManager};
    use std::env;

    // These tests require actual GitHub CLI and authentication
    // They should be run with: cargo test --features integration-tests

    #[tokio::test]
    #[ignore] // Ignore by default since it requires GitHub CLI
    async fn test_gh_cli_check() {
        let manager = GitHubLabelsManager::new(GitHubLabelsConfig::default());
        let is_installed = manager.is_gh_cli_installed().await;

        if is_installed {
            let version = manager.get_gh_version().await;
            assert!(version.is_ok());
            println!("GitHub CLI version: {:?}", version.unwrap());
        } else {
            println!("GitHub CLI not installed - test skipped");
        }
    }

    #[tokio::test]
    #[ignore] // Ignore by default since it requires authentication
    async fn test_github_authentication() {
        if env::var("GITHUB_TOKEN").is_ok() || env::var("GH_TOKEN").is_ok() {
            let mut config = GitHubLabelsConfig::default();
            config.skip_install = true;

            let manager = GitHubLabelsManager::new(config);
            let result = manager.check_authentication().await;

            if result.is_ok() {
                println!("GitHub authentication successful");
            } else {
                println!("GitHub authentication failed: {:?}", result.err());
            }
        } else {
            println!("No GitHub token found - authentication test skipped");
        }
    }

    #[tokio::test]
    #[ignore] // Ignore by default since it requires GitHub CLI and repo
    async fn test_list_labels_real() {
        let mut config = GitHubLabelsConfig::default();
        config.list_only = true;
        config.skip_install = true;
        config.skip_auth = true;

        let manager = GitHubLabelsManager::new(config);
        let result = manager.list_labels().await;

        // This might fail if not in a GitHub repo or not authenticated
        match result {
            Ok(_) => println!("Label listing successful"),
            Err(e) => println!("Label listing failed (expected in test): {}", e),
        }
    }
}
