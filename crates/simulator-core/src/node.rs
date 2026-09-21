use crate::color::Color;
use crate::geometry::{Insets, Point, Rect};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Node#{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentType {
    View,
    Text,
    TextInput,
    Pressable,
    TouchableOpacity,
    ScrollView,
    SafeAreaView,
    Image,
    ImageBackground,
    Other(String),
}

impl ComponentType {
    pub fn from_name(name: &str) -> Self {
        match name {
            "RCTView" | "View" => Self::View,
            "RCTText" | "RCTParagraph" | "Text" => Self::Text,
            "RCTSinglelineTextInputView" | "RCTMultilineTextInputView" | "TextInput" => Self::TextInput,
            "Pressable" => Self::Pressable,
            "TouchableOpacity" => Self::TouchableOpacity,
            "RCTScrollView" | "ScrollView" => Self::ScrollView,
            "RCTSafeAreaView" | "SafeAreaView" => Self::SafeAreaView,
            "RCTImageView" | "Image" => Self::Image,
            "ImageBackground" => Self::ImageBackground,
            other => Self::Other(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlexDirection {
    Column,
    Row,
    ColumnReverse,
    RowReverse,
}

impl Default for FlexDirection {
    fn default() -> Self {
        Self::Column
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JustifyContent {
    FlexStart,
    Center,
    FlexEnd,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl Default for JustifyContent {
    fn default() -> Self {
        Self::FlexStart
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlignItems {
    Stretch,
    FlexStart,
    Center,
    FlexEnd,
    Baseline,
}

impl Default for AlignItems {
    fn default() -> Self {
        Self::Stretch
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Style {
    pub flex: Option<f32>,
    pub flex_grow: Option<f32>,
    pub flex_shrink: Option<f32>,
    pub flex_direction: FlexDirection,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub align_self: Option<AlignItems>,

    pub width: Option<f32>,
    pub height: Option<f32>,
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,

    pub padding: Insets,
    pub margin: Insets,

    pub background_color: Option<Color>,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Option<Color>,
    pub opacity: f32,

    // Text specific
    pub color: Option<Color>,
    pub font_size: f32,
    pub font_weight: String,
    pub text_align: String,
    pub line_height: Option<f32>,
}

impl Style {
    pub fn new() -> Self {
        Self {
            opacity: 1.0,
            font_size: 14.0,
            font_weight: "normal".to_string(),
            text_align: "left".to_string(),
            ..Default::default()
        }
    }

    pub fn from_json(val: &serde_json::Value) -> Self {
        let mut style = Self::new();
        style.apply_json(val);
        style
    }

    pub fn apply_json(&mut self, val: &serde_json::Value) {
        if let serde_json::Value::Array(arr) = val {
            for item in arr {
                self.apply_json(item);
            }
            return;
        }

        if let serde_json::Value::Object(map) = val {
            if let Some(flex) = map.get("flex").and_then(|v| v.as_f64()) {
                self.flex = Some(flex as f32);
            }
            if let Some(w) = map.get("width").and_then(|v| v.as_f64()) {
                self.width = Some(w as f32);
            }
            if let Some(h) = map.get("height").and_then(|v| v.as_f64()) {
                self.height = Some(h as f32);
            }
            if let Some(bg) = map.get("backgroundColor").and_then(|v| v.as_str()) {
                self.background_color = Color::parse(bg);
            }
            if let Some(c) = map.get("color").and_then(|v| v.as_str()) {
                self.color = Color::parse(c);
            }
            if let Some(r) = map.get("borderRadius").and_then(|v| v.as_f64()) {
                self.border_radius = r as f32;
            }
            if let Some(bw) = map.get("borderWidth").and_then(|v| v.as_f64()) {
                self.border_width = bw as f32;
            }
            if let Some(bc) = map.get("borderColor").and_then(|v| v.as_str()) {
                self.border_color = Color::parse(bc);
            }
            if let Some(op) = map.get("opacity").and_then(|v| v.as_f64()) {
                self.opacity = op as f32;
            }
            if let Some(fs) = map.get("fontSize").and_then(|v| v.as_f64()) {
                self.font_size = fs as f32;
            }
            if let Some(fw) = map.get("fontWeight").and_then(|v| v.as_str()) {
                self.font_weight = fw.to_string();
            }
            if let Some(ta) = map.get("textAlign").and_then(|v| v.as_str()) {
                self.text_align = ta.to_string();
            }

            // Padding
            let p_all = map.get("padding").and_then(|v| v.as_f64()).map(|v| v as f32);
            let p_h = map.get("paddingHorizontal").and_then(|v| v.as_f64()).map(|v| v as f32);
            let p_v = map.get("paddingVertical").and_then(|v| v.as_f64()).map(|v| v as f32);
            let p_t = map.get("paddingTop").and_then(|v| v.as_f64()).map(|v| v as f32);
            let p_b = map.get("paddingBottom").and_then(|v| v.as_f64()).map(|v| v as f32);
            let p_l = map.get("paddingLeft").and_then(|v| v.as_f64()).map(|v| v as f32);
            let p_r = map.get("paddingRight").and_then(|v| v.as_f64()).map(|v| v as f32);

            let top = p_t.or(p_v).or(p_all).unwrap_or(self.padding.top);
            let bottom = p_b.or(p_v).or(p_all).unwrap_or(self.padding.bottom);
            let left = p_l.or(p_h).or(p_all).unwrap_or(self.padding.left);
            let right = p_r.or(p_h).or(p_all).unwrap_or(self.padding.right);
            self.padding = Insets::new(top, bottom, left, right);

            // Margin
            let m_all = map.get("margin").and_then(|v| v.as_f64()).map(|v| v as f32);
            let m_h = map.get("marginHorizontal").and_then(|v| v.as_f64()).map(|v| v as f32);
            let m_v = map.get("marginVertical").and_then(|v| v.as_f64()).map(|v| v as f32);
            let m_t = map.get("marginTop").and_then(|v| v.as_f64()).map(|v| v as f32);
            let m_b = map.get("marginBottom").and_then(|v| v.as_f64()).map(|v| v as f32);
            let m_l = map.get("marginLeft").and_then(|v| v.as_f64()).map(|v| v as f32);
            let m_r = map.get("marginRight").and_then(|v| v.as_f64()).map(|v| v as f32);

            let m_top = m_t.or(m_v).or(m_all).unwrap_or(self.margin.top);
            let m_bottom = m_b.or(m_v).or(m_all).unwrap_or(self.margin.bottom);
            let m_left = m_l.or(m_h).or(m_all).unwrap_or(self.margin.left);
            let m_right = m_r.or(m_h).or(m_all).unwrap_or(self.margin.right);
            self.margin = Insets::new(m_top, m_bottom, m_left, m_right);

            if let Some(fd) = map.get("flexDirection").and_then(|v| v.as_str()) {
                self.flex_direction = match fd {
                    "row" => FlexDirection::Row,
                    "column-reverse" => FlexDirection::ColumnReverse,
                    "row-reverse" => FlexDirection::RowReverse,
                    _ => FlexDirection::Column,
                };
            }

            if let Some(jc) = map.get("justifyContent").and_then(|v| v.as_str()) {
                self.justify_content = match jc {
                    "center" => JustifyContent::Center,
                    "flex-end" => JustifyContent::FlexEnd,
                    "space-between" => JustifyContent::SpaceBetween,
                    "space-around" => JustifyContent::SpaceAround,
                    "space-evenly" => JustifyContent::SpaceEvenly,
                    _ => JustifyContent::FlexStart,
                };
            }

            if let Some(ai) = map.get("alignItems").and_then(|v| v.as_str()) {
                self.align_items = match ai {
                    "flex-start" => AlignItems::FlexStart,
                    "center" => AlignItems::Center,
                    "flex-end" => AlignItems::FlexEnd,
                    "baseline" => AlignItems::Baseline,
                    _ => AlignItems::Stretch,
                };
            }
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub component_type: ComponentType,
    pub props: HashMap<String, serde_json::Value>,
    pub style: Style,
    pub children: Vec<NodeId>,
    pub layout: Rect,
    pub text_content: Option<String>,
    pub placeholder: Option<String>,
    pub placeholder_text_color: Option<Color>,
    pub scroll_offset: Point,
    pub content_container_style: Option<Style>,
    pub is_focused: bool,
    pub image_source: Option<String>,
    pub resize_mode: Option<String>,
}

impl Node {
    pub fn new(id: NodeId, component_type: ComponentType) -> Self {
        Self {
            id,
            component_type,
            props: HashMap::new(),
            style: Style::new(),
            children: Vec::new(),
            layout: Rect::ZERO,
            text_content: None,
            placeholder: None,
            placeholder_text_color: None,
            scroll_offset: Point::ZERO,
            content_container_style: None,
            is_focused: false,
            image_source: None,
            resize_mode: None,
        }
    }

    pub fn update_props(&mut self, new_props: serde_json::Value) {
        if let serde_json::Value::Object(map) = new_props {
            if let Some(style_val) = map.get("style") {
                self.style = Style::from_json(style_val);
                // Also check style for resizeMode
                if let serde_json::Value::Object(style_map) = style_val {
                    if let Some(rm) = style_map.get("resizeMode").and_then(|v| v.as_str()) {
                        self.resize_mode = Some(rm.to_string());
                    }
                }
            }
            if let Some(ccs) = map.get("contentContainerStyle") {
                self.content_container_style = Some(Style::from_json(ccs));
            }
            if let Some(ph) = map.get("placeholder").and_then(|v| v.as_str()) {
                self.placeholder = Some(ph.to_string());
            }
            if let Some(pc) = map.get("placeholderTextColor").and_then(|v| v.as_str()) {
                self.placeholder_text_color = Color::parse(pc);
            }
            if let Some(text) = map.get("text").and_then(|v| v.as_str()) {
                self.text_content = Some(text.to_string());
            } else if let Some(children_str) = map.get("children").and_then(|v| v.as_str()) {
                self.text_content = Some(children_str.to_string());
            }

            // Image source parsing
            if let Some(source_val) = map.get("source") {
                if let Some(uri_str) = source_val.as_str() {
                    self.image_source = Some(uri_str.to_string());
                } else if let Some(uri_obj) = source_val.get("uri").and_then(|v| v.as_str()) {
                    self.image_source = Some(uri_obj.to_string());
                } else if let Some(arr) = source_val.as_array() {
                    if let Some(first_uri) = arr.first().and_then(|item| item.get("uri")).and_then(|v| v.as_str()) {
                        self.image_source = Some(first_uri.to_string());
                    }
                }
            }

            // Image resizeMode prop
            if let Some(rm) = map.get("resizeMode").and_then(|v| v.as_str()) {
                self.resize_mode = Some(rm.to_string());
            }

            for (k, v) in map {
                self.props.insert(k, v);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_array_merging_for_nativewind() {
        let json = serde_json::json!([
            { "flex": 1.0, "backgroundColor": "blue" },
            { "backgroundColor": "orangered", "padding": 16.0 }
        ]);

        let style = Style::from_json(&json);
        assert_eq!(style.flex, Some(1.0));
        assert_eq!(style.background_color, Some(Color::rgb(255, 69, 0)));
        assert_eq!(style.padding.top, 16.0);
        assert_eq!(style.padding.bottom, 16.0);
    }
}
