//! Integration mode — JWT-based file editing for embedding in other products.
//!
//! Host product generates a JWT containing:
//! - fileId: document identifier
//! - userId, userName: editor identity
//! - permissions: "edit" | "view" | "comment"
//! - downloadUrl: where to fetch the document
//! - callbackUrl: where to POST modifications
//!
//! Flow:
//! 1. Host sends user to: /edit?token=<jwt>
//! 2. Server validates JWT, fetches file from downloadUrl
//! 3. Creates editing session, redirects to editor with ?file=<fileId>
//! 4. On save/close, server POSTs modified file to callbackUrl

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

use crate::routes::AppState;

/// JWT claims for integration mode.
#[derive(Debug, Serialize, Deserialize)]
pub struct IntegrationClaims {
    /// Document identifier in the host system.
    pub file_id: String,
    /// User ID.
    pub user_id: String,
    /// User display name.
    pub user_name: String,
    /// Permission level: "edit", "view", "comment".
    #[serde(default = "default_permissions")]
    pub permissions: String,
    /// URL to download the document from.
    pub download_url: Option<String>,
    /// URL to POST modifications to.
    pub callback_url: Option<String>,
    /// Expiry timestamp (Unix seconds).
    #[serde(default)]
    pub exp: u64,
}

fn default_permissions() -> String {
    "edit".to_string()
}

/// Query params for /edit endpoint.
#[derive(Debug, Deserialize)]
pub struct EditQuery {
    pub token: Option<String>,
}

/// Handle /edit?token=<jwt> — entry point for integrated editing.
///
/// Validates the JWT, fetches the document, creates a session,
/// and redirects to the editor with ?file=<fileId>.
pub async fn handle_edit(
    State(state): State<Arc<AppState>>,
    Query(query): Query<EditQuery>,
) -> Result<Response, (StatusCode, String)> {
    let token = query
        .token
        .ok_or((StatusCode::BAD_REQUEST, "Missing token parameter".into()))?;

    // Validate JWT
    let jwt_secret = std::env::var("S1_JWT_SECRET").unwrap_or_default();
    let claims = validate_integration_jwt(&token, &jwt_secret)
        .map_err(|e| (StatusCode::UNAUTHORIZED, format!("Invalid token: {e}")))?;

    // Check if session already exists for this fileId
    if state.sessions.exists(&claims.file_id).await {
        // Session exists — redirect directly
        return Ok(Redirect::to(&format!(
            "/?file={}&mode={}",
            claims.file_id, claims.permissions
        ))
        .into_response());
    }

    // Fetch document from downloadUrl
    let data = if let Some(url) = &claims.download_url {
        fetch_document(url).await.map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                format!("Failed to fetch document: {e}"),
            )
        })?
    } else {
        // No download URL — create empty session
        Vec::new()
    };

    // Detect format from URL or default
    let format = claims
        .download_url
        .as_deref()
        .and_then(|u| u.rsplit('.').next())
        .unwrap_or("docx")
        .to_lowercase();

    let filename = format!("{}.{}", claims.file_id, format);

    // Create session
    state
        .sessions
        .create(
            claims.file_id.clone(),
            filename,
            data,
            format,
            Some(claims.user_id.clone()),
            claims.callback_url.clone(),
        )
        .await;

    // Redirect to editor
    Ok(Redirect::to(&format!(
        "/?file={}&mode={}",
        claims.file_id, claims.permissions
    ))
    .into_response())
}

/// Get integration session info.
#[allow(dead_code)]
pub async fn get_integration_info(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let info = state
        .sessions
        .get_info(&id)
        .await
        .ok_or((StatusCode::NOT_FOUND, format!("Session not found: {id}")))?;

    Ok(Json(json!({
        "fileId": info.file_id,
        "filename": info.filename,
        "format": info.format,
        "size": info.size,
        "editorCount": info.editor_count,
        "editors": info.editors,
        "mode": info.mode,
        "status": info.status,
        "durationSecs": info.created_at_secs_ago,
    })))
}

/// Trigger a save callback — POST current document to callbackUrl.
pub async fn trigger_save_callback(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let data = state
        .sessions
        .get_data(&id)
        .await
        .ok_or((StatusCode::NOT_FOUND, "Session not found".into()))?;

    // Get callback URL from session
    let _info = state.sessions.get_info(&id).await;
    // For now, the callback_url is stored on the session but not exposed in SessionInfo.
    // We'll send via a direct lookup.

    // TODO: Store callback_url in SessionInfo and POST to it
    tracing::info!("Save callback triggered for {} ({} bytes)", id, data.len());

    Ok(StatusCode::OK)
}

/// Validate a JWT token for integration mode.
fn validate_integration_jwt(token: &str, secret: &str) -> Result<IntegrationClaims, String> {
    if secret.is_empty() {
        return Err("S1_JWT_SECRET not configured".into());
    }

    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err("Invalid JWT format".into());
    }

    // Decode payload
    use base64::Engine as _;
    let payload_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|e| format!("Base64 error: {e}"))?;

    let claims: IntegrationClaims =
        serde_json::from_slice(&payload_bytes).map_err(|e| format!("Invalid claims: {e}"))?;

    // Verify signature (HS256)
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;

    let signing_input = format!("{}.{}", parts[0], parts[1]);
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).map_err(|e| format!("HMAC error: {e}"))?;
    mac.update(signing_input.as_bytes());

    let sig_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(parts[2])
        .map_err(|e| format!("Signature decode error: {e}"))?;

    mac.verify_slice(&sig_bytes)
        .map_err(|_| "Invalid signature".to_string())?;

    // Check expiry
    if claims.exp > 0 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if now > claims.exp {
            return Err("Token expired".into());
        }
    }

    Ok(claims)
}

/// Fetch a document from a URL.
async fn fetch_document(url: &str) -> Result<Vec<u8>, String> {
    let resp = reqwest::Client::new()
        .get(url)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| format!("Fetch error: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }

    resp.bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| format!("Read error: {e}"))
}
