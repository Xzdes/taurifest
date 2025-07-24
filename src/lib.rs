// taurifest/src/lib.rs

use std::path::PathBuf;
use std::sync::Arc;

// Объявляем наши основные модули.
pub mod core;
mod commands;

// Импортируем все необходимые структуры из наших модулей.
use crate::core::manifest::Manifest;
use crate::core::connectors::ConnectorManager;
use crate::core::action_engine::ActionEngine;
use crate::core::renderer::Renderer;
use crate::core::asset_loader::AssetLoader;
use crate::core::sockets::SocketManager;
use crate::commands::run_action;

// Делаем `Builder` публичным, чтобы пользователи могли его импортировать
// из нашего крейта (`use taurifest::Builder;`).
pub use self::builder::Builder;

/// `AppState` - это центральное хранилище состояния нашего приложения.
/// Tauri будет владеть этим состоянием и предоставлять к нему безопасный доступ
/// из асинхронных команд. Все "сервисы" (менеджеры, движки) хранятся здесь.
/// Мы используем `Arc` (Atomic Reference Counting) для безопасного совместного
/// владения этими сервисами из разных потоков, что является требованием Tauri.
pub struct AppState {
    pub manifest: Manifest,
    pub connector_manager: Arc<ConnectorManager>,
    pub action_engine: Arc<ActionEngine>,
    pub renderer: Arc<Renderer>,
    pub asset_loader: Arc<AssetLoader>,
    pub socket_manager: Arc<SocketManager>,
}

/// Внутренний модуль `builder` для инкапсуляции логики создания движка.
mod builder {
    // Импортируем все из родительского модуля (`lib.rs`), чтобы иметь доступ
    // к `AppState` и другим необходимым структурам.
    use super::*;

    /// `Builder` - это основной способ инициализации движка `Taurifest`.
    /// Он использует паттерн "Строитель" для удобной и понятной настройки.
    pub struct Builder {
        app_path: PathBuf,
    }

    impl Builder {
        /// Создает новый экземпляр строителя.
        ///
        /// # Arguments
        ///
        /// * `app_path` - Путь к корневой папке приложения пользователя (например, "app" или "ui"),
        ///   внутри которой находится `manifest.json`, а также папки `components`, `actions` и `data`.
        pub fn new(app_path: impl Into<PathBuf>) -> Self {
            Self { app_path: app_path.into() }
        }

        /// Финальный метод, который выполняет всю работу по настройке:
        /// 1. Читает и парсит `manifest.json`.
        /// 2. Инициализирует все сервисы.
        /// 3. Собирает их в `AppState`.
        /// 4. Возвращает `tauri::Builder`, готовый к запуску, с уже настроенным состоянием и командами.
        pub fn build(self) -> tauri::Builder<tauri::Wry> {
            // --- Шаг 1: Загрузка и парсинг манифеста ---
            let manifest_path = self.app_path.join("manifest.json");
            let manifest_content = std::fs::read_to_string(&manifest_path)
                .unwrap_or_else(|error| panic!("FATAL: Failed to read manifest.json from {:?}. Error: {}", manifest_path, error));
            let manifest: Manifest = serde_json::from_str(&manifest_content)
                .unwrap_or_else(|error| panic!("FATAL: Failed to parse manifest.json. Error: {}", error));

            // --- Шаг 2: Инициализация всех сервисов движка ---

            // Загрузчик ассетов (HTML шаблонов, CSS файлов)
            let asset_loader = AssetLoader::new(&self.app_path, &manifest)
                .expect("FATAL: Failed to load application assets");

            // Менеджер коннекторов (баз данных)
            let data_path = self.app_path.join("data");
            let connector_manager = ConnectorManager::new(&manifest.connectors, data_path)
                .expect("FATAL: Failed to initialize connectors");
            
            // Движок выполнения логики `steps`
            let action_engine = ActionEngine::new();
            
            // Рендерер HTML
            let renderer = Renderer::new();

            // Менеджер real-time событий
            let socket_manager = SocketManager::new();

            // --- Шаг 3: Сборка глобального состояния `AppState` ---
            // Оборачиваем все сервисы в `Arc`, чтобы Tauri мог безопасно
            // передавать ссылки на них между потоками.
            let state = AppState {
                manifest,
                connector_manager: Arc::new(connector_manager),
                action_engine: Arc::new(action_engine),
                renderer: Arc::new(renderer),
                asset_loader: Arc::new(asset_loader),
                socket_manager: Arc::new(socket_manager),
            };

            // --- Шаг 4: Конфигурирование и возврат строителя Tauri ---
            tauri::Builder::default()
                .manage(state) // Передаем наше состояние под управление Tauri
                .invoke_handler(tauri::generate_handler![
                    run_action,
                    // Здесь будут регистрироваться другие команды
                ])
        }
    }
}