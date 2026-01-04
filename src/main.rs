mod dbus;
mod hyprctl;

#[derive(Debug, PartialEq)]
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
            },
        }
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
}

impl Context {
    pub fn lock(&mut self) {
        self.is_locked = true;
    }

    pub fn unlock(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.is_locked = false;
        if let Some(orientation) = self.queued.take() {
            self.orient(orientation)?;
            self.queued = None;
        }
        Ok(())
    }

    pub fn orient(&mut self, orientation: Orientation) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_locked {
            self.queued = Some(orientation);
            return Ok(());
        }

        if self.now == orientation {
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

        println!("Orientation changed {:?} -> {:?}", self.now, orientation);
        self.now = orientation;

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = zbus::blocking::connection::Connection::system()?;
    let proxy = dbus::SensorProxyBlocking::new(&conn)?;
    proxy.claim_accelerometer()?;
    println!("Accelerometer claimed successfully.");

    let devices = hyprctl::devices()?;
    println!("Input devices: {:?}", devices);

    let monitor = hyprctl::monitor("eDP-1")?;
    println!("Monitor: {:?}", monitor);

    let orientation = proxy.accelerometer_orientation()?;
    println!("Initial orientation: {}", orientation);

    let mut context = Context {
        output: monitor,
        queued: None,
        now: Orientation::new(&orientation).unwrap(),
        transforms: [0, 1, 2, 3],
        is_locked: false,
        has_touch: devices.has_touch(),
        has_tablet: devices.has_tablet(),
    };

    println!("Listening for orientation changes...");
    let mut props = proxy.receive_accelerometer_orientation_changed();
    while let Some(prop) = props.next() {
        let orientation = match prop.get() {
            Err(_) => {
                eprintln!("Failed to get orientation property.");
                continue;
            }
            Ok(s) => match Orientation::new(&s) {
                Some(o) => o,
                None => continue,
            },
        };

        context.orient(orientation)?;
    }

    Ok(())
}
