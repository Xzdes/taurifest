// taurifest/src/core/asset_loader.rs

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf}; // Удалили неиспользуемый PathBuf
use crate::core::manifest::{ComponentConfig, Manifest};

/// Структура для хранения загруженного и готового к использованию компонента.
#[derive(Clone, Debug)]
pub struct ComponentAsset {
    pub template: String,
    pub style: Option<String>,
}

/// `AssetLoader` - это сервис, который при старте приложения загружает
/// все необходимые файлы в память.
pub struct AssetLoader {
    components: HashMap<String, ComponentAsset>,
    // В будущем здесь может быть кэш для `run` скриптов и т.д.
}

impl AssetLoader {
    /// Создает новый экземпляр `AssetLoader`, сканируя папку приложения
    /// на основе конфигурации из `manifest`.
    pub fn new(app_path: &Path, manifest: &Manifest) -> Result<Self, String> {
        let mut components = HashMap::new();
        let components_path = app_path.join("components");

        for (name, config) in &manifest.components {
            // Определяем пути к файлам в зависимости от формата конфигурации компонента
            let (template_path, style_path): (PathBuf, Option<PathBuf>) = match config {
                // Простой формат: "componentName": "template.html"
                ComponentConfig::Simple(template_file) => {
                    (components_path.join(template_file), None)
                },
                // Расширенный формат: "componentName": { "template": "...", "style": "..." }
                ComponentConfig::Detailed { template, style } => {
                    (
                        components_path.join(template),
                        style.as_ref().map(|style_file| components_path.join(style_file))
                    )
                }
            };

            // Читаем файл шаблона. Если не удалось, возвращаем ошибку.
            let template = fs::read_to_string(&template_path)
                .map_err(|e| format!("Failed to read template for component '{}' at {:?}: {}", name, template_path, e))?;
            
            // Читаем файл стилей, если он указан.
            let style = match style_path {
                Some(path) => Some(fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read style for component '{}' at {:?}: {}", name, path, e))?),
                None => None,
            };

            components.insert(name.clone(), ComponentAsset { template, style });
        }
        
        Ok(Self { components })
    }

    /// Возвращает ссылку на закэшированный компонент по его имени.
    pub fn get_component(&self, name: &str) -> Option<&ComponentAsset> {
        self.components.get(name)
    }
}