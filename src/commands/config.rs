use anyhow::{anyhow, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePool, Pool, Row, Sqlite};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub gemini_api_key: Option<String>,
    pub gemini_model: String,
    pub translation_delay_seconds: u64,
    pub messages_dir: String,
    pub source_file: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            gemini_api_key: None,
            gemini_model: "gemini-1.5-flash".to_string(),
            translation_delay_seconds: 2,
            messages_dir: "messages".to_string(),
            source_file: "source.json".to_string(),
        }
    }
}

pub struct ConfigManager {
    pub pool: Pool<Sqlite>,
    pub config_dir: PathBuf,
}

impl ConfigManager {
    pub async fn new() -> Result<Self> {
        let config_dir = Self::get_config_dir()?;

        // Ensure config directory exists
        if let Err(e) = std::fs::create_dir_all(&config_dir) {
            return Err(anyhow!(
                "Failed to create config directory {}: {}",
                config_dir.display(),
                e
            ));
        }

        let db_path = config_dir.join("nitrokit.db");

        // Check if we can write to the directory
        if !config_dir.exists() || std::fs::metadata(&config_dir)?.permissions().readonly() {
            return Err(anyhow!(
                "Config directory is not writable: {}",
                config_dir.display()
            ));
        }

        let database_url = format!("sqlite:{}?mode=rwc", db_path.display());

        println!(
            "{}",
            format!("📁 Config directory: {}", config_dir.display()).dimmed()
        );
        println!(
            "{}",
            format!("🗄️  Database path: {}", db_path.display()).dimmed()
        );

        let pool = match SqlitePool::connect(&database_url).await {
            Ok(pool) => pool,
            Err(e) => {
                // Fallback: try to create database in current directory
                println!(
                    "{}",
                    "⚠️  Cannot access home config directory, using local config".yellow()
                );
                let local_db_path = "./nitrokit.db";
                let local_url = format!("sqlite:{}?mode=rwc", local_db_path);

                match SqlitePool::connect(&local_url).await {
                    Ok(pool) => {
                        println!(
                            "{}",
                            format!("📁 Using local database: {}", local_db_path).blue()
                        );
                        pool
                    }
                    Err(local_e) => {
                        return Err(anyhow!(
                            "Failed to connect to database. Home: {} Local: {}",
                            e,
                            local_e
                        ));
                    }
                }
            }
        };

        // Initialize database schema
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS config (
                id INTEGER PRIMARY KEY,
                key TEXT UNIQUE NOT NULL,
                value TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await
        .map_err(|e| anyhow!("Failed to create config table: {}", e))?;

        Ok(Self { pool, config_dir })
    }

    pub fn get_config_dir() -> Result<PathBuf> {
        // Try multiple fallback locations
        if let Some(home_dir) = dirs::home_dir() {
            let config_dir = home_dir.join(".config").join("nitrokit");
            if Self::test_directory_writable(&config_dir) {
                return Ok(config_dir);
            }
        }

        // Fallback 1: XDG_CONFIG_HOME
        if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
            let config_dir = PathBuf::from(xdg_config).join("nitrokit");
            if Self::test_directory_writable(&config_dir) {
                return Ok(config_dir);
            }
        }

        // Fallback 2: Current directory
        let current_dir = std::env::current_dir()?.join(".nitrokit");
        if Self::test_directory_writable(&current_dir) {
            return Ok(current_dir);
        }

        // Fallback 3: Temp directory
        let temp_dir = std::env::temp_dir().join("nitrokit");
        Ok(temp_dir)
    }

    pub fn test_directory_writable(dir: &PathBuf) -> bool {
        // Try to create directory and test write access
        if std::fs::create_dir_all(dir).is_err() {
            return false;
        }

        // Test write by creating a temporary file
        let test_file = dir.join(".test_write");
        if std::fs::write(&test_file, "test").is_ok() {
            let _ = std::fs::remove_file(&test_file);
            return true;
        }

        false
    }

