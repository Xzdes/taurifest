// src/core/connectors.rs

use async_trait::async_trait;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::core::errors::ConnectorError;
use crate::core::manifest::ConnectorConfig;

type Pool = r2d2::Pool<SqliteConnectionManager>;

/// Единый интерфейс для всех типов коннекторов.
#[async_trait]
pub trait DataConnector: Send + Sync {
    async fn read(&self) -> Result<Value, ConnectorError>;
    async fn write(&self, data: &Value) -> Result<(), ConnectorError>;
    // async fn migrate(&self) -> Result<(), ConnectorError>; // Понадобится позже
}

/// Коннектор, использующий SQLite для хранения данных.
struct SqliteConnector {
    pool: Pool,
}

#[async_trait]
impl DataConnector for SqliteConnector {
    async fn read(&self) -> Result<Value, ConnectorError> {
        let conn = self.pool.get().map_err(|e| ConnectorError::PoolConnection(e.to_string()))?;

        // wise-json-db хранит "items" и "meta" в разных местах.
        // Мы эмулируем это поведение с двумя таблицами.
        let items: Vec<Value> = {
            let mut stmt = conn.prepare("SELECT data FROM items")?;
            let rows = stmt.query_map([], |row| {
                let json_str: String = row.get(0)?;
                serde_json::from_str(&json_str).map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))
            })?;
            
            rows.collect::<Result<Vec<_>, _>>()?
        };

        let meta: Value = {
             let mut stmt = conn.prepare("SELECT data FROM meta WHERE id = 1")?;
             stmt.query_row([], |row| {
                let json_str: String = row.get(0)?;
                serde_json::from_str(&json_str).map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))
             }).unwrap_or(Value::Object(serde_json::Map::new()))
        };
        
        // Собираем финальный объект, совместимый с Serverokey
        let mut result = meta;
        if let Some(obj) = result.as_object_mut() {
            obj.insert("items".to_string(), Value::Array(items));
        }

        Ok(result)
    }

    async fn write(&self, data: &Value) -> Result<(), ConnectorError> {
        let mut conn = self.pool.get().map_err(|e| ConnectorError::PoolConnection(e.to_string()))?;

        let tx = conn.transaction()?;

        tx.execute("DELETE FROM items", [])?;
        tx.execute("DELETE FROM meta", [])?;

        if let Some(items) = data.get("items").and_then(|i| i.as_array()) {
            for item in items {
                let json_str = serde_json::to_string(item)?;
                tx.execute("INSERT INTO items (data) VALUES (?)", params![json_str])?;
            }
        }

        let mut meta = data.clone();
        if let Some(obj) = meta.as_object_mut() {
            obj.remove("items");
        }
        
        let meta_str = serde_json::to_string(&meta)?;
        tx.execute("INSERT INTO meta (id, data) VALUES (1, ?)", params![meta_str])?;
        
        tx.commit()?;
        Ok(())
    }
}

/// Менеджер, который управляет всеми коннекторами.
pub struct ConnectorManager {
    connectors: HashMap<String, Arc<dyn DataConnector>>,
}

impl ConnectorManager {
    pub fn new(configs: &HashMap<String, ConnectorConfig>, data_path: PathBuf) -> Result<Self, ConnectorError> {
        // Создаем папку для данных, если ее нет
        std::fs::create_dir_all(&data_path).expect("Failed to create data directory");

        let mut connectors = HashMap::new();
        for (name, config) in configs {
            match config.connector_type.as_str() {
                "sqlite" => {
                    let collection_name = config.collection.as_deref().unwrap_or(name);
                    let db_file = data_path.join(format!("{}.db", collection_name));
                    
                    let manager = SqliteConnectionManager::file(&db_file);
                    let pool = Pool::new(manager)
                        .map_err(|e| ConnectorError::PoolInitialization(e.to_string()))?;
                    
                    // Создаем таблицы при первом запуске
                    let conn = pool.get().map_err(|e| ConnectorError::PoolConnection(e.to_string()))?;
                    conn.execute_batch(
                        "BEGIN;
                         CREATE TABLE IF NOT EXISTS items (data TEXT NOT NULL);
                         CREATE TABLE IF NOT EXISTS meta (id INTEGER PRIMARY KEY, data TEXT NOT NULL);
                         COMMIT;"
                    )?;

                    connectors.insert(name.clone(), Arc::new(SqliteConnector { pool }) as Arc<dyn DataConnector>);
                }
                "in-memory" => {
                    // Реализация InMemoryConnector будет следующим шагом
                }
                _ => return Err(ConnectorError::UnsupportedType(config.connector_type.clone())),
            }
        }
        Ok(Self { connectors })
    }

    pub fn get(&self, name: &str) -> Option<&Arc<dyn DataConnector>> {
        self.connectors.get(name)
    }
}