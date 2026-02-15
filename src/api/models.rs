use crate::telemetry::{FlightMetadata, TelemetryPacket};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TelemetryResponse {
    pub count: usize,
    pub packets: Vec<TelemetryPacket>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FlightListResponse {
    pub count: usize,
    pub flights: Vec<FlightMetadata>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StatsResponse {
    pub total_packets: usize,
    pub total_flights: usize,
    pub oldest_timestamp: Option<u64>,
    pub newest_timestamp: Option<u64>,
    pub storage_size_bytes: usize,
}

#[derive(Deserialize, Debug)]
pub struct TelemetryQuery {
    pub start: Option<u64>,
    pub end: Option<u64>,
    pub limit: Option<usize>,
}
