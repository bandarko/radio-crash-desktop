use crate::panel::{self, Panel};
use crate::{audio::Engine, dsp::Meters, platform, settings::Settings, skins::*};
use eframe::egui::*;
use std::time::{Duration, Instant};
fn minimize_screw(ui: &mut Ui, rect: Rect) -> Response {
    ui.interact(
        rect,
        Id::unique("upper-left-minimize-screw"),
        Sense::click(),
    )
    .on_hover_cursor(CursorIcon::PointingHand)
    .on_hover_text("Minimize to tray: live L/R meters. Click the tray icon to restore.")
}
fn skin_selector(ui: &mut Ui, selected: &mut Skin) {
    ComboBox::from_label("Receiver skin")
        .selected_text(selected.label())
        .show_ui(ui, |ui| {
            for skin in [Skin::VfdBlack, Skin::AnalogSilver] {
                ui.selectable_value(selected, skin, skin.label());
            }
        });
}
pub struct RadioApp {
    updater: crate::updates::Checker,
    show_update: bool,
    update_seen: Option<crate::updates::Version>,
    materials: Materials,
    analog: Option<crate::analog::AnalogSkin>,
    graphics: crate::graphics::Graphics,
    #[cfg(target_os = "windows")]
    receiver_window: Option<std::sync::Arc<winit::window::Window>>,
    clock: crate::clock_face::ClockFace,
    clock_updated: Instant,
    observed_window: Vec2,
    resize_save: Option<Instant>,
    applied_connection: (String, String, String),
    displayed_track: (String, String),
    track_changed: Instant,
    capture: Option<std::path::PathBuf>,
    qa_exit: Option<Duration>,
    tray: Option<platform::Tray>,
    hidden_to_tray: bool,
    restore_until: Instant,
    tray_check: Option<std::path::PathBuf>,
    tray_check_phase: u8,
    tray_check_before: (u64, u64),
    engine: Engine,
    settings: Settings,
    show_settings: bool,
    focus_settings: bool,
    power_menu_on_launch: bool,
    power: bool,
    mute: bool,
    meters: Meters,
    last: Instant,
    start: Instant,
    devices: Vec<String>,
    error: String,
}
impl RadioApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        #[cfg(target_os = "windows")]
        crate::platform::configure_receiver_window(cc);
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "vfd".into(),
            FontData::from_static(include_bytes!("../assets/Doto.ttf")).into(),
        );
        fonts.families.insert(
            FontFamily::Name("vfd".into()),
            vec!["vfd".into(), "Hack".into()],
        );
        fonts.font_data.insert(
            "badge".into(),
            FontData::from_static(include_bytes!("../assets/Audiowide-Regular.ttf")).into(),
        );
        fonts
            .families
            .insert(FontFamily::Name("badge".into()), vec!["badge".into()]);
        fonts.font_data.insert(
            "analog-display".into(),
            FontData::from_static(include_bytes!("../assets/Rajdhani-SemiBold.ttf")).into(),
        );
        fonts.families.insert(
            FontFamily::Name("analog-display".into()),
            vec!["analog-display".into(), "Hack".into()],
        );
        cc.egui_ctx.set_fonts(fonts);
        cc.egui_ctx.set_visuals(Visuals::dark());
        let mut settings = Settings::load();
        if let Some(name) = std::env::args().skip_while(|a| a != "--skin").nth(1) {
            settings.skin = if name == "analog-silver" {
                Skin::AnalogSilver
            } else {
                Skin::VfdBlack
            };
        }
        if settings.start_minimized {
            cc.egui_ctx
                .send_viewport_cmd(ViewportCommand::Minimized(true));
        }
        let materials = Materials::new(&cc.egui_ctx);
        let graphics = crate::graphics::Graphics::new(cc);
        let mut engine = Engine::new();
        engine.volume(settings.volume);
        if settings.autoplay || std::env::args().any(|a| a == "--autoplay") {
            engine.start(settings.clone());
        }
        Self {
            updater: crate::updates::Checker::new(),
            show_update: false,
            update_seen: None,
            materials,
            analog: None,
            graphics,
            #[cfg(target_os = "windows")]
            receiver_window: cc.winit_window().cloned(),
            clock: crate::clock_face::ClockFace::now(),
            clock_updated: Instant::now(),
            observed_window: Vec2::ZERO,
            resize_save: None,
            applied_connection: (
                settings.stream_url.clone(),
                settings.metadata_url.clone(),
                settings.device.clone(),
            ),
            displayed_track: (String::new(), String::new()),
            track_changed: Instant::now(),
            capture: std::env::args()
                .skip_while(|a| a != "--capture")
                .nth(1)
                .map(Into::into),
            qa_exit: std::env::args()
                .skip_while(|a| a != "--qa-exit-after")
                .nth(1)
                .and_then(|v| v.parse::<u64>().ok())
                .filter(|v| (1..=120).contains(v))
                .map(Duration::from_secs),
            tray: platform::Tray::new()
                .map_err(|e| log::warn!("Tray: {e}"))
                .ok(),
            hidden_to_tray: false,
            restore_until: Instant::now(),
            tray_check: std::env::args()
                .skip_while(|a| a != "--tray-check")
                .nth(1)
                .map(Into::into),
            tray_check_phase: 0,
            tray_check_before: (0, 0),
            power: true,
            mute: false,
            devices: platform::devices(),
            engine,
            settings,
            show_settings: std::env::args().any(|a| a == "--settings"),
            focus_settings: true,
            power_menu_on_launch: std::env::args().any(|a| a == "--power-menu"),
            meters: Meters::default(),
            last: Instant::now(),
            start: Instant::now(),
            error: String::new(),
        }
    }
    fn minimize_to_tray(&mut self, ctx: &Context) {
        if self.tray.is_none() {
            ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
            return;
        }
        self.hidden_to_tray = true;
        platform::set_tray_mode(true);
        if self.show_settings {
            ctx.send_viewport_cmd_to(
                ViewportId::from_hash_of("receiver-settings"),
                ViewportCommand::Close,
            );
        }
        self.show_settings = false;
        ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
        ctx.send_viewport_cmd(ViewportCommand::Visible(false));
        ctx.request_repaint_after(Duration::from_millis(33));
    }
    fn restore_from_tray(&mut self, ctx: &Context) {
        self.hidden_to_tray = false;
        platform::set_tray_mode(false);
        self.restore_until = Instant::now() + Duration::from_millis(600);
        ctx.send_viewport_cmd(ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd(ViewportCommand::Focus);
        ctx.request_repaint();
    }
    fn button(&self, ui: &mut Ui, r: Rect, kind: &str, lit: bool) -> Response {
        let response = panel::click_target(ui, r, kind);
        let p = ui.painter();
        if response.hovered() {
            p.rect_filled(
                r.shrink(1.),
                1.,
                Color32::from_white_alpha(if response.is_pointer_button_down_on() {
                    2
                } else {
                    5
                }),
            );
        }
        let s = match kind {
            "power" => 40.,
            "transport" | "mute" => 42.,
            _ => 40.,
        } * (r.width()
            / match kind {
                "power" => 66.,
                "transport" => 128.,
                "mute" => 79.,
                _ => 60.,
            })
            / 44.;
        let c = r.center();
        let analog = self.settings.skin == Skin::AnalogSilver;
        let silver = if analog {
            Color32::from_gray(43)
        } else {
            SILVER
        };
        let ink = if analog {
            Color32::from_rgb(36, 31, 24)
        } else if lit {
            CYAN
        } else {
            Color32::from_rgb(166, 173, 169)
        };
        match kind {
            "transport" => {
                if lit {
                    p.rect_filled(Rect::from_center_size(c, vec2(12., 12.) * s), 0.8 * s, ink);
                } else {
                    p.add(Shape::convex_polygon(
                        vec![
                            c + vec2(-5., -9.) * s,
                            c + vec2(-5., 9.) * s,
                            c + vec2(10., 0.) * s,
                        ],
                        ink,
                        Stroke::NONE,
                    ));
                }
            }
            "power" => {
                let led = Rect::from_center_size(c + vec2(0., 1.) * s, vec2(18., 2.) * s);
                p.rect_filled(
                    led.expand(1.4 * s),
                    1.,
                    Color32::from_rgba_unmultiplied(24, 220, 228, 16),
                );
                p.rect_filled(
                    led,
                    0.5 * s,
                    if lit {
                        CYAN
                    } else {
                        Color32::from_rgb(150, 38, 18)
                    },
                );
            }
            "mute" => {
                p.add(Shape::convex_polygon(
                    vec![
                        c + vec2(-10., -3.) * s,
                        c + vec2(-5., -3.) * s,
                        c + vec2(1., -8.) * s,
                        c + vec2(1., 8.) * s,
                        c + vec2(-5., 3.) * s,
                        c + vec2(-10., 3.) * s,
                    ],
                    silver,
                    Stroke::NONE,
                ));
                if lit {
                    p.line_segment(
                        [c + vec2(-12., 11.) * s, c + vec2(10., -10.) * s],
                        Stroke::new(1.5 * s, Color32::from_rgb(233, 179, 90)),
                    );
                } else {
                    let arc: Vec<_> = (0..=12)
                        .map(|i| {
                            let a = -0.9 + i as f32 / 12. * 1.8;
                            c + vec2(2. + a.cos() * 8., a.sin() * 8.) * s
                        })
                        .collect();
                    p.add(Shape::line(arc, Stroke::new(s, silver)));
                }
            }
            "settings" => {
                let pts: Vec<_> = (0..48)
                    .map(|i| {
                        let a = i as f32 / 48. * std::f32::consts::TAU;
                        let radius = if i % 6 < 3 { 8.5 } else { 6.6 };
                        c + vec2(a.cos(), a.sin()) * radius * s
                    })
                    .collect();
                p.add(Shape::convex_polygon(pts, silver, Stroke::NONE));
                p.circle_filled(c, 3.8 * s, Color32::from_gray(25));
                p.circle_stroke(c, 3.8 * s, Stroke::new(0.6 * s, Color32::from_gray(115)));
            }
            _ => {}
        }
        response
            .on_hover_cursor(CursorIcon::PointingHand)
            .on_hover_text(match kind {
                "transport" => {
                    if lit {
                        "Stop stream"
                    } else {
                        "Play Radio Crash"
                    }
                }
                "power" => "Power menu: ON / OFF / EXIT",
                "mute" => {
                    if lit {
                        "Unmute"
                    } else {
                        "Mute"
                    }
                }
                _ => "Receiver settings",
            })
    }
    fn window_controls(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        let area = ui.max_rect();
        let current = area.size();
        if (current - self.observed_window).length() > 0.5 && current.x >= 200. && current.y >= 100.
        {
            let previous = self.observed_window;
            self.observed_window = current;
            let mut width = current.x;
            if self.settings.lock_aspect
                && previous.x > 0.
                && (current.y - current.x / crate::settings::PANEL_ASPECT).abs() > 1.5
            {
                if (current.y - previous.y).abs() * crate::settings::PANEL_ASPECT
                    > (current.x - previous.x).abs()
                {
                    width = current.y * crate::settings::PANEL_ASPECT;
                }
                let size = crate::settings::window_size(width, current.y, true);
                ctx.send_viewport_cmd(ViewportCommand::InnerSize(vec2(size[0], size[1])));
            }
            self.settings.window_width = width;
            self.settings.window_height = if self.settings.lock_aspect {
                width / crate::settings::PANEL_ASPECT
            } else {
                current.y
            };
            self.resize_save = Some(Instant::now());
        }
        // Hit-test the visible metal rim, not the transparent canvas around the feet.
        let panel = Panel::new(area);
        let handle = |x, y, w, h| panel.rect([x, y, w, h]);
        let handles = [
            (
                0,
                handle(1177., 28., 19., 350.),
                CursorIcon::ResizeHorizontal,
            ),
            (1, handle(24., 386., 1140., 13.), CursorIcon::ResizeVertical),
            (2, handle(1170., 380., 26., 20.), CursorIcon::ResizeNwSe),
        ];
        for (axis, rect, cursor) in handles {
            let response = ui
                .interact(rect, Id::unique(("receiver-resize", axis)), Sense::drag())
                .on_hover_cursor(cursor);
            if response.dragged() {
                let d = response.drag_delta();
                let width = if self.settings.lock_aspect
                    && (axis == 1
                        || (axis == 2 && d.y.abs() * crate::settings::PANEL_ASPECT > d.x.abs()))
                {
                    (current.y + d.y) * crate::settings::PANEL_ASPECT
                } else {
                    current.x + if axis == 1 { 0. } else { d.x }
                };
                let height = current.y + if axis == 0 { 0. } else { d.y };
                let size = crate::settings::window_size(width, height, self.settings.lock_aspect);
                ctx.send_viewport_cmd(ViewportCommand::InnerSize(vec2(size[0], size[1])));
                self.resize_save = Some(Instant::now());
            }
        }
        if self
            .resize_save
            .is_some_and(|at| at.elapsed() > Duration::from_millis(750))
            && !ui.input(|i| i.pointer.any_down())
        {
            if let Err(e) = self.settings.save() {
                self.error = e.to_string();
            }
            self.resize_save = None;
        }
    }
    fn settings(&mut self, ctx: &Context) {
        let mut apply = false;
        let mut closed = false;
        ctx.show_viewport_immediate(
            ViewportId::from_hash_of("receiver-settings"),
            ViewportBuilder::default().with_title("Radio Crash · Receiver settings")
                .with_inner_size([560., 650.]).with_min_inner_size([520.,420.]).with_resizable(true).with_decorations(true),
            |ctx, _| {
                closed = ctx.input(|i| i.viewport().close_requested());
                CentralPanel::default().show(ctx, |ui| {
                    ui.heading("Receiver settings");
                    ui.label(concat!("Radio Crash Desktop ",env!("CARGO_PKG_VERSION")));
                    ui.separator();
                    ScrollArea::vertical().id_salt("receiver-settings-scroll").max_height((ui.available_height()-164.).max(80.)).show(ui,|ui|{
                    ComboBox::from_label("Audio output").selected_text(if self.settings.device.is_empty() { "System default" } else { &self.settings.device }).show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.settings.device, String::new(), "System default");
                        for d in &self.devices { ui.selectable_value(&mut self.settings.device, d.clone(), d); }
                    });
                    if ui.button("Refresh devices").clicked() { self.devices=platform::devices(); }
                    ui.checkbox(&mut self.settings.autoplay,"Autoplay on launch");
                    ui.checkbox(&mut self.settings.start_minimized,"Start minimized");
                    ui.add_enabled(self.tray.is_some(), Checkbox::new(&mut self.settings.minimize_to_tray,"Minimize to tray"));
                    ui.separator();
                    ui.label("Window size · desktop points");
                    ui.horizontal(|ui|{
                        for (label,width) in [("Small",640.),("Everyday",780.),("Large",1000.),("Full",1200.)] {
                            if ui.button(label).clicked(){self.settings.window_width=width;self.settings.window_height=width/crate::settings::PANEL_ASPECT;}
                        }
                    });
                    ui.horizontal(|ui|{
                        ui.label("Width");
                        let width=ui.add(DragValue::new(&mut self.settings.window_width).range(560.0..=1920.0).speed(2.));
                        if width.changed() && self.settings.lock_aspect {self.settings.window_height=self.settings.window_width/crate::settings::PANEL_ASPECT;}
                        ui.label("Height");
                        let height=ui.add(DragValue::new(&mut self.settings.window_height).range(206.0..=1080.0).speed(1.));
                        if height.changed() && self.settings.lock_aspect {self.settings.window_width=self.settings.window_height*crate::settings::PANEL_ASPECT;}
                    });
                    ui.checkbox(&mut self.settings.lock_aspect,"Keep receiver proportions");
                    ui.small("Drag the right/bottom edge or lower-right corner to resize. Size is remembered.");
                    ui.separator();
                    skin_selector(ui, &mut self.settings.skin);
                    self.graphics.settings(ui, &mut self.settings.graphics_quality);
                    if ui.button("Copy graphics and window diagnostics").clicked() {
                        let report=self.graphics.diagnostics(self.settings.graphics_quality);
                        #[cfg(target_os="windows")]
                        let report={let mut report=report;if let Some(window)=&self.receiver_window {report["window"]=platform::receiver_window_report(window);}report};
                        ui.ctx().copy_text(serde_json::to_string_pretty(&report).unwrap_or_default());
                    }
                    ui.horizontal(|ui| {
                        let mut brightness=self.settings.vfd_brightness*100.;
                        if ui.add(Slider::new(&mut brightness,25.0..=100.0).text("Instrument illumination · %")).changed() { self.settings.vfd_brightness=brightness/100.; }
                        if ui.button("Day").clicked() { self.settings.vfd_brightness=1.; }
                        if ui.button("Night").clicked() { self.settings.vfd_brightness=0.45; }
                    });
                    ui.checkbox(&mut self.settings.standby_clock,"Show local clock when stopped");
                    if self.settings.skin == Skin::AnalogSilver {
                        ui.label("Classic analog VU · fixed 300 ms movement · scale to +3 VU");
                        ui.small("Full-wave average detection, sine calibrated. No RMS/Peak switching or peak-hold needle.");
                        let mut sensitivity = -self.settings.analog_reference_dbfs;
                        if ui.add(Slider::new(&mut sensitivity,0.0..=24.0).step_by(0.5).text("Analog VU level · dB")).changed() {
                            self.settings.analog_reference_dbfs = -sensitivity;
                        }
                        ui.small(format!("0 VU = {:.1} dBFS for a sine wave. Display sensitivity only; audio is unchanged.", self.settings.analog_reference_dbfs));
                    } else {
                    ComboBox::from_label("VU response").selected_text(self.settings.vu_response.label()).show_ui(ui, |ui| {
                        for response in [crate::dsp::VuResponse::SmoothRms, crate::dsp::VuResponse::FastRms, crate::dsp::VuResponse::Peak] {
                            ui.selectable_value(&mut self.settings.vu_response, response, response.label());
                        }
                    });
                    ui.checkbox(&mut self.settings.fast_vu,"Faster VU decay");
                    ComboBox::from_label("VU calibration").selected_text(match self.settings.vu_reference_dbfs {0.=>"Digital: 0 dBFS".to_owned(),-15.=>"Radio Crash Hi-Fi: -15 dBFS".to_owned(),-18.=>"Studio: -18 dBFS".to_owned(),other=>format!("Custom: {other} dBFS")}).show_ui(ui,|ui|{
                        ui.selectable_value(&mut self.settings.vu_reference_dbfs,-15.,"Radio Crash Hi-Fi (recommended)");
                        ui.selectable_value(&mut self.settings.vu_reference_dbfs,0.,"Digital: 0 dBFS (actual level)");
                        ui.selectable_value(&mut self.settings.vu_reference_dbfs,-18.,"Studio: -18 dBFS (relative VU)");
                    });
                    let mut sensitivity=-self.settings.vu_reference_dbfs;
                    if ui.add(Slider::new(&mut sensitivity,0.0..=24.0).step_by(0.5).text("VU sensitivity · dB")).changed(){self.settings.vu_reference_dbfs = -sensitivity;}
                    ui.small(match self.settings.vu_response {
                        crate::dsp::VuResponse::SmoothRms => "Hi-Fi RMS: the smooth response from 1.1.5.",
                        crate::dsp::VuResponse::FastRms => "Fast RMS: quicker response to rhythm, without changing meter gain.",
                        crate::dsp::VuResponse::Peak => "Peak: immediate response to short sample peaks, with controlled decay.",
                    });
                    ui.small("Peak hold remembers the highest level of the SAME selected detector for 2 seconds.\nThe scale is relative to the selected reference; red segments alone do not mean clipping.");
                    }
                    CollapsingHeader::new("Live signal measurements").show(ui, |ui| {
                        let snap = self.engine.state.lock().unwrap().clone();
                        let m = &snap.meters;
                        let value = |v: Option<f64>, unit: &str| v.map(|v| format!("{v:.1} {unit}")).unwrap_or_else(|| "Unavailable / warming up".into());
                        ui.label(format!("Momentary (400 ms): {}", value(m.momentary_lufs, "LUFS")));
                        ui.label(format!("Short-term (3 s): {}", value(m.short_term_lufs, "LUFS")));
                        if m.true_peaks.is_some() {
                            ui.label(format!("True Peak hold: L {:+.1} / R {:+.1} dBTP", m.held_peaks[0], m.held_peaks[1]));
                            if m.held_peaks.iter().any(|v| *v > 0.) {
                                let color = if ui.visuals().dark_mode { Color32::LIGHT_RED } else { Color32::from_rgb(157, 37, 26) };
                                ui.colored_label(color,"Above 0 dBTP in the decoded stream (before volume).");
                            }
                        } else { ui.label("True Peak: unavailable"); }
                        ui.small("Measured from playback PCM before volume. No loudness normalization or audio processing. Silence has no finite LUFS value.");
                    });
                    if self.settings.skin == Skin::VfdBlack {
                        ui.checkbox(&mut self.settings.spectrum,"Spectrum analyzer");
                    ui.add(Slider::new(&mut self.settings.hd_kbps,64..=512).text("HD badge lights from · kbps"));
                    ui.small("Badge only. Audio always uses the original stream quality; this does not limit bitrate.\nHD SOUND is a Radio Crash badge, not a standardized HD codec. Meters are measured before volume.");
                    }
                    });
                    ui.separator();
                    ui.label("Automatic update checks are always on");
                    ui.small("Checks on launch and every 6 hours. Updates are downloaded in your browser.");
                    if ui.add_enabled(!self.updater.busy(),Button::new("Check for updates now")).clicked(){self.updater.request();self.update_seen=None;}
                    ui.label(match &self.updater.state {
                        crate::updates::State::Idle=>"Updates have not been checked yet".into(),
                        crate::updates::State::Checking=>"Checking for updates...".into(),
                        crate::updates::State::Current=>"You have the latest version".into(),
                        crate::updates::State::Available(r)=>format!("New version available: {}",r.version.text()),
                        crate::updates::State::Failed(message)=>message.clone(),
                    });
                    ui.add_space(8.);
                    apply=ui.button("Save and apply").clicked();
                    if !self.error.is_empty() { ui.colored_label(Color32::LIGHT_RED,&self.error); }
                });
            },
        );
        if std::mem::take(&mut self.focus_settings) {
            let id = ViewportId::from_hash_of("receiver-settings");
            ctx.send_viewport_cmd_to(id, ViewportCommand::Visible(true));
            ctx.send_viewport_cmd_to(id, ViewportCommand::Focus);
        }
        if closed {
            self.show_settings = false;
        }
        if apply {
            match self.settings.save() {
                Ok(()) => {
                    self.error.clear();
                    let size = crate::settings::window_size(
                        self.settings.window_width,
                        self.settings.window_height,
                        self.settings.lock_aspect,
                    );
                    self.settings.window_width = size[0];
                    self.settings.window_height = size[1];
                    ctx.send_viewport_cmd(ViewportCommand::InnerSize(vec2(size[0], size[1])));
                    let connection = (
                        self.settings.stream_url.clone(),
                        self.settings.metadata_url.clone(),
                        self.settings.device.clone(),
                    );
                    if self.engine.running() && connection != self.applied_connection {
                        self.engine.start(self.settings.clone());
                    }
                    self.applied_connection = connection;
                    self.show_settings = false;
                }
                Err(e) => self.error = e.to_string(),
            }
        }
    }
    fn marquee(&self, p: &Painter, r: Rect, s: &str, size: f32) {
        let font = FontId::new(size, FontFamily::Name("vfd".into()));
        let galley = p.layout_no_wrap(s.to_owned(), font, CYAN);
        let w = galley.size().x;
        let clipped = p.with_clip_rect(r);
        if w <= r.width() {
            glow(&clipped, r.min, s, size);
        } else {
            let phase = (self.track_changed.elapsed().as_secs_f32() - 4.).max(0.) * 25.;
            let offset = phase % (w + 45.);
            glow(&clipped, r.min - vec2(offset, 0.), s, size);
            glow(&clipped, r.min + vec2(w + 45. - offset, 0.), s, size);
        }
    }
    fn update_notice(&mut self, ctx: &Context) {
        if !self.show_update {
            return;
        }
        let crate::updates::State::Available(release) = self.updater.state.clone() else {
            return;
        };
        let mut dismiss = false;
        Window::new("Dostupna je nova verzija").collapsible(false).resizable(false).default_width(420.).anchor(Align2::CENTER_CENTER,Vec2::ZERO).open(&mut self.show_update).show(ctx,|ui|{
            ui.heading(format!("Radio Crash Desktop {}",release.version.text()));
            ui.label(format!("Tvoja verzija: {}",env!("CARGO_PKG_VERSION")));
            ui.add_space(8.);
            ui.label("Preuzmi novi ZIP, zatvori player i zamijeni aplikaciju novom verzijom. Tvoje postavke ostaju spremljene.");
            ui.add_space(10.);
            ui.horizontal(|ui|{
                if ui.button("Preuzmi novu verziju").clicked(){ctx.open_url(OpenUrl::new_tab(release.download()));dismiss=true;}
                if ui.button("Što je novo").clicked(){ctx.open_url(OpenUrl::new_tab(release.page()));}
                if ui.button("Kasnije").clicked(){dismiss=true;}
            });
        });
        if dismiss {
            self.show_update = false;
        }
    }
}
impl eframe::App for RadioApp {
    fn logic(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        #[cfg(target_os = "windows")]
        if let Some(window) = &self.receiver_window
            && let Err(e) = platform::maintain_receiver_frame(window)
        {
            log::warn!("Receiver frame: {e}");
        }
        if self.clock_updated.elapsed() >= Duration::from_secs(1) {
            self.clock = crate::clock_face::ClockFace::now();
            self.clock_updated = Instant::now();
        }
        if self
            .qa_exit
            .is_some_and(|duration| self.start.elapsed() >= duration)
        {
            self.qa_exit = None;
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
        self.graphics.tick(self.settings.graphics_quality);
        self.updater.tick(true);
        if let crate::updates::State::Available(release) = &self.updater.state
            && self.update_seen != Some(release.version)
        {
            self.update_seen = Some(release.version);
            self.show_update = true;
        }
        let snapshot = self.engine.state.lock().unwrap().clone();
        let update_version = match &self.updater.state {
            crate::updates::State::Available(r) => Some(r.version.text()),
            _ => None,
        };
        let action = if let Some(tray) = &mut self.tray {
            tray.set_update(update_version.as_deref());
            tray.update(
                &snapshot.meters,
                self.engine.running(),
                if self.settings.skin == Skin::AnalogSilver {
                    self.settings.analog_reference_dbfs
                } else {
                    self.settings.vu_reference_dbfs
                },
                self.settings.fast_vu,
                self.settings.vu_response,
                self.settings.skin == Skin::AnalogSilver,
            );
            tray.poll()
        } else {
            platform::TrayAction::None
        };
        match action {
            platform::TrayAction::Restore => self.restore_from_tray(ctx),
            platform::TrayAction::Quit => ctx.send_viewport_cmd(ViewportCommand::Close),
            platform::TrayAction::Updates => {
                self.restore_from_tray(ctx);
                if matches!(self.updater.state, crate::updates::State::Available(_)) {
                    self.show_update = true;
                } else {
                    self.updater.request();
                    self.show_settings = true;
                    self.focus_settings = true;
                }
            }
            platform::TrayAction::None => {}
        }
        if self.settings.minimize_to_tray
            && self.tray.is_some()
            && !self.hidden_to_tray
            && Instant::now() > self.restore_until
            && ctx.input(|i| i.viewport().minimized.unwrap_or(false))
        {
            self.minimize_to_tray(ctx);
        }
        // This timer also drives App::logic when no GUI frame is painted.
        ctx.request_repaint_after(Duration::from_millis(if self.engine.running() {
            33
        } else {
            150
        }));
        if self.tray_check.is_some() {
            let seconds = self.start.elapsed().as_secs_f32();
            if self.tray_check_phase == 0 && seconds > 2. {
                if let Some(tray) = &self.tray {
                    self.tray_check_before = (tray.ticks, tray.uploads);
                }
                self.minimize_to_tray(ctx);
                self.tray_check_phase = 1;
                log::info!("TRAY_CHECK hidden");
            }
            if self.tray_check_phase == 1 && seconds > 8. {
                if let Some(tray) = &self.tray {
                    let ticks = tray.ticks - self.tray_check_before.0;
                    let uploads = tray.uploads - self.tray_check_before.1;
                    let dir = self.tray_check.as_ref().unwrap();
                    let _ = std::fs::create_dir_all(dir);
                    let _ = tray.export(&dir.join("tray-live.png"));
                    let report = serde_json::json!({"hidden_ticks":ticks,"changed_icons":uploads,"status":snapshot.status,"rms_dbfs":snapshot.meters.rms,"tray_only":platform::is_tray_mode(),"success":ticks>20&&uploads>0&&platform::is_tray_mode()});
                    let _ = std::fs::write(dir.join("tray-check.json"), report.to_string());
                    log::info!("TRAY_CHECK {report}");
                }
                self.restore_from_tray(ctx);
                self.tray_check_phase = 2;
            }
            if self.tray_check_phase == 2 && seconds > 12. {
                ctx.send_viewport_cmd(ViewportCommand::Close);
                self.tray_check_phase = 3;
            }
        }
    }
    fn clear_color(&self, _: &Visuals) -> [f32; 4] {
        [0.; 4]
    }
    fn ui(&mut self, ui: &mut Ui, _: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        ctx.request_repaint_after(Duration::from_millis(
            if ctx.input(|i| i.viewport().minimized.unwrap_or(false)) || !self.engine.running() {
                200
            } else {
                16
            },
        ));
        let now = Instant::now();
        let dt = (now - self.last).as_secs_f32().min(0.2);
        self.last = now;
        let snap = self.engine.state.lock().unwrap().clone();
        if self.displayed_track.0 != snap.artist || self.displayed_track.1 != snap.title {
            self.displayed_track = (snap.artist.clone(), snap.title.clone());
            self.track_changed = now;
        }
        for c in 0..2 {
            let target = if self.engine.running() {
                snap.meters.vu(self.settings.vu_response).level[c]
            } else {
                -60.
            };
            self.meters.rms[c] = crate::dsp::follow(
                self.meters.rms[c],
                target,
                dt,
                if self.settings.fast_vu { 180. } else { 120. },
            );
        }
        for i in 0..10 {
            let t = if self.engine.running() {
                snap.meters.spectrum[i]
            } else {
                -60.
            };
            self.meters.spectrum[i] = crate::dsp::follow(self.meters.spectrum[i], t, dt, 90.);
        }
        let area = ui.max_rect();
        let panel = Panel::new(area);
        let scale = panel.scale;
        let base = panel.origin;
        let rect = |x, y, w, h| panel.rect([x, y, w, h]);
        let p = ui.painter().clone();
        p.rect_filled(area, 0., Color32::TRANSPARENT);
        let analog = self.settings.skin == Skin::AnalogSilver;
        let controls = panel::Controls::for_skin(self.settings.skin);
        let hd = snap.bitrate.is_some_and(|v| v >= self.settings.hd_kbps) && self.engine.running();
        if analog {
            self.analog
                .get_or_insert_with(|| crate::analog::AnalogSkin::new(&ctx))
                .background(&p, panel);
        } else {
            self.materials.paint(&p, rect(0., 0., 1200., 440.));
            self.materials
                .logo(&p, rect(94., 49., 205., 205. * 67. / 900.));
        }
        let screw = minimize_screw(ui, panel.rect(panel::MINIMIZE));
        if screw.clicked() {
            self.minimize_to_tray(&ctx);
        }
        let drag = ui.interact(
            panel.rect(panel::DRAG),
            Id::unique("chassis-drag"),
            Sense::click_and_drag(),
        );
        if drag.drag_started() {
            ctx.send_viewport_cmd(ViewportCommand::StartDrag);
        }
        drag.context_menu(|ui| {
            for (label, width) in [
                ("Small · 640", 640.),
                ("Everyday · 780", 780.),
                ("Large · 1000", 1000.),
            ] {
                if ui.button(label).clicked() {
                    ctx.send_viewport_cmd(ViewportCommand::InnerSize(vec2(
                        width,
                        width / crate::settings::PANEL_ASPECT,
                    )));
                    ui.close();
                }
            }
            ui.separator();
            if ui.button("Receiver settings").clicked() {
                self.show_settings = true;
                self.focus_settings = true;
                ui.close();
            }
            if ui
                .add_enabled(
                    self.tray.is_some(),
                    Button::new("Minimize to tray · L/R meters"),
                )
                .clicked()
            {
                self.minimize_to_tray(&ctx);
                ui.close();
            }
            if ui.button("Minimize to Dock / taskbar").clicked() {
                ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
                ui.close();
            }
            if ui.button("Quit Radio Crash").clicked() {
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
        });
        let standby_clock = self.settings.standby_clock && !self.engine.running();
        if analog {
            self.analog.as_mut().unwrap().instruments(
                &p,
                panel,
                &snap,
                &self.settings,
                self.engine.running(),
                self.power,
                &self.clock,
                self.track_changed.elapsed().as_secs_f32(),
                dt,
                self.mute,
            );
            self.graphics.paint(
                ui.painter(),
                panel.rect(crate::analog::GLASS),
                true,
                self.settings.vfd_brightness,
                0.,
                self.settings.volume,
            );
        } else {
            // Coordinates below map the live phosphor plane inside the photographed glass aperture.
            let pt = |x: f32, y: f32| {
                base + vec2(
                    70. + (x - 48.) * 1060. / 1104.,
                    99. + (y - 111.) * 136. / 175.,
                ) * scale
            };
            let rect = |x: f32, y: f32, w: f32, h: f32| {
                Rect::from_min_size(pt(x, y), vec2(w * 1060. / 1104., h * 136. / 175.) * scale)
            };
            let scale = scale * 0.9;
            let standby_clock = self.settings.standby_clock && !self.engine.running();
            let mut p = p.clone();
            p.multiply_opacity(crate::settings::display_brightness(
                self.settings.vfd_brightness,
            ));
            for x in [443., 820.] {
                p.line_segment(
                    [pt(x, 129.), pt(x, 269.)],
                    Stroke::new(0.6 * scale, Color32::from_rgb(31, 62, 63)),
                );
            }
            let muted_display = if self.power || standby_clock {
                CYAN
            } else {
                DIM
            };
            text(
                &p,
                pt(70., 129.),
                "STEREO LEVEL",
                9.2 * scale,
                muted_display.gamma_multiply(0.7),
            );
            text(
                &p,
                pt(348., 129.),
                "PEAK HOLD",
                8.5 * scale,
                Color32::from_rgb(202, 172, 69),
            );
            for (db, caption) in [
                (-40., "−40"),
                (-12., "−12"),
                (-8., "−8"),
                (-4., "−4"),
                (0., "0"),
                (4., "+4"),
                (8., "+8"),
                (12., "+12"),
                (18., "+18"),
            ] {
                let x = 98. + crate::dsp::vu_position(db) * 312.;
                let color = vu_color(db);
                p.text(
                    pt(x + 3., 153.),
                    Align2::CENTER_TOP,
                    caption,
                    FontId::monospace(9.4 * scale),
                    color,
                );
                p.line_segment(
                    [pt(x + 3., 169.), pt(x + 3., 175.)],
                    Stroke::new(
                        if db == 0. { 1.3 * scale } else { 0.7 * scale },
                        color.gamma_multiply(0.65),
                    ),
                );
            }
            text(&p, pt(70., 153.), "dB", 8.5 * scale, DIM);
            for c in 0..2 {
                let y = 187. + c as f32 * 38.;
                text(
                    &p,
                    pt(70., y - 2.),
                    if c == 0 { "L" } else { "R" },
                    17. * scale,
                    muted_display,
                );
                let level = crate::dsp::vu_segment_index(
                    self.meters.rms[c],
                    self.settings.vu_reference_dbfs,
                );
                let held = crate::dsp::vu_segment_index(
                    snap.meters.vu(self.settings.vu_response).held[c],
                    self.settings.vu_reference_dbfs,
                );
                for (i, db) in crate::dsp::VU_SCALE.iter().enumerate() {
                    let x = 98. + i as f32 * 8.;
                    // Keep the approved full, zone-colored filament in the L/R
                    // row. Hold is the maximum of the SAME selected detector.
                    let on = level.is_some_and(|last| i <= last) || held == Some(i);
                    for row in 0..2 {
                        segment_with_color(
                            &p,
                            rect(x, y + row as f32 * 5.3, 6., 3.7),
                            on,
                            vu_color(*db),
                        );
                    }
                }
            }
            text(
                &p,
                pt(99., 257.),
                match self.settings.vu_response {
                    crate::dsp::VuResponse::SmoothRms => "RMS",
                    crate::dsp::VuResponse::FastRms => "FAST RMS",
                    crate::dsp::VuResponse::Peak => "PEAK",
                },
                7.8 * scale,
                DIM,
            );
            let calibration = if self.settings.vu_reference_dbfs == 0. {
                "DIGITAL dBFS".to_owned()
            } else {
                format!("0 VU = {} dBFS", self.settings.vu_reference_dbfs)
            };
            text(&p, pt(245., 257.), &calibration, 7.8 * scale, DIM);
            let status = if standby_clock {
                "STANDBY"
            } else if !self.power {
                "POWER OFF"
            } else {
                &snap.status
            };
            text(
                &p,
                pt(466., 129.),
                if standby_clock {
                    "LOCAL TIME"
                } else {
                    "NOW PLAYING"
                },
                9.5 * scale,
                muted_display.gamma_multiply(0.7),
            );
            let status_size = if status.len() > 10 { 8.5 } else { 10. };
            p.text(
                pt(797., 129.),
                Align2::RIGHT_TOP,
                status,
                FontId::monospace(status_size * scale),
                muted_display,
            );
            if status == "ON AIR" {
                p.circle_filled(pt(732., 134.), 2.5 * scale, CYAN);
                p.circle_filled(
                    pt(732., 134.),
                    4.6 * scale,
                    Color32::from_rgba_unmultiplied(80, 245, 245, 16),
                );
            }
            let artist = if snap.artist.is_empty() {
                "RADIO CRASH"
            } else {
                &snap.artist
            };
            let title = if snap.title.is_empty() {
                if self.engine.running() {
                    "Waiting for metadata"
                } else {
                    "Good music lives on"
                }
            } else {
                &snap.title
            };
            if standby_clock {
                let font = FontId::new(43. * scale, FontFamily::Name("vfd".into()));
                let center = pt(631., 158.);
                for offset in [vec2(-0.7, 0.), vec2(0.7, 0.), vec2(0., -0.7), vec2(0., 0.7)] {
                    p.text(
                        center + offset * scale,
                        Align2::CENTER_TOP,
                        &self.clock.time,
                        font.clone(),
                        Color32::from_rgba_unmultiplied(65, 232, 245, 25),
                    );
                }
                p.text(
                    center,
                    Align2::CENTER_TOP,
                    &self.clock.time,
                    font,
                    muted_display,
                );
                p.text(
                    pt(631., 218.),
                    Align2::CENTER_TOP,
                    &self.clock.date,
                    FontId::monospace(11. * scale),
                    CYAN.gamma_multiply(0.75),
                );
            } else {
                self.marquee(&p, rect(466., 158., 331., 34.), artist, 26. * scale);
                self.marquee(&p, rect(466., 201., 331., 28.), title, 20.5 * scale);
            }
            p.line_segment(
                [pt(465., 242.), pt(799., 242.)],
                Stroke::new(0.6 * scale, DIM),
            );
            let tech = format!(
                "{}   {}   {}   {}",
                snap.bitrate
                    .map(|v| format!("{v} kbps"))
                    .unwrap_or("- kbps".into()),
                if snap.codec.is_empty() {
                    "-"
                } else {
                    &snap.codec
                },
                if snap.rate > 0 {
                    format!("{:.1} kHz", snap.rate as f32 / 1000.)
                } else {
                    "- kHz".into()
                },
                match snap.channels {
                    1 => "MONO",
                    2 => "STEREO",
                    _ => "-",
                }
            );
            text(
                &p,
                pt(467., 255.),
                if standby_clock {
                    "RADIO CRASH  ·  SINCE 1986"
                } else {
                    &tech
                },
                10. * scale,
                muted_display,
            );
            text(
                &p,
                pt(842., 129.),
                "SPECTRUM ANALYZER",
                9. * scale,
                muted_display.gamma_multiply(0.7),
            );
            text(&p, pt(1117., 129.), "Hz", 8. * scale, DIM);
            for (y, l) in [(148., "0"), (179., "−20"), (210., "−40"), (241., "−60")] {
                p.text(
                    pt(850., y),
                    Align2::RIGHT_TOP,
                    l,
                    FontId::monospace(7. * scale),
                    DIM,
                );
            }
            for i in 0..10 {
                let x = 860. + i as f32 * 27.;
                for j in 0..20 {
                    segment(
                        &p,
                        rect(x, 243. - j as f32 * 4.6, 17., 2.9),
                        self.settings.spectrum
                            && snap.meters.available[i]
                            && self.meters.spectrum[i] > -60. + j as f32 * 3.,
                        j,
                        20,
                    );
                }
                p.text(
                    pt(x + 8.5, 257.),
                    Align2::CENTER_TOP,
                    [
                        "32", "64", "125", "250", "500", "1k", "2k", "4k", "8k", "16k",
                    ][i],
                    FontId::monospace(8.3 * scale),
                    if snap.meters.available[i] {
                        CYAN.gamma_multiply(0.85)
                    } else {
                        DIM
                    },
                );
            }
            // Fine optical reflections are composited over the emitted phosphor, keeping it behind glass.
            gradient(
                &p,
                rect(51., 114., 1095., 46.),
                Color32::from_rgba_unmultiplied(154, 192, 198, 9),
                Color32::TRANSPARENT,
            );
            text(
                &p,
                pt(466., 273.),
                "HD SOUND",
                7.7 * scale,
                if hd {
                    Color32::from_rgb(83, 228, 207)
                } else {
                    DIM
                },
            );
            p.text(
                pt(798., 273.),
                Align2::RIGHT_TOP,
                if self.mute {
                    "MUTED".into()
                } else {
                    format!("VOL {:02}%", (self.settings.volume * 100.).round() as u32)
                },
                FontId::monospace(7.7 * scale),
                if self.mute {
                    Color32::from_rgb(238, 176, 77)
                } else {
                    DIM
                },
            );
            self.graphics.paint(
                ui.painter(),
                panel.rect([70., 99., 1060., 136.]),
                true,
                self.settings.vfd_brightness,
                0.,
                self.settings.volume,
            );
        }
        // The photographed hardware surfaces are native interactive controls, never baked indicators.
        let p = ui.painter().clone();
        let scale = (area.width() / 1200.).min(area.height() / 440.);
        let pt = |x: f32, y: f32| base + vec2(x, y) * scale;
        let rect =
            |x: f32, y: f32, w: f32, h: f32| Rect::from_min_size(pt(x, y), vec2(w, h) * scale);
        if !analog {
            let badge = rect(458., 307., 164., 43.);
            hd_radio(
                &p,
                badge,
                self.power,
                hd && snap.status == "ON AIR" && self.power,
            );
            ui.interact(badge,Id::unique("hd-radio-quality"),Sense::hover()).on_hover_text(format!("Radio Crash oznaka kvalitete streama. Pun sjaj od {} kbps. Internetski radio; nije HD Radio/IBOC prijamnik.",self.settings.hd_kbps));
        }
        let power = self.button(ui, panel.rect(controls.power), "power", self.power);
        if std::mem::take(&mut self.power_menu_on_launch) {
            Popup::open_id(&ctx, Popup::default_response_id(&power));
        }
        if let Some(action) = panel::power_menu(&power, self.power) {
            self.power = action == panel::PowerAction::On;
            if self.power {
                self.engine.start(self.settings.clone());
            } else {
                self.engine.stop();
            }
        }
        let running = self.engine.running();
        if self
            .button(ui, panel.rect(controls.play), "transport", running)
            .clicked()
            && self.power
        {
            if running {
                self.engine.stop();
            } else {
                self.engine.start(self.settings.clone());
            }
        }
        if self
            .button(ui, panel.rect(controls.mute), "mute", self.mute)
            .clicked()
        {
            self.mute = !self.mute;
        }
        let center = pt(controls.knob_center[0], controls.knob_center[1]);
        let radius = controls.knob_radius * scale;
        let knob = ui
            .interact(
                panel.rect(controls.volume),
                Id::unique("volume"),
                Sense::click_and_drag(),
            )
            .on_hover_cursor(CursorIcon::Grab)
            .on_hover_text(format!(
                "Volume: {:.0}% · drag or scroll · double-click to mute",
                self.settings.volume * 100.
            ));
        if knob.dragged() {
            self.settings.volume = (self.settings.volume - knob.drag_delta().y / (180. * scale)
                + knob.drag_delta().x / (260. * scale))
                .clamp(0., 1.);
        }
        if knob.hovered() {
            self.settings.volume = (self.settings.volume
                + ui.input(|i| i.smooth_scroll_delta.y) * 0.002)
                .clamp(0., 1.);
        }
        if knob.double_clicked() {
            self.mute = !self.mute;
        }
        let a = 2.3561945 + self.settings.volume * std::f32::consts::PI * 1.5;
        let d = vec2(a.cos(), a.sin());
        p.line_segment(
            [center + d * radius * 0.72, center + d * radius * 0.87],
            Stroke::new(
                1.4 * scale,
                if analog {
                    Color32::from_gray(26)
                } else {
                    Color32::from_rgb(213, 214, 193)
                },
            ),
        );
        if self
            .button(ui, panel.rect(controls.settings), "settings", false)
            .clicked()
        {
            self.show_settings = true;
            self.focus_settings = true;
        }
        if !snap.error.is_empty() {
            self.error = snap.error.clone();
        }
        self.engine.volume(if self.mute || !self.power {
            0.
        } else {
            self.settings.volume
        });
        if (knob.drag_stopped() || knob.hovered() && ui.input(|i| i.smooth_scroll_delta.y != 0.))
            && let Err(e) = self.settings.save()
        {
            self.error = e.to_string();
        }
        if ui.input(|i| i.modifiers.command && i.key_pressed(Key::M)) {
            if self.settings.minimize_to_tray {
                self.minimize_to_tray(&ctx);
            } else {
                ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
            }
        }
        let signal = if standby_clock {
            0.18
        } else if self.engine.running() {
            10.0_f32.powf(self.meters.rms[0].max(self.meters.rms[1]) / 20.)
        } else {
            0.
        };
        if !analog {
            self.graphics.paint(
                ui.painter(),
                panel.rect([0., 0., 1200., 440.]),
                false,
                self.settings.vfd_brightness,
                signal,
                self.settings.volume,
            );
        }
        self.window_controls(ui);
        if self.show_settings {
            self.settings(&ctx);
        }
        self.update_notice(&ctx);
        if self.start.elapsed() > Duration::from_secs(4)
            && let Some(path) = self.capture.take()
        {
            ctx.request_screenshot(move |frame| {
                let bytes: Vec<u8> = frame.pixels.iter().flat_map(|c| c.to_array()).collect();
                if let Err(e) = image::save_buffer(
                    &path,
                    &bytes,
                    frame.size[0] as u32,
                    frame.size[1] as u32,
                    image::ColorType::Rgba8,
                ) {
                    log::error!("Capture: {e}");
                } else {
                    log::info!("Saved native front panel to {}", path.display());
                }
            });
        }
    }

    fn on_exit(&mut self, gl: Option<&eframe::glow::Context>) {
        self.graphics.destroy(gl);
        if let Err(e) = self.settings.save() {
            log::error!("save settings: {e}");
        }
        self.engine.stop();
    }
}

#[cfg(test)]
mod interaction_tests {
    use super::*;
    #[test]
    fn skin_menu_selects_analog_then_restores_the_original_skin() {
        fn draw(ctx: &Context, skin: &mut Skin, events: Vec<Event>) -> FullOutput {
            let mut out = ctx.run_ui(
                RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(700., 400.))),
                    events,
                    ..Default::default()
                },
                |ui| {
                    skin_selector(ui, skin);
                },
            );
            out.textures_delta.clear();
            out
        }
        fn label(out: &FullOutput, text: &str) -> Pos2 {
            out.shapes
                .iter()
                .find_map(|s| match &s.shape {
                    Shape::Text(t) if t.galley.text() == text => {
                        Some(t.pos + t.galley.size() * 0.5)
                    }
                    _ => None,
                })
                .unwrap_or_else(|| panic!("missing skin choice: {text}"))
        }
        fn click(ctx: &Context, skin: &mut Skin, point: Pos2) {
            for pressed in [true, false] {
                draw(
                    ctx,
                    skin,
                    vec![
                        Event::PointerMoved(point),
                        Event::PointerButton {
                            pos: point,
                            button: PointerButton::Primary,
                            pressed,
                            modifiers: Modifiers::NONE,
                        },
                    ],
                );
            }
        }
        let ctx = Context::default();
        let mut selected = Skin::VfdBlack;
        for next in [Skin::AnalogSilver, Skin::VfdBlack] {
            draw(&ctx, &mut selected, vec![]);
            let out = draw(&ctx, &mut selected, vec![]);
            let point = label(&out, selected.label());
            click(&ctx, &mut selected, point);
            let out = draw(&ctx, &mut selected, vec![]);
            let point = label(&out, next.label());
            click(&ctx, &mut selected, point);
            assert_eq!(selected, next);
        }
    }
    #[test]
    fn left_screw_accepts_primary_click_at_compact_and_large_sizes() {
        for width in [560., 640., 780., 1200.] {
            let ctx = Context::default();
            let scale = width / 1200.;
            let area = Rect::from_min_size(Pos2::ZERO, vec2(width, 440. * scale));
            let panel = Panel::new(area);
            let r = panel.rect(panel::MINIMIZE);
            let point = r.center();
            let mut clicked = false;
            for (frame, events) in [
                vec![],
                vec![
                    Event::PointerMoved(point),
                    Event::PointerButton {
                        pos: point,
                        button: PointerButton::Primary,
                        pressed: true,
                        modifiers: Modifiers::NONE,
                    },
                ],
                vec![Event::PointerButton {
                    pos: point,
                    button: PointerButton::Primary,
                    pressed: false,
                    modifiers: Modifiers::NONE,
                }],
            ]
            .into_iter()
            .enumerate()
            {
                let input = RawInput {
                    screen_rect: Some(area),
                    time: Some(frame as f64 / 60.),
                    events,
                    ..Default::default()
                };
                let mut output = ctx.run_ui(input, |root| {
                    CentralPanel::default().frame(Frame::NONE).show(root, |ui| {
                        clicked |= minimize_screw(ui, r).clicked();
                        // The chassis drag target must not steal screw clicks.
                        ui.interact(
                            panel.rect(panel::DRAG),
                            Id::unique("test-drag"),
                            Sense::click_and_drag(),
                        );
                    });
                });
                output.textures_delta.clear();
            }
            assert!(clicked, "width={width}");
        }
    }
}
