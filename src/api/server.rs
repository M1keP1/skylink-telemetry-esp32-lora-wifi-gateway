use axum::{
    routing::get,
    Router,
};

use tower_http::cors::{CorsLayer, Any};
use std::sync::{Arc, Mutex};
use crate::Store;
use super::handlers::*;

pub fn create_router(store: Arc<Mutex<Store>>, broadcast_tx: tokio::sync::broadcast::Sender<crate::telemetry::TelemetryPacket>) -> Router {
    let app_state = super::handlers::AppState {
        store,
        broadcast_tx,
    };

    Router::new()

        .route("/api/telemetry",get(get_telemetry_range))
        .route("/api/telemetry/{timestamp}",get(get_telemetry_by_id))

        .route("/api/flights",get(get_all_flights))
        .route("/api/flights/{id}",get(get_telemetry_by_id))
        //.route("/api/flights/:id/data",get(get_flight_telemetry))

        //.route("/api/stats",get(get_stats))

        // WebSocket endpoint for live streaming
        .route("/ws/telemetry", get(super::websocket::websocket_handler))

        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        )
        .with_state(app_state)
}

pub async fn start_server(
    store: Arc<Mutex<Store>>, 
    broadcast_tx: tokio::sync::broadcast::Sender<crate::telemetry::TelemetryPacket>,
    host: &str, 
    port: u16
) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_router(store, broadcast_tx);
    let addr = format!("{}:{}", host, port);

    println!("Skylink API Server starting on http://{}", addr);
    println!("Available Endpoints:");
    println!(" GET /api/telemetry?start=X&end=Y&limit=Z");
    println!(" GET /api/telemetry/:timestamp");
    println!(" GET /api/flights");
    println!(" GET /api/flights/:id");
    println!(" GET /api/flights/:id/data");
    println!(" GET /api/stats");
    println!(" WS  /ws/telemetry (WebSocket live stream)");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener,app).await?;
    Ok(())
}