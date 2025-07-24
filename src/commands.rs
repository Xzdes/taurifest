// taurifest/src/commands.rs

use tauri::{AppHandle, State};
use serde_json::Value;

use crate::AppState;
use crate::core::context::Context;
use crate::core::errors::AppError;

/// Основная команда, которая выполняет `action`-роут из манифеста.
/// Вызывается из JavaScript как `invoke('run_action', { name: '...', body: ... })`.
#[tauri::command]
pub async fn run_action(
    name: String,
    body: Value,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<Value, AppError> {

    // --- Шаг 1: Найти нужный роут в манифесте по его имени ---
    let route = state.manifest.routes.get(&name)
        .ok_or_else(|| AppError::Config(format!("Route '{}' not found in manifest", name)))?;

    // --- Шаг 2: Прочитать все необходимые данные из коннекторов (`reads`) ---
    let mut data_map = serde_json::Map::new();
    for connector_name in &route.reads {
        if let Some(connector) = state.connector_manager.get(connector_name) {
            let connector_data = connector.read().await?; 
            data_map.insert(connector_name.clone(), connector_data);
        } else {
            return Err(AppError::Config(format!("Connector '{}' in reads for action '{}' is not defined.", connector_name, name)));
        }
    }
    
    // --- Шаг 3: Создать начальный контекст выполнения ---
    let initial_context = Context::new(Value::Object(data_map), body, Value::Null); // `user` пока `Null`

    // --- Шаг 4: Запустить ActionEngine для выполнения `steps` ---
    let final_context = state.action_engine.run(
        &route.steps, 
        initial_context, 
        &state, 
        &app_handle
    ).await?;
    
    // --- Шаг 5: Сохранить измененные данные и отправить real-time события ---
    if let Some(data_object) = final_context.data.as_object() {
        for connector_name in &route.writes {
            if let (Some(connector), Some(data_to_write)) = (
                state.connector_manager.get(connector_name),
                data_object.get(connector_name)
            ) {
                // Сначала асинхронно записываем новые данные в базу
                connector.write(data_to_write).await?;
                
                // Затем, после успешной записи, уведомляем SocketManager.
                // Он проверит, нужно ли отправлять событие по этому поводу.
                state.socket_manager.notify_on_write(
                    connector_name,
                    &state,
                    &app_handle
                ).await?;
            }
        }
    }

    // --- Шаг 6: Подготовить и вернуть JSON-ответ для UI ---
    let mut response_map = serde_json::Map::new();

    if let Some(component_to_update) = &route.update {
        // Создаем контекст специально для рендеринга.
        let render_context = serde_json::json!({
            "data": final_context.data,
            "user": final_context.user,
            "globals": state.manifest.globals,
        });
        
        // Вызываем рендерер, чтобы получить новый HTML для компонента.
        let html = state.renderer.render_component(
            &state.asset_loader,
            component_to_update,
            &render_context
        ).await?;
        
        response_map.insert("html".to_string(), Value::String(html));
        
        // TODO: Добавить сюда же стили (`styles`) и другую мета-информацию.
    }
    
    // TODO: Добавить обработку `redirect`.

    // Возвращаем финальный JSON-объект, который получит JavaScript в `.then()`.
    Ok(Value::Object(response_map))
}