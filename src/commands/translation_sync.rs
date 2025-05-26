use crate::commands::config::{AppConfig, ConfigManager};
use anyhow::{anyhow, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct TranslationConfig {
    pub api_key: String,
    pub model: String,
    pub delay_seconds: u64,
    pub messages_dir: PathBuf,
    pub source_file: String,
}

impl From<AppConfig> for TranslationConfig {
    fn from(app_config: AppConfig) -> Self {
        Self {
            api_key: app_config.gemini_api_key.unwrap_or_default(),
            model: app_config.gemini_model,
            delay_seconds: app_config.translation_delay_seconds,
            messages_dir: PathBuf::from(app_config.messages_dir),
            source_file: app_config.source_file,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Language {
    pub code: String,
    pub name: String,
    pub flag: String,
}

impl Language {
    pub fn new(code: &str, name: &str, flag: &str) -> Self {
        Self {
            code: code.to_string(),
            name: name.to_string(),
            flag: flag.to_string(),
        }
    }

    pub fn from_code(code: &str) -> Self {
        // Basic language mapping - you can extend this
        let (name, flag) = match code {
            "tr" => ("Turkish", "🇹🇷"),
            "en" => ("English", "🇺🇸"),
            "es" => ("Spanish", "🇪🇸"),
            "fr" => ("French", "🇫🇷"),
            "de" => ("German", "🇩🇪"),
            "it" => ("Italian", "🇮🇹"),
            "pt" => ("Portuguese", "🇵🇹"),
            "ru" => ("Russian", "🇷🇺"),
            "ja" => ("Japanese", "🇯🇵"),
            "ko" => ("Korean", "🇰🇷"),
            "zh" => ("Chinese", "🇨🇳"),
            "ar" => ("Arabic", "🇸🇦"),
            "hi" => ("Hindi", "🇮🇳"),
            "nl" => ("Dutch", "🇳🇱"),
            "sv" => ("Swedish", "🇸🇪"),
            "no" => ("Norwegian", "🇳🇴"),
            "da" => ("Danish", "🇩🇰"),
            "fi" => ("Finnish", "🇫🇮"),
            "pl" => ("Polish", "🇵🇱"),
            "cs" => ("Czech", "🇨🇿"),
            "hu" => ("Hungarian", "🇭🇺"),
            "ro" => ("Romanian", "🇷🇴"),
            "bg" => ("Bulgarian", "🇧🇬"),
            "hr" => ("Croatian", "🇭🇷"),
            "sk" => ("Slovak", "🇸🇰"),
            "sl" => ("Slovenian", "🇸🇮"),
            "et" => ("Estonian", "🇪🇪"),
            "lv" => ("Latvian", "🇱🇻"),
            "lt" => ("Lithuanian", "🇱🇹"),
            "uk" => ("Ukrainian", "🇺🇦"),
            "he" => ("Hebrew", "🇮🇱"),
            "th" => ("Thai", "🇹🇭"),
            "vi" => ("Vietnamese", "🇻🇳"),
            "id" => ("Indonesian", "🇮🇩"),
            "ms" => ("Malay", "🇲🇾"),
            "az" => ("Azerbaijani", "🇦🇿"),
            "bs" => ("Bosnian", "🇧🇦"),
            "ur" => ("Urdu", "🇵🇰"),
            "uz" => ("Uzbek", "🇺🇿"),
            _ => (code, "🌍"), // Fallback for unknown languages
        };

        Self::new(code, name, flag)
    }
}

// Statik dil listesini kaldırdık, artık dynamic olacak
pub fn discover_language_files(messages_dir: &Path, source_file: &str) -> Result<Vec<Language>> {
    if !messages_dir.exists() {
        return Err(anyhow!(
            "Messages directory does not exist: {}",
            messages_dir.display()
        ));
    }

    let mut languages = Vec::new();

    // messages/ klasöründeki tüm .json dosyalarını oku
    for entry in fs::read_dir(messages_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Sadece .json dosyalarını kontrol et
        if let Some(extension) = path.extension() {
            if extension == "json" {
                if let Some(file_name) = path.file_name() {
                    if let Some(file_name_str) = file_name.to_str() {
                        // source.json'u atla
                        if file_name_str != source_file {
                            // Dosya isminden dil kodunu çıkar (örn: "tr.json" -> "tr")
                            if let Some(lang_code) = file_name_str.strip_suffix(".json") {
                                let language = Language::from_code(lang_code);
                                languages.push(language);
                                println!(
                                    "{}",
                                    format!(
                                        "📁 Found language file: {} ({})",
                                        file_name_str, lang_code
                                    )
                                    .dimmed()
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    // Alfabetik sırala
    languages.sort_by(|a, b| a.code.cmp(&b.code));

    println!(
        "{}",
        format!("🌍 Discovered {} language files", languages.len()).blue()
    );

    Ok(languages)
}

// Eksik dilleri tespit et ve kullanıcıya sor
pub async fn get_target_languages(messages_dir: &Path, source_file: &str) -> Result<Vec<Language>> {
    // Mevcut dil dosyalarını keşfet
    let existing_languages = discover_language_files(messages_dir, source_file)?;

    if existing_languages.is_empty() {
        println!("{}", "ℹ️  No existing language files found.".yellow());
        println!("{}", "Creating translations for common languages...".blue());

        // Yaygın diller için varsayılan liste
        let default_languages = vec![
            Language::from_code("tr"),
            Language::from_code("es"),
            Language::from_code("fr"),
            Language::from_code("de"),
            Language::from_code("it"),
        ];

        return Ok(default_languages);
    }

    // Kullanıcıya yeni dil eklemek isteyip istemediğini sor
    println!("\n{}", "Would you like to add new languages? (y/n):".cyan());
    print!("{}", "nitrokit> ".cyan().bold());

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes" {
        println!(
            "{}",
            "Enter language codes separated by commas (e.g., 'ja,ko,zh'):".cyan()
        );
        print!("{}", "nitrokit> ".cyan().bold());

        let mut lang_input = String::new();
        std::io::stdin().read_line(&mut lang_input)?;

        let new_codes: Vec<&str> = lang_input.trim().split(',').map(|s| s.trim()).collect();
        let mut all_languages = existing_languages;

        for code in new_codes {
            if !code.is_empty() && !all_languages.iter().any(|l| l.code == code) {
                all_languages.push(Language::from_code(code));
                println!(
                    "{}",
                    format!(
                        "➕ Added language: {} ({})",
                        Language::from_code(code).name,
                        code
                    )
                    .green()
                );
            }
        }

        return Ok(all_languages);
    }

    Ok(existing_languages)
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "generationConfig")]
    generation_config: GeminiGenerationConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiGenerationConfig {
    temperature: f32,
    #[serde(rename = "topK")]
    top_k: i32,
    #[serde(rename = "topP")]
    top_p: f32,
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: i32,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

pub struct TranslationSync {
    config: TranslationConfig,
    client: reqwest::Client,
}

impl TranslationSync {
    pub fn new(config: TranslationConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    pub async fn sync_translations(&self) -> Result<()> {
        println!("{}", "🔄 Starting translation sync...".cyan().bold());

        // messages/ klasörünün var olup olmadığını kontrol et
        if !self.config.messages_dir.exists() {
            println!(
                "{}",
                format!(
                    "📁 Creating messages directory: {}",
                    self.config.messages_dir.display()
                )
                .blue()
            );
            fs::create_dir_all(&self.config.messages_dir)?;
        }

        // Load source JSON
        let source_path = self.config.messages_dir.join(&self.config.source_file);
        if !source_path.exists() {
            return Err(anyhow!("Source file not found: {}", source_path.display()));
        }

        let source_content = fs::read_to_string(&source_path)?;
        let source_json: Value = serde_json::from_str(&source_content)?;

        println!(
            "{}",
            format!("📖 Loaded source file: {}", source_path.display()).green()
        );

        // Get all translation paths
        let all_paths = self.extract_all_paths(&source_json, "");
        println!(
            "{}",
            format!("🔍 Found {} translation keys", all_paths.len()).blue()
        );

        // Dinamik olarak dil dosyalarını keşfet
        let languages =
            get_target_languages(&self.config.messages_dir, &self.config.source_file).await?;

        if languages.is_empty() {
            println!("{}", "⚠️  No target languages found.".yellow());
            return Ok(());
        }

        // Process each language
        for language in &languages {
            println!(
                "\n{}",
                format!(
                    "🌍 Processing {} {} ({})",
                    language.flag, language.name, language.code
                )
                .yellow()
                .bold()
            );

            match self
                .process_language(&source_json, &all_paths, language)
                .await
            {
                Ok(updated_count) => {
                    if updated_count > 0 {
                        println!(
                            "{}",
                            format!("✅ Updated {} translations", updated_count).green()
                        );
                    } else {
                        println!("{}", "✅ All translations up to date".green());
                    }
                }
                Err(e) => {
                    println!(
                        "{}",
                        format!("❌ Failed to process {}: {}", language.name, e).red()
                    );
                }
            }

            // Rate limiting
            if self.config.delay_seconds > 0 {
                tokio::time::sleep(Duration::from_secs(self.config.delay_seconds)).await;
            }
        }

        println!("\n{}", "🎉 Translation sync completed!".green().bold());
        Ok(())
    }

    // Geri kalan metodlar aynı kalacak...
    async fn process_language(
        &self,
        source_json: &Value,
        all_paths: &[String],
        language: &Language,
    ) -> Result<usize> {
        let lang_file = self
            .config
            .messages_dir
            .join(format!("{}.json", language.code));

        // Load existing translations or create empty
        let mut existing_json = if lang_file.exists() {
            let content = fs::read_to_string(&lang_file)?;
            serde_json::from_str(&content)?
        } else {
            serde_json::json!({})
        };

        // Find missing translations
        let missing_paths = self.find_missing_paths(&existing_json, all_paths);

        if missing_paths.is_empty() {
            return Ok(0);
        }

        println!(
            "{}",
            format!("📝 Found {} missing translations", missing_paths.len()).yellow()
        );

        // Translate missing keys in batches
        let batch_size = 10; // Avoid overwhelming the API
        let mut updated_count = 0;

        for chunk in missing_paths.chunks(batch_size) {
            let translations = self.translate_batch(chunk, source_json, language).await?;

            for (path, translation) in translations {
                self.set_nested_value(&mut existing_json, &path, Value::String(translation))?;
                updated_count += 1;
            }
        }

        // Save updated translations
        if updated_count > 0 {
            let formatted_json = serde_json::to_string_pretty(&existing_json)?;
            fs::write(&lang_file, formatted_json)?;
        }

        Ok(updated_count)
    }

    async fn translate_batch(
        &self,
        paths: &[String],
        source_json: &Value,
        language: &Language,
    ) -> Result<Vec<(String, String)>> {
        let mut batch_text = String::new();
        let mut path_mapping = Vec::new();

        for path in paths {
            if let Some(source_text) = self.get_nested_value(source_json, path) {
                if let Some(text) = source_text.as_str() {
                    batch_text.push_str(&format!("{}||{}\n", path, text));
                    path_mapping.push(path.clone());
                }
            }
        }

        if batch_text.is_empty() {
            return Ok(Vec::new());
        }

        let prompt = format!(
            "Translate the following key-value pairs to {}. Keep the exact format with || separator and preserve any HTML tags, placeholders like {{appName}}, {{min}}, {{max}}, etc. Only translate the text content, not the keys or placeholders:\n\n{}",
            language.name,
            batch_text
        );

        let translated_text = self.call_gemini_api(&prompt).await?;
        self.parse_translation_response(&translated_text, &path_mapping)
    }

    async fn call_gemini_api(&self, prompt: &str) -> Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.config.model, self.config.api_key
        );

        let request = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart {
                    text: prompt.to_string(),
                }],
            }],
            generation_config: GeminiGenerationConfig {
                temperature: 0.3,
                top_k: 40,
                top_p: 0.95,
                max_output_tokens: 2048,
            },
        };

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Gemini API error: {}", error_text));
        }

        let gemini_response: GeminiResponse = response.json().await?;

        if let Some(candidate) = gemini_response.candidates.first() {
            if let Some(part) = candidate.content.parts.first() {
                return Ok(part.text.clone());
            }
        }

        Err(anyhow!("No response from Gemini API"))
    }

    fn parse_translation_response(
        &self,
        response: &str,
        paths: &[String],
    ) -> Result<Vec<(String, String)>> {
        let mut results = Vec::new();

        for line in response.lines() {
            if let Some((path, translation)) = line.split_once("||") {
                let path = path.trim();
                let translation = translation.trim();

                if paths.contains(&path.to_string()) {
                    results.push((path.to_string(), translation.to_string()));
                }
            }
        }

        Ok(results)
    }

    fn extract_all_paths(&self, value: &Value, prefix: &str) -> Vec<String> {
        let mut paths = Vec::new();

        match value {
            Value::Object(map) => {
                for (key, val) in map {
                    let current_path = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };

                    if val.is_string() {
                        paths.push(current_path);
                    } else {
                        paths.extend(self.extract_all_paths(val, &current_path));
                    }
                }
            }
            _ => {}
        }

        paths
    }

    fn find_missing_paths(&self, target: &Value, all_paths: &[String]) -> Vec<String> {
        let mut missing = Vec::new();

        for path in all_paths {
            if self.get_nested_value(target, path).is_none() {
                missing.push(path.clone());
            }
        }

        missing
    }

    fn get_nested_value<'a>(&self, value: &'a Value, path: &str) -> Option<&'a Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = value;

        for part in parts {
            match current {
                Value::Object(map) => {
                    current = map.get(part)?;
                }
                _ => return None,
            }
        }

        Some(current)
    }

    fn set_nested_value(&self, value: &mut Value, path: &str, new_value: Value) -> Result<()> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = value;

        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Last part, set the value
                if let Value::Object(map) = current {
                    map.insert(part.to_string(), new_value);
                    return Ok(());
                }
            } else {
                // Intermediate part, navigate or create
                if !current.is_object() {
                    *current = serde_json::json!({});
                }

                if let Value::Object(map) = current {
                    if !map.contains_key(*part) {
                        map.insert(part.to_string(), serde_json::json!({}));
                    }
                    current = map.get_mut(*part).unwrap();
                }
            }
        }

        Err(anyhow!("Failed to set nested value"))
    }
}

