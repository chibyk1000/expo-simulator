use anyhow::{Context, Result};
use reqwest::Client;

#[derive(Debug, Clone)]
pub struct MetroClient {
    base_url: String,
    http_client: Client,
}

impl Default for MetroClient {
    fn default() -> Self {
        Self::new("http://localhost:8081")
    }
}

impl MetroClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            http_client: Client::builder()
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub async fn is_running(&self) -> bool {
        let status_url = format!("{}/status", self.base_url);
        match self.http_client.get(&status_url).send().await {
            Ok(res) => res.status().is_success(),
            Err(_) => false,
        }
    }

    pub async fn fetch_bundle(&self, entry_file: Option<&str>) -> Result<String> {
        let entry = entry_file.unwrap_or("index");
        let bundle_url = format!(
            "{}/{}.bundle?platform=android&dev=true&minify=false",
            self.base_url, entry
        );

        log::info!("Fetching React Native bundle from {}", bundle_url);
        let res = self
            .http_client
            .get(&bundle_url)
            .send()
            .await
            .context("Failed to connect to Metro bundler")?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            anyhow::bail!("Metro returned HTTP {}: {}", status, body);
        }

        let body = res.text().await.context("Failed to read bundle text")?;
        log::info!("Fetched bundle ({} bytes)", body.len());
        Ok(body)
    }

    pub fn hmr_url(&self, entry_file: Option<&str>) -> String {
        let entry = entry_file.unwrap_or("index");
        let ws_base = if let Some(stripped) = self.base_url.strip_prefix("http://") {
            format!("ws://{}", stripped)
        } else if let Some(stripped) = self.base_url.strip_prefix("https://") {
            format!("wss://{}", stripped)
        } else {
            "ws://localhost:8081".to_string()
        };

        format!("{}/hot?bundleEntry={}&platform=android", ws_base, entry)
    }

    pub async fn listen_hmr<F>(&self, entry_file: Option<&str>, mut on_update: F) -> Result<()>
    where
        F: FnMut() + Send + 'static,
    {
        use futures_util::{SinkExt, StreamExt};
        use tokio_tungstenite::connect_async;
        use tokio_tungstenite::tungstenite::protocol::Message;

        let hmr_url = self.hmr_url(entry_file);
        let (ws_stream, _) = connect_async(&hmr_url)
            .await
            .context("Failed to connect to Metro HMR WebSocket")?;

        let (mut write, mut read) = ws_stream.split();

        let entry = entry_file.unwrap_or("index");
        let register_payload = serde_json::json!({
            "type": "register-entrypoints",
            "entrypoints": [entry],
        });
        let _ = write.send(Message::Text(register_payload.to_string().into())).await;

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                        if let Some(msg_type) = val.get("type").and_then(|v| v.as_str()) {
                            if msg_type == "update" || msg_type == "update-done" {
                                on_update();
                            }
                        }
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(e) => {
                    log::debug!("Metro HMR disconnected: {}", e);
                    break;
                }
                _ => {}
            }
        }
        Ok(())
    }
}

