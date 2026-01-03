use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Deserialize)]
pub struct Monitor {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub disabled: bool,
}

pub fn monitor(name: &str) -> Result<Monitor, Box<dyn std::error::Error>> {
    let output = Command::new("hyprctl")
        .args(["monitors", "-j"])
        .output()?;

    if !output.status.success() {
        return Err("Failed to execute hyprctl".into());
    }

    let monitors: Vec<Monitor> = serde_json::from_slice(&output.stdout)?;

    for monitor in monitors {
        if monitor.name == name {
            if monitor.disabled {
                return Err(format!("Monitor '{}' is disabled", name).into());
            }
            return Ok(monitor);
        }
    }

    Err(format!("Monitor '{}' not found", name).into())
}
