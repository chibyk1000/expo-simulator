use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsValue {
    Null,
    Undefined,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<JsValue>),
    Object(serde_json::Map<String, serde_json::Value>),
}

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("JavaScript evaluation error: {0}")]
    EvalError(String),

    #[error("Function not found: {0}")]
    FunctionNotFound(String),

    #[error("Internal engine error: {0}")]
    Internal(String),
}

pub type HostCallback = Box<dyn Fn(&[JsValue]) -> Result<JsValue, String> + Send>;

pub trait JsEngine: 'static {
    fn name(&self) -> &'static str;
    fn eval(&mut self, code: &str, source_url: &str) -> Result<JsValue, EngineError>;
    fn call_global(&mut self, func: &str, args: &[JsValue]) -> Result<JsValue, EngineError>;
    fn register_host_fn(&mut self, name: &str, callback: HostCallback) -> Result<(), EngineError>;
}
