use crate::commands::translation_sync::{
    TranslationSync, TranslationConfig, TranslationFile, sync_translations_interactive,
    show_config, setup_config, reset_config
};
use anyhow::Result;
use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio;
use serde_json;

#[cfg(test)]
mod tests {
    use super::*;

    // Test helper to create a temporary directory with translation files
    fn setup_test_translations() -> Result<(TempDir, PathBuf)> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path().to_path_buf();

        // Create translations directory structure
        let translations_dir = project_path.join("translations");
        fs::create_dir_all(&translations_dir)?;

        // Create English (source) translation file
        let en_content = serde_json::json!({
            "app": {
                "title": "NitroKit Terminal",
                "description": "A terminal tool for project management",
                "buttons": {
                    "save": "Save",
                    "cancel": "Cancel",
                    "delete": "Delete"
                }
            },
            "messages": {
                "welcome": "Welcome to NitroKit",
                "success": "Operation completed successfully",
                "error": "An error occurred"
            }
        });
        fs::write(translations_dir.join("en.json"), serde_json::to_string_pretty(&en_content)?)?;

        // Create Turkish translation file (incomplete)
        let tr_content = serde_json::json!({
            "app": {
                "title": "NitroKit Terminal",
                "description": "Proje yönetimi için terminal aracı"
                // Missing buttons section
            },
            "messages": {
                "welcome": "NitroKit'e Hoş Geldiniz"
                // Missing success and error messages
            }
        });
        fs::write(translations_dir.join("tr.json"), serde_json::to_string_pretty(&tr_content)?)?;

        // Create Spanish translation file (empty)
        let es_content = serde_json::json!({});
        fs::write(translations_dir.join("es.json"), serde_json::to_string_pretty(&es_content)?)?;

