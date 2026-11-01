//! One logical-point transform for fascia, painted controls and pointer targets.
//! OS/egui converts physical mouse pixels once; do not apply DPI scaling here.
use eframe::egui::*;

pub const POWER: [f32; 4] = [84., 302., 66., 55.];
pub const PLAY: [f32; 4] = [222., 299., 128., 60.];
pub const MUTE: [f32; 4] = [699., 302., 79., 55.];
pub const SETTINGS: [f32; 4] = [1060., 303., 60., 52.];
pub const MINIMIZE: [f32; 4] = [14., 19., 43., 41.];
pub const DRAG: [f32; 4] = [62., 30., 1093., 62.];

#[derive(Clone, Copy)]
pub struct Panel {
    pub origin: Pos2,
    pub scale: f32,
}
impl Panel {
    pub fn new(area: Rect) -> Self {
        let scale = (area.width() / 1200.).min(area.height() / 440.);
        Self {
            origin: area.center() - vec2(600., 220.) * scale,
            scale,
        }
    }
    pub fn point(self, x: f32, y: f32) -> Pos2 {
        self.origin + vec2(x, y) * self.scale
    }
    pub fn rect(self, [x, y, w, h]: [f32; 4]) -> Rect {
        Rect::from_min_size(self.point(x, y), vec2(w, h) * self.scale)
    }
    pub fn volume(self) -> Rect {
        Rect::from_center_size(self.point(911., 319.), vec2(124., 124.) * self.scale)
    }
}

pub fn click_target(ui: &mut Ui, rect: Rect, name: &str) -> Response {
    ui.interact(rect, Id::unique(name), Sense::click())
        .on_hover_cursor(CursorIcon::PointingHand)
}

#[derive(Debug, PartialEq, Eq)]
pub enum PowerAction {
    On,
    Off,
}

