use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const TRANSPARENT: Self = Self { r: 0, g: 0, b: 0, a: 0 };
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0, a: 255 };
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255, a: 255 };

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        if s.is_empty() || s.eq_ignore_ascii_case("transparent") {
            return Some(Self::TRANSPARENT);
        }

        if let Some(hex) = s.strip_prefix('#') {
            return match hex.len() {
                3 => {
                    let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
                    let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
                    let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
                    Some(Self::rgba(r, g, b, 255))
                }
                4 => {
                    let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
                    let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
                    let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
                    let a = u8::from_str_radix(&hex[3..4], 16).ok()? * 17;
                    Some(Self::rgba(r, g, b, a))
                }
                6 => {
                    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                    Some(Self::rgba(r, g, b, 255))
                }
                8 => {
                    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                    let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                    Some(Self::rgba(r, g, b, a))
                }
                _ => None,
            };
        }

        if s.starts_with("rgb(") && s.ends_with(')') {
            let inner = &s[4..s.len() - 1];
            let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
            if parts.len() == 3 {
                let r: u8 = parts[0].parse().ok()?;
                let g: u8 = parts[1].parse().ok()?;
                let b: u8 = parts[2].parse().ok()?;
                return Some(Self::rgba(r, g, b, 255));
            }
        }

        if s.starts_with("rgba(") && s.ends_with(')') {
            let inner = &s[5..s.len() - 1];
            let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
            if parts.len() == 4 {
                let r: u8 = parts[0].parse().ok()?;
                let g: u8 = parts[1].parse().ok()?;
                let b: u8 = parts[2].parse().ok()?;
                let a_f: f32 = parts[3].parse().ok()?;
                let a = (a_f.clamp(0.0, 1.0) * 255.0).round() as u8;
                return Some(Self::rgba(r, g, b, a));
            }
        }

        // Full CSS / VS Code standard named color palette (148 colors)
        match s.to_ascii_lowercase().as_str() {
            "aliceblue" => Some(Self::rgb(240, 248, 255)),
            "antiquewhite" => Some(Self::rgb(250, 235, 215)),
            "aqua" => Some(Self::rgb(0, 255, 255)),
            "aquamarine" => Some(Self::rgb(127, 255, 212)),
            "azure" => Some(Self::rgb(240, 255, 255)),
            "beige" => Some(Self::rgb(245, 245, 220)),
            "bisque" => Some(Self::rgb(255, 228, 196)),
            "black" => Some(Self::BLACK),
            "blanchedalmond" => Some(Self::rgb(255, 235, 205)),
            "blue" => Some(Self::rgb(0, 0, 255)),
            "blueviolet" => Some(Self::rgb(138, 43, 226)),
            "brown" => Some(Self::rgb(165, 42, 42)),
            "burlywood" => Some(Self::rgb(222, 184, 135)),
            "cadetblue" => Some(Self::rgb(95, 158, 160)),
            "chartreuse" => Some(Self::rgb(127, 255, 0)),
            "chocolate" => Some(Self::rgb(210, 105, 30)),
            "coral" => Some(Self::rgb(255, 127, 80)),
            "cornflowerblue" => Some(Self::rgb(100, 149, 237)),
            "cornsilk" => Some(Self::rgb(255, 248, 220)),
            "crimson" => Some(Self::rgb(220, 20, 60)),
            "cyan" => Some(Self::rgb(0, 255, 255)),
            "darkblue" => Some(Self::rgb(0, 0, 139)),
            "darkcyan" => Some(Self::rgb(0, 139, 139)),
            "darkgoldenrod" => Some(Self::rgb(184, 134, 11)),
            "darkgray" | "darkgrey" => Some(Self::rgb(169, 169, 169)),
            "darkgreen" => Some(Self::rgb(0, 100, 0)),
            "darkkhaki" => Some(Self::rgb(189, 183, 107)),
            "darkmagenta" => Some(Self::rgb(139, 0, 139)),
            "darkolivegreen" => Some(Self::rgb(85, 107, 47)),
            "darkorange" => Some(Self::rgb(255, 140, 0)),
            "darkorchid" => Some(Self::rgb(153, 50, 204)),
            "darkred" => Some(Self::rgb(139, 0, 0)),
            "darksalmon" => Some(Self::rgb(233, 150, 122)),
            "darkseagreen" => Some(Self::rgb(143, 188, 143)),
            "darkslateblue" => Some(Self::rgb(72, 61, 139)),
            "darkslategray" | "darkslategrey" => Some(Self::rgb(47, 79, 79)),
            "darkturquoise" => Some(Self::rgb(0, 206, 209)),
            "darkviolet" => Some(Self::rgb(148, 0, 211)),
            "deeppink" => Some(Self::rgb(255, 20, 147)),
            "deepskyblue" => Some(Self::rgb(0, 191, 255)),
            "dimgray" | "dimgrey" => Some(Self::rgb(105, 105, 105)),
            "dodgerblue" => Some(Self::rgb(30, 144, 255)),
            "firebrick" => Some(Self::rgb(178, 34, 34)),
            "floralwhite" => Some(Self::rgb(255, 250, 240)),
            "forestgreen" => Some(Self::rgb(34, 139, 34)),
            "fuchsia" => Some(Self::rgb(255, 0, 255)),
            "gainsboro" => Some(Self::rgb(220, 220, 220)),
            "ghostwhite" => Some(Self::rgb(248, 248, 255)),
            "gold" => Some(Self::rgb(255, 215, 0)),
            "goldenrod" => Some(Self::rgb(218, 165, 32)),
            "gray" | "grey" => Some(Self::rgb(128, 128, 128)),
            "green" => Some(Self::rgb(0, 128, 0)),
            "greenyellow" => Some(Self::rgb(173, 255, 47)),
            "honeydew" => Some(Self::rgb(240, 255, 240)),
            "hotpink" => Some(Self::rgb(255, 105, 180)),
            "indianred" => Some(Self::rgb(205, 92, 92)),
            "indigo" => Some(Self::rgb(75, 0, 130)),
            "ivory" => Some(Self::rgb(255, 255, 240)),
            "khaki" => Some(Self::rgb(240, 230, 140)),
            "lavender" => Some(Self::rgb(230, 230, 250)),
            "lavenderblush" => Some(Self::rgb(255, 240, 245)),
            "lawngreen" => Some(Self::rgb(124, 252, 0)),
            "lemonchiffon" => Some(Self::rgb(255, 250, 205)),
            "lightblue" => Some(Self::rgb(173, 216, 230)),
            "lightcoral" => Some(Self::rgb(240, 128, 128)),
            "lightcyan" => Some(Self::rgb(224, 255, 255)),
            "lightgoldenrodyellow" => Some(Self::rgb(250, 250, 210)),
            "lightgray" | "lightgrey" => Some(Self::rgb(211, 211, 211)),
            "lightgreen" => Some(Self::rgb(144, 238, 144)),
            "lightpink" => Some(Self::rgb(255, 182, 193)),
            "lightsalmon" => Some(Self::rgb(255, 160, 122)),
            "lightseagreen" => Some(Self::rgb(32, 178, 170)),
            "lightskyblue" => Some(Self::rgb(135, 206, 250)),
            "lightslategray" | "lightslategrey" => Some(Self::rgb(119, 136, 153)),
            "lightsteelblue" => Some(Self::rgb(176, 196, 222)),
            "lightyellow" => Some(Self::rgb(255, 255, 224)),
            "lime" => Some(Self::rgb(0, 255, 0)),
            "limegreen" => Some(Self::rgb(50, 205, 50)),
            "linen" => Some(Self::rgb(250, 240, 230)),
            "magenta" => Some(Self::rgb(255, 0, 255)),
            "maroon" => Some(Self::rgb(128, 0, 0)),
            "mediumaquamarine" => Some(Self::rgb(102, 205, 170)),
            "mediumblue" => Some(Self::rgb(0, 0, 205)),
            "mediumorchid" => Some(Self::rgb(186, 85, 211)),
            "mediumpurple" => Some(Self::rgb(147, 112, 219)),
            "mediumseagreen" => Some(Self::rgb(60, 179, 113)),
            "mediumslateblue" => Some(Self::rgb(123, 104, 238)),
            "mediumspringgreen" => Some(Self::rgb(0, 250, 154)),
            "mediumturquoise" => Some(Self::rgb(72, 209, 204)),
            "mediumvioletred" => Some(Self::rgb(199, 21, 133)),
            "midnightblue" => Some(Self::rgb(25, 25, 112)),
            "mintcream" => Some(Self::rgb(245, 255, 250)),
            "mistyrose" => Some(Self::rgb(255, 228, 225)),
            "moccasin" => Some(Self::rgb(255, 228, 181)),
            "navajowhite" => Some(Self::rgb(255, 222, 173)),
            "navy" => Some(Self::rgb(0, 0, 128)),
            "oldlace" => Some(Self::rgb(253, 245, 230)),
            "olive" => Some(Self::rgb(128, 128, 0)),
            "olivedrab" => Some(Self::rgb(107, 142, 35)),
            "orange" => Some(Self::rgb(255, 165, 0)),
            "orangered" => Some(Self::rgb(255, 69, 0)),
            "orchid" => Some(Self::rgb(218, 112, 214)),
            "palegoldenrod" => Some(Self::rgb(238, 232, 170)),
            "palegreen" => Some(Self::rgb(152, 251, 152)),
            "paleturquoise" => Some(Self::rgb(175, 238, 238)),
            "palevioletred" => Some(Self::rgb(219, 112, 147)),
            "papayawhip" => Some(Self::rgb(255, 239, 213)),
            "peachpuff" => Some(Self::rgb(255, 218, 185)),
            "peru" => Some(Self::rgb(205, 133, 63)),
            "pink" => Some(Self::rgb(255, 192, 203)),
            "plum" => Some(Self::rgb(221, 160, 221)),
            "powderblue" => Some(Self::rgb(176, 224, 230)),
            "purple" => Some(Self::rgb(128, 0, 128)),
            "rebeccapurple" => Some(Self::rgb(102, 51, 153)),
            "red" => Some(Self::rgb(255, 0, 0)),
            "rosybrown" => Some(Self::rgb(188, 143, 143)),
            "royalblue" => Some(Self::rgb(65, 105, 225)),
            "saddlebrown" => Some(Self::rgb(139, 69, 19)),
            "salmon" => Some(Self::rgb(250, 128, 114)),
            "sandybrown" => Some(Self::rgb(244, 164, 96)),
            "seagreen" => Some(Self::rgb(46, 139, 87)),
            "seashell" => Some(Self::rgb(255, 245, 238)),
            "sienna" => Some(Self::rgb(160, 82, 45)),
            "silver" => Some(Self::rgb(192, 192, 192)),
            "skyblue" => Some(Self::rgb(135, 206, 235)),
            "slateblue" => Some(Self::rgb(106, 90, 205)),
            "slategray" | "slategrey" => Some(Self::rgb(112, 128, 144)),
            "snow" => Some(Self::rgb(255, 250, 250)),
            "springgreen" => Some(Self::rgb(0, 255, 127)),
            "steelblue" => Some(Self::rgb(70, 130, 180)),
            "tan" => Some(Self::rgb(210, 180, 140)),
            "teal" => Some(Self::rgb(0, 128, 128)),
            "thistle" => Some(Self::rgb(216, 191, 216)),
            "tomato" => Some(Self::rgb(255, 99, 71)),
            "turquoise" => Some(Self::rgb(64, 224, 208)),
            "violet" => Some(Self::rgb(238, 130, 238)),
            "wheat" => Some(Self::rgb(245, 222, 179)),
            "white" => Some(Self::WHITE),
            "whitesmoke" => Some(Self::rgb(245, 245, 245)),
            "yellow" => Some(Self::rgb(255, 255, 0)),
            "yellowgreen" => Some(Self::rgb(154, 205, 50)),
            _ => None,
        }

    }

    pub fn to_u32(&self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub fn to_rgba8(&self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
    }
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Color::parse(&s).ok_or_else(|| serde::de::Error::custom(format!("Invalid color format: {s}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_parsing() {
        assert_eq!(Color::parse("#111827"), Some(Color::rgb(0x11, 0x18, 0x27)));
        assert_eq!(Color::parse("#fff"), Some(Color::WHITE));
        assert_eq!(Color::parse("white"), Some(Color::WHITE));
        assert_eq!(Color::parse("yellow"), Some(Color::rgb(255, 255, 0)));
        assert_eq!(Color::parse("orangered"), Some(Color::rgb(255, 69, 0)));
        assert_eq!(Color::parse("cornflowerblue"), Some(Color::rgb(100, 149, 237)));
        assert_eq!(Color::parse("rebeccapurple"), Some(Color::rgb(102, 51, 153)));
        assert_eq!(Color::parse("#2563EB"), Some(Color::rgb(0x25, 0x63, 0xEB)));
    }
}
