use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
pub struct TelemetryConfig {
    pub esp32: Esp32Config,
    pub storage: StorageConfig,
    pub flight_detection: FlightDetectionConfig,
}
#[derive(Debug, Deserialize)]
pub struct Esp32Config {
    pub ip: String,
    pub port: u16,
    pub reconnect_delay_secs: u64,
}

#[derive(Debug, Deserialize)]
pub struct StorageConfig {
    pub persists: bool,
    pub path: String,
    pub auto_compact: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FlightDetectionConfig {
    pub start_altitude_m: f32,
    pub start_speed_ms: f32,
    pub end_altitude_m: f32,
    pub min_takeoff_altitude_m: f32,
    pub end_speed_ms: f32,
    pub ground_stable_duration_ms: u64,
    pub timeout_duration_ms: u64,
}

impl TelemetryConfig {
    pub fn load() -> Result<Self, Box<dyn Error>> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("config"))
            .build()?;

        let settings = config.try_deserialize()?;
        Ok(settings)
    }

    pub fn esp32_address(&self) -> String {
        format!("{}:{}", self.esp32.ip, self.esp32.port)
    }
}