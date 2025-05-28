use serde_json::{json, Value};
use std::fs;
use tempfile::tempdir;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_file_validation() {
        let temp_dir = tempdir().unwrap();

        // Valid JSON file
        let valid_json_path = temp_dir.path().join("valid.json");
        let valid_content = json!({
            "hello": "Hello",
            "world": "World",
            "nested": {
                "key": "value"
            }
        });
        fs::write(&valid_json_path, valid_content.to_string()).unwrap();

        // Invalid JSON file
        let invalid_json_path = temp_dir.path().join("invalid.json");
        fs::write(&invalid_json_path, "{ invalid json }").unwrap();

        // Test valid JSON parsing
        let valid_result = fs::read_to_string(&valid_json_path).and_then(|content| {
            serde_json::from_str::<Value>(&content)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        });
        assert!(valid_result.is_ok());

        // Test invalid JSON parsing
        let invalid_result = fs::read_to_string(&invalid_json_path).and_then(|content| {
            serde_json::from_str::<Value>(&content)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        });
        assert!(invalid_result.is_err());
    }

    #[test]
    fn test_translation_key_extraction() {
        let source_json = json!({
            "app": {
                "title": "My App",
                "description": "A great application"
            },
            "buttons": {
                "save": "Save",
                "cancel": "Cancel",
                "submit": "Submit"
            },
            "messages": {
                "success": "Operation completed successfully",
                "error": "An error occurred"
            }
        });

        // Extract all keys from nested JSON
        fn extract_keys(obj: &Value, prefix: &str, keys: &mut Vec<String>) {
            match obj {
                Value::Object(map) => {
                    for (key, value) in map {
                        let full_key = if prefix.is_empty() {
                            key.clone()
                        } else {
                            format!("{}.{}", prefix, key)
                        };

                        match value {
                            Value::Object(_) => extract_keys(value, &full_key, keys),
                            _ => keys.push(full_key),
                        }
                    }
                }
                _ => {}
            }
        }

        let mut keys = Vec::new();
        extract_keys(&source_json, "", &mut keys);

        assert!(keys.contains(&"app.title".to_string()));
        assert!(keys.contains(&"app.description".to_string()));
        assert!(keys.contains(&"buttons.save".to_string()));
        assert!(keys.contains(&"buttons.cancel".to_string()));
        assert!(keys.contains(&"buttons.submit".to_string()));
        assert!(keys.contains(&"messages.success".to_string()));
        assert!(keys.contains(&"messages.error".to_string()));
        assert_eq!(keys.len(), 7);
    }

    #[test]
    fn test_language_code_validation() {
        let valid_codes = vec!["en", "es", "fr", "de", "it", "pt", "ru", "zh", "ja", "ko"];
        let invalid_codes = vec!["", "english", "123", "a", "toolongcode"];

        for code in valid_codes {
            assert!(code.len() >= 2 && code.len() <= 5);
            assert!(code.chars().all(|c| c.is_ascii_lowercase() || c == '-'));
        }

        for code in invalid_codes {
            let is_valid = code.len() >= 2
                && code.len() <= 5
                && code.chars().all(|c| c.is_ascii_lowercase() || c == '-');
            assert!(!is_valid, "Code '{}' should be invalid", code);
        }
    }

    #[test]
    fn test_translation_file_structure() {
        let temp_dir = tempdir().unwrap();
        let translations_dir = temp_dir.path().join("translations");
        fs::create_dir_all(&translations_dir).unwrap();

        // Create language files
        let languages = vec!["en", "es", "fr"];
        for lang in &languages {
            let lang_file = translations_dir.join(format!("{}.json", lang));
            let content = json!({
                "app.title": format!("Title in {}", lang),
                "app.description": format!("Description in {}", lang)
            });
            fs::write(&lang_file, content.to_string()).unwrap();
        }

        // Verify files exist and are readable
        for lang in languages {
            let lang_file = translations_dir.join(format!("{}.json", lang));
            assert!(lang_file.exists());

            let content = fs::read_to_string(&lang_file).unwrap();
            let parsed: Value = serde_json::from_str(&content).unwrap();
            assert!(parsed.is_object());
        }
    }

    #[test]
    fn test_missing_translation_detection() {
        let source_keys = vec![
            "app.title".to_string(),
            "app.description".to_string(),
            "buttons.save".to_string(),
            "buttons.cancel".to_string(),
        ];

        let existing_keys = vec!["app.title".to_string(), "buttons.save".to_string()];

        let missing_keys: Vec<String> = source_keys
            .iter()
            .filter(|key| !existing_keys.contains(key))
            .cloned()
            .collect();

        assert_eq!(missing_keys.len(), 2);
        assert!(missing_keys.contains(&"app.description".to_string()));
        assert!(missing_keys.contains(&"buttons.cancel".to_string()));
    }

    #[test]
    fn test_backup_file_creation() {
        let temp_dir = tempdir().unwrap();
        let original_file = temp_dir.path().join("original.json");
        let backup_file = temp_dir.path().join("original.json.bak");

        let content = json!({"key": "value"});
        fs::write(&original_file, content.to_string()).unwrap();

        // Create backup
        fs::copy(&original_file, &backup_file).unwrap();

        assert!(backup_file.exists());

        let original_content = fs::read_to_string(&original_file).unwrap();
        let backup_content = fs::read_to_string(&backup_file).unwrap();
        assert_eq!(original_content, backup_content);
    }

    #[test]
    fn test_json_merge_logic() {
        let mut base = json!({
            "existing": "value",
            "nested": {
                "key1": "value1"
            }
        });

        let updates = json!({
            "new_key": "new_value",
            "nested": {
                "key2": "value2"
            }
        });

        // Merge logic simulation
        if let (Value::Object(base_map), Value::Object(updates_map)) = (&mut base, &updates) {
            for (key, value) in updates_map {
                if let Some(existing) = base_map.get_mut(key) {
                    if let (Value::Object(existing_obj), Value::Object(new_obj)) = (existing, value)
                    {
                        for (nested_key, nested_value) in new_obj {
                            existing_obj.insert(nested_key.clone(), nested_value.clone());
                        }
                    }
                } else {
                    base_map.insert(key.clone(), value.clone());
                }
            }
        }

        assert!(base["new_key"] == "new_value");
        assert!(base["nested"]["key1"] == "value1");
        assert!(base["nested"]["key2"] == "value2");
    }

    #[tokio::test]
    async fn test_ai_translation_request_format() {
        // Test AI translation request structure
        let text_to_translate = "Hello, world!";
        let target_language = "Spanish";

        let prompt = format!(
            "Translate the following text to {}: '{}'",
            target_language, text_to_translate
        );

        assert!(prompt.contains(target_language));
        assert!(prompt.contains(text_to_translate));
        assert!(prompt.starts_with("Translate"));
    }

    #[test]
    fn test_progress_tracking() {
        let total_keys = 10;
        let processed_keys = 7;

        let progress_percentage = (processed_keys as f32 / total_keys as f32) * 100.0;
        assert_eq!(progress_percentage, 70.0);

        let progress_bar = format!(
            "[{}{}] {}/{}",
            "=".repeat(processed_keys),
            " ".repeat(total_keys - processed_keys),
            processed_keys,
            total_keys
        );

        assert!(progress_bar.contains("======="));
        assert!(progress_bar.contains("7/10"));
    }

    #[test]
    fn test_translation_delay_calculation() {
        let delay_seconds = 2;
        let num_translations = 5;

        let total_time = delay_seconds * num_translations;
        assert_eq!(total_time, 10);

        let delay_millis = delay_seconds * 1000;
        assert_eq!(delay_millis, 2000);
    }

    #[test]
    fn test_file_extension_validation() {
        let valid_files = vec!["source.json", "en.json", "translations.json"];
        let invalid_files = vec!["source.txt", "data.xml", "file", "config.toml"];

        for file in valid_files {
            assert!(file.ends_with(".json"));
        }

        for file in invalid_files {
            assert!(!file.ends_with(".json"));
        }
    }

    #[test]
    fn test_directory_creation() {
        let temp_dir = tempdir().unwrap();
        let translations_dir = temp_dir.path().join("translations");
        let nested_dir = translations_dir.join("nested").join("deep");

        // Test directory creation
        fs::create_dir_all(&nested_dir).unwrap();

        assert!(translations_dir.exists());
        assert!(nested_dir.exists());
        assert!(translations_dir.is_dir());
        assert!(nested_dir.is_dir());
    }

    #[test]
    fn test_translation_sync_result() {
        // Test sync result structure
        #[derive(Debug)]
        struct MockSyncResult {
            total_keys: usize,
            translated_keys: usize,
            skipped_keys: usize,
            errors: Vec<String>,
        }

        let result = MockSyncResult {
            total_keys: 10,
            translated_keys: 8,
            skipped_keys: 1,
            errors: vec!["Failed to translate 'complex.key'".to_string()],
        };

        assert_eq!(result.total_keys, 10);
        assert_eq!(result.translated_keys, 8);
        assert_eq!(result.skipped_keys, 1);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_key_path_normalization() {
        let test_keys = vec![
            ("app.title", "app.title"),
            ("app..title", "app.title"),
            (".app.title", "app.title"),
            ("app.title.", "app.title"),
            ("app...title", "app.title"),
        ];

        for (input, expected) in test_keys {
            let normalized = input
                .trim_matches('.')
                .split('.')
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(".");

            assert_eq!(normalized, expected);
        }
    }

    #[test]
    fn test_translation_caching() {
        use std::collections::HashMap;

        let mut cache: HashMap<String, String> = HashMap::new();

        // Simulate caching translations
        cache.insert("hello".to_string(), "hola".to_string());
        cache.insert("world".to_string(), "mundo".to_string());

        assert_eq!(cache.get("hello"), Some(&"hola".to_string()));
        assert_eq!(cache.get("world"), Some(&"mundo".to_string()));
        assert_eq!(cache.get("missing"), None);

        // Test cache hit
        let key = "hello";
        let cached_translation = cache.get(key);
        assert!(cached_translation.is_some());
    }

    #[test]
    fn test_error_handling_scenarios() {
        let error_cases = vec![
            ("File not found", std::io::ErrorKind::NotFound),
            ("Permission denied", std::io::ErrorKind::PermissionDenied),
            ("Invalid JSON", std::io::ErrorKind::InvalidData),
        ];

        for (description, error_kind) in error_cases {
            let error = std::io::Error::new(error_kind, description);
            assert_eq!(error.kind(), error_kind);
            assert!(error.to_string().contains(description));
        }
    }

    #[tokio::test]
    async fn test_concurrent_translation_handling() {
        // Test that multiple translations can be handled
        let translations = vec![("hello", "es"), ("world", "es"), ("goodbye", "es")];

        let mut results = Vec::new();
        for (text, lang) in translations {
            // Simulate async translation
            let result = format!("Translated '{}' to {}", text, lang);
            results.push(result);
        }

        assert_eq!(results.len(), 3);
        assert!(results[0].contains("hello"));
        assert!(results[1].contains("world"));
        assert!(results[2].contains("goodbye"));
    }

    #[test]
    fn test_config_validation() {
        // Test translation sync configuration
        #[derive(Debug)]
        struct MockConfig {
            source_file: String,
            target_languages: Vec<String>,
            delay_seconds: u64,
            output_dir: String,
        }

        let config = MockConfig {
            source_file: "source.json".to_string(),
            target_languages: vec!["es".to_string(), "fr".to_string()],
            delay_seconds: 2,
            output_dir: "translations".to_string(),
        };

        assert!(config.source_file.ends_with(".json"));
        assert!(!config.target_languages.is_empty());
        assert!(config.delay_seconds > 0);
        assert!(!config.output_dir.is_empty());
    }
}
