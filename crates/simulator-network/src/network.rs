use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkCondition {
    Online,
    Offline,
    Slow3G,
    Fast3G,
    FourG,
    FiveG,
    WiFi,
}

impl Default for NetworkCondition {
    fn default() -> Self {
        Self::Online
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkManager {
    condition: NetworkCondition,
    simulated_latency_ms: u32,
}

impl Default for NetworkManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkManager {
    pub fn new() -> Self {
        Self {
            condition: NetworkCondition::Online,
            simulated_latency_ms: 0,
        }
    }

    pub fn condition(&self) -> NetworkCondition {
        self.condition
    }

    pub fn set_condition(&mut self, condition: NetworkCondition) {
        self.condition = condition;
        self.simulated_latency_ms = match condition {
            NetworkCondition::Online | NetworkCondition::WiFi => 0,
            NetworkCondition::Offline => 0,
            NetworkCondition::Slow3G => 400,
            NetworkCondition::Fast3G => 150,
            NetworkCondition::FourG => 50,
            NetworkCondition::FiveG => 15,
        };
    }

    pub fn is_connected(&self) -> bool {
        self.condition != NetworkCondition::Offline
    }

    pub fn latency_ms(&self) -> u32 {
        self.simulated_latency_ms
    }
}