    pub async fn get_config(&self) -> Result<AppConfig> {
        let mut config = AppConfig::default();

        // Load from database using runtime queries
        let rows = match sqlx::query("SELECT key, value FROM config")
            .fetch_all(&self.pool)
            .await
        {
            Ok(rows) => rows,
            Err(e) => {
                println!("{}", format!("⚠️  Database read error: {}", e).yellow());
                return Ok(config); // Return default config if database read fails
            }
        };

        for row in rows {
            let key: String = row.get("key");
            let value: String = row.get("value");

            match key.as_str() {
                "gemini_api_key" => {
                    if !value.is_empty() {
                        config.gemini_api_key = Some(value);
                    }
                }
                "gemini_model" => config.gemini_model = value,
                "translation_delay_seconds" => {
                    config.translation_delay_seconds = value.parse().unwrap_or(2);
                }
                "messages_dir" => config.messages_dir = value,
                "source_file" => config.source_file = value,
                _ => {}
            }
        }

        Ok(config)
    }

    pub async fn save_config(&self, config: &AppConfig) -> Result<()> {
        let delay_string = config.translation_delay_seconds.to_string();
        let config_items = vec![
            (
                "gemini_api_key",
                config.gemini_api_key.as_deref().unwrap_or(""),
            ),
            ("gemini_model", &config.gemini_model),
            ("translation_delay_seconds", &delay_string),
            ("messages_dir", &config.messages_dir),
            ("source_file", &config.source_file),
        ];

        for (key, value) in config_items {
            sqlx::query(
                r#"
                INSERT INTO config (key, value, updated_at) 
                VALUES (?, ?, CURRENT_TIMESTAMP)
                ON CONFLICT(key) DO UPDATE SET 
                    value = excluded.value,
                    updated_at = CURRENT_TIMESTAMP
                "#,
            )
            .bind(key)
            .bind(value)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to save config key '{}': {}", key, e))?;
        }

        Ok(())
    }

    pub async fn is_first_run(&self) -> Result<bool> {
        let row = match sqlx::query("SELECT COUNT(*) as count FROM config")
            .fetch_one(&self.pool)
            .await
        {
            Ok(row) => row,
            Err(_) => return Ok(true), // If query fails, assume first run
        };

        let count: i64 = row.get("count");
        Ok(count == 0)
    }

    pub async fn interactive_setup(&self) -> Result<AppConfig> {
        println!("{}", "🎯 Nitrokit Configuration Setup".cyan().bold());
        println!("{}", "═".repeat(40).dimmed());
        println!();

        let mut config = self.get_config().await.unwrap_or_default();

        // Gemini API Key
        println!("{}", "🔑 Gemini API Configuration".yellow().bold());
        config.gemini_api_key = self
            .prompt_for_api_key(config.gemini_api_key.as_deref())
            .await?;

        // Gemini Model
        config.gemini_model = self.prompt_for_model(&config.gemini_model).await?;

        // Delay
        config.translation_delay_seconds = self
            .prompt_for_delay(config.translation_delay_seconds)
            .await?;

        // Messages Directory
        config.messages_dir = self.prompt_for_messages_dir(&config.messages_dir).await?;

        // Source File
        config.source_file = self.prompt_for_source_file(&config.source_file).await?;

        // Save configuration
        match self.save_config(&config).await {
            Ok(_) => {
                println!(
                    "\n{}",
                    "✅ Configuration saved successfully!".green().bold()
                );
                println!(
                    "{}",
                    format!("📁 Config stored in: {}", self.config_dir.display()).dimmed()
                );
            }
            Err(e) => {
                println!(
                    "{}",
                    format!("⚠️  Warning: Could not save config to database: {}", e).yellow()
                );
                println!(
                    "{}",
                    "Configuration will work for this session only.".dimmed()
                );
            }
        }

        Ok(config)
    }

    async fn prompt_for_api_key(&self, current: Option<&str>) -> Result<Option<String>> {
        if let Ok(env_key) = std::env::var("GEMINI_API_KEY") {
            if !env_key.is_empty() {
                println!("{}", "✅ Using GEMINI_API_KEY from environment".green());
                return Ok(Some(env_key));
            }
        }

        let prompt = if let Some(current_key) = current {
            let masked = format!("{}***", &current_key[..std::cmp::min(8, current_key.len())]);
            format!("Gemini API Key [current: {}]: ", masked)
        } else {
            "Gemini API Key (required): ".to_string()
        };

        print!("{}", prompt.cyan());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            if let Some(current_key) = current {
                println!("{}", "✅ Keeping current API key".green());
                return Ok(Some(current_key.to_string()));
            } else {
                println!("{}", "❌ API key is required for translation sync!".red());
                return Ok(None);
            }
        }