/// The existing physical POWER switch owns standby and application exit.
/// Opening or dismissing this menu never changes playback by itself.
pub fn power_menu(response: &Response, powered: bool) -> Option<PowerAction> {
    let mut action = None;
    Popup::menu(response)
        .align(emath::RectAlign::TOP_START)
        .frame(
            Frame::popup(&response.ctx.global_style())
                .fill(Color32::from_rgb(22, 25, 25))
                .stroke(Stroke::new(1., Color32::from_rgb(100, 121, 123)))
                .inner_margin(8.),
        )
        .show(|ui| {
            // The popup has a dark hardware surface even when macOS/Windows
            // uses a light theme. Scope both foreground and interaction colors
            // here instead of inheriting dark-on-light system text.
            *ui.visuals_mut() = Visuals::dark();
            ui.visuals_mut().override_text_color = Some(Color32::from_rgb(240, 246, 245));
            ui.visuals_mut().widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
            ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::NONE;
            ui.visuals_mut().widgets.hovered.bg_fill = Color32::from_rgb(39, 72, 76);
            ui.visuals_mut().widgets.hovered.weak_bg_fill = Color32::from_rgb(39, 72, 76);
            ui.visuals_mut().widgets.hovered.bg_stroke =
                Stroke::new(1., Color32::from_rgb(100, 224, 231));
            ui.visuals_mut().widgets.active.weak_bg_fill = Color32::from_rgb(49, 89, 93);
            ui.visuals_mut().widgets.active.bg_fill = Color32::from_rgb(49, 89, 93);
            ui.set_min_width(160.);
            let choice = |label: &str| {
                Button::new(RichText::new(label).size(16.).strong()).min_size(vec2(160., 36.))
            };
            if ui
                .add(choice(if powered { "OFF" } else { "ON" }))
                .on_hover_text(if powered {
                    "Stop radio and enter standby"
                } else {
                    "Turn on and play Radio Crash"
                })
                .clicked()
            {
                action = Some(if powered {
                    PowerAction::Off
                } else {
                    PowerAction::On
                });
                ui.close();
            }
            ui.separator();
            if ui
                .add(choice("EXIT"))
                .on_hover_text("Stop audio and close the entire application")
                .clicked()
            {
                ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                ui.close();
            }
        });
    action
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(ctx: &Context, area: Rect, dpi: f32, events: Vec<Event>) -> (Vec<usize>, FullOutput) {
        let mut input = RawInput {
            screen_rect: Some(area),
            events,
            ..Default::default()
        };
        input
            .viewports
            .get_mut(&ViewportId::ROOT)
            .unwrap()
            .native_pixels_per_point = Some(dpi);
        let mut clicked = Vec::new();
        let mut out = ctx.run_ui(input, |root| {
            let panel = Panel::new(root.max_rect());
            for (n, geometry) in [POWER, PLAY, MUTE, SETTINGS, MINIMIZE]
                .into_iter()
                .enumerate()
            {
                if click_target(root, panel.rect(geometry), &format!("control-{n}")).clicked() {
                    clicked.push(n);
                }
            }
            root.interact(
                panel.volume(),
                Id::unique("volume"),
                Sense::click_and_drag(),
            );
            root.interact(
                panel.rect(DRAG),
                Id::unique("chassis-drag"),
                Sense::click_and_drag(),
            );
        });
        out.textures_delta.clear();
        (clicked, out)
    }
    fn click(ctx: &Context, area: Rect, dpi: f32, point: Pos2) -> (Vec<usize>, FullOutput) {
        frame(ctx, area, dpi, vec![]);
        frame(
            ctx,
            area,
            dpi,
            vec![
                Event::PointerMoved(point),
                Event::PointerButton {
                    pos: point,
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
        frame(
            ctx,
            area,
            dpi,
            vec![Event::PointerButton {
                pos: point,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            }],
        )
    }
    #[test]
    fn entire_visible_buttons_click_at_all_scales_and_aspects() {
        for dpi in [1., 1.25, 1.5, 2.] {
            for size in [
                vec2(560., 206.),
                vec2(780., 286.),
                vec2(1200., 440.),
                vec2(1000., 500.),
                vec2(1200., 300.),
            ] {
                let ctx = Context::default();
                let area = Rect::from_min_size(pos2(0., 0.), size);
                let panel = Panel::new(area);
                for (n, geometry) in [POWER, PLAY, MUTE, SETTINGS, MINIMIZE]
                    .into_iter()
                    .enumerate()
                {
                    let r = panel.rect(geometry).shrink(2. / dpi);
                    for point in [
                        r.center(),
                        r.left_top(),
                        r.right_top(),
                        r.left_bottom(),
                        r.right_bottom(),
                    ] {
                        let (clicked, _) = click(&ctx, area, dpi, point);
                        assert_eq!(
                            clicked,
                            [n],
                            "dpi={dpi}, size={size:?}, control={n}, point={point:?}"
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn pointer_targets_follow_dpi_and_client_size_changes() {
        let ctx = Context::default();
        for (dpi, size) in [
            (1., vec2(780., 286.)),
            (1.5, vec2(640., 235.)),
            (2., vec2(1200., 440.)),
            (1., vec2(1000., 500.)),
        ] {
            let area = Rect::from_min_size(pos2(0., 0.), size);
            let point = Panel::new(area).rect(SETTINGS).center();
            assert_eq!(click(&ctx, area, dpi, point).0, [3]);
        }
    }
    fn menu_frame(
        ctx: &Context,
        powered: bool,
        events: Vec<Event>,
    ) -> (Option<PowerAction>, FullOutput) {
        let mut action = None;
        let mut out = ctx.run_ui(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(640., 235.))),
                events,
                ..Default::default()
            },
            |root| {
                let panel = Panel::new(root.max_rect());
                let response = click_target(root, panel.rect(POWER), "power");
                action = power_menu(&response, powered);
            },
        );
        out.textures_delta.clear();
        (action, out)
    }
    fn menu_click(ctx: &Context, powered: bool, point: Pos2) -> (Option<PowerAction>, FullOutput) {
        menu_frame(
            ctx,
            powered,
            vec![
                Event::PointerMoved(point),
                Event::PointerButton {
                    pos: point,
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
        menu_frame(
            ctx,
            powered,
            vec![Event::PointerButton {
                pos: point,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            }],
        )
    }
    fn open_power(ctx: &Context, powered: bool) -> FullOutput {
        menu_frame(ctx, powered, vec![]);
        let area = Rect::from_min_size(Pos2::ZERO, vec2(640., 235.));
        let (action, _) = menu_click(ctx, powered, Panel::new(area).rect(POWER).center());
        assert_eq!(action, None, "opening the menu must not toggle playback");
        menu_frame(ctx, powered, vec![]).1
    }
    fn label_center(out: &FullOutput, label: &str) -> Pos2 {
        out.shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                Shape::Text(text) if text.galley.text() == label => {
                    Some(text.pos + text.galley.size() / 2.)
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("missing menu label {label}"))
    }
    #[test]
    fn power_menu_off_and_on_are_explicit_actions() {
        for (powered, label, expected) in [
            (true, "OFF", PowerAction::Off),
            (false, "ON", PowerAction::On),
        ] {
            let ctx = Context::default();
            let out = open_power(&ctx, powered);
            let point = label_center(&out, label);
            let (action, out) = menu_click(&ctx, powered, point);
            assert_eq!(action, Some(expected));
            assert!(
                !out.viewport_output[&ViewportId::ROOT]
                    .commands
                    .iter()
                    .any(|cmd| matches!(cmd, ViewportCommand::Close))
            );
        }
    }
    #[test]
    fn power_menu_exit_closes_root_instead_of_minimizing() {
        for powered in [true, false] {
            let ctx = Context::default();
            let out = open_power(&ctx, powered);
            let (action, out) = menu_click(&ctx, powered, label_center(&out, "EXIT"));
            assert_eq!(action, None);
            let commands = &out.viewport_output[&ViewportId::ROOT].commands;
            assert!(
                commands
                    .iter()
                    .any(|cmd| matches!(cmd, ViewportCommand::Close))
            );
            assert!(!commands.iter().any(|cmd| matches!(
                cmd,
                ViewportCommand::Visible(false) | ViewportCommand::Minimized(true)
            )));
        }
    }
    #[test]
    fn dismissing_power_menu_leaves_audio_state_alone() {
        let ctx = Context::default();
        open_power(&ctx, true);
        assert_eq!(menu_click(&ctx, true, pos2(500., 50.)).0, None);
        let (_, out) = menu_frame(&ctx, true, vec![]);
        assert!(
            !out.shapes.iter().any(
                |shape| matches!(&shape.shape,Shape::Text(text) if text.galley.text()=="EXIT")
            )
        );
    }
    #[test]
    fn blank_metal_does_not_activate_neighboring_controls() {
        let ctx = Context::default();
        let area = Rect::from_min_size(pos2(25., 15.), vec2(1000., 500.));
        let point = Panel::new(area).point(188., 325.);
        assert!(click(&ctx, area, 1., point).0.is_empty());
    }
}
