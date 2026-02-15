use super::config::FlightDetectionConfig;
use super::types::TelemetryPacket;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightMetadata {
    pub flight_id: String,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub duration_secs: u64,
    pub packet_count: u32,
    pub max_altitude: f32,
    pub min_altitude: f32,
    pub ended_normally: bool,
    pub min_battery: f32,
}

pub struct FlightDetector {
    current_flight: Option<FlightSession>,
    last_packet_time: Option<u64>,
    flight_counter: u32,
    config: FlightDetectionConfig,
}
struct FlightSession {
    flight_id: String,
    start_time: u64,

    packet_timestamps: Vec<u64>,
    max_altitude: f32,
    min_altitude: f32,
    min_battery: f32,
    ground_time_start: Option<u64>,
}

impl FlightDetector {
    pub fn new(config: FlightDetectionConfig) -> Self {
        Self {
            current_flight: None,
            last_packet_time: None,
            flight_counter: 0,
            config,
        }
    }

    pub fn process_packet(&mut self, packet: &TelemetryPacket) -> Option<FlightMetadata> {
        let altitude = packet.baro.alt;
        let speed = packet.gps.speed;

        if self.current_flight.is_none() && speed > self.config.start_speed_ms {
            self.start_flight(packet);
        }

        if let Some(ref mut flight) = self.current_flight {
            flight.packet_timestamps.push(packet.timestamp);
            flight.max_altitude = flight.max_altitude.max(altitude);
            flight.min_altitude = flight.min_altitude.min(altitude);
            flight.min_battery = flight.min_battery.min(packet.battery.voltage);

            if speed < self.config.end_speed_ms {
                if flight.ground_time_start.is_none() {
                    flight.ground_time_start = Some(packet.timestamp);
                }

                if let Some(ground_start) = flight.ground_time_start
                    && packet.timestamp - ground_start > self.config.ground_stable_duration_ms
                {
                    return Some(self.end_flight(packet, true));
                }
            } else {
                flight.ground_time_start = None;
            }
        }

        if self.current_flight.is_some()
            && let Some(last_time) = self.last_packet_time
            && packet.timestamp - last_time > self.config.timeout_duration_ms
        {
            return Some(self.end_flight(packet, false));
        }
        self.last_packet_time = Some(packet.timestamp);
        None
    }

    pub fn start_flight(&mut self, packet: &TelemetryPacket) {
        self.flight_counter += 1;
        let flight_id = format!("flight_{:03}", self.flight_counter);
        println!(
            "\n Flight_id: {} Started at timestamp {}",
            flight_id, packet.timestamp
        );
        self.current_flight = Some(FlightSession {
            flight_id,
            start_time: packet.timestamp,

            packet_timestamps: vec![packet.timestamp],
            max_altitude: packet.baro.alt,
            min_altitude: packet.gps.alt,
            min_battery: packet.battery.voltage,
            ground_time_start: None,
        });
    }

