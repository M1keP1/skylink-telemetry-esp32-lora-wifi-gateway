use crate::telemetry::{
    AccelData, Barodata, BatteryData, GpsData, GyroData, ImuData, LinkQuality, TelemetryPacket,
};

pub struct FlightSimulator {
    start_timestamp: u64,
}

impl Default for FlightSimulator {
    fn default() -> Self {
        Self::new(0) // Default to 0, caller should usually set this
    }
}

impl FlightSimulator {
    pub fn new(start_timestamp: u64) -> Self {
        Self { start_timestamp }
    }

    pub fn generate_packet(&self, offset_seconds: u32) -> TelemetryPacket {
        let ts = self.start_timestamp + (offset_seconds as u64 * 1000);

        // Simulating a flight profile
        // 0-10s: Ground
        // 10-40s: Ascent
        // 40-70s: Cruise
        // 70-90s: Descent
        // 90-100s: Ground/Landed

        let (alt, speed, phase) = if offset_seconds < 10 {
            (0.0, 0.0, "GROUND")
        } else if offset_seconds < 40 {
            ((offset_seconds - 10) as f32 * 10.0, 15.0, "ASCENT")
        } else if offset_seconds < 70 {
            (300.0, 20.0, "CRUISE")
        } else if offset_seconds < 90 {
            (300.0 - (offset_seconds - 70) as f32 * 15.0, 15.0, "DESCENT")
        } else {
            (0.0, 0.0, "LANDED")
        };

        TelemetryPacket {
            seq: offset_seconds,
            timestamp: ts,
            phase: phase.to_string(),
            gps: GpsData {
                lat: 0.0,
                lon: 0.0,
                alt,
                speed,
                heading: 0.0,
                sats: 8,
                fix: 1,
            },
            baro: Barodata {
                alt,
                vspeed: 0.0,
                temp: 25.0,
            },
            imu: ImuData {
                roll: 0.0,
                pitch: 0.0,
                yaw: 0.0,
                gyro: GyroData {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                accel: AccelData {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            battery: BatteryData {
                voltage: 12.0,
                current: 1.5,
                power: 18.0,
                mah_used: 10.0,
            },
            link: LinkQuality {
                rssi: -80.0,
                snr: 10.0,
            },
            status: 0,
        }
    }
}
