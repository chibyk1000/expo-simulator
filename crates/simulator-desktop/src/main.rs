use simulator_bridge::{BridgeChannel, BridgeRequest};
use simulator_core::{Color, ComponentType, Node, NodeId, Point, Rect};
use simulator_device::profile::Orientation;
use simulator_device::DeviceManager;
use simulator_input::{InputEvent, InputHandler};
use simulator_renderer::{LayoutEngine, Renderer};
use simulator_runtime::{JsEngine, MetroClient, QuickJsEngine};
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tiny_skia::{Paint, PathBuilder, Pixmap, PixmapMut, Transform};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::window::{Window, WindowId};

#[derive(Debug)]
pub enum SimulatorEvent {
    MetroStatus(bool),
    FastRefreshUpdate,
    FileChanged,
    Reload,
}

fn find_project_app_file() -> Option<PathBuf> {
    let candidates = [
        "examples/expo-app/App.tsx",
        "examples/expo-app/App.jsx",
        "examples/expo-app/App.js",
        "App.tsx",
        "App.jsx",
        "App.js",
        "src/App.tsx",
        "src/App.jsx",
        "src/App.js",
    ];

    for c in candidates {
        let p = PathBuf::from(c);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn transpile_app_file(path: &Path) -> Option<String> {
    let script_candidates = [
        "packages/runtime/transpile.js",
        "../packages/runtime/transpile.js",
        "../../packages/runtime/transpile.js",
    ];

    // The npm launcher points us at its bundled copy via EXPO_SIM_TRANSPILE
    let env_script = std::env::var("EXPO_SIM_TRANSPILE").ok().filter(|p| Path::new(p).exists());
    let transpile_script = match env_script.as_deref() {
        Some(p) => p,
        None => *script_candidates.iter().find(|p| Path::new(p).exists())?,
    };

    let output = std::process::Command::new("node")
        .arg(transpile_script)
        .arg(path)
        .output()
        .ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout).ok()
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        log::warn!("Transpile error: {}", err);
        None
    }
}

#[derive(Debug, Clone)]
pub struct ConsoleLogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleTab {
    Console,
    Network,
}

#[derive(Debug, Clone)]
pub struct NetworkLogEntry {
    pub id: String,
    pub method: String,
    pub url: String,
    pub status: u16,
    pub duration_ms: u64,
    pub timestamp: String,
}

struct AppState {
    window: Option<Arc<Window>>,
    softbuffer_context: Option<softbuffer::Context<Arc<Window>>>,
    softbuffer_surface: Option<softbuffer::Surface<Arc<Window>, Arc<Window>>>,
    device_manager: DeviceManager,
    renderer: Renderer,
    bridge: BridgeChannel,
    input_handler: InputHandler,
    nodes: HashMap<NodeId, Node>,
    root_nodes: Vec<NodeId>,
    metro_client: MetroClient,
    metro_connected: bool,
    fast_refresh_count: u32,
    window_width: u32,
    window_height: u32,
    max_window: (u32, u32),
    cursor_position: Point,
    engine: QuickJsEngine,
    rt: Arc<tokio::runtime::Runtime>,
    _proxy: EventLoopProxy<SimulatorEvent>,
    active_pressed_node: Option<NodeId>,
    app_file: Option<PathBuf>,
    modifiers: ModifiersState,

    // Dev Menu State
    is_dev_menu_open: bool,
    dev_menu_rect: Rect,
    dev_menu_btn_rect: Rect,
    dev_menu_items: Vec<(String, Rect)>,

    // Element Inspector State
    is_inspector_active: bool,
    inspected_node_id: Option<NodeId>,
    inspector_btn_rect: Rect,

    // Orientation & Theme State
    rotate_btn_rect: Rect,
    theme_btn_rect: Rect,
    color_scheme: String,

    // Console & Network Window State
    is_console_open: bool,
    console_active_tab: ConsoleTab,
    console_logs: Vec<ConsoleLogEntry>,
    network_logs: Vec<NetworkLogEntry>,
    console_scroll_offset: usize,
    console_toggle_btn_rect: Rect,
    console_tab_btn_rect: Rect,
    network_tab_btn_rect: Rect,
    console_clear_btn_rect: Rect,
    console_close_btn_rect: Rect,
    console_drawer_rect: Rect,
    console_detached: bool,
    console_dock_btn_rect: Rect,
    last_desired_size: (u32, u32),
}

impl AppState {
    fn new(
        bridge: BridgeChannel,
        engine: QuickJsEngine,
        rt: Arc<tokio::runtime::Runtime>,
        proxy: EventLoopProxy<SimulatorEvent>,
        app_file: Option<PathBuf>,
    ) -> Self {
        let mut device_manager = DeviceManager::new();
        // Built-in profiles ship inside the binary; a local ./devices dir can add or override them
        for json in [
            include_str!("../../../devices/iphone-16-pro.json"),
            include_str!("../../../devices/iphone-16.json"),
            include_str!("../../../devices/iphone-se.json"),
            include_str!("../../../devices/pixel-9.json"),
            include_str!("../../../devices/pixel-9-pro.json"),
            include_str!("../../../devices/custom.json"),
        ] {
            device_manager.load_from_json(json);
        }
        let _ = device_manager.load_from_dir("devices");

        Self {
            window: None,
            softbuffer_context: None,
            softbuffer_surface: None,
            device_manager,
            renderer: Renderer::new(),
            input_handler: InputHandler::new(bridge.clone()),
            bridge,
            nodes: HashMap::new(),
            root_nodes: Vec::new(),
            metro_client: MetroClient::default(),
            metro_connected: false,
            fast_refresh_count: 0,
            window_width: 860,
            window_height: 1040,
            max_window: (u32::MAX, u32::MAX),
            cursor_position: Point::ZERO,
            engine,
            rt,
            _proxy: proxy,
            active_pressed_node: None,
            app_file,
            modifiers: ModifiersState::empty(),
            is_dev_menu_open: false,
            dev_menu_rect: Rect::ZERO,
            dev_menu_btn_rect: Rect::ZERO,
            dev_menu_items: Vec::new(),
            is_inspector_active: false,
            inspected_node_id: None,
            inspector_btn_rect: Rect::ZERO,
            rotate_btn_rect: Rect::ZERO,
            theme_btn_rect: Rect::ZERO,
            color_scheme: "dark".to_string(),
            is_console_open: false,
            console_active_tab: ConsoleTab::Console,
            console_logs: Vec::new(),
            network_logs: Vec::new(),
            console_scroll_offset: 0,
            console_toggle_btn_rect: Rect::ZERO,
            console_tab_btn_rect: Rect::ZERO,
            network_tab_btn_rect: Rect::ZERO,
            console_clear_btn_rect: Rect::ZERO,
            console_close_btn_rect: Rect::ZERO,
            console_drawer_rect: Rect::ZERO,
            console_detached: false,
            console_dock_btn_rect: Rect::ZERO,
            last_desired_size: (0, 0),
        }
    }

    fn add_console_log(&mut self, level: &str, message: &str) {
        let timestamp = if let Ok(duration) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            let secs = duration.as_secs();
            let h = (secs % 86400) / 3600;
            let m = (secs % 3600) / 60;
            let s = secs % 60;
            format!("{:02}:{:02}:{:02}", h, m, s)
        } else {
            "00:00:00".to_string()
        };

        self.console_logs.push(ConsoleLogEntry {
            timestamp,
            level: level.to_string(),
            message: message.to_string(),
        });

        if self.console_logs.len() > 500 {
            self.console_logs.remove(0);
        }

        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    fn toggle_device_orientation(&mut self) {
        let new_ori = self.device_manager.toggle_orientation();
        let (w, h) = match new_ori {
            Orientation::Portrait => (860, 1040),
            Orientation::Landscape => (1040, 860),
        };
        self.window_width = w.min(self.max_window.0);
        self.window_height = h.min(self.max_window.1);
        if let Some(surface) = self.softbuffer_surface.as_mut() {
            let _ = surface.resize(
                NonZeroU32::new(self.window_width).unwrap(),
                NonZeroU32::new(self.window_height).unwrap(),
            );
        }
        if let Some(w) = &self.window {
            let _ = w.request_inner_size(PhysicalSize::new(self.window_width, self.window_height));
            w.request_redraw();
        }
        let screen_rect = self.calculate_device_screen_rect();
        let density = self.device_manager.active_profile().density;
        let _ = self.engine.eval(&format!(
            "if (global.__turboModuleProxy) {{ var di = global.__turboModuleProxy('DeviceInfo'); if (di) {{ di.getConstants = function() {{ return {{ Dimensions: {{ window: {{ width: {}, height: {}, scale: {}, fontScale: 1.0 }}, screen: {{ width: {}, height: {}, scale: {}, fontScale: 1.0 }} }} }}; }}; }}",
            screen_rect.width, screen_rect.height, density, screen_rect.width, screen_rect.height, density
        ), "dimensions.js");
        self.add_console_log("system", &format!("📱 Device rotated to {:?}", new_ori));
    }

