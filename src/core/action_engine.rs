// taurifest/src/core/action_engine.rs - ИСПРАВЛЕННАЯ ВЕРСИЯ С BOX::PIN

use rhai::{Engine, Scope, Dynamic};
use serde_json::Value;
use tauri::AppHandle;
use futures::future::BoxFuture; // <-- СНОВА НУЖЕН ЭТОТ ИМПОРТ

use crate::core::context::Context;
use crate::core::manifest::Step; // <-- Убрали ActionRunStep, он внутри Step
use crate::core::errors::ActionError;
use crate::AppState;

pub struct ActionEngine {
    engine: Engine,
}

impl ActionEngine {
    pub fn new() -> Self {
        let mut engine = Engine::new();
        engine.set_max_operations(1_000_000);
        engine.set_max_call_levels(100);
        engine.set_max_string_size(1024 * 1024);
        Self { engine }
    }

    // --- ИЗМЕНЕНИЕ: Сигнатура теперь возвращает BoxFuture ---
    // Мы также добавляем 's (static lifetime) для безопасности,
    // так как наш Future будет жить в многопоточной среде Tauri.
    pub fn run<'s>(
        &'s self,
        steps: &'s [Step],
        mut context: Context,
        state: &'s AppState,
        app_handle: &'s AppHandle,
    ) -> BoxFuture<'s, Result<Context, ActionError>> {
        // --- ИЗМЕНЕНИЕ: Оборачиваем всю логику в Box::pin ---
        Box::pin(async move {
            let mut scope = Scope::new();
            scope.push_constant("data", context.data.clone());
            scope.push_constant("body", context.body.clone());
            scope.push_constant("user", context.user.clone());
            scope.push("context", context.temp.clone());

            let mut steps_to_process = steps.iter().rev().collect::<Vec<_>>();

            while let Some(step) = steps_to_process.pop() {
                if let Some(action_run_config) = &step.action_run {
                    let sub_route_name = &action_run_config.name;
                    
                    if let Some(sub_route) = state.manifest.routes.get(sub_route_name) {
                        println!("[ActionEngine] Running sub-action: '{}'", sub_route_name);
                        
                        let sub_context = Context::new(
                            context.data.clone(),
                            context.body.clone(),
                            context.user.clone()
                        );
                        
                        // Рекурсивный вызов теперь внутри `Box::pin`, все легально
                        let result_context = self.run(&sub_route.steps, sub_context, state, app_handle).await?;
                        
                        context.data = result_context.data;
                        context.temp = result_context.temp;
                        
                        scope = Scope::new();
                        scope.push_constant("data", context.data.clone());
                        scope.push_constant("body", context.body.clone());
                        scope.push_constant("user", context.user.clone());
                        scope.push("context", context.temp.clone());
                    } else {
                        return Err(ActionError::InvalidSetPath(format!("Sub-action '{}' not found", sub_route_name)));
                    }
                    continue;
                }

                match self.execute_step_sync(step, &mut scope) {
                    Ok(Some(next_steps)) => {
                        steps_to_process.extend(next_steps.iter().rev());
                    }
                    Ok(None) => {},
                    Err(e) => return Err(e),
                }
            }
            
            context.temp = scope.get_value("context").unwrap();
            Ok(context)
        })
    }
    
    fn execute_step_sync<'a>(
        &self,
        step: &'a Step,
        scope: &mut Scope<'a>
    ) -> Result<Option<&'a Vec<Step>>, ActionError> {
        if let Some(path) = &step.set {
            if let Some(expr) = &step.to {
                let result = self.engine.eval_with_scope::<Dynamic>(scope, expr)
                    .map_err(|e| ActionError::Rhai(e.to_string()))?;
                let json_result = serde_json::to_value(result).unwrap_or(Value::Null);
                set_value_by_path(scope, path, json_result)?;
            }
        } else if let Some(condition) = &step.condition {
            let result = self.engine.eval_with_scope::<bool>(scope, condition).unwrap_or(false);
            if result {
                return Ok(step.then.as_ref());
            } else {
                return Ok(step.an_else.as_ref());
            }
        }
        
        Ok(None)
    }
}

fn set_value_by_path(scope: &mut Scope, full_path: &str, value: Value) -> Result<(), ActionError> {
    let mut parts = full_path.split('.').peekable();
    let root_name = parts.next().ok_or_else(|| ActionError::InvalidSetPath(full_path.to_string()))?;
    if root_name != "context" {
        return Err(ActionError::NotMutable(full_path.to_string()));
    }
    if let Some(root_val) = scope.get_value_mut::<Value>(root_name) {
        let mut current = root_val;
        while let Some(part) = parts.next() {
            if parts.peek().is_none() {
                if let Some(obj) = current.as_object_mut() {
                    obj.insert(part.to_string(), value);
                }
                break;
            } else {
                if !current.is_object() {
                    *current = Value::Object(Default::default());
                }
                current = current.as_object_mut().unwrap().entry(part).or_insert(Value::Object(Default::default()));
            }
        }
    }
    Ok(())
}