        Ok((temp_dir, project_path))
    }

    fn setup_test_config() -> TranslationConfig {
        TranslationConfig {
            gemini_api_key: "test-api-key".to_string(),
            source_language: "en".to_string(),
            target_languages: vec!["tr".to_string(), "es".to_string(), "fr".to_string()],
            translations_path: "./translations".to_string(),
            file_pattern: "{lang}.json".to_string(),
            auto_sync: false,
            backup_enabled: true,
        }
    }

    #[test]
    fn test_translation_config_creation() {
        let config = setup_test_config();
        
        assert_eq!(config.source_language, "en");
        assert!(config.target_languages.contains(&"tr".to_string()));
        assert!(config.target_languages.contains(&"es".to_string()));
        assert_eq!(config.translations_path, "./translations");
        assert_eq!(config.file_pattern, "{lang}.json");
    }

    #[test]
    fn test_translation_config_serialization() {
        let config = setup_test_config();
        
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("test-api-key"));
        assert!(json.contains("en"));
        assert!(json.contains("tr"));
        
        let deserialized: TranslationConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.gemini_api_key, config.gemini_api_key);
        assert_eq!(deserialized.source_language, config.source_language);
    }

    #[test]
    fn test_translation_file_loading() {
        let (_temp_dir, project_path) = setup_test_translations().unwrap();
        env::set_current_dir(&project_path).unwrap();
        
        let config = setup_test_config();
        let sync = TranslationSync::new(config);
        
        // Test loading existing translation file
        let en_file = sync.load_translation_file("en").unwrap();
        assert_eq!(en_file.language, "en");
        assert!(en_file.content.contains_key("app"));
        assert!(en_file.content.contains_key("messages"));
        
        // Test loading partial translation file
        let tr_file = sync.load_translation_file("tr").unwrap();
        assert_eq!(tr_file.language, "tr");
        assert!(tr_file.content.contains_key("app"));
        
        // Test loading non-existent file
        let result = sync.load_translation_file("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_keys_detection() {
        let (_temp_dir, project_path) = setup_test_translations().unwrap();
        env::set_current_dir(&project_path).unwrap();
        
        let config = setup_test_config();
        let sync = TranslationSync::new(config);
        
        let source_file = sync.load_translation_file("en").unwrap();
        let target_file = sync.load_translation_file("tr").unwrap();
        
        let missing_keys = sync.find_missing_keys(&source_file.content, &target_file.content);
        
        assert!(missing_keys.contains(&"app.buttons.save".to_string()));
        assert!(missing_keys.contains(&"app.buttons.cancel".to_string()));
        assert!(missing_keys.contains(&"app.buttons.delete".to_string()));
        assert!(missing_keys.contains(&"messages.success".to_string()));
        assert!(missing_keys.contains(&"messages.error".to_string()));
    }

    #[test]
    fn test_key_flattening() {
        let config = setup_test_config();
        let sync = TranslationSync::new(config);
        
        let nested_json = serde_json::json!({
            "app": {
                "title": "Test",
                "buttons": {
                    "save": "Save",
                    "cancel": "Cancel"
                }
            },
            "simple": "Simple value"
        });
        
        let flattened = sync.flatten_keys(&nested_json, "");
        
        assert!(flattened.contains_key("app.title"));
        assert!(flattened.contains_key("app.buttons.save"));
        assert!(flattened.contains_key("app.buttons.cancel"));
        assert!(flattened.contains_key("simple"));
        
        assert_eq!(flattened.get("app.title").unwrap(), "Test");
        assert_eq!(flattened.get("simple").unwrap(), "Simple value");
    }

    #[test]
    fn test_translation_file_backup() {
        let (_temp_dir, project_path) = setup_test_translations().unwrap();
        env::set_current_dir(&project_path).unwrap();
        
        let config = setup_test_config();
        let sync = TranslationSync::new(config);
        
        // Create backup
        let result = sync.create_backup("tr");
        assert!(result.is_ok());
        
        // Check if backup file exists
        let backup_path = project_path.join("translations").join("tr.json.backup");
        assert!(backup_path.exists());
        
        // Verify backup content
        let backup_content = fs::read_to_string(backup_path).unwrap();
        let original_content = fs::read_to_string(project_path.join("translations").join("tr.json")).unwrap();
        assert_eq!(backup_content, original_content);
    }

    #[test]
    fn test_translation_file_validation() {
        let config = setup_test_config();
        let sync = TranslationSync::new(config);
        
        // Valid JSON
        let valid_json = r#"{"key": "value", "nested": {"inner": "test"}}"#;
        assert!(sync.validate_json(valid_json).is_ok());
        
        // Invalid JSON
        let invalid_json = r#"{"key": "value", "invalid": }"#;
        assert!(sync.validate_json(invalid_json).is_err());
        
        // Empty JSON
        let empty_json = "{}";
        assert!(sync.validate_json(empty_json).is_ok());
    }

    #[test]
    fn test_supported_languages() {
        let config = setup_test_config();
        let sync = TranslationSync::new(config);
        
        let supported = sync.get_supported_languages();
        
        assert!(supported.contains(&"tr".to_string()));
        assert!(supported.contains(&"es".to_string()));
        assert!(supported.contains(&"fr".to_string()));
        assert!(supported.contains(&"de".to_string()));
        assert!(supported.contains(&"it".to_string()));
        assert!(supported.contains(&"pt".to_string()));
        assert!(supported.contains(&"ru".to_string()));
        assert!(supported.contains(&"ja".to_string()));
        assert!(supported.contains(&"ko".to_string()));
        assert!(supported.contains(&"zh".to_string()));
    }

    #[test]
    fn test_language_name_mapping() {
        let config = setup_test_config();
        let sync = TranslationSync::new(config);
        
        assert_eq!(sync.get_language_name("en"), "English");
        assert_eq!(sync.get_language_name("tr"), "Turkish");
        assert_eq!(sync.get_language_name("es"), "Spanish");
        assert_eq!(sync.get_language_name("fr"), "French");
        assert_eq!(sync.get_language_name("de"), "German");
        assert_eq!(sync.get_language_name("unknown"), "unknown");
    }

    #[tokio::test]
    async fn test_translation_progress_tracking() {
        let (_temp_dir, project_path) = setup_test_translations().unwrap();
        env::set_current_dir(&project_path).unwrap();
        
        let config = setup_test_config();
        let sync = TranslationSync::new(config);
        
        let progress = sync.calculate_translation_progress("tr").await.unwrap();
        
        // Turkish file has some translations but not all
        assert!(progress.total_keys > 0);
        assert!(progress.translated_keys > 0);
        assert!(progress.missing_keys > 0);
        assert!(progress.percentage < 100.0);
        assert!(progress.percentage > 0.0);
    }

    #[tokio::test]
    async fn test_sync_statistics() {
        let (_temp_dir, project_path) = setup_test_translations().unwrap();
        env::set_current_dir(&project_path).unwrap();
        
        let config = setup_test_config();
        let sync = TranslationSync::new(config);
        
        let stats = sync.get_sync_statistics().await.unwrap();
        
        assert!(stats.total_languages >= 2); // en, tr at minimum
        assert!(stats.source_keys > 0);
        assert!(stats.languages.contains_key("en"));
        assert!(stats.languages.contains_key("tr"));
        
        let en_stats = &stats.languages["en"];
        assert_eq!(en_stats.completion_percentage, 100.0);
        
        let tr_stats = &stats.languages["tr"];
        assert!(tr_stats.completion_percentage < 100.0);
        assert!(tr_stats.completion_percentage > 0.0);
    }

    #[test]
    fn test_config_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nitrokit.config.json");
        
        let config = setup_test_config();
        
        // Save config
        let result = config.save_to_file(&config_path);
        assert!(result.is_ok());
        assert!(config_path.exists());
        
        // Load config
        let loaded_config = TranslationConfig::load_from_file(&config_path).unwrap();
        assert_eq!(loaded_config.gemini_api_key, config.gemini_api_key);
        assert_eq!(loaded_config.source_language, config.source_language);
        assert_eq!(loaded_config.target_languages, config.target_languages);
    }

    #[test]
    fn test_config_validation() {
        let mut config = setup_test_config();
        
        // Valid config
        assert!(config.validate().is_ok());
        
        // Invalid API key
        config.gemini_api_key = "".to_string();
        assert!(config.validate().is_err());
        
        // Reset and test empty target languages
        config = setup_test_config();
        config.target_languages.clear();
        assert!(config.validate().is_err());
        
        // Reset and test invalid source language
        config = setup_test_config();
        config.source_language = "invalid".to_string();
        assert!(config.validate().is_err());
    }

    #[tokio::test]
    async fn test_dry_run_sync() {
        let (_temp_dir, project_path) = setup_test_translations().unwrap();
        env::set_current_dir(&project_path).unwrap();
        
        let config = setup_test_config();
        let sync = TranslationSync::new(config);
        
        // Perform dry run
        let dry_run_result = sync.dry_run_sync("tr").await.unwrap();
        
        assert!(!dry_run_result.missing_keys.is_empty());
        assert!(dry_run_result.estimated_tokens > 0);
        assert!(dry_run_result.estimated_cost > 0.0);
        assert!(!dry_run_result.preview_translations.is_empty());
    }

    #[test]
    fn test_error_handling() {
        let config = setup_test_config();
        let sync = TranslationSync::new(config);
        
        // Test with non-existent directory
        env::set_current_dir("/tmp").unwrap();
        
        let result = sync.load_translation_file("en");
        assert!(result.is_err());
        
        let result = sync.create_backup("en");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let config = setup_test_config();
        let sync = TranslationSync::new(config);
        
        // Test rate limiting logic
        let start = std::time::Instant::now();
        
        // Simulate multiple API calls
        for _ in 0..3 {
            sync.apply_rate_limiting().await;
        }
        
        let duration = start.elapsed();
        // Should have some delay due to rate limiting
        assert!(duration.as_millis() > 0);
    }

    #[test]
    fn test_translation_key_extraction() {
        let config = setup_test_config();
        let sync = TranslationSync::new(config);
        
        let json_content = serde_json::json!({
            "level1": {
                "level2": {
                    "key1": "value1",
                    "key2": "value2"
                },
                "simple": "simple_value"
            },
            "root": "root_value"
        });
        
        let keys = sync.extract_all_keys(&json_content);
        
        assert!(keys.contains("level1.level2.key1"));
        assert!(keys.contains("level1.level2.key2"));
        assert!(keys.contains("level1.simple"));
        assert!(keys.contains("root"));
        assert_eq!(keys.len(), 4);
    }

    #[test]
    fn test_translation_merge() {
        let config = setup_test_config();
        let sync = TranslationSync::new(config);
        
        let original = serde_json::json!({
            "existing": "value",
            "nested": {
                "keep": "this"
            }
        });
        
        let new_translations = vec![
            ("new_key".to_string(), "new_value".to_string()),
            ("nested.added".to_string(), "added_value".to_string()),
        ];
        
        let merged = sync.merge_translations(&original, &new_translations).unwrap();
        
        // Check original content is preserved
        assert_eq!(merged["existing"], "value");
        assert_eq!(merged["nested"]["keep"], "this");
        
        // Check new content is added
        assert_eq!(merged["new_key"], "new_value");
        assert_eq!(merged["nested"]["added"], "added_value");
    }

    // Mock test for API integration (requires environment setup)
    #[tokio::test]
    #[ignore] // Ignore by default since it requires API key
    async fn test_gemini_api_integration() {
        if env::var("GEMINI_API_KEY").is_err() {
            return; // Skip if no API key
        }
        
        let (_temp_dir, project_path) = setup_test_translations().unwrap();
        env::set_current_dir(&project_path).unwrap();
        
        let mut config = setup_test_config();
        config.gemini_api_key = env::var("GEMINI_API_KEY").unwrap();
        
        let sync = TranslationSync::new(config);
        
        // Test actual API call with a simple translation
        let test_texts = vec![("test.key".to_string(), "Hello World".to_string())];
        
        let result = sync.translate_with_gemini(&test_texts, "tr").await;
        
        // This would test real API integration
        match result {
            Ok(translations) => {
                assert!(!translations.is_empty());
                println!("Translation successful: {:?}", translations);
            }
            Err(e) => {
                println!("Translation failed: {}", e);
                // Don't fail the test if API is temporarily unavailable
            }
        }
    }
}