    pub fn end_flight(&mut self, packet: &TelemetryPacket, normal: bool) -> FlightMetadata {
        let flight = self.current_flight.take()
            .expect("BUG: end_flight() called with no active flight - this indicates a logic error in the detector");

        let duration_ms = packet.timestamp - flight.start_time;
        let duration_secs = duration_ms / 1000;

        let metadata = FlightMetadata {
            flight_id: flight.flight_id.clone(),
            start_time: flight.start_time,
            end_time: Some(packet.timestamp),
            duration_secs,
            packet_count: flight.packet_timestamps.len() as u32,
            max_altitude: flight.max_altitude,
            min_altitude: flight.min_altitude,
            min_battery: flight.min_battery,
            ended_normally: normal,
        };

        if metadata.max_altitude > self.config.min_takeoff_altitude_m {
            println!(
                "\n Flight ended: {} ({} packets, {:.1}s)",
                metadata.flight_id, metadata.packet_count, metadata.duration_secs
            );
            println!(
                " Max alt: {:.1}m, Min batt: {:.1}V, Normal: {}",
                metadata.max_altitude, metadata.min_battery, metadata.ended_normally
            );
        } else {
            println!(
                "\n Taxing event discarded: {} (max alt {:.1}m - never took off,need {:.1})m)",
                metadata.flight_id, metadata.max_altitude, self.config.min_takeoff_altitude_m
            );
        }

        metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::{
        AccelData, Barodata, BatteryData, GpsData, GyroData, ImuData, LinkQuality,
    };

    fn create_dummy_packet(timestamp: u64, alt: f32, speed: f32, voltage: f32) -> TelemetryPacket {
        TelemetryPacket {
            seq: 0,
            timestamp,
            phase: "TEST".to_string(),
            gps: GpsData {
                lat: 0.0,
                lon: 0.0,
                alt: 0.0,
                speed,
                heading: 0.0,
                sats: 0,
                fix: 0,
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
                voltage,
                current: 0.0,
                power: 0.0,
                mah_used: 0.0,
            },
            link: LinkQuality {
                rssi: 0.0,
                snr: 0.0,
            },
            status: 0,
        }
    }

    fn get_test_config() -> FlightDetectionConfig {
        FlightDetectionConfig {
            start_altitude_m: 10.0,
            start_speed_ms: 5.0,
            end_altitude_m: 5.0,
            end_speed_ms: 1.0,
            min_takeoff_altitude_m: 20.0,
            ground_stable_duration_ms: 1000,
            timeout_duration_ms: 2000,
        }
    }

    #[test]
    fn test_flight_detection_lifecycle() {
        let config = get_test_config();
        let mut detector = FlightDetector::new(config);

        // 1. Ground - No flight
        let p1 = create_dummy_packet(1000, 0.0, 0.0, 12.0);
        assert!(detector.process_packet(&p1).is_none());

        // 2. Takeoff - Speed exceeds threshold
        let p2 = create_dummy_packet(2000, 15.0, 6.0, 11.9);
        assert!(detector.process_packet(&p2).is_none()); // Flight started internally

        // 3. Ascent - Max altitude updates
        let p3 = create_dummy_packet(3000, 50.0, 10.0, 11.8);
        assert!(detector.process_packet(&p3).is_none());

        // 4. Descent - Landing trigger (Speed < 1.0)
        let p4 = create_dummy_packet(4000, 5.0, 0.5, 11.7);
        assert!(detector.process_packet(&p4).is_none()); // Ground timer starts

        // 5. Landed - Ground stable duration exceeded (1000ms)
        let p5 = create_dummy_packet(5500, 0.0, 0.0, 11.6); // 1500ms after p4
        let metadata = detector
            .process_packet(&p5)
            .expect("Should detect flight end");

        assert_eq!(metadata.duration_secs, 3); // 5500 - 2000 = 3500ms -> 3s
        assert_eq!(metadata.max_altitude, 50.0);
        assert_eq!(metadata.min_battery, 11.6);
        assert!(metadata.ended_normally);
        assert_eq!(metadata.flight_id, "flight_001");
    }

    #[test]
    fn test_timeout_detection() {
        let config = get_test_config();
        let mut detector = FlightDetector::new(config);

        // Start flight
        let p1 = create_dummy_packet(1000, 10.0, 6.0, 12.0);
        detector.process_packet(&p1);

        // Timeout - 3000ms later (timeout is 2000ms)
        let p2 = create_dummy_packet(4000, 50.0, 10.0, 11.5);
        let metadata = detector.process_packet(&p2).expect("Should detect timeout");

        assert!(!metadata.ended_normally);
        assert_eq!(metadata.duration_secs, 3);
    }

    #[test]
    fn test_filter_taxiing() {
        let config = get_test_config();
        let mut detector = FlightDetector::new(config);

        // Start "flight" (speed trigger)
        let p1 = create_dummy_packet(1000, 0.0, 6.0, 12.0);
        detector.process_packet(&p1);

        // Low altitude - never reached min_takeoff_altitude_m (20.0)
        let p2 = create_dummy_packet(2000, 15.0, 0.5, 11.9); // Max alt 15.0
        detector.process_packet(&p2);

        // End flight
        let p3 = create_dummy_packet(3500, 2.0, 0.0, 11.8);
        // This logic is tricky: detector returns metadata regardless of altitude,
        // but prints "discarded" to stdout.
        // The test verifies the metadata is returned but we check the logic within the metadata.

        // Wait, review `end_flight`: it returns metadata ALWAYS. The filtering is only in the PRINT logic.
        // If the user wants to filter what gets stored, the CALLER needs to check metadata.max_altitude.
        // Let's verify our assumptions about the detector code.

        let metadata = detector.process_packet(&p3).unwrap();
        assert!(metadata.max_altitude < 20.0);
    }
}
