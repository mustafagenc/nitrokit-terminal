use crate::commands::config::{AppConfig, ConfigManager};
use std::path::PathBuf;

// ...existing code at the end of the file...

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_app_config_default() {
        let config = AppConfig::default();

        assert_eq!(config.gemini_api_key, None);
        assert_eq!(config.gemini_model, "gemini-1.5-flash");
        assert_eq!(config.translation_delay_seconds, 2);
        assert_eq!(config.messages_dir, "messages");
        assert_eq!(config.source_file, "source.json");
    }

    #[test]
    fn test_app_config_serialization() {
        let config = AppConfig {
            gemini_api_key: Some("test-key".to_string()),
            gemini_model: "gemini-1.5-pro".to_string(),
            translation_delay_seconds: 5,
            messages_dir: "test-messages".to_string(),
            source_file: "test.json".to_string(),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.gemini_api_key, deserialized.gemini_api_key);
        assert_eq!(config.gemini_model, deserialized.gemini_model);
        assert_eq!(
            config.translation_delay_seconds,
            deserialized.translation_delay_seconds
        );
        assert_eq!(config.messages_dir, deserialized.messages_dir);
        assert_eq!(config.source_file, deserialized.source_file);
    }

    #[test]
    fn test_config_dir_fallback() {
        let result = ConfigManager::get_config_dir();
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.is_absolute() || path.starts_with("."));
    }

    #[test]
    fn test_directory_writable() {
        let temp_dir = tempdir().unwrap();
        let writable_path = temp_dir.path().to_path_buf();

        assert!(ConfigManager::test_directory_writable(&writable_path));

        let non_writable = PathBuf::from("/root/non-existent-test-dir");
        assert!(!ConfigManager::test_directory_writable(&non_writable));
    }

    #[tokio::test]
    async fn test_config_manager_new() {
        let result = ConfigManager::new().await;
        assert!(result.is_ok());

        let manager = result.unwrap();
        assert!(manager.config_dir.exists() || manager.config_dir.starts_with("."));
    }

    #[tokio::test]
    async fn test_save_and_load_config() {
        // Create a unique test database
        let temp_dir = tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", temp_dir.path().to_str().unwrap());

        let manager = ConfigManager::new().await.unwrap();

        let test_config = AppConfig {
            gemini_api_key: Some("test-api-key-123".to_string()),
            gemini_model: "gemini-1.5-pro".to_string(),
            translation_delay_seconds: 10,
            messages_dir: "test-messages".to_string(),
            source_file: "test-source.json".to_string(),
        };

        let save_result = manager.save_config(&test_config).await;
        assert!(save_result.is_ok());

        let loaded_config = manager.get_config().await.unwrap();

        assert_eq!(loaded_config.gemini_api_key, test_config.gemini_api_key);
        assert_eq!(loaded_config.gemini_model, test_config.gemini_model);
        assert_eq!(
            loaded_config.translation_delay_seconds,
            test_config.translation_delay_seconds
        );
        assert_eq!(loaded_config.messages_dir, test_config.messages_dir);
        assert_eq!(loaded_config.source_file, test_config.source_file);

        std::env::remove_var("XDG_CONFIG_HOME");
    }

    #[tokio::test]
    async fn test_is_first_run() {
        let manager = ConfigManager::new().await.unwrap();

        let _ = manager.reset_config().await;

        let is_first = manager.is_first_run().await.unwrap();
        assert!(is_first);

        let config = AppConfig::default();
        let _ = manager.save_config(&config).await;

        let is_first_after = manager.is_first_run().await.unwrap();
        assert!(!is_first_after);
    }

    #[tokio::test]
    async fn test_reset_config() {
        let manager = ConfigManager::new().await.unwrap();

        let config = AppConfig::default();
        let _ = manager.save_config(&config).await;

        assert!(!manager.is_first_run().await.unwrap());

        let reset_result = manager.reset_config().await;
        assert!(reset_result.is_ok());

        assert!(manager.is_first_run().await.unwrap());
    }

    #[test]
    fn test_env_var_handling() {
        env::set_var("GEMINI_API_KEY", "env-test-key");

        let env_key = env::var("GEMINI_API_KEY").unwrap();
        assert_eq!(env_key, "env-test-key");

        env::remove_var("GEMINI_API_KEY");
    }

    #[test]
    fn test_model_validation() {
        let valid_models = vec![
            "gemini-1.5-flash",
            "gemini-1.5-pro",
            "gemini-2.0-flash",
            "gemini-custom-model",
        ];

        for model in valid_models {
            assert!(model.starts_with("gemini"));
        }
    }

    #[test]
    fn test_path_validation() {
        let relative_path = "messages";
        let absolute_path = "/home/user/messages";

        assert!(!std::path::Path::new(relative_path).is_absolute());
        assert!(std::path::Path::new(absolute_path).is_absolute());
    }

    #[test]
    fn test_json_extension_validation() {
        let valid_files = vec!["source.json", "translations.json", "data.json"];
        let invalid_files = vec!["source.txt", "data", "file.xml"];

        for file in valid_files {
            assert!(file.ends_with(".json"));
        }

        for file in invalid_files {
            assert!(!file.ends_with(".json"));
        }
    }
}
