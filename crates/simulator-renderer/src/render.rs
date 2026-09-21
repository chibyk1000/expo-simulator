use crate::text::TextEngine;
use simulator_core::{Color, ComponentType, Node, NodeId, Point, Rect};
use std::collections::HashMap;
use tiny_skia::{FillRule, Mask, Paint, PathBuilder, Pixmap, PixmapMut, Stroke, Transform};

pub struct Renderer {
    pub text_engine: TextEngine,
    pub image_cache: HashMap<String, Pixmap>,
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            text_engine: TextEngine::new(),
            image_cache: HashMap::new(),
        }
    }

    pub fn create_placeholder_image(w: u32, h: u32, label: &str) -> Pixmap {
        let mut pm = Pixmap::new(w.max(1), h.max(1)).unwrap_or_else(|| Pixmap::new(1, 1).unwrap());
        pm.fill(tiny_skia::Color::from_rgba8(30, 41, 59, 255)); // slate-800

        let fw = w as f32;
        let fh = h as f32;

        // Draw mountains / landscape graphic
        let mut pb = PathBuilder::new();
        pb.move_to(fw * 0.15, fh * 0.80);
        pb.line_to(fw * 0.45, fh * 0.35);
        pb.line_to(fw * 0.70, fh * 0.70);
        pb.line_to(fw * 0.85, fh * 0.50);
        pb.line_to(fw * 0.95, fh * 0.80);
        pb.close();

        if let Some(path) = pb.finish() {
            let mut paint = Paint::default();
            paint.set_color_rgba8(71, 85, 105, 255); // slate-600
            pm.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
        }

        // Sun / Moon circle
        let sun_r = (fw.min(fh) * 0.08).max(4.0);
        let sun_cx = fw * 0.75;
        let sun_cy = fh * 0.28;
        let mut pb_sun = PathBuilder::new();
        pb_sun.push_circle(sun_cx, sun_cy, sun_r);
        if let Some(sun_path) = pb_sun.finish() {
            let mut paint = Paint::default();
            paint.set_color_rgba8(250, 204, 21, 255); // Yellow-400
            pm.fill_path(&sun_path, &paint, FillRule::Winding, Transform::identity(), None);
        }

        // Label at bottom
        if !label.is_empty() {
            let mut text_engine = TextEngine::new();
            text_engine.render_text(
                &mut pm.as_mut(),
                label,
                Rect::new(fw * 0.05, fh * 0.82, fw * 0.9, fh * 0.16),
                (fh * 0.11).max(9.0).min(14.0),
                "600",
                Color::rgb(203, 213, 225),
                Some(fw * 0.9),
            );
        }

        pm
    }

    pub fn get_or_load_image(&mut self, source: &str) -> &Pixmap {
        if !self.image_cache.contains_key(source) {
            let candidates = [
                source.to_string(),
                format!("examples/expo-app/{}", source),
                format!("examples/expo-app/assets/{}", source),
                format!("assets/{}", source),
            ];

            let mut loaded = None;
            for path_str in &candidates {
                let p = std::path::Path::new(path_str);
                if p.exists() {
                    if let Ok(pixmap) = Pixmap::load_png(p) {
                        loaded = Some(pixmap);
                        break;
                    }
                }
            }

            let pixmap = loaded.unwrap_or_else(|| {
                let label = if source.starts_with("http") {
                    source.split('/').last().unwrap_or("Network Image")
                } else {
                    std::path::Path::new(source)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(source)
                };
                Self::create_placeholder_image(300, 200, label)
            });

            self.image_cache.insert(source.to_string(), pixmap);
        }

        self.image_cache.get(source).unwrap()
    }

    pub fn render_image(
        &mut self,
        pixmap: &mut PixmapMut,
        source: &str,
        resize_mode: Option<&str>,
        bounds: Rect,
        border_radius: f32,
        opacity: f32,
    ) {
        if bounds.width <= 0.0 || bounds.height <= 0.0 {
            return;
        }

        let img_pixmap = self.get_or_load_image(source).clone();
        let img_w = img_pixmap.width() as f32;
        let img_h = img_pixmap.height() as f32;
        if img_w <= 0.0 || img_h <= 0.0 {
            return;
        }

        let bw = bounds.width.max(1.0);
        let bh = bounds.height.max(1.0);

        let mode = resize_mode.unwrap_or("cover");
        let (scale_x, scale_y, offset_x, offset_y) = match mode {
            "stretch" => {
                (bw / img_w, bh / img_h, 0.0_f32, 0.0_f32)
            }
            "contain" => {
                let s = (bw / img_w).min(bh / img_h);
                let dw = img_w * s;
                let dh = img_h * s;
                (s, s, (bw - dw) / 2.0, (bh - dh) / 2.0)
            }
            _ => { // "cover"
                let s = (bw / img_w).max(bh / img_h);
                let dw = img_w * s;
                let dh = img_h * s;
                (s, s, (bw - dw) / 2.0, (bh - dh) / 2.0)
            }
        };

        let mut temp = match Pixmap::new(bw as u32, bh as u32) {
            Some(p) => p,
            None => return,
        };

        let transform = Transform::from_translate(offset_x, offset_y).pre_scale(scale_x, scale_y);
        temp.draw_pixmap(
            0,
            0,
            img_pixmap.as_ref(),
            &tiny_skia::PixmapPaint {
                opacity: 1.0,
                blend_mode: tiny_skia::BlendMode::SourceOver,
                quality: tiny_skia::FilterQuality::Bilinear,
            },
            transform,
            None,
        );

        // Apply rounded clipping if border_radius > 0.0
        let r = border_radius.min(bw / 2.0).min(bh / 2.0);
        if r > 0.5 {
            if let Some(mut mask) = Mask::new(bw as u32, bh as u32) {
                let mut pb = PathBuilder::new();
                pb.move_to(r, 0.0);
                pb.line_to(bw - r, 0.0);
                pb.quad_to(bw, 0.0, bw, r);
                pb.line_to(bw, bh - r);
                pb.quad_to(bw, bh, bw - r, bh);
                pb.line_to(r, bh);
                pb.quad_to(0.0, bh, 0.0, bh - r);
                pb.line_to(0.0, r);
                pb.quad_to(0.0, 0.0, r, 0.0);
                pb.close();
                if let Some(path) = pb.finish() {
                    mask.fill_path(&path, FillRule::Winding, false, Transform::identity());
                    temp.apply_mask(&mask);
                }
            }
        }

        pixmap.draw_pixmap(
            bounds.x as i32,
            bounds.y as i32,
            temp.as_ref(),
            &tiny_skia::PixmapPaint {
                opacity,
                blend_mode: tiny_skia::BlendMode::SourceOver,
                quality: tiny_skia::FilterQuality::Bilinear,
            },
            Transform::identity(),
            None,
        );
    }

    pub fn render_inspector_overlay(
        &self,
        pixmap: &mut PixmapMut,
        node: &Node,
        origin: Point,
        parent_scroll: Point,
    ) {
        let abs_x = origin.x + node.layout.x - parent_scroll.x;
        let abs_y = origin.y + node.layout.y - parent_scroll.y;
        let w = node.layout.width;
        let h = node.layout.height;

        // 1. Margin box (orange)
        let margin_rect = Rect::new(
            abs_x - node.style.margin.left,
            abs_y - node.style.margin.top,
            (w + node.style.margin.left + node.style.margin.right).max(1.0),
            (h + node.style.margin.top + node.style.margin.bottom).max(1.0),
        );
        self.draw_rounded_rect(pixmap, margin_rect, 0.0, Color::rgba(249, 115, 22, 45), 1.0);
        self.draw_border(pixmap, margin_rect, 0.0, 1.0, Color::rgba(249, 115, 22, 220));

        // 2. Padding box (green)
        let padding_rect = Rect::new(abs_x, abs_y, w.max(1.0), h.max(1.0));
        self.draw_rounded_rect(pixmap, padding_rect, node.style.border_radius, Color::rgba(34, 197, 94, 55), 1.0);
        self.draw_border(pixmap, padding_rect, node.style.border_radius, 1.0, Color::rgba(34, 197, 94, 220));

        // 3. Content box (blue)
        let content_x = abs_x + node.style.padding.left;
        let content_y = abs_y + node.style.padding.top;
        let content_w = (w - node.style.padding.left - node.style.padding.right).max(1.0);
        let content_h = (h - node.style.padding.top - node.style.padding.bottom).max(1.0);
        let content_rect = Rect::new(content_x, content_y, content_w, content_h);
        self.draw_rounded_rect(pixmap, content_rect, 0.0, Color::rgba(59, 130, 246, 75), 1.0);
        self.draw_border(pixmap, content_rect, 0.0, 1.0, Color::rgba(59, 130, 246, 240));

        // 4. Element Inspector Tooltip Badge
        let type_name = match &node.component_type {
            ComponentType::View => "View",
            ComponentType::Text => "Text",
            ComponentType::TextInput => "TextInput",
            ComponentType::Pressable => "Pressable",
            ComponentType::TouchableOpacity => "TouchableOpacity",
            ComponentType::ScrollView => "ScrollView",
            ComponentType::SafeAreaView => "SafeAreaView",
            ComponentType::Image => "Image",
            ComponentType::ImageBackground => "ImageBackground",
            ComponentType::Other(s) => s.as_str(),
        };
        let badge_text = format!("<{}> #{:?}  {:.0}×{:.0}pt", type_name, node.id.0, w, h);
        let badge_w = 180.0_f32;
        let badge_h = 22.0_f32;
        let badge_x = abs_x.max(4.0).min((pixmap.width() as f32) - badge_w - 4.0);
        let badge_y = if abs_y >= badge_h + 4.0 {
            abs_y - badge_h - 2.0
        } else {
            abs_y + h + 2.0
        };
        let badge_rect = Rect::new(badge_x, badge_y, badge_w, badge_h);
        self.draw_rounded_rect(pixmap, badge_rect, 4.0, Color::rgb(15, 23, 42), 0.95);
        self.draw_border(pixmap, badge_rect, 4.0, 1.0, Color::rgb(6, 182, 212));

        let mut text_engine = TextEngine::new();
        text_engine.render_text(
            pixmap,
            &badge_text,
            Rect::new(badge_x + 6.0, badge_y + 4.0, badge_w - 12.0, 14.0),
            11.0,
            "bold",
            Color::WHITE,
            None,
        );
    }

    pub fn render_tree(
        &mut self,
        pixmap: &mut PixmapMut,
        nodes: &HashMap<NodeId, Node>,
        root_nodes: &[NodeId],
        screen_rect: Rect,
    ) {
        // Clear screen with default dark background
        pixmap.fill(tiny_skia::Color::from_rgba8(17, 24, 39, 255));

        for &root_id in root_nodes {
            self.render_node(pixmap, root_id, nodes, screen_rect.origin(), Point::ZERO);
        }
    }

    fn render_node(
        &mut self,
        pixmap: &mut PixmapMut,
        node_id: NodeId,
        nodes: &HashMap<NodeId, Node>,
        origin: Point,
        parent_scroll: Point,
    ) {
        let node = match nodes.get(&node_id) {
            Some(n) => n,
            None => return,
        };

        let abs_x = origin.x + node.layout.x - parent_scroll.x;
        let abs_y = origin.y + node.layout.y - parent_scroll.y;
        let abs_rect = Rect::new(abs_x, abs_y, node.layout.width, node.layout.height);

        // Draw background and borders
        if let Some(bg_color) = node.style.background_color {
            self.draw_rounded_rect(
                pixmap,
                abs_rect,
                node.style.border_radius,
                bg_color,
                node.style.opacity,
            );
        }

        // Draw image if Image or ImageBackground
        if node.component_type == ComponentType::Image || node.component_type == ComponentType::ImageBackground {
            if let Some(ref src) = node.image_source {
                self.render_image(
                    pixmap,
                    src,
                    node.resize_mode.as_deref(),
                    abs_rect,
                    node.style.border_radius,
                    node.style.opacity,
                );
            }
        }

        if node.style.border_width > 0.0 {
            if let Some(border_color) = node.style.border_color {
                self.draw_border(
                    pixmap,
                    abs_rect,
                    node.style.border_radius,
                    node.style.border_width,
                    border_color,
                );
            }
        }

        // Draw focused outline for TextInput
        if node.component_type == ComponentType::TextInput && node.is_focused {
            self.draw_border(
                pixmap,
                abs_rect,
                node.style.border_radius,
                2.0,
                Color::rgb(37, 99, 235), // active blue focus
            );
        }

        // Draw text
        let has_text = node.text_content.as_ref().map_or(false, |t| !t.is_empty());
        let text_bounds = Rect::new(
            abs_x + node.style.padding.left,
            abs_y + node.style.padding.top,
            (abs_rect.width - node.style.padding.left - node.style.padding.right).max(1.0),
            (abs_rect.height - node.style.padding.top - node.style.padding.bottom).max(1.0),
        );

        if has_text {
            let text = node.text_content.as_ref().unwrap();
            let text_color = node.style.color.unwrap_or(Color::WHITE);

            // Centered text handling if textAlign is center
            let text_x = if node.style.text_align == "center" {
                let measured = self.text_engine.measure_text(
                    text,
                    node.style.font_size,
                    node.style.line_height,
                    Some(text_bounds.width),
                );
                text_bounds.x + (text_bounds.width - measured.width).max(0.0) / 2.0
            } else {
                text_bounds.x
            };

            let draw_bounds = Rect::new(text_x, text_bounds.y, text_bounds.width, text_bounds.height);

            self.text_engine.render_text(
                pixmap,
                text,
                draw_bounds,
                node.style.font_size,
                &node.style.font_weight,
                text_color,
                Some(text_bounds.width),
            );
        } else if node.component_type == ComponentType::TextInput {
            // Placeholder text
            if let Some(ref ph) = node.placeholder {
                let ph_color = node.placeholder_text_color.unwrap_or(Color::rgb(156, 163, 175));
                self.text_engine.render_text(
                    pixmap,
                    ph,
                    text_bounds,
                    node.style.font_size,
                    &node.style.font_weight,
                    ph_color,
                    Some(text_bounds.width),
                );
            }
        }

        // Draw cursor if TextInput is focused
        if node.component_type == ComponentType::TextInput && node.is_focused {
            let cursor_x = if has_text {
                let measured = self.text_engine.measure_text(
                    node.text_content.as_ref().unwrap(),
                    node.style.font_size,
                    node.style.line_height,
                    Some(text_bounds.width),
                );
                (text_bounds.x + measured.width + 1.0).min(text_bounds.x + text_bounds.width - 2.0)
            } else {
                text_bounds.x
            };

            let cursor_h = (node.style.font_size * 1.2).min(text_bounds.height);
            let cursor_y = text_bounds.y + (text_bounds.height - cursor_h) / 2.0;

            if let Some(sk_rect) = tiny_skia::Rect::from_xywh(cursor_x, cursor_y, 2.0, cursor_h) {
                let mut paint = Paint::default();
                paint.set_color_rgba8(96, 165, 250, 255); // soft blue caret
                pixmap.fill_rect(sk_rect, &paint, Transform::identity(), None);
            }
        }

        // Render children
        let current_scroll = if node.component_type == ComponentType::ScrollView {
            node.scroll_offset
        } else {
            Point::ZERO
        };

        for &child_id in &node.children {
            self.render_node(
                pixmap,
                child_id,
                nodes,
                Point::new(abs_x, abs_y),
                current_scroll,
            );
        }
    }

    pub fn draw_rounded_rect(
        &self,
        pixmap: &mut PixmapMut,
        rect: Rect,
        radius: f32,
        color: Color,
        opacity: f32,
    ) {
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return;
        }

        let mut pb = PathBuilder::new();
        let r = radius.min(rect.width / 2.0).min(rect.height / 2.0);

        if r <= 0.5 {
            if let Some(skia_rect) = tiny_skia::Rect::from_xywh(rect.x, rect.y, rect.width, rect.height) {
                let mut paint = Paint::default();
                paint.set_color_rgba8(color.r, color.g, color.b, (color.a as f32 * opacity) as u8);
                pixmap.fill_rect(skia_rect, &paint, Transform::identity(), None);
            }
            return;
        }

        pb.move_to(rect.x + r, rect.y);
        pb.line_to(rect.x + rect.width - r, rect.y);
        pb.quad_to(rect.x + rect.width, rect.y, rect.x + rect.width, rect.y + r);
        pb.line_to(rect.x + rect.width, rect.y + rect.height - r);
        pb.quad_to(rect.x + rect.width, rect.y + rect.height, rect.x + rect.width - r, rect.y + rect.height);
        pb.line_to(rect.x + r, rect.y + rect.height);
        pb.quad_to(rect.x, rect.y + rect.height, rect.x, rect.y + rect.height - r);
        pb.line_to(rect.x, rect.y + r);
        pb.quad_to(rect.x, rect.y, rect.x + r, rect.y);
        pb.close();

        if let Some(path) = pb.finish() {
            let mut paint = Paint::default();
            paint.set_color_rgba8(color.r, color.g, color.b, (color.a as f32 * opacity) as u8);
            pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
        }
    }

    pub fn draw_border(
        &self,
        pixmap: &mut PixmapMut,
        rect: Rect,
        radius: f32,
        width: f32,
        color: Color,
    ) {
        let mut pb = PathBuilder::new();
        let r = radius.min(rect.width / 2.0).min(rect.height / 2.0);

        pb.move_to(rect.x + r, rect.y);
        pb.line_to(rect.x + rect.width - r, rect.y);
        pb.quad_to(rect.x + rect.width, rect.y, rect.x + rect.width, rect.y + r);
        pb.line_to(rect.x + rect.width, rect.y + rect.height - r);
        pb.quad_to(rect.x + rect.width, rect.y + rect.height, rect.x + rect.width - r, rect.y + rect.height);
        pb.line_to(rect.x + r, rect.y + rect.height);
        pb.quad_to(rect.x, rect.y + rect.height, rect.x, rect.y + rect.height - r);
        pb.line_to(rect.x, rect.y + r);
        pb.quad_to(rect.x, rect.y, rect.x + r, rect.y);
        pb.close();

        if let Some(path) = pb.finish() {
            let mut paint = Paint::default();
            paint.set_color_rgba8(color.r, color.g, color.b, color.a);
            let mut stroke = Stroke::default();
            stroke.width = width;
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_placeholder_and_asset_creation() {
        let pm = Renderer::create_placeholder_image(128, 128, "Expo App");
        let _ = std::fs::create_dir_all("examples/expo-app/assets");
        let _ = pm.save_png("examples/expo-app/assets/expo-icon.png");
        assert_eq!(pm.width(), 128);
        assert_eq!(pm.height(), 128);
    }
}
