use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TelemetryPacket {
    pub seq: u32,
    pub timestamp: u64,
    pub phase: String,
    pub gps: GpsData,
    pub baro: Barodata,
    pub imu: ImuData,
    pub battery: BatteryData,
    pub link: LinkQuality,
    pub status: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GpsData {
    pub lat: f64,
    pub lon: f64,
    pub alt: f32,
    pub speed: f32,
    pub heading: f32,
    pub sats: u8,
    pub fix: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Barodata {
    pub alt: f32,
    pub vspeed: f32,
    pub temp: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ImuData {
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub gyro: GyroData,
    pub accel: AccelData,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GyroData {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AccelData {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BatteryData {
    pub voltage: f32,
    pub current: f32,
    pub power: f32,
    pub mah_used: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LinkQuality {
    pub rssi: f32,
    pub snr: f32,
}
