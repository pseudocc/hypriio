use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_transforms")]
    pub transforms: [u8; 4],
    #[serde(default)]
    pub lock: bool,
    #[serde(default = "default_output")]
    pub output: String,
}

fn default_transforms() -> [u8; 4] {
    [0, 1, 2, 3]
}

fn default_output() -> String {
    String::from("eDP-1")
}

impl Default for Config {
    fn default() -> Self {
        Self {
            transforms: default_transforms(),
            lock: false,
            output: default_output(),
        }
    }
}

impl Config {
    fn config_path() -> PathBuf {
        if let Ok(path) = std::env::var("HYPRIIO_CONFIG") {
            return PathBuf::from(path);
        }
        let home = std::env::var("HOME").unwrap_or_else(|_| String::from("/tmp"));
        PathBuf::from(home).join(".config/hypriio/config.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(config) => return config,
                    Err(err) => eprintln!("Failed to parse config file: {}", err),
                },
                Err(err) => eprintln!("Failed to read config file: {}", err),
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn set_lock(&mut self, lock: bool) -> Result<(), Box<dyn std::error::Error>> {
        self.lock = lock;
        self.save()
    }
}
