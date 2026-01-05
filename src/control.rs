use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Static {
    #[serde(default = "default_output")]
    pub output: String,
    #[serde(default)]
    pub restart_services: Vec<String>,
}

fn default_output() -> String {
    String::from("eDP-1")
}

impl Default for Static {
    fn default() -> Self {
        Self {
            output: default_output(),
            restart_services: Vec::new(),
        }
    }
}

fn default_transforms() -> [u8; 4] {
    [0, 1, 2, 3]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dynamic {
    #[serde(default)]
    pub lock: bool,
    #[serde(default = "default_transforms")]
    pub transforms: [u8; 4],
}

impl Default for Dynamic {
    fn default() -> Self {
        Self {
            lock: false,
            transforms: default_transforms(),
        }
    }
}

enum Role {
    Static,
    Dynamic,
}

fn config_path(role: Role) -> PathBuf {
    let key = match role {
        Role::Static => "HYPRIIO_STATIC",
        Role::Dynamic => "HYPRIIO_CONFIG",
    };
    if let Ok(path) = std::env::var(key) {
        return PathBuf::from(path);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| String::from("/tmp"));
    let file_name = match role {
        Role::Static => "static.toml",
        Role::Dynamic => "config.toml",
    };
    PathBuf::from(home).join(".config").join("hypriio").join(file_name)
}

impl Static {
    pub fn load() -> Self {
        let path = config_path(Role::Static);
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match toml::from_str::<Self>(&content) {
                    Ok(config) => return config,
                    Err(err) => eprintln!("Failed to parse config file: {}", err),
                },
                Err(err) => eprintln!("Failed to read config file: {}", err),
            }
        }
        Self::default()
    }
}

impl Dynamic {
    pub fn load() -> Self {
        let path = config_path(Role::Dynamic);
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match toml::from_str::<Self>(&content) {
                    Ok(config) => return config,
                    Err(err) => eprintln!("Failed to parse config file: {}", err),
                },
                Err(err) => eprintln!("Failed to read config file: {}", err),
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = config_path(Role::Dynamic);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(&self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn set_lock(&mut self, lock: bool) -> Result<(), Box<dyn std::error::Error>> {
        self.lock = lock;
        self.save()
    }
}
