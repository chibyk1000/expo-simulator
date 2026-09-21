use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum BridgeRequest {
    #[serde(rename = "ui.createNode")]
    CreateNode {
        id: u64,
        #[serde(rename = "viewName")]
        view_name: String,
        #[serde(rename = "rootTag")]
        root_tag: u64,
        props: Value,
    },

    #[serde(rename = "ui.cloneNode")]
    CloneNode {
        id: u64,
        #[serde(rename = "newId")]
        new_id: u64,
        props: Option<Value>,
        children: Option<Vec<u64>>,
    },

    #[serde(rename = "ui.updateProps")]
    UpdateProps {
        id: u64,
        props: Value,
    },

    #[serde(rename = "ui.setChildren")]
    SetChildren {
        id: u64,
        children: Vec<u64>,
    },

    #[serde(rename = "ui.appendChild")]
    AppendChild {
        #[serde(rename = "parentId")]
        parent_id: u64,
        #[serde(rename = "childId")]
        child_id: u64,
    },

    #[serde(rename = "ui.removeChild")]
    RemoveChild {
        #[serde(rename = "parentId")]
        parent_id: u64,
        #[serde(rename = "childId")]
        child_id: u64,
    },

    #[serde(rename = "ui.completeRoot")]
    CompleteRoot {
        #[serde(rename = "rootTag")]
        root_tag: u64,
        children: Vec<u64>,
    },

    #[serde(rename = "ui.measure")]
    Measure {
        id: u64,
        #[serde(rename = "callbackId")]
        callback_id: u64,
    },

    #[serde(rename = "app.log")]
    Log {
        level: String,
        message: String,
    },

    #[serde(rename = "network.request")]
    NetworkRequest {
        id: String,
        url: String,
        method: String,
        timestamp: String,
    },

    #[serde(rename = "network.response")]
    NetworkResponse {
        id: String,
        status: u16,
        #[serde(rename = "durationMs")]
        duration_ms: u64,
        #[serde(rename = "sizeBytes")]
        size_bytes: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum BridgeResponse {
    #[serde(rename = "event.dispatch")]
    DispatchEvent {
        #[serde(rename = "targetId")]
        target_id: u64,
        #[serde(rename = "eventName")]
        event_name: String,
        payload: Value,
    },

    #[serde(rename = "ui.measureResult")]
    MeasureResult {
        #[serde(rename = "callbackId")]
        callback_id: u64,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        #[serde(rename = "pageX")]
        page_x: f32,
        #[serde(rename = "pageY")]
        page_y: f32,
    },

    #[serde(rename = "app.reload")]
    Reload,

    #[serde(rename = "device.updateDimensions")]
    UpdateDimensions {
        width: f32,
        height: f32,
        scale: f32,
        #[serde(rename = "fontScale")]
        font_scale: f32,
    },

    #[serde(rename = "appearance.update")]
    UpdateAppearance {
        #[serde(rename = "colorScheme")]
        color_scheme: String,
    },
}
