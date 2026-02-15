use super::models::*;
use crate::telemetry::{FlightMetadata, TelemetryPacket};
use crate::types::BorrowedEntry;
use crate::{Key, Store};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use std::sync::{Arc, Mutex};

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
        if let Key::String(s) = key
            && let Some(timestamp) = s.strip_prefix("telem:").and_then(|t| t.parse::<u64>().ok())
            && timestamp >= start
            && timestamp <= end
            && let Ok(BorrowedEntry::Text(json)) = store.get(key)
            && let Ok(packet) = serde_json::from_str::<TelemetryPacket>(json)
        {
            packets.push(packet);
            if packets.len() >= limit {
                break;
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
        if let Key::String(s) = key
            && s.starts_with("flight:")
            && let Ok(BorrowedEntry::Text(json)) = store.get(key)
            && let Ok(metadata) = serde_json::from_str::<FlightMetadata>(json)
        {
            flights.push(metadata);
        }
    }

    flights.sort_by_key(|f| f.start_time);

    Ok(Json(FlightListResponse {
        count: flights.len(),
        flights,
    }))
}

pub async fn get_stats(State(state): State<AppState>) -> Result<Json<StatsResponse>, StatusCode> {
    let store = state.store.lock().unwrap();

    let mut total_packets = 0;
    let mut total_flights = 0;
    let mut oldest_timestamp = None;
    let mut newest_timestamp = None;
    let mut storage_size_bytes = 0; // Approximate

    for key in store.keys() {
        if let Key::String(s) = key {
            if let Some(ts_str) = s.strip_prefix("telem:") {
                if let Ok(timestamp) = ts_str.parse::<u64>() {
                    total_packets += 1;

                    if oldest_timestamp.is_none_or(|t| timestamp < t) {
                        oldest_timestamp = Some(timestamp);
                    }
                    if newest_timestamp.is_none_or(|t| timestamp > t) {
                        newest_timestamp = Some(timestamp);
                    }

                    // Approximate size: key length + value length (need to get value)
                    if let Ok(BorrowedEntry::Text(json)) = store.get(key) {
                        storage_size_bytes += s.len() + json.len();
                    }
                }
            } else if s.starts_with("flight:") {
                total_flights += 1;
                // Add flight metadata size
                if let Ok(BorrowedEntry::Text(json)) = store.get(key) {
                    storage_size_bytes += s.len() + json.len();
                }
            }
        }
    }

    Ok(Json(StatsResponse {
        total_packets,
        total_flights,
        oldest_timestamp,
        newest_timestamp,
        storage_size_bytes,
    }))
}

