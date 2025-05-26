use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub project_name: String,
    pub git_remote: String,
    pub release_format: String,
}

impl Config {
    pub fn load_config() -> Self {
        Self::default()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            project_name: "nitrokit".to_string(),
            git_remote: "origin".to_string(),
            release_format: "markdown".to_string(),
        }
    }
}
