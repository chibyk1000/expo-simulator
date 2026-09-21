use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, SwashCache, Weight};
use simulator_core::{Color, Rect, Size};

pub struct TextEngine {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
}

impl Default for TextEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TextEngine {
    pub fn new() -> Self {
        Self {
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
        }
    }

    pub fn measure_text(
        &mut self,
        text: &str,
        font_size: f32,
        line_height: Option<f32>,
        max_width: Option<f32>,
    ) -> Size {
        let lh = line_height.unwrap_or(font_size * 1.25);
        let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(font_size, lh));

        buffer.set_size(max_width, None);
        buffer.set_text(
            text,
            &Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(&mut self.font_system, false);

        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for run in buffer.layout_runs() {
            width = width.max(run.line_w);
            height += run.line_height;
        }

        Size::new(width.max(1.0), height.max(font_size))
    }

    pub fn render_text(
        &mut self,
        pixmap: &mut tiny_skia::PixmapMut,
        text: &str,
        bounds: Rect,
        font_size: f32,
        font_weight: &str,
        color: Color,
        max_width: Option<f32>,
    ) {
        let weight = match font_weight {
            "700" | "bold" => Weight::BOLD,
            "600" | "semibold" => Weight::SEMIBOLD,
            "500" | "medium" => Weight::MEDIUM,
            "300" | "light" => Weight::LIGHT,
            _ => Weight::NORMAL,
        };

        let lh = font_size * 1.25;
        let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(font_size, lh));

        buffer.set_size(max_width.or(Some(bounds.width)), None);
        buffer.set_text(
            text,
            &Attrs::new().family(Family::SansSerif).weight(weight),
            Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(&mut self.font_system, false);

        let text_color = cosmic_text::Color::rgba(color.r, color.g, color.b, color.a);
        let px_w = pixmap.width() as i32;
        let px_h = pixmap.height() as i32;

        buffer.draw(
            &mut self.font_system,
            &mut self.swash_cache,
            text_color,
            |x, y, w, h, col| {
                let dst_x = bounds.x as i32 + x;
                let dst_y = bounds.y as i32 + y;

                for py in 0..h as i32 {
                    for px in 0..w as i32 {
                        let target_x = dst_x + px;
                        let target_y = dst_y + py;

                        if target_x >= 0 && target_x < px_w && target_y >= 0 && target_y < px_h {
                            let idx = (target_y as usize * px_w as usize + target_x as usize) * 4;
                            let data = pixmap.data_mut();

                            let src_a = col.a() as f32 / 255.0;
                            let src_r = col.r() as f32;
                            let src_g = col.g() as f32;
                            let src_b = col.b() as f32;

                            let dst_r = data[idx] as f32;
                            let dst_g = data[idx + 1] as f32;
                            let dst_b = data[idx + 2] as f32;
                            let dst_a = data[idx + 3] as f32;

                            let out_a = src_a + (dst_a / 255.0) * (1.0 - src_a);
                            if out_a > 0.0 {
                                data[idx] = ((src_r * src_a + dst_r * (1.0 - src_a)).min(255.0)) as u8;
                                data[idx + 1] = ((src_g * src_a + dst_g * (1.0 - src_a)).min(255.0)) as u8;
                                data[idx + 2] = ((src_b * src_a + dst_b * (1.0 - src_a)).min(255.0)) as u8;
                                data[idx + 3] = ((out_a * 255.0).min(255.0)) as u8;
                            }
                        }
                    }
                }
            },
        );
    }
}
