use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Deserialize)]
pub struct Device {
    pub address: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Devices {
    touch: Vec<Device>,
    tablets: Vec<Device>,
}

impl Devices {
    pub fn has_touch(&self) -> bool {
        !self.touch.is_empty()
    }
    pub fn has_tablet(&self) -> bool {
        !self.tablets.is_empty()
    }
}

pub fn devices() -> Result<Devices, Box<dyn std::error::Error>> {
    let output = Command::new("hyprctl")
        .args(["devices", "-j"])
        .output()?;

    if !output.status.success() {
        return Err("Failed to execute hyprctl".into());
    }

    let devices: Devices = serde_json::from_slice(&output.stdout)?;
    Ok(devices)
}

pub trait Rule {
    const KEYWORD: &'static str;
    fn value(&self) -> String;
}

pub struct Rules {
    rules: Vec<String>,
}

impl Rules {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add<R: Rule>(&mut self, rule: R) {
        self.rules.push(format!("keyword {} {}", R::KEYWORD, rule.value()));
    }

    pub fn exec(&self) -> Result<(), Box<dyn std::error::Error>> {
        let rules = self.rules.join(";");
        let output = Command::new("hyprctl")
            .args(["--batch", &rules])
            .output()?;
        if !output.status.success() {
            return Err("Failed to execute hyprctl".into());
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct Monitor {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub disabled: bool,
}

pub fn monitor(name: &str) -> Result<Monitor, Box<dyn std::error::Error>> {
    let output = Command::new("hyprctl")
        .args(["monitors", "-j", "all"])
        .output()?;

    if !output.status.success() {
        return Err("Failed to execute hyprctl".into());
    }

    let monitors: Vec<Monitor> = serde_json::from_slice(&output.stdout)?;

    for monitor in monitors {
        if monitor.name == name {
            return Ok(monitor);
        }
    }

    Err(format!("Monitor '{}' not found", name).into())
}

pub struct MonitorTransform<'a>(&'a Monitor, u8);
impl<'a> MonitorTransform<'a> {
    pub fn new(monitor: &'a Monitor, rotation: u8) -> Self {
        Self(monitor, rotation)
    }
}
impl Rule for MonitorTransform<'_> {
    const KEYWORD: &'static str = "monitor";

    fn value(&self) -> String {
        format!("{},transform,{}", self.0.name, self.1)
    }
}

pub struct TouchDeviceTransform(u8);
impl TouchDeviceTransform {
    pub fn new(transform: u8) -> Self {
        Self(transform)
    }
}
impl Rule for TouchDeviceTransform {
    const KEYWORD: &'static str = "input:touchdevice:transform";
    fn value(&self) -> String {
        format!("{}", self.0)
    }
}

pub struct TabletTransform(u8);
impl TabletTransform {
    pub fn new(transform: u8) -> Self {
        Self(transform)
    }
}
impl Rule for TabletTransform {
    const KEYWORD: &'static str = "input:tablet:transform";
    fn value(&self) -> String {
        format!("{}", self.0)
    }
}
