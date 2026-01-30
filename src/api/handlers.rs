use std::env::SplitPaths;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json
};
use std::sync::{Arc, Mutex};
use crate::{Store, Key};
use crate::types::BorrowedEntry;
use crate::telemetry::{TelemetryPacket, FlightMetadata};
use super::models::*;

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<Mutex<Store>>,
    pub broadcast_tx: tokio::sync::broadcast::Sender<TelemetryPacket>,
}

pub async fn get_telemetry_range(
    State(state): State<AppState>,
    Query(params): Query<TelemetryQuery>,
) -> Result<Json<TelemetryResponse>, StatusCode> {
    let store = state.store.lock().unwrap();

    let start = params.start.unwrap_or(0);
    let end = params.end.unwrap_or(u64::MAX);
    let limit = params.limit.unwrap_or(1000);

    let mut packets = Vec::new();

    for key in store.keys() {
        if let Key::String(s) = key {
            if let Some(ts_str) = s.strip_prefix("telem:") {
            if let Ok(timestamp) = ts_str.parse::<u64>() {
            if timestamp >=start && timestamp <= end {
                if let Ok(entry) = store.get(&key) {
                    if let BorrowedEntry::Text(json) = entry {
                        if let Ok(packet) = serde_json::from_str::<TelemetryPacket>(json) {
                            packets.push(packet);
                            if packets.len() >= limit {
                                break;
                            }
                        }
                    }
                }
            }
            }
            }
        }
    }

    packets.sort_by_key(|p| p.timestamp);

    Ok(Json(TelemetryResponse {
        count: packets.len(),
        packets,
    }))

}

pub async fn get_telemetry_by_id(
    State(state): State<AppState>,
    Path(timestamp): Path<u64>,
) -> Result<Json<TelemetryPacket>, StatusCode> {
    let store = state.store.lock().unwrap();
    let key = Key::String(format!("telem:{}", timestamp));

    match store.get(&key) {
        Ok(entry) => {
            if let BorrowedEntry::Text(json) = entry {
                match serde_json::from_str::<TelemetryPacket>(json) {
                    Ok(packet) => Ok(Json(packet)),
                    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
                }
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn get_all_flights(
    State(state): State<AppState>,
) -> Result<Json<FlightListResponse>, StatusCode> {
    let store = state.store.lock().unwrap();

    let mut flights = Vec::new();

    for key in store.keys() {
        if let Key::String(s) = key {
            if s.starts_with("flight:") {
                if let Ok(entry) = store.get(&key) {
                    if let BorrowedEntry::Text(json) = entry {
                        if let Ok(metadata) = serde_json::from_str::<FlightMetadata>(json) {
                            flights.push(metadata);
                        }
                    }
                }
            }
        }
    }

    flights.sort_by_key(|f| f.start_time);

    Ok(Json(FlightListResponse {
        count: flights.len(),
        flights,
    }))
}




