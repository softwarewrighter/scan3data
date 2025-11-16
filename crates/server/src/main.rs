//! scan3data REST API server
//!
//! Three-phase processing pipeline: Scan -> Classify & Correct -> Convert
//!
//! Copyright (c) 2025 Michael A Wright

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
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

    // API routes
    let api_routes = Router::new()
        .route("/health", get(health_check))
        .route("/api/scan_sets", post(create_scan_set))
        .route("/api/scan_sets/:id/upload", post(upload_image))
        .route("/api/scan_sets/:id/artifacts", get(get_artifacts))
        .route("/api/clean-image", post(clean_image))
        .with_state(state);

    // Serve static files from dist directory (WASM frontend)
    let serve_dir = ServeDir::new("dist").not_found_service(ServeDir::new("dist/index.html"));

    // Combine routes: API routes take precedence, then static files
    let app = Router::new()
        .merge(api_routes)
        .nest_service("/", serve_dir)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr = "127.0.0.1:7214";
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

#[derive(Deserialize)]
struct CleanImageRequest {
    /// Base64-encoded image data
    image_data: String,
}

#[derive(Serialize)]
struct CleanImageResponse {
    /// Base64-encoded cleaned image data
    cleaned_image_data: String,
}

async fn clean_image(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<CleanImageRequest>,
) -> Result<Json<CleanImageResponse>, StatusCode> {
    // Decode base64 image
    let image_bytes = general_purpose::STANDARD
        .decode(&payload.image_data)
        .map_err(|e| {
            tracing::error!("Failed to decode base64 image: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    // Create Gemini client from environment
    let gemini_client = llm_bridge::GeminiClient::from_env().map_err(|e| {
        tracing::error!("Failed to create Gemini client: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Clean the image
    let cleaned_bytes = gemini_client.clean_image(&image_bytes).await.map_err(|e| {
        tracing::error!("Failed to clean image: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Encode back to base64
    let cleaned_b64 = general_purpose::STANDARD.encode(&cleaned_bytes);

    Ok(Json(CleanImageResponse {
        cleaned_image_data: cleaned_b64,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_image_request_deserialize() {
        let json = r#"{"image_data": "dGVzdA=="}"#;
        let req: CleanImageRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.image_data, "dGVzdA==");
    }

    #[test]
    fn test_clean_image_response_serialize() {
        let response = CleanImageResponse {
            cleaned_image_data: "Y2xlYW5lZA==".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("cleaned_image_data"));
        assert!(json.contains("Y2xlYW5lZA=="));
    }

    #[test]
    fn test_base64_roundtrip() {
        let original = b"test image data";
        let encoded = general_purpose::STANDARD.encode(original);
        let decoded = general_purpose::STANDARD.decode(&encoded).unwrap();
        assert_eq!(original, decoded.as_slice());
    }
}
