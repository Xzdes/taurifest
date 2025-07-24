// src/core/manifest.rs

use serde::Deserialize;
use std::collections::HashMap;

// --- Главная структура манифеста ---
#[derive(Debug, Deserialize, Clone)]
pub struct Manifest {
    #[serde(default)] // Если поле отсутствует, используется значение по умолчанию
    pub globals: serde_json::Value,
    #[serde(default)]
    pub sockets: HashMap<String, SocketConfig>,
    pub auth: Option<AuthConfig>,
    pub connectors: HashMap<String, ConnectorConfig>,
    pub components: HashMap<String, ComponentConfig>,
    pub routes: HashMap<String, Route>,
}

// --- Структуры для каждой секции ---

#[derive(Debug, Deserialize, Clone)]
pub struct SocketConfig {
    pub watch: String,
    pub emit: EmitConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmitConfig {
    pub event: String,
    pub payload: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    #[serde(rename = "userConnector")]
    pub user_connector: String,
    #[serde(rename = "identityField")]
    pub identity_field: String,
    #[serde(rename = "passwordField")]
    pub password_field: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ConnectorConfig {
    #[serde(rename = "type")]
    pub connector_type: String,
    pub collection: Option<String>,
    #[serde(default)]
    pub initial_state: serde_json::Value,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)] // Позволяет парсить либо строку, либо объект
pub enum ComponentConfig {
    Simple(String),
    Detailed {
        template: String,
        style: Option<String>,
    },
}

#[derive(Debug, Deserialize, Clone)]
pub struct Route {
    #[serde(rename = "type")]
    pub route_type: String,
    #[serde(default)]
    pub reads: Vec<String>,
    #[serde(default)]
    pub writes: Vec<String>,
    pub update: Option<String>,
    #[serde(default)]
    pub steps: Vec<Step>,
    pub internal: Option<bool>,
    // ... другие поля роутов (layout, inject, auth)
}

#[derive(Debug, Deserialize, Clone)]
pub struct Step {
    // Используем Option, т.к. в шаге будет только одно из этих полей
    pub set: Option<String>,
    pub to: Option<String>,
    #[serde(rename = "if")]
    pub condition: Option<String>,
    pub then: Option<Vec<Step>>,
    #[serde(rename = "else")] // `else` - ключевое слово в Rust
    pub an_else: Option<Vec<Step>>,
    #[serde(rename = "action:run")]
    pub action_run: Option<ActionRunStep>,
    // ... другие типы шагов (run, http:get, auth:login, etc.)
}

#[derive(Debug, Deserialize, Clone)]
pub struct ActionRunStep {
    pub name: String,
}