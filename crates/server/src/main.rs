//! scan2data REST API server

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
struct AppState {
    // TODO: Add database connection, job queue, etc.
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let state = Arc::new(AppState {});

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/scan_sets", post(create_scan_set))
        .route("/api/scan_sets/:id/upload", post(upload_image))
        .route("/api/scan_sets/:id/artifacts", get(get_artifacts))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = "127.0.0.1:3000";
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}

async fn create_scan_set(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<CreateScanSetResponse>, StatusCode> {
    // TODO: Create new scan set
    Ok(Json(CreateScanSetResponse {
        id: uuid::Uuid::new_v4().to_string(),
    }))
}

async fn upload_image(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<UploadResponse>, StatusCode> {
    // TODO: Handle image upload
    Ok(Json(UploadResponse {
        artifact_id: uuid::Uuid::new_v4().to_string(),
        status: "uploaded".to_string(),
    }))
}

async fn get_artifacts(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<ArtifactsResponse>, StatusCode> {
    // TODO: Get artifacts for scan set
    Ok(Json(ArtifactsResponse {
        artifacts: Vec::new(),
    }))
}

#[derive(Serialize)]
struct CreateScanSetResponse {
    id: String,
}

#[derive(Serialize)]
struct UploadResponse {
    artifact_id: String,
    status: String,
}

#[derive(Serialize)]
struct ArtifactsResponse {
    artifacts: Vec<ArtifactInfo>,
}

#[derive(Serialize, Deserialize)]
struct ArtifactInfo {
    id: String,
    kind: String,
}
