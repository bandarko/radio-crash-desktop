//! One logical-point transform for fascia, painted controls and pointer targets.
//! OS/egui converts physical mouse pixels once; do not apply DPI scaling here.
use eframe::egui::*;

pub const POWER: [f32; 4] = [84., 302., 66., 55.];
pub const PLAY: [f32; 4] = [222., 299., 128., 60.];
pub const MUTE: [f32; 4] = [699., 302., 79., 55.];
pub const SETTINGS: [f32; 4] = [1060., 303., 60., 52.];
pub const EXIT: [f32; 4] = [1063., 23., 76., 25.];
pub const MINIMIZE: [f32; 4] = [14., 19., 43., 41.];
pub const DRAG: [f32; 4] = [62., 30., 987., 62.];

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

pub fn exit_button(ui: &mut Ui, rect: Rect) -> Response {
    let response = click_target(ui, rect, "receiver-exit")
        .on_hover_text("Exit Radio Crash - stop audio and close the application");
    let s = rect.height() / 25.;
    let p = ui.painter();
    // A small recessed equipment switch above the heritage inscription.
    p.rect_filled(rect, 3. * s, Color32::from_rgb(16, 18, 18));
    p.rect_stroke(
        rect,
        3. * s,
        Stroke::new(0.8 * s, Color32::from_gray(84)),
        StrokeKind::Inside,
    );
    let ink = if response.hovered() {
        Color32::from_rgb(255, 174, 143)
    } else {
        Color32::from_rgb(205, 205, 195)
    };
    p.text(
        rect.center(),
        Align2::CENTER_CENTER,
        "EXIT",
        FontId::monospace(11. * s),
        ink,
    );
    if response.clicked() {
        ui.ctx().send_viewport_cmd(ViewportCommand::Close);
    }
    response
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
            exit_button(root, panel.rect(EXIT));
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
    #[test]
    fn exit_closes_root_instead_of_minimizing() {
        for dpi in [1., 1.25, 1.5, 2.] {
            let ctx = Context::default();
            let area = Rect::from_min_size(Pos2::ZERO, vec2(640., 235.));
            let point = Panel::new(area).rect(EXIT).center();
            let (clicked, out) = click(&ctx, area, dpi, point);
            assert!(clicked.is_empty());
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
    fn blank_metal_does_not_activate_neighboring_controls() {
        let ctx = Context::default();
        let area = Rect::from_min_size(pos2(25., 15.), vec2(1000., 500.));
        let point = Panel::new(area).point(188., 325.);
        assert!(click(&ctx, area, 1., point).0.is_empty());
    }
}
