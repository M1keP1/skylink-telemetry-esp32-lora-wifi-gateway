use super::types::TelemetryPacket;
use super::config::FlightDetectionConfig;
use serde::{Deserialize, Serialize};
use tokio::macros::support::SelectBiased;

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
    start_seq: u32,
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

        if self.current_flight.is_none() {
            if speed > self.config.start_speed_ms {
                self.start_flight(packet);
            }
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


            if let Some(ground_start) = flight.ground_time_start {
                if packet.timestamp - ground_start > self.config.ground_stable_duration_ms {
                    return Some(self.end_flight(packet, true));
                }
            }
        } else {
            flight.ground_time_start = None;
        }
    }

        if self.current_flight.is_some() {
            if let Some(last_time) = self.last_packet_time {
                if packet.timestamp - last_time > self.config.timeout_duration_ms {
                    return Some(self.end_flight(packet, false));
                }
            }
        }
        self.last_packet_time = Some(packet.timestamp);
        None

    }

    pub fn start_flight(&mut self, packet: &TelemetryPacket) {
        self.flight_counter += 1;
        let flight_id = format!("flight_{:03}", self.flight_counter);
        println!("\n Flight_id: {} Started at timestamp {}", flight_id, packet.timestamp);
        self.current_flight = Some(FlightSession {
            flight_id,
            start_time: packet.timestamp,
            start_seq: packet.seq,
            packet_timestamps: vec![packet.timestamp],
            max_altitude: packet.baro.alt,
            min_altitude: packet.gps.alt,
            min_battery: packet.battery.voltage,
            ground_time_start: None,
        });
    }

    pub fn end_flight(&mut self, packet: &TelemetryPacket, normal: bool) -> FlightMetadata {
            let flight = self.current_flight.take().unwrap();

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
                println!("\n Flight ended: {} ({} packets, {:.1}s)",
                         metadata.flight_id,
                         metadata.packet_count,
                         metadata.duration_secs
                );
                println!(" Max alt: {:.1}m, Min batt: {:.1}V, Normal: {}",
                         metadata.max_altitude,
                         metadata.min_battery,
                         metadata.ended_normally);
            } else {
                println!("\n Taxing event discarded: {} (max alt {:.1}m - never took off,need {:.1})m)",
                         metadata.flight_id,
                         metadata.max_altitude,
                         self.config.min_takeoff_altitude_m);
            }

            metadata
    }

}

