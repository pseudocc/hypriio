use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hardware {
    #[serde(default = "default_output")]
    pub output: String,
    #[serde(default = "default_transforms")]
    pub transforms: [u8; 4],
}

fn default_transforms() -> [u8; 4] {
    [0, 1, 2, 3]
}

fn default_output() -> String {
    String::from("eDP-1")
}

impl Default for Hardware {
    fn default() -> Self {
        Self {
            output: default_output(),
            transforms: default_transforms(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dynamic {
    #[serde(default)]
    pub lock: bool,
    #[serde(default)]
    pub restart_services: Vec<String>,
}

impl Default for Dynamic {
    fn default() -> Self {
        Self {
            lock: false,
            restart_services: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ConfigFile {
    #[serde(flatten)]
    hardware: Hardware,
    #[serde(flatten)]
    dynamic: Dynamic,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            hardware: Hardware::default(),
            dynamic: Dynamic::default(),
        }
    }
}

enum Role {
    Hardware,
    Dynamic,
}

fn config_path(role: Role) -> PathBuf {
    let key = match role {
        Role::Hardware => "HYPRIIO_HARDWARE",
        Role::Dynamic => "HYPRIIO_CONFIG",
    };
    if let Ok(path) = std::env::var(key) {
        return PathBuf::from(path);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| String::from("/tmp"));
    let file_name = match role {
        Role::Hardware => "hardware.toml",
        Role::Dynamic => "config.toml",
    };
    PathBuf::from(home).join(".config").join("hypriioctl").join(file_name)
}

impl Hardware {
    pub fn load() -> Self {
        let path = config_path(Role::Hardware);
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match toml::from_str::<ConfigFile>(&content) {
                    Ok(config) => return config.hardware,
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
                Ok(content) => match toml::from_str::<ConfigFile>(&content) {
                    Ok(config) => return config.dynamic,
                    Err(err) => eprintln!("Failed to parse config file: {}", err),
                },
                Err(err) => eprintln!("Failed to read config file: {}", err),
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = config_path(Role::Dynamic);
        let hardware = Hardware::load();
        let config_file = ConfigFile {
            hardware,
            dynamic: self.clone(),
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(&config_file)?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn set_lock(&mut self, lock: bool) -> Result<(), Box<dyn std::error::Error>> {
        self.lock = lock;
        self.save()
    }
}
