//! Real-time collaborative editing via WebSocket.
//!
//! Each file being edited gets a room keyed by `file_id`.
//! The server maintains authoritative document state via file sessions.
//! New editors receive the latest snapshot on connect.

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, Query, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

use crate::routes::AppState;
use crate::storage::StorageBackend;

/// A collaboration room for a document.
struct Room {
    tx: broadcast::Sender<String>,
    peer_count: usize,
    ops_log: Vec<String>,
    #[allow(dead_code)]
    doc_id: String,
    dirty: bool,
}

/// Manages all active collaboration rooms.
pub struct RoomManager {
    rooms: Mutex<HashMap<String, Room>>,
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            rooms: Mutex::new(HashMap::new()),
        }
    }

    async fn join(&self, room_id: &str) -> (broadcast::Sender<String>, Vec<String>) {
        let mut rooms = self.rooms.lock().await;
        if let Some(room) = rooms.get_mut(room_id) {
            room.peer_count += 1;
            let catch_up = room.ops_log.clone();
            (room.tx.clone(), catch_up)
        } else {
            let (tx, _) = broadcast::channel(512);
            let room = Room {
                tx: tx.clone(),
                peer_count: 1,
                ops_log: Vec::new(),
                doc_id: room_id.to_string(),
                dirty: false,
            };
            rooms.insert(room_id.to_string(), room);
            tracing::info!("Room created: {}", room_id);
            (tx, Vec::new())
        }
    }

    async fn record_op(&self, room_id: &str, op: &str) {
        let mut rooms = self.rooms.lock().await;
        if let Some(room) = rooms.get_mut(room_id) {
            room.ops_log.push(op.to_string());
            room.dirty = true;
            if room.ops_log.len() > 10_000 {
                room.ops_log.drain(..5_000);
            }
        }
    }

    fn validate_op(msg: &str) -> bool {
        serde_json::from_str::<serde_json::Value>(msg)
            .map(|v| v.get("type").is_some() || v.get("action").is_some())
            .unwrap_or(false)
    }

    async fn leave(&self, room_id: &str) -> bool {
        let mut rooms = self.rooms.lock().await;
        if let Some(room) = rooms.get_mut(room_id) {
            room.peer_count = room.peer_count.saturating_sub(1);
            if room.peer_count == 0 {
                rooms.remove(room_id);
                tracing::info!("Room closed: {}", room_id);
                return true;
            }
        }
        false
    }

    pub async fn save_dirty_rooms(&self, storage: &dyn StorageBackend) {
        let mut rooms = self.rooms.lock().await;
        for (room_id, room) in rooms.iter_mut() {
            if room.dirty && !room.ops_log.is_empty() {
                let ops_json = serde_json::to_string(&room.ops_log).unwrap_or_default();
                let meta = crate::storage::DocumentMeta {
                    id: format!("{}_ops", room_id),
                    filename: format!("{}_ops.json", room_id),
                    format: "json".to_string(),
                    size: ops_json.len(),
                    title: None,
                    author: None,
                    word_count: 0,
                    created_at: chrono::Utc::now().to_rfc3339(),
                    updated_at: chrono::Utc::now().to_rfc3339(),
                };
                if let Err(e) = storage.put(&meta.id, ops_json.as_bytes(), &meta) {
                    tracing::warn!("Failed to save room {} ops: {}", room_id, e);
                } else {
                    room.dirty = false;
                }
            }
        }
    }

    #[allow(dead_code)]
    pub async fn room_count(&self) -> usize {
        self.rooms.lock().await.len()
    }
}

/// Query params for WebSocket connection.
#[derive(Debug, Deserialize)]
pub struct WsParams {
    /// User name for presence display.
    #[serde(default = "default_user_name")]
    pub user: String,
    /// User ID for session tracking.
    #[serde(default = "default_user_id")]
    pub uid: String,
    /// Editing mode.
    #[serde(default = "default_mode")]
    pub mode: String,
}

fn default_user_name() -> String {
    format!("User-{}", rand_id())
}
fn default_user_id() -> String {
    rand_id()
}
fn default_mode() -> String {
    "edit".to_string()
}
fn rand_id() -> String {
    uuid::Uuid::new_v4().to_string()[..8].to_string()
}

/// WebSocket upgrade handler.
///
/// Route: `GET /ws/edit/{file_id}?user=Alice&uid=u123&mode=edit`
pub async fn ws_collab_handler(
    ws: WebSocketUpgrade,
    Path(file_id): Path<String>,
    Query(params): Query<WsParams>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, file_id, params, state))
}

/// Handle a WebSocket connection for collaborative editing.
async fn handle_socket(socket: WebSocket, file_id: String, params: WsParams, state: Arc<AppState>) {
    // Check if file session exists
    let has_session = state.sessions.exists(&file_id).await;

    // Join the room (creates if first peer)
    let (tx, catch_up_ops) = state.rooms.join(&file_id).await;
    let mut rx = tx.subscribe();

    // Track editor in session
    if has_session {
        state
            .sessions
            .editor_join(&file_id, &params.uid, &params.user, &params.mode)
            .await;
    }

    let (mut sender, mut receiver) = socket.split();

    // Send welcome with session info
    let file_info = if has_session {
        state
            .sessions
            .get_info(&file_id)
            .await
            .map(|i| {
                serde_json::json!({
                    "filename": i.filename,
                    "size": i.size,
                    "editorCount": i.editor_count,
                })
            })
            .unwrap_or(serde_json::json!(null))
    } else {
        serde_json::json!(null)
    };

    let welcome = serde_json::json!({
        "type": "welcome",
        "fileId": file_id,
        "user": params.user,
        "opsCount": catch_up_ops.len(),
        "file": file_info,
    });
    let _ = sender.send(Message::Text(welcome.to_string().into())).await;

    // Send document snapshot if session has data (new editor gets full doc)
    if has_session {
        if let Some(data) = state.sessions.get_data(&file_id).await {
            // Send as base64 for transport
            use base64::Engine as _;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
            let snapshot = serde_json::json!({
                "type": "snapshot",
                "data": b64,
                "size": data.len(),
            });
            let _ = sender
                .send(Message::Text(snapshot.to_string().into()))
                .await;
        }
    }

    // Send catch-up ops
    for op in &catch_up_ops {
        let msg = serde_json::json!({
            "type": "catchUp",
            "op": serde_json::from_str::<serde_json::Value>(op).unwrap_or_default(),
        });
        if sender
            .send(Message::Text(msg.to_string().into()))
            .await
            .is_err()
        {
            break;
        }
    }

    // Broadcast → this peer
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // This peer → validate → broadcast + record
    let rooms = state.rooms.clone();
    let tx_clone = tx.clone();
    let file_id_recv = file_id.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    let text_str = text.to_string();
                    if RoomManager::validate_op(&text_str) {
                        let _ = tx_clone.send(text_str.clone());
                        rooms.record_op(&file_id_recv, &text_str).await;
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    // Cleanup: remove editor from session, leave room
    if has_session {
        state.sessions.editor_leave(&file_id, &params.uid).await;
    }
    let room_closed = state.rooms.leave(&file_id).await;

    tracing::debug!(
        "Editor {} disconnected from {} (room {})",
        params.user,
        file_id,
        if room_closed {
            "closed"
        } else {
            "still active"
        }
    );
}