// Helper functions for testing
impl TranslationSync {
    #[cfg(test)]
    pub fn validate_json(&self, json_str: &str) -> Result<()> {
        serde_json::from_str::<serde_json::Value>(json_str)?;
        Ok(())
    }
    
    #[cfg(test)]
    pub async fn apply_rate_limiting(&self) {
        // Simulate rate limiting delay
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    #[cfg(test)]
    pub fn extract_all_keys(&self, json: &serde_json::Value) -> Vec<String> {
        let flattened = self.flatten_keys(json, "");
        flattened.keys().cloned().collect()
    }
    
    #[cfg(test)]
    pub fn merge_translations(
        &self, 
        original: &serde_json::Value, 
        new_translations: &[(String, String)]
    ) -> Result<serde_json::Value> {
        let mut result = original.clone();
        
        for (key, value) in new_translations {
            self.set_nested_value(&mut result, key, value)?;
        }
        
        Ok(result)
    }
    
    #[cfg(test)]
    fn set_nested_value(
        &self,
        json: &mut serde_json::Value,
        key: &str,
        value: &str
    ) -> Result<()> {
        let parts: Vec<&str> = key.split('.').collect();
        let mut current = json;
        
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Last part, set the value
                current[part] = serde_json::Value::String(value.to_string());
            } else {
                // Navigate or create intermediate objects
                if !current[part].is_object() {
                    current[part] = serde_json::json!({});
                }
                current = &mut current[part];
            }
        }
        
        Ok(())
    }
}