pub async fn delete_flight_by_id(
    State(state): State<AppState>,
    Path(flight_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let mut store = state.store.lock().unwrap();
    let flight_key = Key::String(format!("flight:{}", flight_id));

    // Check if flight exists and get time range
    let (start_time, end_time) = match store.get(&flight_key) {
        Ok(entry) => {
            if let BorrowedEntry::Text(json) = entry {
                if let Ok(metadata) = serde_json::from_str::<FlightMetadata>(json) {
                    (metadata.start_time, metadata.end_time.unwrap_or(u64::MAX))
                } else {
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            } else {
                return Err(StatusCode::NOT_FOUND);
            }
        }
        Err(_) => return Err(StatusCode::NOT_FOUND),
    };

    // Identify keys to delete
    let mut keys_to_delete = Vec::new();
    keys_to_delete.push(flight_key.clone());

    for key in store.keys() {
        if let Key::String(s) = key
            && let Some(timestamp) = s.strip_prefix("telem:").and_then(|t| t.parse::<u64>().ok())
            && timestamp >= start_time
            && timestamp <= end_time
        {
            keys_to_delete.push(key.clone());
        }
    }

    // Delete keys
    for key in keys_to_delete {
        let _ = store.delete(&key);
    }

    Ok(StatusCode::OK)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Store;
    use crate::telemetry::{FlightMetadata, TelemetryPacket};
    use crate::types::{Key, Value};
    use axum::http::StatusCode;

    fn populate_store() -> Arc<Mutex<Store>> {
        let mut store = Store::new();

        // 1. Create Flight Metadata (flight_001)
        let flight_meta = FlightMetadata {
            flight_id: "flight_001".to_string(),
            start_time: 1000,
            end_time: Some(2000),
            duration_secs: 10,
            packet_count: 5,
            max_altitude: 100.0,
            min_altitude: 0.0,
            min_battery: 3.7,
            ended_normally: true, // Should be true
        };
        let flight_json = serde_json::to_string(&flight_meta).unwrap();
        store.put(
            Key::String("flight:flight_001".to_string()),
            Value::String(flight_json),
        );

        // 2. Create Flight Telemetry (telem:1000 to telem:2000)
        let base_packet = TelemetryPacket {
            seq: 0,
            timestamp: 0,
            phase: "flight".into(),
            gps: crate::telemetry::GpsData {
                lat: 0.0,
                lon: 0.0,
                alt: 0.0,
                speed: 0.0,
                heading: 0.0,
                sats: 0,
                fix: 0,
            },
            baro: crate::telemetry::Barodata {
                alt: 0.0,
                vspeed: 0.0,
                temp: 0.0,
            },
            imu: crate::telemetry::ImuData {
                roll: 0.0,
                pitch: 0.0,
                yaw: 0.0,
                gyro: crate::telemetry::GyroData {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                accel: crate::telemetry::AccelData {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            battery: crate::telemetry::BatteryData {
                voltage: 0.0,
                current: 0.0,
                power: 0.0,
                mah_used: 0.0,
            },
            link: crate::telemetry::LinkQuality {
                rssi: 0.0,
                snr: 0.0,
            },
            status: 0,
        };

        for i in 0..5 {
            let mut packet = base_packet.clone();
            packet.seq = i;
            packet.timestamp = 1000 + (i as u64 * 250); // 1000, 1250, 1500, 1750, 2000
            let json = serde_json::to_string(&packet).unwrap();
            store.put(
                Key::String(format!("telem:{}", packet.timestamp)),
                Value::String(json),
            );
        }

        // 3. Create Unrelated Telemetry (telem:3000)
        let mut packet = base_packet.clone();
        packet.timestamp = 3000;
        let json = serde_json::to_string(&packet).unwrap();
        store.put(Key::String("telem:3000".to_string()), Value::String(json));

        Arc::new(Mutex::new(store))
    }

    #[tokio::test]
    async fn test_get_telemetry_range() {
        let store = populate_store();
        let (tx, _) = tokio::sync::broadcast::channel(100);
        let state = AppState {
            store,
            broadcast_tx: tx,
        };

        // Query range 1000-1500
        let query = TelemetryQuery {
            start: Some(1000),
            end: Some(1500),
            limit: None,
        };

        let result = get_telemetry_range(State(state), Query(query)).await;
        assert!(result.is_ok());
        let response = result.unwrap().0;

        // Should have 1000, 1250, 1500 (3 packets)
        assert_eq!(response.count, 3);
        assert_eq!(response.packets.len(), 3);
        assert_eq!(response.packets[0].timestamp, 1000);
        assert_eq!(response.packets[2].timestamp, 1500);
    }

    #[tokio::test]
    async fn test_get_telemetry_by_id() {
        let store = populate_store();
        let (tx, _) = tokio::sync::broadcast::channel(100);
        let state = AppState {
            store,
            broadcast_tx: tx,
        };

        // Test existing
        let result = get_telemetry_by_id(State(state.clone()), Path(1000)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0.timestamp, 1000);

        // Test non-existing
        let result = get_telemetry_by_id(State(state), Path(9999)).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_all_flights() {
        let store = populate_store();
        let (tx, _) = tokio::sync::broadcast::channel(100);
        let state = AppState {
            store,
            broadcast_tx: tx,
        };

        let result = get_all_flights(State(state)).await;
        assert!(result.is_ok());
        let response = result.unwrap().0;

        assert_eq!(response.count, 1);
        assert_eq!(response.flights[0].flight_id, "flight_001");
    }

    #[tokio::test]
    async fn test_delete_flight() {
        let store_arc = populate_store();
        let (tx, _) = tokio::sync::broadcast::channel(100);
        let state = AppState {
            store: store_arc.clone(),
            broadcast_tx: tx,
        };

        // Deletion
        let result =
            delete_flight_by_id(State(state.clone()), Path("flight_001".to_string())).await;
        assert_eq!(result, Ok(StatusCode::OK));

        // Verification
        let store = store_arc.lock().unwrap();
        assert!(
            store
                .get(&Key::String("flight:flight_001".to_string()))
                .is_err()
        );
        // 1000-2000 should be gone
        assert!(store.get(&Key::String("telem:1000".to_string())).is_err());
        assert!(store.get(&Key::String("telem:2000".to_string())).is_err());
        // 3000 should remain
        assert!(store.get(&Key::String("telem:3000".to_string())).is_ok());
    }

    #[tokio::test]
    async fn test_get_stats() {
        let store = populate_store();
        let (tx, _) = tokio::sync::broadcast::channel(100);
        let state = AppState {
            store,
            broadcast_tx: tx,
        };

        let result = get_stats(State(state)).await;
        assert!(result.is_ok());
        let stats = result.unwrap().0;

        assert_eq!(stats.total_packets, 6); // 5 flight + 1 unrelated
        assert_eq!(stats.total_flights, 1);
        assert_eq!(stats.oldest_timestamp, Some(1000));
        assert_eq!(stats.newest_timestamp, Some(3000));
        assert!(stats.storage_size_bytes > 0);
    }
}