    fn toggle_color_scheme(&mut self) {
        self.color_scheme = if self.color_scheme == "dark" {
            "light".to_string()
        } else {
            "dark".to_string()
        };
        let _ = self.engine.eval(&format!(
            "if (global.Appearance && global.Appearance.setColorScheme) {{ global.Appearance.setColorScheme('{}'); }}",
            self.color_scheme
        ), "theme.js");
        self.add_console_log("system", &format!("🌓 Color scheme changed to {}", self.color_scheme));
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    fn hit_test_leaf_node(&self, point: Point, screen_rect: Rect) -> Option<NodeId> {
        if !screen_rect.contains(point) {
            return None;
        }
        let local_pt = Point::new(point.x - screen_rect.x, point.y - screen_rect.y);
        let mut best: Option<(NodeId, f32)> = None;

        for (id, node) in &self.nodes {
            let r = node.layout;
            if r.width > 0.0 && r.height > 0.0 {
                if local_pt.x >= r.x && local_pt.x <= r.x + r.width &&
                   local_pt.y >= r.y && local_pt.y <= r.y + r.height {
                    let area = r.width * r.height;
                    match best {
                        None => best = Some((*id, area)),
                        Some((_, best_area)) => {
                            if area <= best_area {
                                best = Some((*id, area));
                            }
                        }
                    }
                }
            }
        }
        best.map(|(id, _)| id)
    }

    fn inspect_node(&mut self, node_id: NodeId) {
        self.inspected_node_id = Some(node_id);
        if let Some(node) = self.nodes.get(&node_id) {
            let type_str = match &node.component_type {
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
            let info = format!(
                "🔍 Inspected <{}> #{:?}: layout [x:{:.0}, y:{:.0}, w:{:.0}, h:{:.0}] bg:{:?} pad:[{:.0},{:.0},{:.0},{:.0}]",
                type_str, node.id.0, node.layout.x, node.layout.y, node.layout.width, node.layout.height,
                node.style.background_color,
                node.style.padding.top, node.style.padding.right, node.style.padding.bottom, node.style.padding.left
            );
            log::info!("{}", info);
            self.add_console_log("info", &info);
        }
    }

    fn reload_bundle(&mut self) {
        self.nodes.clear();
        self.root_nodes.clear();

        if self.metro_connected {
            log::info!("⚡ Fetching updated bundle from Metro bundler...");
            let metro = self.metro_client.clone();
            let bundle_res = self.rt.block_on(async move {
                metro.fetch_bundle(None).await
            });
            match bundle_res {
                Ok(bundle) => {
                    log::info!("⚡ Evaluating Metro bundle ({} bytes)...", bundle.len());
                    let _ = self.engine.eval(&bundle, "index.bundle.js");
                }
                Err(e) => {
                    log::warn!("Failed to fetch bundle from Metro: {}. Falling back to local app.", e);
                    self.load_local_or_reference_app();
                }
            }
        } else {
            self.load_local_or_reference_app();
        }

        self.process_bridge_messages();
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    fn load_local_or_reference_app(&mut self) {
        if let Some(ref path) = self.app_file {
            log::info!("⚡ Transpiling and loading local app from {:?}...", path);
            if let Some(bundle) = transpile_app_file(path) {
                if let Err(e) = self.engine.eval(&bundle, "App.tsx") {
                    log::warn!("Evaluation of {:?} failed: {:?}. Falling back to reference app.", path, e);
                    self.load_reference_app();
                }
                return;
            }
        }
        self.load_reference_app();
    }

    fn load_reference_app(&mut self) {
        let sample_script = r##"
        (function() {
            var ui = global.nativeFabricUIManager;

            // Root ScrollView
            var root = ui.createNode(1, "ScrollView", 1, {
                style: {
                    flex: 1,
                    backgroundColor: "#0D1117"
                }
            });

            // ── Header card ──────────────────────────────────────────────
            var header = ui.createNode(10, "View", 1, {
                style: {
                    backgroundColor: "#1E1B4B",
                    padding: 24,
                    paddingTop: 68,
                    marginBottom: 4
                }
            });

            var headerTitle = ui.createNode(11, "Text", 1, {
                text: "Expo Simulator",
                style: {
                    color: "#E0E7FF",
                    fontSize: 26,
                    fontWeight: "700",
                    marginBottom: 6
                }
            });

            var headerSub = ui.createNode(12, "Text", 1, {
                text: "React Native runtime • Ready to render",
                style: {
                    color: "#818CF8",
                    fontSize: 13,
                    fontWeight: "400"
                }
            });

            ui.appendChild(header, headerTitle);
            ui.appendChild(header, headerSub);

            // ── Stats row ─────────────────────────────────────────────────
            var statsRow = ui.createNode(20, "View", 1, {
                style: {
                    flexDirection: "row",
                    padding: 16,
                    paddingTop: 20,
                    paddingBottom: 4
                }
            });

            // Card 1 — Render
            var card1 = ui.createNode(21, "View", 1, {
                style: {
                    flex: 1,
                    backgroundColor: "#111827",
                    borderRadius: 14,
                    padding: 16,
                    marginRight: 8
                }
            });
            var card1Val = ui.createNode(22, "Text", 1, {
                text: "60fps",
                style: { color: "#34D399", fontSize: 20, fontWeight: "700", marginBottom: 4 }
            });
            var card1Lbl = ui.createNode(23, "Text", 1, {
                text: "Render rate",
                style: { color: "#6B7280", fontSize: 11 }
            });
            ui.appendChild(card1, card1Val);
            ui.appendChild(card1, card1Lbl);

            // Card 2 — Layout
            var card2 = ui.createNode(24, "View", 1, {
                style: {
                    flex: 1,
                    backgroundColor: "#111827",
                    borderRadius: 14,
                    padding: 16,
                    marginRight: 8
                }
            });
            var card2Val = ui.createNode(25, "Text", 1, {
                text: "Yoga",
                style: { color: "#60A5FA", fontSize: 20, fontWeight: "700", marginBottom: 4 }
            });
            var card2Lbl = ui.createNode(26, "Text", 1, {
                text: "Layout engine",
                style: { color: "#6B7280", fontSize: 11 }
            });
            ui.appendChild(card2, card2Val);
            ui.appendChild(card2, card2Lbl);

            // Card 3 — JS
            var card3 = ui.createNode(27, "View", 1, {
                style: {
                    flex: 1,
                    backgroundColor: "#111827",
                    borderRadius: 14,
                    padding: 16
                }
            });
            var card3Val = ui.createNode(28, "Text", 1, {
                text: "QuickJS",
                style: { color: "#A78BFA", fontSize: 20, fontWeight: "700", marginBottom: 4 }
            });
            var card3Lbl = ui.createNode(29, "Text", 1, {
                text: "JS engine",
                style: { color: "#6B7280", fontSize: 11 }
            });
            ui.appendChild(card3, card3Val);
            ui.appendChild(card3, card3Lbl);

            ui.appendChild(statsRow, card1);
            ui.appendChild(statsRow, card2);
            ui.appendChild(statsRow, card3);

            // ── Section label ─────────────────────────────────────────────
            var section = ui.createNode(30, "Text", 1, {
                text: "COMPONENTS",
                style: {
                    color: "#374151",
                    fontSize: 11,
                    fontWeight: "700",
                    marginLeft: 20,
                    marginTop: 20,
                    marginBottom: 10,
                    letterSpacing: 2
                }
            });

            // ── TextInput ─────────────────────────────────────────────────
            var inputWrapper = ui.createNode(40, "View", 1, {
                style: {
                    marginLeft: 16, marginRight: 16, marginBottom: 12
                }
            });
            var input = ui.createNode(41, "TextInput", 1, {
                placeholder: "Search components...",
                placeholderTextColor: "#4B5563",
                style: {
                    backgroundColor: "#111827",
                    color: "#F9FAFB",
                    padding: 14,
                    borderRadius: 12,
                    fontSize: 15
                }
            });
            ui.appendChild(inputWrapper, input);

            // ── Pressable CTA button ──────────────────────────────────────
            var btnWrapper = ui.createNode(50, "View", 1, {
                style: {
                    marginLeft: 16, marginRight: 16, marginBottom: 12
                }
            });
            var button = ui.createNode(51, "Pressable", 1, {
                style: {
                    backgroundColor: "#4F46E5",
                    padding: 16,
                    borderRadius: 14
                }
            });
            var buttonText = ui.createNode(52, "Text", 1, {
                text: "Open Developer Console",
                style: {
                    color: "#E0E7FF",
                    textAlign: "center",
                    fontWeight: "700",
                    fontSize: 15
                }
            });
            ui.appendChild(button, buttonText);
            ui.appendChild(btnWrapper, button);

            // ── Secondary button ──────────────────────────────────────────
            var btn2Wrapper = ui.createNode(60, "View", 1, {
                style: {
                    marginLeft: 16, marginRight: 16
                }
            });
            var button2 = ui.createNode(61, "Pressable", 1, {
                style: {
                    backgroundColor: "#111827",
                    padding: 16,
                    borderRadius: 14
                }
            });
            var btn2Text = ui.createNode(62, "Text", 1, {
                text: "Rotate Device  ( press o )",
                style: {
                    color: "#9CA3AF",
                    textAlign: "center",
                    fontWeight: "600",
                    fontSize: 14
                }
            });
            ui.appendChild(button2, btn2Text);
            ui.appendChild(btn2Wrapper, button2);

            // ── Assemble ──────────────────────────────────────────────────
            ui.appendChild(root, header);
            ui.appendChild(root, statsRow);
            ui.appendChild(root, section);
            ui.appendChild(root, inputWrapper);
            ui.appendChild(root, btnWrapper);
            ui.appendChild(root, btn2Wrapper);
            ui.completeRoot(1, [root]);
        })();
        "##;
        let _ = self.engine.eval(sample_script, "sample.js");
    }

    const DETACHED_CONSOLE_W: f32 = 440.0;
    const DETACHED_GAP: f32 = 24.0;

    fn console_is_detached(&self) -> bool {
        self.is_console_open && self.console_detached
    }

    /// Extra horizontal space taken by a console panel that sits beside the phone.
    fn console_side_extra(&self) -> f32 {
        if self.console_is_detached() { Self::DETACHED_CONSOLE_W + Self::DETACHED_GAP } else { 0.0 }
    }

    fn rounded_path(x: f32, y: f32, w: f32, h: f32, r: f32) -> Option<tiny_skia::Path> {
        let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
        let mut pb = PathBuilder::new();
        pb.move_to(x + r, y);
        pb.line_to(x + w - r, y);
        pb.quad_to(x + w, y, x + w, y + r);
        pb.line_to(x + w, y + h - r);
        pb.quad_to(x + w, y + h, x + w - r, y + h);
        pb.line_to(x + r, y + h);
        pb.quad_to(x, y + h, x, y + h - r);
        pb.line_to(x, y + r);
        pb.quad_to(x, y, x + r, y);
        pb.close();
        pb.finish()
    }

    /// Composite `layer` onto `pixmap`, clipped to a rounded rect.
    fn composite_clipped(pixmap: &mut Pixmap, mut layer: Pixmap, rect: Rect, radius: f32) {
        if let (Some(path), Some(mut mask)) = (
            Self::rounded_path(rect.x, rect.y, rect.width, rect.height, radius),
            tiny_skia::Mask::new(layer.width(), layer.height()),
        ) {
            mask.fill_path(&path, tiny_skia::FillRule::Winding, true, Transform::identity());
            layer.apply_mask(&mask);
        }
        pixmap.draw_pixmap(0, 0, layer.as_ref(), &tiny_skia::PixmapPaint::default(), Transform::identity(), None);
    }

    /// Scale that shrinks the whole UI so the device and title bar always fit the window.
    fn ui_scale(&self) -> f32 {
        let size = self.device_manager.active_profile().screen_size(self.device_manager.orientation());
        let need_w = (size.width + 64.0 + self.console_side_extra()).max(640.0);
        let need_h = size.height + 72.0 + 32.0;
        (self.window_width as f32 / need_w)
            .min(self.window_height as f32 / need_h)
            .min(1.0)
            .max(0.1)
    }

    fn virtual_size(&self) -> (u32, u32) {
        let s = self.ui_scale();
        ((self.window_width as f32 / s).ceil() as u32, (self.window_height as f32 / s).ceil() as u32)
    }

    fn calculate_device_screen_rect(&self) -> Rect {
        let profile = self.device_manager.active_profile();
        let orientation = self.device_manager.orientation();
        let screen_size = profile.screen_size(orientation);

        // Center simulated phone in desktop window, below the toolbar
        let group_w = screen_size.width + self.console_side_extra();
        let x = ((self.virtual_size().0 as f32 - group_w) / 2.0).max(0.0);
        let y = 72.0; // below the title bar + padding
        Rect::new(x, y, screen_size.width, screen_size.height)
    }

    /// Fill a rounded rect with a solid RGBA color.
    fn fill_rrect(pixmap: &mut PixmapMut, x: f32, y: f32, w: f32, h: f32, r: f32, rgba: (u8, u8, u8, u8)) {
        let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
        let mut pb = PathBuilder::new();
        pb.move_to(x + r, y);
        pb.line_to(x + w - r, y);
        pb.quad_to(x + w, y, x + w, y + r);
        pb.line_to(x + w, y + h - r);
        pb.quad_to(x + w, y + h, x + w - r, y + h);
        pb.line_to(x + r, y + h);
        pb.quad_to(x, y + h, x, y + h - r);
        pb.line_to(x, y + r);
        pb.quad_to(x, y, x + r, y);
        pb.close();
        if let Some(path) = pb.finish() {
            let mut paint = Paint::default();
            paint.set_color_rgba8(rgba.0, rgba.1, rgba.2, rgba.3);
            paint.anti_alias = true;
            pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
        }
    }

    fn draw_device_chrome(&self, pixmap: &mut PixmapMut, screen_rect: Rect) {
        let profile = self.device_manager.active_profile();
        let orientation = self.device_manager.orientation();
        let portrait = orientation == Orientation::Portrait;
        let corner_radius = profile.corner_radius;
        let bezel = if corner_radius > 0.0 { 12.0_f32 } else { 40.0_f32 };

        let bx = screen_rect.x - bezel;
        let by = screen_rect.y - bezel;
        let bw = screen_rect.width + bezel * 2.0;
        let bh = screen_rect.height + bezel * 2.0;
        let br = corner_radius + bezel * 0.9;

        // Side buttons (portrait only): action + volume on the left, power on the right
        if portrait {
            let btn = (70, 70, 74, 255);
            let rail = 3.0_f32;
            Self::fill_rrect(pixmap, bx - rail, by + bh * 0.13, rail + 2.0, 26.0, 1.5, btn); // action
            Self::fill_rrect(pixmap, bx - rail, by + bh * 0.20, rail + 2.0, 50.0, 1.5, btn); // volume up
            Self::fill_rrect(pixmap, bx - rail, by + bh * 0.28, rail + 2.0, 50.0, 1.5, btn); // volume down
            Self::fill_rrect(pixmap, bx + bw - 2.0, by + bh * 0.24, rail + 2.0, 80.0, 1.5, btn); // power
        }

        // Drop shadow
        Self::fill_rrect(pixmap, bx - 2.0, by + 6.0, bw + 4.0, bh + 4.0, br + 2.0, (0, 0, 0, 70));
        // Titanium outer rim, then inner body
        Self::fill_rrect(pixmap, bx, by, bw, bh, br, (108, 108, 114, 255));
        Self::fill_rrect(pixmap, bx + 1.5, by + 1.5, bw - 3.0, bh - 3.0, br - 1.5, (18, 18, 20, 255));
        // Screen well
        Self::fill_rrect(pixmap, screen_rect.x, screen_rect.y, screen_rect.width, screen_rect.height, corner_radius, (0, 0, 0, 255));
    }

    /// Chrome drawn above the app content: Dynamic Island, iOS status bar, home indicator.
    fn draw_device_overlay(&self, pixmap: &mut PixmapMut, screen_rect: Rect) {
        let profile = self.device_manager.active_profile();
        let portrait = self.device_manager.orientation() == Orientation::Portrait;
        let fg = if self.color_scheme == "dark" { Color::WHITE } else { Color::rgb(0, 0, 0) };
        let (fr, fg_g, fb) = (fg.r, fg.g, fg.b);
        let mut te = simulator_renderer::TextEngine::new();

        // Dynamic Island / punch-hole
        let mut has_island = false;
        if let Some(ref cutout) = profile.camera_cutout {
            if cutout.cutout_type == "dynamic-island" {
                has_island = true;
                let (w, h) = (cutout.radius * 6.6, cutout.radius * 2.0);
                let (cx, cy) = if portrait {
                    (screen_rect.x + cutout.x, screen_rect.y + cutout.y)
                } else {
                    (screen_rect.x + cutout.y, screen_rect.y + screen_rect.height / 2.0)
                };
                let (pw, ph) = if portrait { (w, h) } else { (h, w) };
                Self::fill_rrect(pixmap, cx - pw / 2.0, cy - ph / 2.0, pw, ph, h / 2.0, (0, 0, 0, 255));
                // Faint camera lens
                let (lx, ly) = if portrait { (cx + w * 0.3, cy) } else { (cx, cy + w * 0.3) };
                Self::fill_rrect(pixmap, lx - 5.0, ly - 5.0, 10.0, 10.0, 5.0, (16, 20, 34, 255));
            } else {
                let (cx, cy) = (screen_rect.x + cutout.x, screen_rect.y + cutout.y);
                Self::fill_rrect(pixmap, cx - cutout.radius, cy - cutout.radius, cutout.radius * 2.0, cutout.radius * 2.0, cutout.radius, (10, 10, 10, 255));
            }
        }

        // iOS status bar (portrait only)
        if portrait {
            let bar_h = profile.safe_area.top.min(62.0);
            let mid_y = if has_island { screen_rect.y + 30.0 } else { screen_rect.y + bar_h / 2.0 };
            let ty = mid_y - 9.0;
            let (left_cx, right_cx) = if has_island {
                (screen_rect.x + 52.0, screen_rect.x + screen_rect.width - 76.0)
            } else {
                (screen_rect.x + screen_rect.width / 2.0, screen_rect.x + screen_rect.width - 60.0)
            };

            te.render_text(pixmap, "9:41", Rect::new(left_cx - 24.0, ty, 48.0, 18.0), 15.5, "bold", fg, None);

            // Cellular bars
            let sx = right_cx - 46.0;
            for i in 0..4_u32 {
                let bh = 4.0 + i as f32 * 2.5;
                Self::fill_rrect(pixmap, sx + i as f32 * 4.5, mid_y + 5.0 - bh, 3.0, bh, 0.8, (fr, fg_g, fb, 255));
            }
            // Wi-Fi (three arcs approximated by stacked bars)
            let wx = sx + 24.0;
            for i in 0..3_u32 {
                let w = 4.0 + i as f32 * 3.5;
                Self::fill_rrect(pixmap, wx + (10.5 - w) / 2.0, mid_y + 2.0 - i as f32 * 3.0, w, 2.2, 1.1, (fr, fg_g, fb, 255));
            }
            // Battery
            let batx = right_cx + 2.0;
            Self::fill_rrect(pixmap, batx, mid_y - 5.5, 25.0, 12.0, 3.5, (fr, fg_g, fb, 90));
            Self::fill_rrect(pixmap, batx + 1.5, mid_y - 4.0, 19.0, 9.0, 2.2, (fr, fg_g, fb, 255));
            Self::fill_rrect(pixmap, batx + 26.0, mid_y - 1.5, 1.6, 4.5, 0.8, (fr, fg_g, fb, 100));
        }

        // Home indicator
        if profile.safe_area.bottom > 0.0 {
            let (nav_w, nav_h) = if portrait { (134.0_f32, 5.0_f32) } else { (134.0, 5.0) };
            Self::fill_rrect(
                pixmap,
                screen_rect.x + (screen_rect.width - nav_w) / 2.0,
                screen_rect.y + screen_rect.height - nav_h - 8.0,
                nav_w, nav_h, nav_h / 2.0,
                (fr, fg_g, fb, 230),
            );
        }
    }

    /// Draw a flat pill-shaped toolbar button with a hairline border.
    fn draw_toolbar_btn(
        &self,
        pixmap: &mut PixmapMut,
        te: &mut simulator_renderer::TextEngine,
        rect: Rect,
        label: &str,
        bg_r: u8, bg_g: u8, bg_b: u8,
        border: (u8, u8, u8),
        text_color: Color,
    ) {
        // Background
        self.renderer.draw_rounded_rect(
            pixmap,
            rect,
            6.0,
            Color::rgb(bg_r, bg_g, bg_b),
            1.0,
        );
        // Border
        self.renderer.draw_border(pixmap, rect, 6.0, 1.0, Color::rgb(border.0, border.1, border.2));
        // Label — centered vertically
        te.render_text(
            pixmap,
            label,
            Rect::new(rect.x + 8.0, rect.y + (rect.height - 14.0) / 2.0, rect.width - 10.0, 14.0),
            10.5,
            "normal",
            text_color,
            None,
        );
    }
    fn render(&mut self) {
        // Grow the window when the console is detached beside the phone
        {
            let (bw, bh) = match self.device_manager.orientation() {
                Orientation::Portrait => (860.0, 1040.0),
                Orientation::Landscape => (1040.0, 860.0),
            };
            let desired = (
                ((bw + self.console_side_extra()) as u32).min(self.max_window.0),
                (bh as u32).min(self.max_window.1),
            );
            if desired != self.last_desired_size {
                self.last_desired_size = desired;
                if let Some(w) = &self.window {
                    let _ = w.request_inner_size(PhysicalSize::new(desired.0, desired.1));
                }
            }
        }

        let scale = self.ui_scale();
        let (width, height) = self.virtual_size();

        let mut pixmap = match Pixmap::new(width, height) {
            Some(p) => p,
            None => return,
        };

        // ── 1. Desktop background: light warm gray ─────────────────────────
        {
            if let Some(bg) = tiny_skia::Rect::from_xywh(0.0, 0.0, width as f32, height as f32) {
                let mut paint = Paint::default();
                // macOS-like window background
                if self.color_scheme == "dark" {
                    paint.set_color_rgba8(28, 28, 30, 255);  // iOS dark system bg
                } else {
                    paint.set_color_rgba8(242, 242, 247, 255); // iOS light system bg
                }
                pixmap.fill_rect(bg, &paint, Transform::identity(), None);
            }
        }

        let mut text_engine = simulator_renderer::TextEngine::new();

        let is_dark = self.color_scheme == "dark";
        let toolbar_text    = if is_dark { Color::rgb(242, 242, 247) } else { Color::rgb(28, 28, 30) };
        let toolbar_subtext = if is_dark { Color::rgb(174, 174, 178) } else { Color::rgb(142, 142, 147) };
        let toolbar_bg_rgb  : (u8,u8,u8) = if is_dark { (44, 44, 46) } else { (255, 255, 255) };
        let toolbar_border  : (u8,u8,u8) = if is_dark { (58, 58, 60) } else { (209, 209, 214) };
        let btn_bg_rgb      : (u8,u8,u8) = if is_dark { (58, 58, 60) } else { (242, 242, 247) };
        let btn_border_rgb  : (u8,u8,u8) = if is_dark { (72, 72, 74) } else { (209, 209, 214) };
        let btn_text        = if is_dark { Color::rgb(242, 242, 247) } else { Color::rgb(28, 28, 30) };

        // ── 2. Top Toolbar ──────────────────────────────────────────────────
        let toolbar_h = 44.0_f32;

        // Toolbar background
        if let Some(tb) = tiny_skia::Rect::from_xywh(0.0, 0.0, width as f32, toolbar_h) {
            let mut paint = Paint::default();
            paint.set_color_rgba8(toolbar_bg_rgb.0, toolbar_bg_rgb.1, toolbar_bg_rgb.2, 255);
            pixmap.fill_rect(tb, &paint, Transform::identity(), None);
        }
        // Bottom hairline border
        if let Some(border) = tiny_skia::Rect::from_xywh(0.0, toolbar_h - 1.0, width as f32, 1.0) {
            let mut paint = Paint::default();
            paint.set_color_rgba8(toolbar_border.0, toolbar_border.1, toolbar_border.2, 255);
            pixmap.fill_rect(border, &paint, Transform::identity(), None);
        }

        // Traffic lights
        for (i, c) in [(255_u8, 95_u8, 87_u8), (254, 188, 46), (40, 200, 64)].iter().enumerate() {
            Self::fill_rrect(&mut pixmap.as_mut(), 16.0 + i as f32 * 20.0, toolbar_h / 2.0 - 6.0, 12.0, 12.0, 6.0, (c.0, c.1, c.2, 255));
        }

        // Device name + iOS version
        let profile_name = self.device_manager.active_profile().name.clone();
        text_engine.render_text(
            &mut pixmap.as_mut(),
            &profile_name,
            Rect::new(84.0, 6.0, 150.0, 16.0),
            12.5,
            "bold",
            toolbar_text,
            None,
        );
        text_engine.render_text(
            &mut pixmap.as_mut(),
            "iOS 18.0",
            Rect::new(84.0, 23.0, 150.0, 14.0),
            10.5,
            "normal",
            toolbar_subtext,
            None,
        );

        // Status indicator — dot + text
        let dot_color = if self.metro_connected {
            Color::rgb(52, 199, 89)   // iOS green
        } else if self.app_file.is_some() {
            Color::rgb(255, 159, 10)  // iOS orange
        } else {
            Color::rgb(142, 142, 147) // iOS gray
        };
        let status_text = if self.metro_connected {
            "Metro"
        } else if self.app_file.is_some() {
            "Hot Reload"
        } else {
            "Local"
        };

        // Status dot
        {
            let mut pb = PathBuilder::new();
            pb.push_circle(252.0, toolbar_h / 2.0, 4.0);
            if let Some(path) = pb.finish() {
                let mut paint = Paint::default();
                paint.set_color_rgba8(dot_color.r, dot_color.g, dot_color.b, 255);
                paint.anti_alias = true;
                pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
            }
        }
        text_engine.render_text(
            &mut pixmap.as_mut(),
            status_text,
            Rect::new(262.0, (toolbar_h / 2.0) - 8.0, 72.0, 16.0),
            11.0,
            "normal",
            toolbar_subtext,
            None,
        );

        // ── Toolbar buttons (right-aligned) ─────────────────────────────────
        let btn_h = 26.0_f32;
        let btn_y = (toolbar_h - btn_h) / 2.0;
        let btn_gap = 5.0_f32;
        let right_x = (width as f32) - 12.0;

        // Helper closure params: store button rects
        // Console
        let console_w = 96.0_f32;
        let console_x = right_x - console_w;
        self.console_toggle_btn_rect = Rect::new(console_x, btn_y, console_w, btn_h);
        let (cb_r, cb_g, cb_b) = if self.is_console_open {
            (0_u8, 122_u8, 255_u8) // iOS blue active
        } else {
            btn_bg_rgb
        };
        let c_text = if self.is_console_open { Color::WHITE } else { btn_text };
        self.draw_toolbar_btn(&mut pixmap.as_mut(), &mut text_engine, self.console_toggle_btn_rect,
            &format!("Console ({})", self.console_logs.len()), cb_r, cb_g, cb_b, btn_border_rgb, c_text);

        // Theme
        let theme_w = 52.0_f32;
        let theme_x = console_x - btn_gap - theme_w;
        self.theme_btn_rect = Rect::new(theme_x, btn_y, theme_w, btn_h);
        let theme_lbl = if is_dark { "Dark" } else { "Light" };
        self.draw_toolbar_btn(&mut pixmap.as_mut(), &mut text_engine, self.theme_btn_rect,
            theme_lbl, btn_bg_rgb.0, btn_bg_rgb.1, btn_bg_rgb.2, btn_border_rgb, btn_text);

        // Rotate
        let rotate_w = 52.0_f32;
        let rotate_x = theme_x - btn_gap - rotate_w;
        self.rotate_btn_rect = Rect::new(rotate_x, btn_y, rotate_w, btn_h);
        self.draw_toolbar_btn(&mut pixmap.as_mut(), &mut text_engine, self.rotate_btn_rect,
            "Rotate", btn_bg_rgb.0, btn_bg_rgb.1, btn_bg_rgb.2, btn_border_rgb, btn_text);

        // Inspect
        let inspect_w = 56.0_f32;
        let inspect_x = rotate_x - btn_gap - inspect_w;
        self.inspector_btn_rect = Rect::new(inspect_x, btn_y, inspect_w, btn_h);
        let (ib_r, ib_g, ib_b) = if self.is_inspector_active {
            (0_u8, 122_u8, 255_u8)
        } else {
            btn_bg_rgb
        };
        let i_text = if self.is_inspector_active { Color::WHITE } else { btn_text };
        self.draw_toolbar_btn(&mut pixmap.as_mut(), &mut text_engine, self.inspector_btn_rect,
            "Inspect", ib_r, ib_g, ib_b, btn_border_rgb, i_text);

        // Dev Menu
        let dev_w = 76.0_f32;
        let dev_x = inspect_x - btn_gap - dev_w;
        self.dev_menu_btn_rect = Rect::new(dev_x, btn_y, dev_w, btn_h);
        let (db_r, db_g, db_b) = if self.is_dev_menu_open {
            (0_u8, 122_u8, 255_u8)
        } else {
            btn_bg_rgb
        };
        let d_text = if self.is_dev_menu_open { Color::WHITE } else { btn_text };
        self.draw_toolbar_btn(&mut pixmap.as_mut(), &mut text_engine, self.dev_menu_btn_rect,
            "Dev Menu", db_r, db_g, db_b, btn_border_rgb, d_text);

        // ── 3. Device screen ────────────────────────────────────────────────
        let screen_rect = self.calculate_device_screen_rect();
        self.draw_device_chrome(&mut pixmap.as_mut(), screen_rect);

        LayoutEngine::calculate_tree_layout(
            &mut self.nodes,
            &self.root_nodes,
            screen_rect.width,
            screen_rect.height,
        );

        let mut screen_pixmap = match Pixmap::new(screen_rect.width as u32, screen_rect.height as u32) {
            Some(sp) => sp,
            None => return,
        };
        self.renderer.render_tree(
            &mut screen_pixmap.as_mut(),
            &self.nodes,
            &self.root_nodes,
            Rect::from_point_size(Point::ZERO, screen_rect.size()),
        );
        // Clip app content to the screen's rounded corners
        let corner = self.device_manager.active_profile().corner_radius;
        if corner > 0.0 {
            let (sw, sh) = (screen_pixmap.width() as f32, screen_pixmap.height() as f32);
            let r = corner.min(sw / 2.0).min(sh / 2.0);
            let mut pb = PathBuilder::new();
            pb.move_to(r, 0.0);
            pb.line_to(sw - r, 0.0);
            pb.quad_to(sw, 0.0, sw, r);
            pb.line_to(sw, sh - r);
            pb.quad_to(sw, sh, sw - r, sh);
            pb.line_to(r, sh);
            pb.quad_to(0.0, sh, 0.0, sh - r);
            pb.line_to(0.0, r);
            pb.quad_to(0.0, 0.0, r, 0.0);
            pb.close();
            if let (Some(path), Some(mut mask)) = (pb.finish(), tiny_skia::Mask::new(screen_pixmap.width(), screen_pixmap.height())) {
                mask.fill_path(&path, tiny_skia::FillRule::Winding, true, Transform::identity());
                screen_pixmap.apply_mask(&mask);
            }
        }
        pixmap.draw_pixmap(
            screen_rect.x as i32,
            screen_rect.y as i32,
            screen_pixmap.as_ref(),
            &tiny_skia::PixmapPaint::default(),
            Transform::identity(),
            None,
        );

        self.draw_device_overlay(&mut pixmap.as_mut(), screen_rect);

        if self.is_inspector_active {
            if let Some(nid) = self.inspected_node_id {
                if let Some(node) = self.nodes.get(&nid) {
                    self.renderer.render_inspector_overlay(
                        &mut pixmap.as_mut(),
                        node,
                        screen_rect.origin(),
                        Point::ZERO,
                    );
                }
            }
        }

        // ── 4. Console Drawer ───────────────────────────────────────────────
        if self.is_console_open {
            let detached = self.console_detached;
            let (drawer_x, drawer_y, drawer_w, drawer_h) = if detached {
                (
                    screen_rect.x + screen_rect.width + 12.0 + Self::DETACHED_GAP,
                    screen_rect.y - 12.0,
                    Self::DETACHED_CONSOLE_W - 12.0,
                    screen_rect.height + 24.0,
                )
            } else {
                (screen_rect.x, screen_rect.y + screen_rect.height - 320.0, screen_rect.width, 320.0)
            };
            self.console_drawer_rect = Rect::new(drawer_x, drawer_y, drawer_w, drawer_h);

            // Draw the drawer on its own layer so it can be clipped to rounded corners
            let mut layer = match Pixmap::new(width, height) {
                Some(l) => l,
                None => return,
            };
            std::mem::swap(&mut pixmap, &mut layer);

            // Drawer background — same as toolbar
            if let Some(dr) = tiny_skia::Rect::from_xywh(drawer_x, drawer_y, drawer_w, drawer_h) {
                let mut paint = Paint::default();
                match (is_dark, detached) {
                    (true, false) => paint.set_color_rgba8(28, 28, 30, 252),
                    (true, true) => paint.set_color_rgba8(20, 20, 22, 255),
                    (false, false) => paint.set_color_rgba8(249, 249, 249, 252),
                    (false, true) => paint.set_color_rgba8(255, 255, 255, 255),
                }
                pixmap.fill_rect(dr, &paint, Transform::identity(), None);
            }

            // Header row (tabs + controls)
            if let Some(hr) = tiny_skia::Rect::from_xywh(drawer_x, drawer_y, drawer_w, 38.0) {
                let mut paint = Paint::default();
                paint.set_color_rgba8(toolbar_bg_rgb.0, toolbar_bg_rgb.1, toolbar_bg_rgb.2, 255);
                pixmap.fill_rect(hr, &paint, Transform::identity(), None);
            }
            if let Some(hb) = tiny_skia::Rect::from_xywh(drawer_x, drawer_y + 38.0, drawer_w, 1.0) {
                let mut paint = Paint::default();
                paint.set_color_rgba8(toolbar_border.0, toolbar_border.1, toolbar_border.2, 255);
                pixmap.fill_rect(hb, &paint, Transform::identity(), None);
            }

            // Tabs
            let tab_h = 24.0_f32;
            let tab_y = drawer_y + 7.0;
            let tab1_w = 90.0_f32;
            let tab2_w = 96.0_f32;
            let tab1_x = drawer_x + 10.0;
            let tab2_x = tab1_x + tab1_w + 4.0;
            self.console_tab_btn_rect = Rect::new(tab1_x, tab_y, tab1_w, tab_h);
            self.network_tab_btn_rect = Rect::new(tab2_x, tab_y, tab2_w, tab_h);

            let (t1_r, t1_g, t1_b) = if self.console_active_tab == ConsoleTab::Console {
                (0_u8, 122_u8, 255_u8)
            } else {
                btn_bg_rgb
            };
            let t1_txt = if self.console_active_tab == ConsoleTab::Console { Color::WHITE } else { btn_text };
            self.draw_toolbar_btn(&mut pixmap.as_mut(), &mut text_engine,
                self.console_tab_btn_rect, &format!("Logs ({})", self.console_logs.len()),
                t1_r, t1_g, t1_b, btn_border_rgb, t1_txt);

            let (t2_r, t2_g, t2_b) = if self.console_active_tab == ConsoleTab::Network {
                (0_u8, 122_u8, 255_u8)
            } else {
                btn_bg_rgb
            };
            let t2_txt = if self.console_active_tab == ConsoleTab::Network { Color::WHITE } else { btn_text };
            self.draw_toolbar_btn(&mut pixmap.as_mut(), &mut text_engine,
                self.network_tab_btn_rect, &format!("Network ({})", self.network_logs.len()),
                t2_r, t2_g, t2_b, btn_border_rgb, t2_txt);

            // Clear + Close
            let clear_w = 46.0_f32;
            let clear_x = drawer_x + drawer_w - clear_w - 36.0;
            let clear_y = tab_y;
            self.console_clear_btn_rect = Rect::new(clear_x, clear_y, clear_w, tab_h);
            self.draw_toolbar_btn(&mut pixmap.as_mut(), &mut text_engine,
                self.console_clear_btn_rect, "Clear",
                btn_bg_rgb.0, btn_bg_rgb.1, btn_bg_rgb.2, btn_border_rgb, btn_text);

            let dock_w = 58.0_f32;
            self.console_dock_btn_rect = Rect::new(clear_x - 4.0 - dock_w, clear_y, dock_w, tab_h);
            self.draw_toolbar_btn(&mut pixmap.as_mut(), &mut text_engine,
                self.console_dock_btn_rect, if detached { "Dock" } else { "Detach" },
                btn_bg_rgb.0, btn_bg_rgb.1, btn_bg_rgb.2, btn_border_rgb, btn_text);

            let close_w = 26.0_f32;
            let close_x = drawer_x + drawer_w - close_w - 6.0;
            self.console_close_btn_rect = Rect::new(close_x, clear_y, close_w, tab_h);
            self.draw_toolbar_btn(&mut pixmap.as_mut(), &mut text_engine,
                self.console_close_btn_rect, "x",
                btn_bg_rgb.0, btn_bg_rgb.1, btn_bg_rgb.2, btn_border_rgb, btn_text);

            // Log content
            let content_y = drawer_y + 44.0;
            let row_h = 22.0_f32;
            let max_visible = (((drawer_h - 48.0) / row_h).floor() as usize).max(1);

            let log_text_color = if is_dark { Color::rgb(235, 235, 240) } else { Color::rgb(28, 28, 30) };
            let ts_color       = if is_dark { Color::rgb(142, 142, 147) } else { Color::rgb(174, 174, 178) };

            if self.console_active_tab == ConsoleTab::Console {
                if self.console_logs.is_empty() {
                    text_engine.render_text(&mut pixmap.as_mut(),
                        "No logs yet — call console.log() in your app.",
                        Rect::new(drawer_x + 16.0, content_y + 60.0, drawer_w - 32.0, 20.0),
                        11.5, "normal", toolbar_subtext, None);
                } else {
                    let total = self.console_logs.len();
                    let clamped = self.console_scroll_offset.min(total.saturating_sub(max_visible));
                    let start  = total.saturating_sub(max_visible + clamped);
                    let end    = (start + max_visible).min(total);

                    for (idx, entry) in self.console_logs[start..end].iter().enumerate() {
                        let ey = content_y + idx as f32 * row_h;

                        // Row separator
                        if idx > 0 {
                            if let Some(sep) = tiny_skia::Rect::from_xywh(drawer_x + 8.0, ey - 0.5, drawer_w - 16.0, 0.5) {
                                let mut paint = Paint::default();
                                paint.set_color_rgba8(toolbar_border.0, toolbar_border.1, toolbar_border.2, 120);
                                pixmap.fill_rect(sep, &paint, Transform::identity(), None);
                            }
                        }

                        // Level pill — color dot only
                        let pill_color = match entry.level.to_lowercase().as_str() {
                            "warn"   => Color::rgb(255, 159, 10),
                            "error"  => Color::rgb(255, 59, 48),
                            "system" => Color::rgb(88, 86, 214),
                            "info"   => Color::rgb(0, 122, 255),
                            _        => Color::rgb(52, 199, 89),
                        };
                        {
                            let mut pb = PathBuilder::new();
                            pb.push_circle(drawer_x + 14.0, ey + row_h / 2.0 - 1.0, 3.5);
                            if let Some(path) = pb.finish() {
                                let mut paint = Paint::default();
                                paint.set_color_rgba8(pill_color.r, pill_color.g, pill_color.b, 255);
                                paint.anti_alias = true;
                                pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
                            }
                        }

                        // Timestamp
                        text_engine.render_text(&mut pixmap.as_mut(),
                            &entry.timestamp,
                            Rect::new(drawer_x + 22.0, ey + 4.0, 54.0, 14.0),
                            9.5, "normal", ts_color, None);

                        // Message
                        let msg_w = drawer_w - 92.0;
                        text_engine.render_text(&mut pixmap.as_mut(),
                            &entry.message,
                            Rect::new(drawer_x + 80.0, ey + 4.0, msg_w, 14.0),
                            10.5, "normal", log_text_color, Some(msg_w));
                    }

                    // Scrollbar
                    if total > max_visible {
                        let track_h = drawer_h - 48.0;
                        let track_x = drawer_x + drawer_w - 4.0;
                        let ratio = max_visible as f32 / total as f32;
                        let thumb_h = (track_h * ratio).max(24.0);
                        let sr = clamped as f32 / (total - max_visible) as f32;
                        let thumb_y = content_y + sr * (track_h - thumb_h);
                        if let Some(thumb) = tiny_skia::Rect::from_xywh(track_x, thumb_y, 3.0, thumb_h) {
                            let mut paint = Paint::default();
                            paint.set_color_rgba8(toolbar_border.0, toolbar_border.1, toolbar_border.2, 200);
                            pixmap.fill_rect(thumb, &paint, Transform::identity(), None);
                        }
                    }
                }
            } else {
                // Network tab
                if self.network_logs.is_empty() {
                    text_engine.render_text(&mut pixmap.as_mut(),
                        "No network requests yet — call fetch() in your app.",
                        Rect::new(drawer_x + 16.0, content_y + 60.0, drawer_w - 32.0, 20.0),
                        11.5, "normal", toolbar_subtext, None);
                } else {
                    let total = self.network_logs.len();
                    let clamped = self.console_scroll_offset.min(total.saturating_sub(max_visible));
                    let start  = total.saturating_sub(max_visible + clamped);
                    let end    = (start + max_visible).min(total);

                    for (idx, entry) in self.network_logs[start..end].iter().enumerate() {
                        let ey = content_y + idx as f32 * row_h;

                        if idx > 0 {
                            if let Some(sep) = tiny_skia::Rect::from_xywh(drawer_x + 8.0, ey - 0.5, drawer_w - 16.0, 0.5) {
                                let mut paint = Paint::default();
                                paint.set_color_rgba8(toolbar_border.0, toolbar_border.1, toolbar_border.2, 120);
                                pixmap.fill_rect(sep, &paint, Transform::identity(), None);
                            }
                        }

                        // Method
                        let m_color = match entry.method.to_uppercase().as_str() {
                            "GET"           => Color::rgb(52, 199, 89),
                            "POST"          => Color::rgb(0, 122, 255),
                            "PUT"|"PATCH"   => Color::rgb(255, 159, 10),
                            "DELETE"        => Color::rgb(255, 59, 48),
                            _               => Color::rgb(142, 142, 147),
                        };
                        text_engine.render_text(&mut pixmap.as_mut(),
                            &entry.method,
                            Rect::new(drawer_x + 10.0, ey + 4.0, 40.0, 14.0),
                            9.5, "bold", m_color, None);

                        // Status
                        let s_color = if entry.status >= 200 && entry.status < 300 {
                            Color::rgb(52, 199, 89)
                        } else if entry.status >= 400 {
                            Color::rgb(255, 59, 48)
                        } else {
                            toolbar_subtext
                        };
                        let s_str = if entry.status > 0 { entry.status.to_string() } else { "···".to_string() };
                        text_engine.render_text(&mut pixmap.as_mut(),
                            &s_str,
                            Rect::new(drawer_x + 54.0, ey + 4.0, 32.0, 14.0),
                            9.5, "600", s_color, None);

                        // URL
                        let url_w = drawer_w - 138.0;
                        text_engine.render_text(&mut pixmap.as_mut(),
                            &entry.url,
                            Rect::new(drawer_x + 90.0, ey + 4.0, url_w, 14.0),
                            10.0, "normal", log_text_color, Some(url_w));

                        // Duration
                        let lat = format!("{}ms", entry.duration_ms);
                        text_engine.render_text(&mut pixmap.as_mut(),
                            &lat,
                            Rect::new(drawer_x + drawer_w - 46.0, ey + 4.0, 42.0, 14.0),
                            9.5, "normal", toolbar_subtext, None);
                    }
                }
            }

            // Composite the drawer layer, clipped to the screen (docked) or its own panel (detached)
            std::mem::swap(&mut pixmap, &mut layer);
            if detached {
                Self::composite_clipped(&mut pixmap, layer, self.console_drawer_rect, 12.0);
                self.renderer.draw_border(&mut pixmap.as_mut(), self.console_drawer_rect, 12.0, 1.0,
                    Color::rgb(toolbar_border.0, toolbar_border.1, toolbar_border.2));
            } else {
                Self::composite_clipped(&mut pixmap, layer, screen_rect, self.device_manager.active_profile().corner_radius);
            }
        }

        // ── 5. Dev Menu Sheet ───────────────────────────────────────────────
        if self.is_dev_menu_open {
            let mut layer = match Pixmap::new(width, height) {
                Some(l) => l,
                None => return,
            };
            std::mem::swap(&mut pixmap, &mut layer);

            // Scrim
            if let Some(scrim) = tiny_skia::Rect::from_xywh(
                screen_rect.x, screen_rect.y, screen_rect.width, screen_rect.height
            ) {
                let mut paint = Paint::default();
                paint.set_color_rgba8(0, 0, 0, 120);
                pixmap.fill_rect(scrim, &paint, Transform::identity(), None);
            }

            // Action sheet card — slides up from bottom
            let sheet_w = screen_rect.width - 16.0;
            let sheet_h = 312.0_f32;
            let sheet_x = screen_rect.x + 8.0;
            let sheet_y = screen_rect.y + screen_rect.height - sheet_h - 16.0;
            self.dev_menu_rect = Rect::new(sheet_x, sheet_y, sheet_w, sheet_h);

            // Card
            {
                let sr = 16.0_f32;
                let sx = sheet_x;
                let sy = sheet_y;
                let sw = sheet_w;
                let sh = sheet_h;
                let mut pb = PathBuilder::new();
                pb.move_to(sx + sr, sy);
                pb.line_to(sx + sw - sr, sy);
                pb.quad_to(sx + sw, sy, sx + sw, sy + sr);
                pb.line_to(sx + sw, sy + sh - sr);
                pb.quad_to(sx + sw, sy + sh, sx + sw - sr, sy + sh);
                pb.line_to(sx + sr, sy + sh);
                pb.quad_to(sx, sy + sh, sx, sy + sh - sr);
                pb.line_to(sx, sy + sr);
                pb.quad_to(sx, sy, sx + sr, sy);
                pb.close();
                if let Some(path) = pb.finish() {
                    let mut paint = Paint::default();
                    if is_dark {
                        paint.set_color_rgba8(44, 44, 46, 255);
                    } else {
                        paint.set_color_rgba8(255, 255, 255, 255);
                    }
                    paint.anti_alias = true;
                    pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
                }
            }

            // Drag handle
            {
                let dw = 36.0_f32;
                let dh = 4.0_f32;
                let dx = sheet_x + (sheet_w - dw) / 2.0;
                let dy = sheet_y + 8.0;
                if let Some(r) = tiny_skia::Rect::from_xywh(dx, dy, dw, dh) {
                    let mut paint = Paint::default();
                    paint.set_color_rgba8(toolbar_border.0, toolbar_border.1, toolbar_border.2, 255);
                    pixmap.fill_rect(r, &paint, Transform::identity(), None);
                }
            }

            // Title
            text_engine.render_text(&mut pixmap.as_mut(),
                "Developer",
                Rect::new(sheet_x + 16.0, sheet_y + 20.0, sheet_w - 32.0, 20.0),
                15.0, "700", toolbar_text, None);

            // Divider
            if let Some(d) = tiny_skia::Rect::from_xywh(sheet_x, sheet_y + 48.0, sheet_w, 1.0) {
                let mut paint = Paint::default();
                paint.set_color_rgba8(toolbar_border.0, toolbar_border.1, toolbar_border.2, 255);
                pixmap.fill_rect(d, &paint, Transform::identity(), None);
            }

            self.dev_menu_items.clear();
            let items: &[(&str, &str, &str)] = &[
                ("reload",    "Reload JavaScript",    "r"),
                ("inspector", if self.is_inspector_active { "Disable Inspector" } else { "Inspector" }, "i"),
                ("console",   if self.is_console_open { "Close Console" } else { "Console" }, "c"),
                ("rotate",    "Rotate",               "o"),
                ("theme",     if is_dark { "Light Mode" } else { "Dark Mode" },   "t"),
                ("dismiss",   "Dismiss",              "Esc"),
            ];

            let item_h = 42.0_f32;
            let mut item_y = sheet_y + 52.0;

            for (action_key, label, shortcut) in items.iter() {
                let item_rect = Rect::new(sheet_x, item_y, sheet_w, item_h);
                self.dev_menu_items.push((action_key.to_string(), item_rect));

                // Row separator (except first)
                if item_y > sheet_y + 52.0 {
                    if let Some(sep) = tiny_skia::Rect::from_xywh(sheet_x + 16.0, item_y - 0.5, sheet_w - 32.0, 0.5) {
                        let mut paint = Paint::default();
                        paint.set_color_rgba8(toolbar_border.0, toolbar_border.1, toolbar_border.2, 160);
                        pixmap.fill_rect(sep, &paint, Transform::identity(), None);
                    }
                }

                // Label
                text_engine.render_text(&mut pixmap.as_mut(),
                    label,
                    Rect::new(sheet_x + 20.0, item_y + 13.0, sheet_w - 80.0, 16.0),
                    13.0, "normal", toolbar_text, None);

                // Shortcut
                text_engine.render_text(&mut pixmap.as_mut(),
                    shortcut,
                    Rect::new(sheet_x + sheet_w - 40.0, item_y + 13.0, 34.0, 16.0),
                    11.0, "normal", toolbar_subtext, None);

                item_y += item_h;
            }

            std::mem::swap(&mut pixmap, &mut layer);
            Self::composite_clipped(&mut pixmap, layer, screen_rect, self.device_manager.active_profile().corner_radius);
        }

        // ── 6. Present ─────────────────────────────────────────────────────
        if scale < 1.0 {
            if let Some(mut out) = Pixmap::new(self.window_width, self.window_height) {
                let paint = tiny_skia::PixmapPaint {
                    quality: tiny_skia::FilterQuality::Bilinear,
                    ..Default::default()
                };
                out.draw_pixmap(0, 0, pixmap.as_ref(), &paint, Transform::from_scale(scale, scale), None);
                pixmap = out;
            }
        }
        if let Some(surface) = self.softbuffer_surface.as_mut() {
            if let Ok(mut buffer) = surface.buffer_mut() {
                let src = pixmap.data();
                for (dst_pixel, src_chunk) in buffer.iter_mut().zip(src.chunks_exact(4)) {
                    let r = src_chunk[0] as u32;
                    let g = src_chunk[1] as u32;
                    let b = src_chunk[2] as u32;
                    let a = src_chunk[3] as u32;
                    *dst_pixel = (a << 24) | (r << 16) | (g << 8) | b;
                }
                let _ = buffer.present();
            }
        }

        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }



    fn process_bridge_messages(&mut self) {
        while let Some(msg) = self.bridge.try_recv_from_js() {
            match msg {
                BridgeRequest::CreateNode { id, view_name, props, .. } => {
                    let mut node = Node::new(NodeId(id), ComponentType::from_name(&view_name));
                    node.update_props(props);
                    self.nodes.insert(NodeId(id), node);
                }
                BridgeRequest::UpdateProps { id, props } => {
                    if let Some(node) = self.nodes.get_mut(&NodeId(id)) {
                        node.update_props(props);
                    }
                }
                BridgeRequest::SetChildren { id, children } => {
                    if let Some(node) = self.nodes.get_mut(&NodeId(id)) {
                        node.children = children.into_iter().map(NodeId).collect();
                    }
                }
                BridgeRequest::AppendChild { parent_id, child_id } => {
                    if let Some(parent) = self.nodes.get_mut(&NodeId(parent_id)) {
                        parent.children.push(NodeId(child_id));
                    }
                }
                BridgeRequest::CompleteRoot { children, .. } => {
                    self.root_nodes = children.into_iter().map(NodeId).collect();
                }
                BridgeRequest::Log { level, message } => {
                    log::info!("[JS {}] {}", level, message);
                    self.add_console_log(&level, &message);
                }
                BridgeRequest::NetworkRequest { id, url, method, timestamp } => {
                    log::info!("[Network Req] {} {}", method, url);
                    self.network_logs.push(NetworkLogEntry {
                        id,
                        method,
                        url,
                        status: 0,
                        duration_ms: 0,
                        timestamp,
                    });
                    if self.network_logs.len() > 300 {
                        self.network_logs.remove(0);
                    }
                    if let Some(w) = &self.window {
                        w.request_redraw();
                    }
                }
                BridgeRequest::NetworkResponse { id, status, duration_ms, size_bytes: _ } => {
                    log::info!("[Network Res] id:{} status:{} {}ms", id, status, duration_ms);
                    if let Some(entry) = self.network_logs.iter_mut().rev().find(|e| e.id == id) {
                        entry.status = status;
                        entry.duration_ms = duration_ms;
                    }
                    if let Some(w) = &self.window {
                        w.request_redraw();
                    }
                }
                _ => {}
            }
        }
    }
}

impl ApplicationHandler<SimulatorEvent> for AppState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        // Keep the initial window inside the screen (leave room for title bar and panels)
        if let Some(mon) = event_loop.primary_monitor().or_else(|| event_loop.available_monitors().next()) {
            let m = mon.size();
            self.max_window = ((m.width as f32 * 0.95) as u32, m.height.saturating_sub(140).max(300));
            self.window_width = self.window_width.min(self.max_window.0);
            self.window_height = self.window_height.min(self.max_window.1);
        }

        let window_attrs = Window::default_attributes()
            .with_title("iPhone 16 Pro — iOS 18.0")
            .with_inner_size(PhysicalSize::new(self.window_width, self.window_height));

        let window = Arc::new(event_loop.create_window(window_attrs).unwrap());
        let context = softbuffer::Context::new(window.clone()).unwrap();
        let mut surface = softbuffer::Surface::new(&context, window.clone()).unwrap();

        let _ = surface.resize(
            NonZeroU32::new(self.window_width).unwrap(),
            NonZeroU32::new(self.window_height).unwrap(),
        );

        self.window = Some(window);
        self.softbuffer_context = Some(context);
        self.softbuffer_surface = Some(surface);
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: SimulatorEvent) {
        match event {
            SimulatorEvent::MetroStatus(connected) => {
                let was_connected = self.metro_connected;
                self.metro_connected = connected;
                if connected && !was_connected {
                    log::info!("⚡ Connected to Metro bundler on port 8081");
                    self.add_console_log("system", "⚡ Connected to Metro bundler on port 8081");
                    self.reload_bundle();
                } else if !connected && was_connected {
                    log::info!("⚡ Disconnected from Metro bundler");
                    self.add_console_log("system", "⚡ Disconnected from Metro bundler");
                }
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            SimulatorEvent::FastRefreshUpdate => {
                self.fast_refresh_count += 1;
                log::info!("⚡ Fast Refresh (Metro): update #{}", self.fast_refresh_count);
                self.add_console_log("system", &format!("⚡ Fast Refresh (Metro): update #{}", self.fast_refresh_count));
                self.reload_bundle();
            }
            SimulatorEvent::FileChanged => {
                self.fast_refresh_count += 1;
                log::info!("⚡ Fast Refresh (File Watcher): update #{}", self.fast_refresh_count);
                self.add_console_log("system", &format!("⚡ Hot reloaded App.tsx (#{})", self.fast_refresh_count));
                self.reload_bundle();
            }
            SimulatorEvent::Reload => {
                log::info!("🔄 Manual reload triggered");
                self.add_console_log("system", "🔄 Manual reload triggered");
                self.reload_bundle();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        self.process_bridge_messages();

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.window_width = size.width.max(1);
                self.window_height = size.height.max(1);
                if let Some(surface) = self.softbuffer_surface.as_mut() {
                    let _ = surface.resize(
                        NonZeroU32::new(self.window_width).unwrap(),
                        NonZeroU32::new(self.window_height).unwrap(),
                    );
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let inv = 1.0 / self.ui_scale();
                self.cursor_position = Point::new(position.x as f32 * inv, position.y as f32 * inv);
                if self.is_inspector_active {
                    let screen_rect = self.calculate_device_screen_rect();
                    let hit = self.hit_test_leaf_node(self.cursor_position, screen_rect);
                    if hit != self.inspected_node_id {
                        self.inspected_node_id = hit;
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left && state == ElementState::Pressed {
                    // 1. If Dev Menu is open
                    if self.is_dev_menu_open {
                        if self.dev_menu_rect.contains(self.cursor_position) {
                            for (action_key, item_rect) in &self.dev_menu_items {
                                if item_rect.contains(self.cursor_position) {
                                    match action_key.as_str() {
                                        "reload" => {
                                            self.is_dev_menu_open = false;
                                            self.reload_bundle();
                                        }
                                        "inspector" => {
                                            self.is_dev_menu_open = false;
                                            self.is_inspector_active = !self.is_inspector_active;
                                            if !self.is_inspector_active {
                                                self.inspected_node_id = None;
                                            }
                                        }
                                        "console" => {
                                            self.is_dev_menu_open = false;
                                            self.is_console_open = !self.is_console_open;
                                        }
                                        "rotate" => {
                                            self.is_dev_menu_open = false;
                                            self.toggle_device_orientation();
                                        }
                                        "theme" => {
                                            self.is_dev_menu_open = false;
                                            self.toggle_color_scheme();
                                        }
                                        "dismiss" => {
                                            self.is_dev_menu_open = false;
                                        }
                                        _ => {}
                                    }
                                    if let Some(w) = &self.window {
                                        w.request_redraw();
                                    }
                                    return;
                                }
                            }
                            return;
                        } else {
                            // Click outside Dev Menu dismisses it
                            self.is_dev_menu_open = false;
                            if let Some(w) = &self.window {
                                w.request_redraw();
                            }
                            return;
                        }
                    }

                    // 2. Toolbar buttons: Dev Menu, Inspect, Rotate, Theme, Console
                    if self.dev_menu_btn_rect.contains(self.cursor_position) {
                        self.is_dev_menu_open = !self.is_dev_menu_open;
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                        return;
                    }

                    if self.inspector_btn_rect.contains(self.cursor_position) {
                        self.is_inspector_active = !self.is_inspector_active;
                        if !self.is_inspector_active {
                            self.inspected_node_id = None;
                        }
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                        return;
                    }

                    if self.rotate_btn_rect.contains(self.cursor_position) {
                        self.toggle_device_orientation();
                        return;
                    }

                    if self.theme_btn_rect.contains(self.cursor_position) {
                        self.toggle_color_scheme();
                        return;
                    }

                    if self.console_toggle_btn_rect.contains(self.cursor_position) {
                        self.is_console_open = !self.is_console_open;
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                        return;
                    }

                    // 3. If console is open, check tabs, clear, close, or absorb clicks
                    if self.is_console_open {
                        if self.console_tab_btn_rect.contains(self.cursor_position) {
                            self.console_active_tab = ConsoleTab::Console;
                            if let Some(w) = &self.window {
                                w.request_redraw();
                            }
                            return;
                        }

                        if self.network_tab_btn_rect.contains(self.cursor_position) {
                            self.console_active_tab = ConsoleTab::Network;
                            if let Some(w) = &self.window {
                                w.request_redraw();
                            }
                            return;
                        }

                        if self.console_close_btn_rect.contains(self.cursor_position) {
                            self.is_console_open = false;
                            if let Some(w) = &self.window {
                                w.request_redraw();
                            }
                            return;
                        }

                        if self.console_dock_btn_rect.contains(self.cursor_position) {
                            self.console_detached = !self.console_detached;
                            if let Some(w) = &self.window {
                                w.request_redraw();
                            }
                            return;
                        }

                        if self.console_clear_btn_rect.contains(self.cursor_position) {
                            if self.console_active_tab == ConsoleTab::Console {
                                self.console_logs.clear();
                            } else {
                                self.network_logs.clear();
                            }
                            self.console_scroll_offset = 0;
                            if let Some(w) = &self.window {
                                w.request_redraw();
                            }
                            return;
                        }

                        if self.console_drawer_rect.contains(self.cursor_position) {
                            // Absorb clicks inside console drawer
                            return;
                        }
                    }

                    // 4. If Inspector is active, clicking inside screen inspects the element
                    let screen_rect = self.calculate_device_screen_rect();
                    if self.is_inspector_active && screen_rect.contains(self.cursor_position) {
                        if let Some(nid) = self.hit_test_leaf_node(self.cursor_position, screen_rect) {
                            self.inspect_node(nid);
                            if let Some(w) = &self.window {
                                w.request_redraw();
                            }
                        }
                        return;
                    }
                }

                if self.is_console_open && self.console_drawer_rect.contains(self.cursor_position) {
                    return;
                }

                if self.is_dev_menu_open {
                    return;
                }

                let screen_rect = self.calculate_device_screen_rect();
                if button == MouseButton::Left {
                    if state == ElementState::Pressed {
                        let input_event = InputEvent::MouseDown {
                            point: self.cursor_position,
                            button: 0,
                        };
                        self.input_handler.handle_event(
                            input_event,
                            &self.nodes,
                            &self.root_nodes,
                            screen_rect,
                        );

                        // Update focus state for TextInput
                        let focused_id = self.input_handler.focused_node();
                        for (id, node) in self.nodes.iter_mut() {
                            node.is_focused = Some(*id) == focused_id;
                        }

                        // Provide visual feedback for Pressable button
                        if let Some(target_id) = self.input_handler.active_touch_target() {
                            self.active_pressed_node = Some(target_id);
                            if let Some(node) = self.nodes.get_mut(&target_id) {
                                if node.component_type == ComponentType::Pressable {
                                    node.style.opacity = 0.82;
                                    println!("Pressed");
                                    log::info!("Pressed");
                                    self.add_console_log("log", "Pressed NativeWind button");
                                }
                            }
                        }
                    } else {
                        let input_event = InputEvent::MouseUp {
                            point: self.cursor_position,
                            button: 0,
                        };
                        self.input_handler.handle_event(
                            input_event,
                            &self.nodes,
                            &self.root_nodes,
                            screen_rect,
                        );

                        // Restore Pressable opacity
                        if let Some(target_id) = self.active_pressed_node.take() {
                            if let Some(node) = self.nodes.get_mut(&target_id) {
                                if node.component_type == ComponentType::Pressable {
                                    node.style.opacity = 1.0;
                                }
                            }
                        }
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if self.is_console_open && self.console_drawer_rect.contains(self.cursor_position) {
                    let dy = match delta {
                        winit::event::MouseScrollDelta::LineDelta(_dx, dy) => dy as i32,
                        winit::event::MouseScrollDelta::PixelDelta(pos) => (pos.y / 15.0) as i32,
                    };
                    if dy > 0 {
                        self.console_scroll_offset = self.console_scroll_offset.saturating_add(dy.unsigned_abs() as usize);
                    } else if dy < 0 {
                        self.console_scroll_offset = self.console_scroll_offset.saturating_sub(dy.unsigned_abs() as usize);
                    }
                    if let Some(w) = &self.window {
                        w.request_redraw();
                    }
                }
            }
            WindowEvent::ModifiersChanged(new_modifiers) => {
                self.modifiers = new_modifiers.state();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    let is_ctrl = self.modifiers.control_key() || self.modifiers.super_key();
                    let focused_id = self.input_handler.focused_node();

                    // Escape closes dev menu, console, or inspector
                    if let Key::Named(NamedKey::Escape) = event.logical_key {
                        if self.is_dev_menu_open {
                            self.is_dev_menu_open = false;
                        } else if self.is_console_open {
                            self.is_console_open = false;
                        } else if self.is_inspector_active {
                            self.is_inspector_active = false;
                            self.inspected_node_id = None;
                        }
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                        return;
                    }

                    // Dev Menu shortcut: Ctrl+D or Cmd+D
                    if let Key::Character(ref ch) = event.logical_key {
                        if ch.as_str().eq_ignore_ascii_case("d") && is_ctrl {
                            self.is_dev_menu_open = !self.is_dev_menu_open;
                            if let Some(w) = &self.window {
                                w.request_redraw();
                            }
                            return;
                        }
                    }

                    // Inspector shortcut: Ctrl+I or 'i' (when unfocused)
                    if let Key::Character(ref ch) = event.logical_key {
                        if ch.as_str().eq_ignore_ascii_case("i") && (is_ctrl || focused_id.is_none()) {
                            self.is_inspector_active = !self.is_inspector_active;
                            if !self.is_inspector_active {
                                self.inspected_node_id = None;
                            }
                            if let Some(w) = &self.window {
                                w.request_redraw();
                            }
                            return;
                        }
                    }

                    // Rotate shortcut: Ctrl+O or 'o' (when unfocused)
                    if let Key::Character(ref ch) = event.logical_key {
                        if ch.as_str().eq_ignore_ascii_case("o") && (is_ctrl || focused_id.is_none()) {
                            self.toggle_device_orientation();
                            return;
                        }
                    }

                    // Theme toggle shortcut: Ctrl+T or 't' (when unfocused)
                    if let Key::Character(ref ch) = event.logical_key {
                        if ch.as_str().eq_ignore_ascii_case("t") && (is_ctrl || focused_id.is_none()) {
                            self.toggle_color_scheme();
                            return;
                        }
                    }

                    // Toggle console shortcut: 'c' or F12 or `
                    let is_c_toggle = match event.logical_key {
                        Key::Character(ref ch) => {
                            (ch.as_str().eq_ignore_ascii_case("c") || ch.as_str() == "`") && (is_ctrl || focused_id.is_none())
                        }
                        Key::Named(NamedKey::F12) => true,
                        _ => false,
                    };
                    if is_c_toggle {
                        self.is_console_open = !self.is_console_open;
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                        return;
                    }

                    // Hot reload shortcut: Ctrl+R (or 'r' when no input is focused)
                    if let Key::Character(ref ch) = event.logical_key {
                        if ch.as_str().eq_ignore_ascii_case("r") && (is_ctrl || focused_id.is_none()) {
                            log::info!("🔄 Reload triggered via keyboard shortcut");
                            self.reload_bundle();
                            return;
                        }
                    }

                    // Text editing when a TextInput is focused
                    if let Some(fid) = focused_id {
                        let screen_rect = self.calculate_device_screen_rect();

                        match event.logical_key {
                            Key::Named(NamedKey::Space) => {
                                let input_event = InputEvent::TextInput { text: " ".to_string() };
                                self.input_handler.handle_event(
                                    input_event,
                                    &self.nodes,
                                    &self.root_nodes,
                                    screen_rect,
                                );

                                if let Some(node) = self.nodes.get_mut(&fid) {
                                    let mut cur = node.text_content.clone().unwrap_or_default();
                                    cur.push(' ');
                                    node.text_content = Some(cur);
                                }
                            }
                            Key::Named(NamedKey::Backspace) => {
                                let input_event = InputEvent::KeyDown { key: "Backspace".to_string() };
                                self.input_handler.handle_event(
                                    input_event,
                                    &self.nodes,
                                    &self.root_nodes,
                                    screen_rect,
                                );

                                if let Some(node) = self.nodes.get_mut(&fid) {
                                    let cur = node.text_content.clone().unwrap_or_default();
                                    if !cur.is_empty() {
                                        let mut chars: Vec<char> = cur.chars().collect();
                                        chars.pop();
                                        let new_text: String = chars.into_iter().collect();
                                        node.text_content = if new_text.is_empty() { None } else { Some(new_text) };
                                    }
                                }
                            }
                            Key::Named(NamedKey::Delete) => {
                                let input_event = InputEvent::KeyDown { key: "Delete".to_string() };
                                self.input_handler.handle_event(
                                    input_event,
                                    &self.nodes,
                                    &self.root_nodes,
                                    screen_rect,
                                );

                                if let Some(node) = self.nodes.get_mut(&fid) {
                                    let cur = node.text_content.clone().unwrap_or_default();
                                    if !cur.is_empty() {
                                        let mut chars: Vec<char> = cur.chars().collect();
                                        chars.pop();
                                        let new_text: String = chars.into_iter().collect();
                                        node.text_content = if new_text.is_empty() { None } else { Some(new_text) };
                                    }
                                }
                            }
                            Key::Named(NamedKey::Tab) => {
                                let input_event = InputEvent::TextInput { text: "  ".to_string() };
                                self.input_handler.handle_event(
                                    input_event,
                                    &self.nodes,
                                    &self.root_nodes,
                                    screen_rect,
                                );

                                if let Some(node) = self.nodes.get_mut(&fid) {
                                    let mut cur = node.text_content.clone().unwrap_or_default();
                                    cur.push_str("  ");
                                    node.text_content = Some(cur);
                                }
                            }
                            Key::Named(NamedKey::Enter) => {
                                let input_event = InputEvent::KeyDown { key: "Enter".to_string() };
                                self.input_handler.handle_event(
                                    input_event,
                                    &self.nodes,
                                    &self.root_nodes,
                                    screen_rect,
                                );
                            }
                            Key::Character(ref ch) => {
                                let input_event = InputEvent::TextInput { text: ch.to_string() };
                                self.input_handler.handle_event(
                                    input_event,
                                    &self.nodes,
                                    &self.root_nodes,
                                    screen_rect,
                                );

                                if let Some(node) = self.nodes.get_mut(&fid) {
                                    let mut cur = node.text_content.clone().unwrap_or_default();
                                    cur.push_str(ch.as_str());
                                    node.text_content = Some(cur);
                                }
                            }
                            _ => {}
                        }

                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                    } else {
                        // Unfocused key triggers (e.g. 'r' for reload)
                        if let Key::Character(ref ch) = event.logical_key {
                            if ch.as_str().eq_ignore_ascii_case("r") {
                                log::info!("🔄 Reload triggered via 'r' key");
                                self.reload_bundle();
                            }
                        }
                    }
                }
            }

            WindowEvent::RedrawRequested => {
                self.render();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.process_bridge_messages();
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
}

/// Linear interpolation between two u8 values.
#[inline(always)]
fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t.clamp(0.0, 1.0)) as u8
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("Starting Expo Simulator...");

    let event_loop: EventLoop<SimulatorEvent> = EventLoop::<SimulatorEvent>::with_user_event().build().unwrap();
    let proxy = event_loop.create_proxy();

    let bridge = BridgeChannel::new();
    let engine = QuickJsEngine::new(bridge.clone()).expect("Failed to initialize QuickJS engine");
    let rt = Arc::new(tokio::runtime::Runtime::new().unwrap());

    let app_file = find_project_app_file();
    if let Some(ref file) = app_file {
        log::info!("⚡ Detected project app file for live reload: {:?}", file);
    }

    let mut app = AppState::new(bridge, engine, rt.clone(), proxy.clone(), app_file.clone());
    app.add_console_log("system", "📱 Expo Simulator initialized (iPhone 16 Pro)");
    app.add_console_log("info", "Press 'c' or click [>_ Console] to toggle window");
    match std::env::var("EXPO_SIM_CONSOLE").as_deref() {
        Ok("docked") => app.is_console_open = true,
        Ok("detached") => {
            app.is_console_open = true;
            app.console_detached = true;
        }
        _ => {}
    }

    // Check if Metro server is running on port 8081
    let metro = MetroClient::default();
    let is_metro_running = rt.block_on(metro.is_running());
    app.metro_connected = is_metro_running;

    // Load initial application bundle
    app.reload_bundle();

    // Spawn Metro background watcher & Fast Refresh HMR listener
    let proxy_clone = proxy.clone();
    let metro_clone = MetroClient::default();
    rt.spawn(async move {
        let mut was_running = is_metro_running;
        loop {
            let is_running = metro_clone.is_running().await;
            if is_running != was_running {
                was_running = is_running;
                let _ = proxy_clone.send_event(SimulatorEvent::MetroStatus(is_running));
            }

            if is_running {
                let p = proxy_clone.clone();
                let _ = metro_clone.listen_hmr(None, move || {
                    let _ = p.send_event(SimulatorEvent::FastRefreshUpdate);
                }).await;

                // If HMR connection closed, recheck status
                let is_still_running = metro_clone.is_running().await;
                if is_still_running != was_running {
                    was_running = is_still_running;
                    let _ = proxy_clone.send_event(SimulatorEvent::MetroStatus(is_still_running));
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
        }
    });

    // Spawn file watcher for live hot reload of source files
    if let Some(app_path) = app_file {
        let proxy_file = proxy.clone();
        rt.spawn(async move {
            let mut last_modified = std::fs::metadata(&app_path)
                .and_then(|m| m.modified())
                .ok();

            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
                if let Ok(meta) = std::fs::metadata(&app_path) {
                    if let Ok(modified) = meta.modified() {
                        if Some(modified) != last_modified {
                            last_modified = Some(modified);
                            let _ = proxy_file.send_event(SimulatorEvent::FileChanged);
                        }
                    }
                }
            }
        });
    }

    event_loop.set_control_flow(ControlFlow::Poll);
    let _ = event_loop.run_app(&mut app);
}
