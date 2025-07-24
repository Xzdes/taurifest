// taurifest/src/core/errors.rs

use thiserror::Error;

/// Ошибки, связанные с работой коннекторов данных.
#[derive(Error, Debug)]
pub enum ConnectorError {
    #[error("Connector with name '{0}' not found")]
    NotFound(String),

    #[error("Failed to initialize connection pool for SQLite: {0}")]
    PoolInitialization(String),

    #[error("Failed to get connection from SQLite pool: {0}")]
    PoolConnection(String),

    #[error("SQLite database error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("JSON serialization or deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Unsupported connector type specified in manifest: '{0}'")]
    UnsupportedType(String),
}

/// Ошибки, возникающие во время выполнения логики в `ActionEngine`.
#[derive(Error, Debug)]
pub enum ActionError {
    #[error("Rhai script evaluation error: {0}")]
    Rhai(String),

    #[error("Path '{0}' is not mutable. Only 'context.*' can be changed via 'set'.")]
    NotMutable(String),

    #[error("Invalid or empty path provided for 'set' step: '{0}'")]
    InvalidSetPath(String),
}

/// Ошибки, возникающие во время рендеринга HTML-компонентов.
#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Component asset '{0}' not found in asset loader cache.")]
    AssetNotFound(String),

    #[error("Mustache template compilation failed: {0}")]
    MustacheCompile(#[from] mustache::Error),

    #[error("Rhai script evaluation error during directive processing: {0}")]
    Rhai(String),
}

/// `AppError` - это "зонтичный" тип ошибки, который объединяет все возможные
/// ошибки нашего движка. Это позволяет нашим `tauri::command` функциям
/// возвращать единый, унифицированный тип `Result<T, AppError>`.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Connector Error: {0}")]
    Connector(#[from] ConnectorError),

    #[error("Action Engine Error: {0}")]
    Action(#[from] ActionError),

    #[error("Renderer Error: {0}")]
    Render(#[from] RenderError),

    #[error("Configuration Error in manifest.json: {0}")]
    Config(String),
}

// Реализуем `serde::Serialize` для `AppError`.
// Это критически важно, чтобы Tauri мог корректно сериализовать нашу ошибку
// в JSON и отправить ее в JavaScript-фронтенд в случае `Err`.
impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        // Мы просто конвертируем ошибку в строку.
        // На фронтенде мы получим `invoke().catch(error => console.error(error))`,
        // где `error` будет этой строкой.
        serializer.serialize_str(&self.to_string())
    }
}