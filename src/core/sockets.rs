// taurifest/src/core/sockets.rs - ПОЛНАЯ ИСПРАВЛЕННАЯ ВЕРСИЯ

use serde_json::Value;
use tauri::{AppHandle, Emitter}; // <-- ИСПРАВЛЕНИЕ 1: Manager -> Emitter

use crate::core::errors::AppError;
use crate::AppState;

pub struct SocketManager;

impl SocketManager {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn notify_on_write(
        &self,
        connector_name: &str,
        state: &AppState,
        app_handle: &AppHandle,
    ) -> Result<(), AppError> {
        // --- ИСПРАВЛЕНИЕ 2: Убираем `if let Some`, так как `sockets` - это HashMap, а не Option ---
        // `#[serde(default)]` гарантирует, что `sockets` всегда будет как минимум пустым HashMap.
        // Поэтому мы можем итерировать по нему напрямую.
        for (_channel_name, socket_config) in &state.manifest.sockets {
            if socket_config.watch == connector_name {
                let event_name = &socket_config.emit.event;
                let payload_source_name = &socket_config.emit.payload;

                let payload = if let Some(connector) = state.connector_manager.get(payload_source_name) {
                    connector.read().await?
                } else {
                    Value::Null
                };
                
                println!("[SocketManager] Emitting event '{}' due to write on connector '{}'", event_name, connector_name);
                
                // Теперь этот вызов корректен, так как трейт `Emitter` в области видимости
                app_handle.emit(event_name, payload)
                    .map_err(|e| AppError::Config(format!("Tauri event emit failed: {}", e)))?;
            }
        }

        Ok(())
    }
}