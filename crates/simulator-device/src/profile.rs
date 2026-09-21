use serde::{Deserialize, Serialize};
use simulator_core::{Insets, Rect, Size};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Orientation {
    Portrait,
    Landscape,
}

impl Default for Orientation {
    fn default() -> Self {
        Self::Portrait
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraCutout {
    #[serde(rename = "type")]
    pub cutout_type: String,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceProfile {
    pub name: String,
    pub id: String,
    pub platform: String,
    pub width: u32,
    pub height: u32,
    #[serde(rename = "logicalWidth")]
    pub logical_width: f32,
    #[serde(rename = "logicalHeight")]
    pub logical_height: f32,
    pub density: f32,
    #[serde(rename = "fontScale")]
    pub font_scale: f32,
    #[serde(rename = "safeArea")]
    pub safe_area: Insets,
    #[serde(rename = "cornerRadius", default)]
    pub corner_radius: f32,
    #[serde(rename = "cameraCutout")]
    pub camera_cutout: Option<CameraCutout>,
}

impl DeviceProfile {
    pub fn pixel_9() -> Self {
        Self {
            name: "Pixel 9".to_string(),
            id: "pixel-9".to_string(),
            platform: "android".to_string(),
            width: 1080,
            height: 2424,
            logical_width: 393.0,
            logical_height: 881.0,
            density: 2.75,
            font_scale: 1.0,
            safe_area: Insets::new(44.0, 34.0, 0.0, 0.0),
            corner_radius: 32.0,
            camera_cutout: Some(CameraCutout {
                cutout_type: "punch-hole".to_string(),
                x: 196.5,
                y: 20.0,
                radius: 14.0,
            }),
        }
    }

    pub fn screen_size(&self, orientation: Orientation) -> Size {
        match orientation {
            Orientation::Portrait => Size::new(self.logical_width, self.logical_height),
            Orientation::Landscape => Size::new(self.logical_height, self.logical_width),
        }
    }

    pub fn screen_rect(&self, orientation: Orientation) -> Rect {
        Rect::from_point_size(simulator_core::Point::ZERO, self.screen_size(orientation))
    }

    pub fn safe_rect(&self, orientation: Orientation) -> Rect {
        let size = self.screen_size(orientation);
        let insets = match orientation {
            Orientation::Portrait => self.safe_area,
            Orientation::Landscape => Insets::new(0.0, 21.0, self.safe_area.top, 0.0),
        };
        Rect::new(
            insets.left,
            insets.top,
            size.width - insets.left - insets.right,
            size.height - insets.top - insets.bottom,
        )
    }
}
