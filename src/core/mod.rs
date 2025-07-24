// src/core/mod.rs

// Объявляем, что внутри `core` есть модули `manifest` и `connectors`
pub mod manifest;
pub mod connectors;
pub mod errors;
pub mod context;
pub mod action_engine;
pub mod renderer;
pub mod asset_loader;
pub mod sockets;