        Ok(Some(input.to_string()))
    }

    async fn prompt_for_model(&self, current: &str) -> Result<String> {
        println!("\n{}", "🤖 Available Gemini Models:".yellow());
        println!(
            "    {{}} gemini-1.5-flash (fast, cost-effective) {}",
            "1.".dimmed(),
        );
        println!(
            "    {{}} gemini-1.5-pro (more accurate, slower) {}",
            "2.".dimmed(),
        );

        print!("{}", format!("Model [current: {}]: ", current).cyan());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            return Ok(current.to_string());
        }

        let model = match input {
            "1" => "gemini-1.5-flash",
            "2" => "gemini-1.5-pro",
            custom if custom.starts_with("gemini") => custom,
            _ => {
                println!("{}", "⚠️  Invalid model, using current".yellow());
                current
            }
        };

        Ok(model.to_string())
    }

    async fn prompt_for_delay(&self, current: u64) -> Result<u64> {
        print!(
            "{}",
            format!("Delay between API calls (seconds) [current: {}]: ", current).cyan()
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            return Ok(current);
        }

        match input.parse::<u64>() {
            Ok(delay) if delay <= 60 => Ok(delay),
            Ok(_) => {
                println!("{}", "⚠️  Delay too high, using current value".yellow());
                Ok(current)
            }
            Err(_) => {
                println!("{}", "⚠️  Invalid number, using current value".yellow());
                Ok(current)
            }
        }
    }

    async fn prompt_for_messages_dir(&self, current: &str) -> Result<String> {
        print!(
            "{}",
            format!("Messages directory [current: {}]: ", current).cyan()
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            return Ok(current.to_string());
        }

        let path = std::path::Path::new(input);
        if path.is_absolute() {
            println!("{}", "⚠️  Using relative paths is recommended".yellow());
        }

        Ok(input.to_string())
    }

    async fn prompt_for_source_file(&self, current: &str) -> Result<String> {
        print!(
            "{}",
            format!("Source file name [current: {}]: ", current).cyan()
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            return Ok(current.to_string());
        }

        if !input.ends_with(".json") {
            println!("{}", "⚠️  File should have .json extension".yellow());
        }

        Ok(input.to_string())
    }

    pub async fn show_config(&self) -> Result<()> {
        let config = self.get_config().await?;

        println!("{}", "⚙️  Current Configuration".cyan().bold());
        println!("{}", "═".repeat(40).dimmed());

        let api_key_display = match &config.gemini_api_key {
            Some(key) => format!("{}***", &key[..std::cmp::min(8, key.len())]),
            None => "Not set".red().to_string(),
        };

        println!("{}: {}", "Gemini API Key".yellow(), api_key_display);
        println!(
            "{}: {}",
            "Gemini Model".yellow(),
            config.gemini_model.green()
        );
        println!(
            "{}: {}",
            "Delay (seconds)".yellow(),
            config.translation_delay_seconds.to_string().green()
        );
        println!(
            "{}: {}",
            "Messages Directory".yellow(),
            config.messages_dir.green()
        );
        println!("{}: {}", "Source File".yellow(), config.source_file.green());
        println!();
        println!(
            "{}",
            format!("📁 Config location: {}", self.config_dir.display()).dimmed()
        );

        Ok(())
    }

    pub async fn reset_config(&self) -> Result<()> {
        match sqlx::query("DELETE FROM config").execute(&self.pool).await {
            Ok(_) => {
                println!("{}", "🗑️  Configuration reset successfully!".green());
            }
            Err(e) => {
                println!(
                    "{}",
                    format!("⚠️  Warning: Could not reset database: {}", e).yellow()
                );
                println!(
                    "{}",
                    "You may need to manually delete the config file.".dimmed()
                );
            }
        }

        Ok(())
    }
}
