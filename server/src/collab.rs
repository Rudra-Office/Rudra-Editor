//! Real-time collaboration via WebSocket.
//!
//! Each document gets a "room". Peers join the room and broadcast operations
//! to each other. The server acts as a relay + optional authoritative CRDT.

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

use crate::routes::AppState;

/// A collaboration room for a document.
struct Room {
    /// Broadcast channel for operations.
    tx: broadcast::Sender<String>,
    /// Connected peer count.
    peer_count: usize,
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

    /// Get or create a room for a document.
    async fn get_or_create(&self, room_id: &str) -> broadcast::Sender<String> {
        let mut rooms = self.rooms.lock().await;
        if let Some(room) = rooms.get_mut(room_id) {
            room.peer_count += 1;
            room.tx.clone()
        } else {
            let (tx, _) = broadcast::channel(256);
            rooms.insert(
                room_id.to_string(),
                Room {
                    tx: tx.clone(),
                    peer_count: 1,
                },
            );
            tracing::info!("Room created: {}", room_id);
            tx
        }
    }

    /// Remove a peer from a room. Closes the room if empty.
    async fn leave(&self, room_id: &str) {
        let mut rooms = self.rooms.lock().await;
        if let Some(room) = rooms.get_mut(room_id) {
            room.peer_count = room.peer_count.saturating_sub(1);
            if room.peer_count == 0 {
                rooms.remove(room_id);
                tracing::info!("Room closed: {}", room_id);
            }
        }
    }

    /// Get the number of active rooms.
    #[allow(dead_code)]
    pub async fn room_count(&self) -> usize {
        self.rooms.lock().await.len()
    }
}

/// WebSocket upgrade handler for collaboration.
///
/// Route: `GET /ws/collab/:room_id`
pub async fn ws_collab_handler(
    ws: WebSocketUpgrade,
    Path(room_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_collab_socket(socket, room_id, state))
}

/// Handle a single WebSocket connection in a collaboration room.
async fn handle_collab_socket(socket: WebSocket, room_id: String, state: Arc<AppState>) {
    let tx = state.rooms.get_or_create(&room_id).await;
    let mut rx = tx.subscribe();

    let (mut sender, mut receiver) = socket.split();

    // Send welcome message
    let welcome = serde_json::json!({
        "type": "welcome",
        "roomId": room_id,
    });
    let _ = sender
        .send(Message::Text(welcome.to_string().into()))
        .await;

    // Spawn task to forward broadcast messages to this peer
    let _room_id_clone = room_id.clone();
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Receive messages from this peer and broadcast to room
    let tx_clone = tx.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    // Broadcast to all peers in the room
                    let _ = tx_clone.send(text.to_string());
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    // Cleanup
    state.rooms.leave(&room_id).await;
    tracing::debug!("Peer disconnected from room: {}", room_id);
}
