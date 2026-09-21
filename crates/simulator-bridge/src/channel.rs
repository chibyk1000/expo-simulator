use crate::protocol::{BridgeRequest, BridgeResponse};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

#[derive(Clone)]
pub struct BridgeChannel {
    to_host_tx: mpsc::UnboundedSender<BridgeRequest>,
    to_host_rx: Arc<Mutex<mpsc::UnboundedReceiver<BridgeRequest>>>,
    to_js_tx: mpsc::UnboundedSender<BridgeResponse>,
    to_js_rx: Arc<Mutex<mpsc::UnboundedReceiver<BridgeResponse>>>,
}

impl Default for BridgeChannel {
    fn default() -> Self {
        Self::new()
    }
}

impl BridgeChannel {
    pub fn new() -> Self {
        let (to_host_tx, to_host_rx) = mpsc::unbounded_channel();
        let (to_js_tx, to_js_rx) = mpsc::unbounded_channel();
        Self {
            to_host_tx,
            to_host_rx: Arc::new(Mutex::new(to_host_rx)),
            to_js_tx,
            to_js_rx: Arc::new(Mutex::new(to_js_rx)),
        }
    }

    pub fn send_to_host(&self, req: BridgeRequest) -> anyhow::Result<()> {
        self.to_host_tx.send(req)?;
        Ok(())
    }

    pub fn send_to_js(&self, res: BridgeResponse) -> anyhow::Result<()> {
        self.to_js_tx.send(res)?;
        Ok(())
    }

    pub async fn recv_from_js(&self) -> Option<BridgeRequest> {
        let mut rx = self.to_host_rx.lock().await;
        rx.recv().await
    }

    pub async fn recv_from_host(&self) -> Option<BridgeResponse> {
        let mut rx = self.to_js_rx.lock().await;
        rx.recv().await
    }

    pub fn try_recv_from_js(&self) -> Option<BridgeRequest> {
        if let Ok(mut rx) = self.to_host_rx.try_lock() {
            rx.try_recv().ok()
        } else {
            None
        }
    }
}
