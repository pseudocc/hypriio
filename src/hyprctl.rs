use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Clone, Deserialize)]
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
    pub fn touch(&self) -> &[Device] {
        &self.touch
    }

    pub fn tablets(&self) -> &[Device] {
        &self.tablets
    }
}

pub fn devices() -> Result<Devices, Box<dyn std::error::Error>> {
    let output = Command::new("hyprctl").args(["devices", "-j"]).output()?;

    if !output.status.success() {
        return Err("Failed to execute hyprctl".into());
    }

    let devices: Devices = serde_json::from_slice(&output.stdout)?;
    Ok(devices)
}

pub trait Rule {
    fn expression(&self) -> String;
}

pub struct Rules {
    expressions: Vec<String>,
}

impl Rules {
    pub fn new() -> Self {
        Self {
            expressions: Vec::new(),
        }
    }

    pub fn add<R: Rule>(&mut self, rule: R) {
        self.expressions.push(rule.expression());
    }

    pub fn exec(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.expressions.iter().try_for_each(|expression| {
            let output = Command::new("hyprctl")
                .args(["eval", expression])
                .output()?;

            output
                .status
                .success()
                .then_some(())
                .ok_or_else(|| "Failed to execute hyprctl".into())
        })
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
    fn expression(&self) -> String {
        format!(
            "hl.monitor {{ output = {}, transform = {} }}",
            lua_string(&self.0.name),
            self.1
        )
    }
}

pub struct TouchDeviceTransform<'a>(&'a Device, u8);
impl<'a> TouchDeviceTransform<'a> {
    pub fn new(device: &'a Device, transform: u8) -> Self {
        Self(device, transform)
    }
}
impl Rule for TouchDeviceTransform<'_> {
    fn expression(&self) -> String {
        format!(
            "hl.device {{ name = {}, transform = {} }}",
            lua_string(&self.0.name),
            self.1
        )
    }
}

pub struct TabletTransform<'a>(&'a Device, u8);
impl<'a> TabletTransform<'a> {
    pub fn new(device: &'a Device, transform: u8) -> Self {
        Self(device, transform)
    }
}
impl Rule for TabletTransform<'_> {
    fn expression(&self) -> String {
        format!(
            "hl.device {{ name = {}, transform = {} }}",
            lua_string(&self.0.name),
            self.1
        )
    }
}

fn lua_string(value: &str) -> String {
    format!("{:?}", value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_expressions() {
        let monitor = Monitor {
            id: 1,
            name: "eDP-1".into(),
            description: String::new(),
            disabled: false,
        };
        let device = Device {
            address: String::new(),
            name: "Wacom \"Pen\"".into(),
        };

        assert_eq!(
            MonitorTransform::new(&monitor, 1).expression(),
            "hl.monitor { output = \"eDP-1\", transform = 1 }"
        );
        assert_eq!(
            TouchDeviceTransform::new(&device, 3).expression(),
            "hl.device { name = \"Wacom \\\"Pen\\\"\", transform = 3 }"
        );
        assert_eq!(
            TabletTransform::new(&device, 2).expression(),
            "hl.device { name = \"Wacom \\\"Pen\\\"\", transform = 2 }"
        );
    }
}
