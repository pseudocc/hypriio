use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_output")]
    pub output: String,
    #[serde(default = "default_transforms")]
    pub transforms: [u8; 4],
    #[serde(default)]
    pub restart_services: Vec<String>,
}

fn default_output() -> String {
    String::from("eDP-1")
}

fn default_transforms() -> [u8; 4] {
    [0, 1, 2, 3]
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output: default_output(),
            transforms: default_transforms(),
            restart_services: Vec::new(),
        }
    }
}

fn config_path() -> PathBuf {
    if let Ok(path) = std::env::var("HYPRIIO_CONFIG") {
        return PathBuf::from(path);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| String::from("/tmp"));
    PathBuf::from(home).join(".config/hypriio/config.toml")
}

impl Config {
    pub fn load() -> Self {
        let path = config_path();
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

pub mod socket {
    use std::path::PathBuf;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    pub struct Server(tokio::net::UnixListener);
    pub struct Client(std::os::unix::net::UnixStream);

    pub fn socket_path() -> PathBuf {
        if let Ok(path) = std::env::var("HYPRIIO_SOCKET") {
            return PathBuf::from(path);
        }
        let home = std::env::var("HOME").unwrap_or_else(|_| String::from("/tmp"));
        PathBuf::from(home).join(".cache/hypriio/socket")
    }

    pub enum Command {
        Lock,
        Unlock,
    }

    impl Into<u32> for Command {
        fn into(self) -> u32 {
            match self {
                Command::Lock => 0,
                Command::Unlock => 1,
            }
        }
    }

    impl TryFrom<u32> for Command {
        type Error = ();

        fn try_from(value: u32) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(Command::Lock),
                1 => Ok(Command::Unlock),
                _ => Err(()),
            }
        }
    }

    impl Client {
        pub fn connect() -> std::io::Result<Self> {
            use std::os::unix::net::UnixStream;
            let path = socket_path();
            let stream = UnixStream::connect(path)?;
            Ok(Self(stream))
        }

        pub fn send(&mut self, cmd: Command) -> std::io::Result<()> {
            use std::io::Write;
            let cmd_value: u32 = cmd.into();
            let data = cmd_value.to_le_bytes();
            self.0.write_all(&data)
        }
    }

    impl Server {
        pub fn bind() -> std::io::Result<Self> {
            use tokio::net::UnixListener;
            let path = socket_path();
            if path.exists() {
                std::fs::remove_file(&path)?;
            }
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let listener = UnixListener::bind(&path)?;
            Ok(Self(listener))
        }

        pub async fn accept(&self) -> std::io::Result<Connection> {
            let (stream, _) = self.0.accept().await?;
            Ok(Connection(stream))
        }
    }

    pub struct Connection(tokio::net::UnixStream);

    impl Connection {
        pub async fn receive(&mut self) -> Option<Command> {
            let mut buf = [0u8; 4];
            match self.0.read_exact(&mut buf).await {
                Ok(_) => {
                    let cmd_value = u32::from_le_bytes(buf);
                    Command::try_from(cmd_value).ok()
                },
                Err(_) => None,
            }
        }
    }
}
