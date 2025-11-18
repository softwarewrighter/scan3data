# REST API

The **server** crate implements the Axum-based REST API backend. It serves both API endpoints and static files for the Yew frontend.

## Overview

**Crate Name:** `server` (binary: scan3data-server)
**Framework:** Axum 0.7
**Runtime:** Tokio (async)
**Port:** 7214 (default)
**Dependencies:** axum, tokio, tower-http, serde, serde_json

## API Endpoints

### Health Check

```http
GET /health
```

**Response:**
```
OK
```

**Status:** ✅ Implemented

### Clean Image

```http
POST /api/clean-image
Content-Type: application/json
```

**Request Body:**
```json
{
  "image_data": "base64-encoded-image-data"
}
```

**Response:**
```json
{
  "cleaned_image_data": "base64-encoded-cleaned-image"
}
```

**Status:** ✅ Implemented (Gemini API integration)

**Example:**
```bash
curl -X POST http://localhost:7214/api/clean-image \
  -H "Content-Type: application/json" \
  -d '{"image_data": "iVBORw0KGgo..."}'
```

### Create Scan Set

```http
POST /api/scan_sets
Content-Type: application/json
```

**Request Body:** (optional metadata)
```json
{
  "name": "forth-scans-1970",
  "notes": "Chuck Moore's Forth source code"
}
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Status:** 🚧 Placeholder (returns UUID, no storage)

### Upload Image

```http
POST /api/scan_sets/:id/upload
Content-Type: multipart/form-data
```

**Form Fields:**
- `file`: Image file (JPEG, PNG, TIFF)

**Response:**
```json
{
  "artifact_id": "660e8400-e29b-41d4-a716-446655440111",
  "status": "uploaded"
}
```

**Status:** 🚧 Placeholder

### Get Artifacts

```http
GET /api/scan_sets/:id/artifacts
```

**Response:**
```json
{
  "artifacts": [
    {
      "artifact_id": "660e8400-e29b-41d4-a716-446655440111",
      "type": "page",
      "layout_label": "SourceListing",
      "confidence": 0.95
    }
  ]
}
```

**Status:** 🚧 Placeholder

### OCR Extract (Planned)

```http
POST /api/ocr-extract
Content-Type: application/json
```

**Request Body:**
```json
{
  "image_data": "base64-encoded-image",
  "use_cleaned": true
}
```

**Response:**
```json
{
  "ocr_text": "      LATEST @ CFA NFA DUP C@ 31 AND\n..."
}
```

**Status:** 📋 Planned

## Implementation

### Server Setup

```rust
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

    // Static file serving
    let serve_dir = ServeDir::new("dist")
        .not_found_service(ServeDir::new("dist/index.html"));

    // Combined routes
    let app = Router::new()
        .merge(api_routes)
        .nest_service("/", serve_dir)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr = "127.0.0.1:7214";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### Clean Image Handler

```rust
async fn clean_image(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CleanImageRequest>,
) -> Result<Json<CleanImageResponse>, StatusCode> {
    // Decode base64 image
    let image_bytes = general_purpose::STANDARD
        .decode(&request.image_data)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Call LLM bridge
    let gemini_client = GeminiClient::new(
        std::env::var("GEMINI_API_KEY")
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    );

    let cleaned_bytes = gemini_client
        .clean_image(&image_bytes)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Encode to base64
    let cleaned_base64 = general_purpose::STANDARD.encode(&cleaned_bytes);

    Ok(Json(CleanImageResponse {
        cleaned_image_data: cleaned_base64,
    }))
}

#[derive(Deserialize)]
struct CleanImageRequest {
    image_data: String,
}

#[derive(Serialize)]
struct CleanImageResponse {
    cleaned_image_data: String,
}
```

## CORS Configuration

```rust
use tower_http::cors::CorsLayer;

// Development: Permissive CORS
let cors = CorsLayer::permissive();

// Production: Restricted CORS
let cors = CorsLayer::new()
    .allow_origin("https://scan3data.example.com".parse::<HeaderValue>().unwrap())
    .allow_methods([Method::GET, Method::POST])
    .allow_headers([CONTENT_TYPE]);
```

## Error Handling

### Error Types

```rust
#[derive(Debug)]
enum ApiError {
    BadRequest(String),
    NotFound(String),
    InternalError(String),
    ServiceUnavailable(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::ServiceUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
        };

        let body = Json(json!({
            "error": message
        }));

        (status, body).into_response()
    }
}
```

### Error Response Format

```json
{
  "error": "Human-readable error message"
}
```

## Middleware

### Tracing

```rust
use tower_http::trace::TraceLayer;

let app = Router::new()
    .route("/api/...", ...)
    .layer(TraceLayer::new_for_http());
```

**Logs:**
```
INFO scan3data_server: request: method=POST path=/api/clean-image
INFO scan3data_server: response: status=200 duration=2341ms
```

### Static File Serving

```rust
use tower_http::services::ServeDir;

let serve_dir = ServeDir::new("dist")
    .not_found_service(ServeDir::new("dist/index.html"));

let app = Router::new()
    .nest_service("/", serve_dir);
```

**Routes:**
- `/` → `dist/index.html`
- `/assets/app.js` → `dist/assets/app.js`
- `/unknown` → `dist/index.html` (SPA fallback)

## Running the Server

### Development

```bash
# Build and run
cargo run -p scan3data-server

# With environment variables
GEMINI_API_KEY=your-key cargo run -p scan3data-server
```

### Production

```bash
# Build release binary
cargo build -p scan3data-server --release

# Run
GEMINI_API_KEY=your-key ./target/release/scan3data-server
```

### Docker (Planned)

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build -p scan3data-server --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/scan3data-server /usr/local/bin/
COPY dist /app/dist
ENV GEMINI_API_KEY=""
EXPOSE 7214
CMD ["scan3data-server"]
```

## Testing

### Integration Tests

```rust
#[tokio::test]
async fn test_health_check() {
    let app = create_app();

    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_clean_image() {
    let app = create_app();

    let body = json!({
        "image_data": "base64-encoded-test-image"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/clean-image")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

## Related Pages

- [Web UI](Web-UI) - Frontend that calls these APIs
- [LLM Bridge](LLM-Bridge) - Service integrations used by API
- [Data Flow](Data-Flow) - API sequence diagrams

---

**Last Updated:** 2025-11-16
