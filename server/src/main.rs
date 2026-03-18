//! s1-server — Unified document editing server.
//!
//! Single binary that serves:
//! - Static editor files (HTML/JS/CSS/WASM) at `/`
//! - REST API at `/api/v1/`
//! - WebSocket collaborative editing at `/ws/edit/{file_id}`
//! - File sessions with auto-cleanup
//!
//! No nginx, no Node.js, no relay.js needed.

use axum::{
    extract::DefaultBodyLimit,
    routing::{delete, get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod collab;
mod config;
mod file_sessions;
mod hooks;
mod plugins;
mod routes;
mod storage;
mod webhooks;

use collab::RoomManager;
use file_sessions::FileSessionManager;
use routes::AppState;
use storage::{LocalStorage, MemoryStorage};
use webhooks::WebhookRegistry;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "s1_server=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::Config::load();

    // Storage backend
    let storage: Arc<dyn storage::StorageBackend> = match config.storage.as_str() {
        "memory" => {
            tracing::info!("Storage: in-memory");
            Arc::new(MemoryStorage::new())
        }
        _ => {
            tracing::info!("Storage: local ({})", config.data_dir);
            Arc::new(
                LocalStorage::new(&config.data_dir).expect("Failed to create storage directory"),
            )
        }
    };

    let webhook_registry = Arc::new(WebhookRegistry::new());
    let room_manager = Arc::new(RoomManager::new());
    let session_manager = Arc::new(FileSessionManager::new(None));

    // Clones for background tasks
    let save_rooms = room_manager.clone();
    let save_storage = storage.clone();
    let cleanup_sessions = session_manager.clone();

    let state = Arc::new(AppState {
        storage,
        webhooks: webhook_registry,
        rooms: room_manager,
        sessions: session_manager,
    });

    // Static editor files directory
    let static_dir = std::env::var("S1_STATIC_DIR").unwrap_or_else(|_| "./public".to_string());

    let app = Router::new()
        // Health
        .route("/health", get(routes::health))
        // WebSocket editing (per file)
        .route("/ws/edit/{file_id}", get(collab::ws_collab_handler))
        // REST API
        .nest("/api/v1", api_routes())
        .with_state(state)
        // Static editor files (fallback for SPA routing)
        .fallback_service(ServeDir::new(&static_dir).append_index_html_on_directories(true))
        // Middleware
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .layer(DefaultBodyLimit::max(64 * 1024 * 1024));

    // Background: save dirty collab rooms every 30s
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            save_rooms.save_dirty_rooms(save_storage.as_ref()).await;
        }
    });

    // Background: clean up expired file sessions every 60s
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            let expired = cleanup_sessions.cleanup_expired().await;
            for (file_id, _callback_url) in &expired {
                tracing::info!("Session expired: {}", file_id);
            }
        }
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("═══════════════════════════════════════");
    tracing::info!("  s1-server v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("───────────────────────────────────────");
    tracing::info!("  Editor:    http://{}/", addr);
    tracing::info!("  API:       http://{}/api/v1/", addr);
    tracing::info!("  WebSocket: ws://{}/ws/edit/{{file_id}}", addr);
    tracing::info!("  Static:    {}", static_dir);
    tracing::info!("═══════════════════════════════════════");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn api_routes() -> Router<Arc<AppState>> {
    Router::new()
        // File sessions (temp editing with TTL)
        .route("/files", post(routes::upload_file))
        .route("/files", get(routes::list_files))
        .route("/files/{id}", get(routes::get_file_info))
        .route("/files/{id}/download", get(routes::download_file))
        .route("/files/{id}", delete(routes::close_file))
        // Documents (persistent storage)
        .route("/documents", post(routes::create_document))
        .route("/documents", get(routes::list_documents))
        .route("/documents/{id}", get(routes::get_document_meta))
        .route("/documents/{id}/content", get(routes::get_document))
        .route("/documents/{id}", delete(routes::delete_document))
        .route("/documents/{id}/thumbnail", get(routes::get_thumbnail))
        // Conversion
        .route("/convert", post(routes::convert_document))
        // Webhooks
        .route("/webhooks", post(routes::register_webhook))
        .route("/webhooks", get(routes::list_webhooks))
        .route("/webhooks/{id}", delete(routes::delete_webhook))
        // Info
        .route("/info", get(routes::server_info))
}
