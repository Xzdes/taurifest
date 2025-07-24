// src/core/context.rs
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Context {
    /// Данные, прочитанные из коннекторов (`reads`)
    pub data: Value,
    /// Данные, пришедшие с клиента (из формы)
    pub body: Value,
    /// Объект текущего пользователя (если есть)
    pub user: Value,
    /// Временное хранилище для промежуточных вычислений
    pub temp: Value,
}

impl Context {
    pub fn new(data: Value, body: Value, user: Value) -> Self {
        Self {
            data,
            body,
            user,
            temp: Value::Object(Default::default()), // Начинаем с пустого объекта
        }
    }
}