pub async fn sync_translations_interactive() -> Result<()> {
    let config_manager = ConfigManager::new().await?;
    // Check if this is the first run
    if config_manager.is_first_run().await? {
        println!(
            "{}",
            "👋 Welcome to Nitrokit Translation Sync!".cyan().bold()
        );
        println!("{}", "Let's set up your configuration...".blue());
        println!();
        let app_config = config_manager.interactive_setup().await?;
        if app_config.gemini_api_key.is_none() {
            println!("{}", "❌ Cannot proceed without API key!".red());
            return Ok(());
        }
        let translation_config = TranslationConfig::from(app_config);
        println!(
            "\n{}",
            "🚀 Starting first translation sync...".green().bold()
        );
        sync_translations_with_config(translation_config).await
    } else {
        let app_config = config_manager.get_config().await?;
        if app_config.gemini_api_key.is_none() {
            println!("{}", "❌ Gemini API key not configured!".red());
            println!("{}", "Run 'nitrokit config' to set up your API key.".blue());
            return Ok(());
        }
        let translation_config = TranslationConfig::from(app_config);
        sync_translations_with_config(translation_config).await
    }
}

pub async fn sync_translations_with_config(config: TranslationConfig) -> Result<()> {
    let sync = TranslationSync::new(config);
    sync.sync_translations().await
}

// Config management commands
pub async fn show_config() -> Result<()> {
    let config_manager = ConfigManager::new().await?;
    config_manager.show_config().await
}

pub async fn setup_config() -> Result<()> {
    let config_manager = ConfigManager::new().await?;
    config_manager.interactive_setup().await?;
    Ok(())
}

pub async fn reset_config() -> Result<()> {
    let config_manager = ConfigManager::new().await?;
    config_manager.reset_config().await
}
