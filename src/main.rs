mod dbus;
mod hyprctl;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = zbus::blocking::connection::Connection::system()?;
    let proxy = dbus::SensorProxyBlocking::new(&conn)?;
    proxy.claim_accelerometer()?;
    println!("Accelerometer claimed successfully.");

    let monitor = hyprctl::monitor("eDP-1")?;
    println!("Monitor: {:?}", monitor);

    let orientation = proxy.accelerometer_orientation()?;
    println!("Current orientation: {}", orientation);

    Ok(())
}
