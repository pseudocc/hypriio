mod control;
mod dbus;
mod hyprctl;

use futures_lite::stream::StreamExt;
use tokio::time::{Duration, sleep};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Orientation {
    Normal,
    LeftUp,
    BottomUp,
    RightUp,
}
impl Orientation {
    pub fn new(s: &str) -> Option<Self> {
        match s {
            "normal" => Some(Self::Normal),
            "left-up" => Some(Self::LeftUp),
            "right-up" => Some(Self::RightUp),
            "bottom-up" => Some(Self::BottomUp),
            _ => {
                eprintln!("Unknown orientation: {}", s);
                None
            }
        }
    }
}

fn has_service(name: &str) -> bool {
    match std::process::Command::new("systemctl")
        .arg("--user")
        .arg("is-active")
        .arg(name)
        .output()
    {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

struct Context {
    now: Orientation,
    queued: Option<Orientation>,

    output: hyprctl::Monitor,
    transforms: [u8; 4],

    is_locked: bool,
    has_touch: bool,
    has_tablet: bool,

    restart_services: Vec<String>,
}

impl Context {
    pub fn orient(&mut self, orientation: Orientation, force: bool) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_locked {
            self.queued = Some(orientation);
            return Ok(());
        }

        if self.now == orientation && !force {
            return Ok(());
        }

        let rotation = match orientation {
            Orientation::Normal => 0,
            Orientation::LeftUp => 1,
            Orientation::BottomUp => 2,
            Orientation::RightUp => 3,
        };
        let transform = self.transforms[rotation];

        let mut hyprctl = hyprctl::Rules::new();
        let monitor = hyprctl::monitor(&self.output.name)?;
        if !monitor.disabled {
            let monitor_transform = hyprctl::MonitorTransform::new(&self.output, transform);
            hyprctl.add(monitor_transform);
        }
        if self.has_touch {
            let touch_transform = hyprctl::TouchDeviceTransform::new(transform);
            hyprctl.add(touch_transform);
        }
        if self.has_tablet {
            let tablet_transform = hyprctl::TabletTransform::new(transform);
            hyprctl.add(tablet_transform);
        }
        hyprctl.exec()?;

        for service in &self.restart_services {
            let _ = std::process::Command::new("systemctl")
                .arg("--user")
                .arg("restart")
                .arg(service)
                .status();
        }

        println!("Orientation changed {:?} -> {:?}", self.now, orientation);
        self.now = orientation;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let static_config = control::Static::load();
    let config = control::Dynamic::load();

    let monitor = hyprctl::monitor(&static_config.output)?;
    println!("Monitor: {:?}", monitor);

    let devices = hyprctl::devices()?;
    println!("Input devices: {:?}", devices);

    let conn = zbus::Connection::system().await?;
    let proxy = dbus::SensorProxy::new(&conn).await?;

    proxy.claim_accelerometer().await?;
    println!("Accelerometer claimed successfully.");

    let orientation = proxy.accelerometer_orientation().await?;
    println!("Initial orientation: {}", orientation);

    let mut context = Context {
        output: monitor,
        queued: None,
        now: Orientation::new(&orientation).unwrap(),
        transforms: config.transforms,
        is_locked: config.lock,
        has_touch: devices.has_touch(),
        has_tablet: devices.has_tablet(),
        restart_services: {
            let mut services = Vec::new();
            for service in static_config.restart_services {
                if has_service(&service) {
                    services.push(service);
                }
            }
            services
        },
    };

    println!("Listening for orientation changes...");
    let mut changes = proxy.receive_accelerometer_orientation_changed().await;

    loop {
        tokio::select! {
            _ = sleep(Duration::from_secs(1)) => {
                let config = control::Dynamic::load();
                let (mut orientation, mut force) = (None, false);
                if config.transforms != context.transforms {
                    context.transforms = config.transforms;
                    println!("Transforms updated: {:?}", context.transforms);
                    (orientation, force) = (Some(context.now), true);
                }
                if config.lock != context.is_locked {
                    if config.lock {
                        context.is_locked = true;
                        println!("Orientation locked.");
                    } else {
                        context.is_locked = false;
                        if let Some(queued) = context.queued {
                            context.queued = None;
                            orientation = Some(queued);
                        }
                        println!("Orientation unlocked.");
                    }
                }
                if let Some(orientation) = orientation {
                    context.orient(orientation, force)?;
                }
            },

            Some(change) = changes.next() => {
                let orientation = match change.get().await {
                    Ok(s) => match Orientation::new(&s) {
                        Some(o) => o,
                        None => continue,
                    },
                    _ => {
                        eprintln!("Failed to get orientation property.");
                        continue;
                    },
                };

                context.orient(orientation, false)?;
            }
        }
    }
}
