use zbus;

#[zbus::proxy(
    interface = "net.hadess.SensorProxy",
    default_service = "net.hadess.SensorProxy",
    default_path = "/net/hadess/SensorProxy"
)]
pub trait Sensor {
    fn claim_accelerometer(&self) -> zbus::Result<()>;

    #[zbus(property)]
    fn accelerometer_orientation(&self) -> zbus::Result<String>;
}
