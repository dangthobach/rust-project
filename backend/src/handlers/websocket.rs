use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
};
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info, warn};

use crate::app_state::AppState;

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    Auth {
        data: AuthData,
    },
    Heartbeat {
        data: HeartbeatData,
    },
    Notification {
        data: NotificationData,
    },
    Status {
        data: StatusData,
    },
    Error {
        data: ErrorData,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthData {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatData {
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationData {
    pub id: String,
    pub title: String,
    pub message: String,
    #[serde(rename = "type")]
    pub notification_type: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusData {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorData {
    pub message: String,
}

/// WebSocket connection manager
#[derive(Clone)]
pub struct WsConnectionManager {
    connections: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Message>>>>,
}

impl WsConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a new connection
    pub async fn add_connection(&self, user_id: String, sender: mpsc::UnboundedSender<Message>) {
        let mut connections = self.connections.write().await;
        connections.insert(user_id.clone(), sender);
        info!("WebSocket connection added for user: {}", user_id);
    }

    /// Remove a connection
    pub async fn remove_connection(&self, user_id: &str) {
        let mut connections = self.connections.write().await;
        connections.remove(user_id);
        info!("WebSocket connection removed for user: {}", user_id);
    }

    /// Send message to specific user
    pub async fn send_to_user(&self, user_id: &str, message: WsMessage) -> Result<(), String> {
        let connections = self.connections.read().await;
        
        if let Some(sender) = connections.get(user_id) {
            let json = serde_json::to_string(&message)
                .map_err(|e| format!("Failed to serialize message: {}", e))?;
            
            sender.send(Message::Text(json))
                .map_err(|e| format!("Failed to send message: {}", e))?;
            
            Ok(())
        } else {
            Err(format!("User {} not connected", user_id))
        }
    }

    /// Broadcast message to all connected users
    pub async fn broadcast(&self, message: WsMessage) {
        let connections = self.connections.read().await;
        let json = match serde_json::to_string(&message) {
            Ok(j) => j,
            Err(e) => {
                error!("Failed to serialize broadcast message: {}", e);
                return;
            }
        };

        for (user_id, sender) in connections.iter() {
            if let Err(e) = sender.send(Message::Text(json.clone())) {
                warn!("Failed to send message to user {}: {}", user_id, e);
            }
        }

        info!("Broadcasted message to {} users", connections.len());
    }

    /// Get number of connected users
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }
}

/// WebSocket upgrade handler
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    let manager = state.ws_manager().clone();
    ws.on_upgrade(move |socket| handle_socket(socket, manager))
}

/// Handle individual WebSocket connection
async fn handle_socket(socket: WebSocket, manager: WsConnectionManager) {
    let (sender, receiver) = socket.split();
    
    // Create channel for sending messages to this client
    let (tx, rx) = mpsc::unbounded_channel();
    
    // Spawn task to handle outgoing messages
    let mut send_task = tokio::spawn(handle_outgoing(rx, sender));
    
    // Handle incoming messages
    let mut recv_task = tokio::spawn(handle_incoming(receiver, manager.clone(), tx.clone()));
    
    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => {
            recv_task.abort();
        }
        _ = &mut recv_task => {
            send_task.abort();
        }
    }
    
    info!("WebSocket connection closed");
}

/// Handle incoming messages from client
async fn handle_incoming(
    mut receiver: SplitStream<WebSocket>,
    manager: WsConnectionManager,
    sender: mpsc::UnboundedSender<Message>,
) {
    let mut user_id: Option<String> = None;
    let mut authenticated = false;

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Parse incoming message
                match serde_json::from_str::<WsMessage>(&text) {
                    Ok(ws_msg) => {
                        match ws_msg {
                            WsMessage::Auth { data: _data } => {
                                // TODO: Validate token
                                // For now, extract user_id from token (simplified)
                                let extracted_user_id = "user_123".to_string(); // TODO: Real JWT validation
                                
                                user_id = Some(extracted_user_id.clone());
                                authenticated = true;
                                
                                // Register connection
                                manager.add_connection(extracted_user_id.clone(), sender.clone()).await;
                                
                                // Send success response
                                let response = WsMessage::Status {
                                    data: StatusData {
                                        message: "Authenticated successfully".to_string(),
                                    },
                                };
                                
                                if let Ok(json) = serde_json::to_string(&response) {
                                    let _ = sender.send(Message::Text(json));
                                }
                                
                                info!("User {} authenticated via WebSocket", extracted_user_id);
                            }
                            WsMessage::Heartbeat { data: _data } => {
                                // Respond to heartbeat
                                let response = WsMessage::Heartbeat {
                                    data: HeartbeatData {
                                        timestamp: chrono::Utc::now().timestamp(),
                                    },
                                };
                                
                                if let Ok(json) = serde_json::to_string(&response) {
                                    let _ = sender.send(Message::Text(json));
                                }
                            }
                            _ => {
                                if !authenticated {
                                    let error = WsMessage::Error {
                                        data: ErrorData {
                                            message: "Not authenticated".to_string(),
                                        },
                                    };
                                    
                                    if let Ok(json) = serde_json::to_string(&error) {
                                        let _ = sender.send(Message::Text(json));
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse WebSocket message: {}", e);
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("Client requested close");
                break;
            }
            Ok(Message::Ping(data)) => {
                let _ = sender.send(Message::Pong(data));
            }
            Ok(Message::Pong(_)) => {
                // Pong received
            }
            Ok(Message::Binary(_)) => {
                warn!("Binary messages not supported");
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    // Clean up connection
    if let Some(uid) = user_id {
        manager.remove_connection(&uid).await;
    }
}

/// Handle outgoing messages to client
async fn handle_outgoing(
    mut rx: mpsc::UnboundedReceiver<Message>,
    mut sender: SplitSink<WebSocket, Message>,
) {
    while let Some(msg) = rx.recv().await {
        if sender.send(msg).await.is_err() {
            error!("Failed to send WebSocket message");
            break;
        }
    }
}

/// Helper function to send notification to user
pub async fn send_notification(
    manager: &WsConnectionManager,
    user_id: &str,
    notification: NotificationData,
) -> Result<(), String> {
    let message = WsMessage::Notification {
        data: notification,
    };
    
    manager.send_to_user(user_id, message).await
}

/// Helper function to broadcast notification to all users
pub async fn broadcast_notification(
    manager: &WsConnectionManager,
    notification: NotificationData,
) {
    let message = WsMessage::Notification {
        data: notification,
    };
    
    manager.broadcast(message).await;
}
