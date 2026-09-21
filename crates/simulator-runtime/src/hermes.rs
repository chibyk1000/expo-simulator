use crate::engine::{EngineError, HostCallback, JsEngine, JsValue};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct HermesEngine {
    hermes_bin: PathBuf,
}

impl HermesEngine {
    pub fn new() -> Result<Self, EngineError> {
        let hermes_bin = Self::find_hermes_binary()
            .ok_or_else(|| EngineError::Internal("Hermes binary not found".to_string()))?;
        Ok(Self { hermes_bin })
    }

    pub fn with_path<P: AsRef<Path>>(path: P) -> Self {
        Self {
            hermes_bin: path.as_ref().to_path_buf(),
        }
    }

    pub fn find_hermes_binary() -> Option<PathBuf> {
        let candidates = [
            PathBuf::from("/tmp/hermes-test/hermes"),
            PathBuf::from("/usr/local/bin/hermes"),
            PathBuf::from("/usr/bin/hermes"),
        ];

        for c in &candidates {
            if c.exists() {
                return Some(c.clone());
            }
        }

        // Try which hermes
        if let Ok(output) = Command::new("which").arg("hermes").output() {
            if output.status.success() {
                let p = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !p.is_empty() {
                    return Some(PathBuf::from(p));
                }
            }
        }

        None
    }
}

impl JsEngine for HermesEngine {
    fn name(&self) -> &'static str {
        "Hermes"
    }

    fn eval(&mut self, code: &str, _source_url: &str) -> Result<JsValue, EngineError> {
        let temp_file = std::env::temp_dir().join(format!("expo_sim_{}.js", std::process::id()));
        std::fs::write(&temp_file, code)
            .map_err(|e| EngineError::Internal(format!("Failed to write temp JS: {e}")))?;

        let output = Command::new(&self.hermes_bin)
            .arg(&temp_file)
            .output()
            .map_err(|e| EngineError::EvalError(format!("Failed to run hermes: {e}")))?;

        let _ = std::fs::remove_file(temp_file);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(EngineError::EvalError(format!("{stderr}\n{stdout}")));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(JsValue::String(stdout))
    }

    fn call_global(&mut self, _func: &str, _args: &[JsValue]) -> Result<JsValue, EngineError> {
        Ok(JsValue::Undefined)
    }

    fn register_host_fn(&mut self, _name: &str, _callback: HostCallback) -> Result<(), EngineError> {
        Ok(())
    }
}
