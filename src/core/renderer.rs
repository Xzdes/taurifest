// taurifest/src/core/renderer.rs - УПРОЩЕННАЯ И РАБОЧАЯ ВЕРСИЯ

use rhai::{Engine, Scope};
use scraper::Html;
use serde_json::Value;

use crate::core::asset_loader::AssetLoader;
use crate::core::errors::RenderError;

pub struct Renderer {
    rhai_engine: Engine,
}

impl Renderer {
    pub fn new() -> Self {
        let mut engine = Engine::new();
        engine.set_max_operations(100_000);
        Self { rhai_engine: engine }
    }

    pub async fn render_component(
        &self,
        asset_loader: &AssetLoader,
        component_name: &str,
        context: &Value,
    ) -> Result<String, RenderError> {
        let component_asset = asset_loader.get_component(component_name)
            .ok_or_else(|| RenderError::AssetNotFound(component_name.to_string()))?;

        let template = mustache::compile_str(&component_asset.template)?;
        
        let mustache_data = mustache::to_data(context)
            .expect("Internal Error: Failed to convert context Value to mustache::Data.");

        let mut rendered_html_bytes = Vec::new();
        template.render_data(&mut rendered_html_bytes, &mustache_data)?;
        
        let html_string = String::from_utf8(rendered_html_bytes)
            .unwrap_or_else(|_| "Error: Template produced invalid UTF-8".to_string());

        let mut document = Html::parse_fragment(&html_string);
        let mut scope = Scope::new();
        scope.push_constant("data", context.get("data").cloned().unwrap_or(Value::Null));
        scope.push_constant("user", context.get("user").cloned().unwrap_or(Value::Null));
        scope.push_constant("globals", context.get("globals").cloned().unwrap_or(Value::Null));

        let mut nodes_to_remove = Vec::new();
        let selector = scraper::Selector::parse("[atom-if]").unwrap();

        // --- ИЗМЕНЕНИЕ: Просто собираем ID узлов для удаления ---
        for element_ref in document.select(&selector) {
            if let Some(condition) = element_ref.value().attr("atom-if") {
                let should_render = self.rhai_engine.eval_with_scope::<bool>(&mut scope, condition)
                    .unwrap_or(false);

                if !should_render {
                    nodes_to_remove.push(element_ref.id());
                }
                // Мы больше не пытаемся удалить атрибут `atom-if` у тех, кто остается.
                // Это компромисс ради работающего кода.
            }
        }

        // Удаляем все отмеченные узлы
        for node_id in nodes_to_remove {
            if let Some(mut node_to_remove) = document.tree.get_mut(node_id) {
                node_to_remove.detach();
            }
        }

        Ok(document.html())
    }
}