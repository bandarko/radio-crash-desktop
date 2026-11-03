//! Photographic silver analog skins. Instruments and metadata use live playback PCM.
use crate::{
    audio::Snapshot,
    clock_face::ClockFace,
    dsp_analog,
    panel::Panel,
    settings::Settings,
    skins::{self, Skin},
};
use eframe::egui::*;

const AMBER: Color32 = Color32::from_rgb(239, 178, 84);
const INK: Color32 = Color32::from_rgb(48, 37, 23);
const RED: Color32 = Color32::from_rgb(168, 42, 25);

/// All material apertures, live graphics and hit regions use this same 1200x440 grid.
#[derive(Clone, Copy)]
pub struct Layout {
    pub power: [f32; 4],
    pub play: [f32; 4],
    pub mute: [f32; 4],
    pub settings: [f32; 4],
    pub knob: [f32; 4],
    pub glass: [f32; 4],
    faces: [[f32; 4]; 2],
    pivots: [[f32; 2]; 2],
    radius: f32,
    vu_offset: f32,
    display: [f32; 4],
    ivory: bool,
}
pub fn layout(skin: Skin) -> Layout {
    match skin {
        Skin::AnalogGlass => Layout {
            power: [117., 315., 73., 32.],
            play: [332., 315., 164., 32.],
            mute: [621., 315., 69., 32.],
            settings: [802., 315., 83., 32.],
            knob: [963., 116., 178., 178.],
            glass: [72., 120., 841., 151.],
            faces: [[89., 120., 276., 150.], [619., 120., 278., 150.]],
            pivots: [[230., 270.], [758., 270.]],
            radius: 119.,
            vu_offset: 58.,
            display: [389., 147., 228., 96.],
            ivory: true,
        },
        _ => Layout {
            power: [74., 299., 58., 34.],
            play: [257., 298., 143., 39.],
            mute: [809., 299., 51., 34.],
            settings: [1044., 299., 80., 34.],
            knob: [534., 246., 132., 132.],
            glass: [456., 142., 288., 70.],
            faces: [[83., 99., 345., 139.], [772., 99., 347., 139.]],
            pivots: [[255.5, 320.], [946.5, 320.]],
            radius: 190.,
            vu_offset: 132.,
            display: [467., 145., 266., 65.],
            ivory: false,
        },
    }
}

struct Photo {
    tiles: [TextureHandle; 2],
    split: f32,
}
impl Photo {
    fn load(ctx: &Context, bytes: &[u8], name: &str) -> Self {
        let photo = image::load_from_memory(bytes)
            .expect("bundled analog materials")
            .to_rgba8();
        let (w, h) = photo.dimensions();
        let split = w / 2;
        // Keep the full Retina source while respecting egui's initial 2048 texture limit.
        let tile = |x, width, suffix| {
            let pixels = image::imageops::crop_imm(&photo, x, 0, width, h).to_image();
            ctx.load_texture(
                format!("{name}-{suffix}"),
                ColorImage::from_rgba_unmultiplied([width as usize, h as usize], pixels.as_raw()),
                TextureOptions::LINEAR,
            )
        };
        Self {
            tiles: [tile(0, split, "left"), tile(split, w - split, "right")],
            split: split as f32 / w as f32,
        }
    }
    fn paint(&self, p: &Painter, panel: Panel) {
        let x = 1200. * self.split;
        for (tile, rect) in [
            (&self.tiles[0], [0., 0., x, 440.]),
            (&self.tiles[1], [x, 0., 1200. - x, 440.]),
        ] {
            p.image(
                tile.id(),
                panel.rect(rect),
                Rect::from_min_max(Pos2::ZERO, pos2(1., 1.)),
                Color32::WHITE,
            );
        }
    }
}

pub struct AnalogSkin {
    silver: Photo,
    glass: Option<Photo>,
    movement: dsp_analog::Reading,
}
impl AnalogSkin {
    pub fn new(ctx: &Context) -> Self {
        Self {
            silver: Photo::load(
                ctx,
                include_bytes!("../assets/receiver-analog-silver-v2-substrate.png"),
                "analog-silver-v2",
            ),
            glass: None,
            movement: dsp_analog::Reading::default(),
        }
    }
    pub fn background(&mut self, p: &Painter, panel: Panel, skin: Skin) {
        if skin == Skin::AnalogGlass {
            self.glass
                .get_or_insert_with(|| {
                    Photo::load(
                        p.ctx(),
                        include_bytes!("../assets/receiver-analog-ivory-glass-substrate.png"),
                        "analog-ivory-glass",
                    )
                })
                .paint(p, panel);
        } else {
            self.silver.paint(p, panel);
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn instruments(
        &mut self,
        p: &Painter,
        panel: Panel,
        snap: &Snapshot,
        settings: &Settings,
        running: bool,
        power: bool,
        clock: &ClockFace,
        track_seconds: f32,
        dt: f32,
        muted: bool,
    ) {
        if running {
            // Mechanically integrated at the consumed audio sample clock. No second GUI envelope.
            self.movement = snap.meters.analog;
        } else {
            let step = dsp_analog::Transition::new(dt as f64);
            for m in &mut self.movement.channels {
                step.advance(m, 0.);
            }
        }
        let l = layout(settings.skin);
        let light = if power { settings.vfd_brightness } else { 0.16 };
        if l.ivory {
            p.rect_filled(
                panel.rect(l.glass),
                0.,
                Color32::from_black_alpha(((1. - light) * 145.) as u8),
            );
        }
        for c in 0..2 {
            self.dial(p, panel, c, settings, power, l);
        }
        let mut display = p.with_clip_rect(panel.rect(l.display));
        let ink = if l.ivory { INK } else { AMBER };
        if !l.ivory {
            display.multiply_opacity(crate::settings::display_brightness(light));
        }
        let [x, y, w, h] = l.display;
        let label = |px, py, text: &str, size, align| {
            display.text(
                panel.point(px, py),
                align,
                text,
                FontId::new(
                    size * panel.scale,
                    FontFamily::Name("analog-display".into()),
                ),
                ink,
            );
        };
        if !running && settings.standby_clock {
            label(
                x + w * 0.5,
                y + h * 0.43,
                &clock.time,
                if l.ivory { 35. } else { 28. },
                Align2::CENTER_CENTER,
            );
            label(
                x + w * 0.5,
                y + h * 0.86,
                &clock.date,
                10.,
                Align2::CENTER_CENTER,
            );
            label(x + w * 0.5, y, "STANDBY", 8.5, Align2::CENTER_TOP);
        } else if power {
            label(x + w * 0.5, y, &snap.status, 9., Align2::CENTER_TOP);
            let artist = if snap.artist.is_empty() {
                "RADIO CRASH"
            } else {
                &snap.artist
            };
            let title = if snap.title.is_empty() {
                "Waiting for song information"
            } else {
                &snap.title
            };
            marquee(
                &display,
                panel.rect([x, y + h * 0.23, w, h * 0.28]),
                artist,
                if l.ivory { 20. } else { 16. },
                panel.scale,
                track_seconds,
                ink,
                l.ivory,
            );
            marquee(
                &display,
                panel.rect([x, y + h * 0.53, w, h * 0.26]),
                title,
                if l.ivory { 18. } else { 14. },
                panel.scale,
                track_seconds,
                ink,
                l.ivory,
            );
            let bitrate = snap
                .bitrate
                .map(|b| format!("{b} kbps"))
                .unwrap_or_else(|| "? kbps".into());
            let rate = if snap.rate == 0 {
                "? kHz".into()
            } else {
                format!("{:.1} kHz", snap.rate as f32 / 1000.)
            };
            let channels = match snap.channels {
                0 => "?",
                1 => "MONO",
                2 => "STEREO",
                _ => "MULTI",
            };
            let codec = if snap.codec.is_empty() {
                "?"
            } else {
                &snap.codec
            };
            label(
                x + w * 0.5,
                y + h * 0.85,
                &format!("{codec}   {bitrate}   {rate}   {channels}"),
                if l.ivory { 10. } else { 9. },
                Align2::CENTER_TOP,
            );
        }
        if muted {
            label(x, y, "MUTE", 8., Align2::LEFT_TOP);
        }
    }
    fn dial(
        &self,
        p: &Painter,
        panel: Panel,
        c: usize,
        settings: &Settings,
        power: bool,
        l: Layout,
    ) {
        let [x, y] = l.pivots[c];
        let at = |offset: Vec2| panel.point(x + offset.x, y + offset.y);
        let scale_point = |t: f32, offset: f32| at(scale_offset(l, t, offset));
        let face = panel.rect(l.faces[c]);
        let p = p.with_clip_rect(face);
        if !l.ivory {
            let light = if power { settings.vfd_brightness } else { 0.16 };
            p.rect_filled(
                face,
                0.,
                Color32::from_black_alpha(((1. - light) * 175.) as u8),
            );
        }
        // Linear deflection per coil current, logarithmically spaced dB labels, fixed +3 VU.
        for db in -20..=3 {
            let t = 10f32.powf((db as f32 - 3.) / 20.);
            p.line_segment(
                [
                    scale_point(t, if l.ivory { 4. } else { 0. }),
                    scale_point(t, if l.ivory { 27. } else { 10. }),
                ],
                Stroke::new(0.9 * panel.scale, if db > 0 { RED } else { INK }),
            );
        }
        for (db, text) in [
            (-20., "−20"),
            (-10., "−10"),
            (-7., "−7"),
            (-5., "−5"),
            (-3., "−3"),
            (-1., "−1"),
            (0., "0"),
            (1., "+1"),
            (2., "+2"),
            (3., "+3"),
        ] {
            let t = 10f32.powf((db - 3.) / 20.);
            p.text(
                scale_point(t, if l.ivory { 45. } else { 24. }),
                Align2::CENTER_CENTER,
                text,
                FontId::proportional(if l.ivory { 10.5 } else { 12. } * panel.scale),
                if db > 0. { RED } else { INK },
            );
        }
        if l.ivory {
            // A straight printed baseline, with radial ticks converging on the shaft.
            // Scale labels lie on a second horizontal plane, rather than on an arc.
            let depth = flat_scale_depth(l);
            p.line_segment(
                [
                    at(vec2(-depth - 18., -depth)),
                    at(vec2(depth + 18., -depth)),
                ],
                Stroke::new(0.9 * panel.scale, INK),
            );
        } else {
            for (start, end, color, width) in [
                (0., 10f32.powf(-3. / 20.), INK, 0.8),
                (10f32.powf(-3. / 20.), 1., RED, 2.2),
            ] {
                let points: Vec<_> = (0..=60)
                    .map(|i| scale_point(start + (end - start) * i as f32 / 60., 0.))
                    .collect();
                p.add(Shape::line(points, Stroke::new(width * panel.scale, color)));
            }
        }
        p.text(
            at(vec2(0., -l.vu_offset)),
            Align2::CENTER_CENTER,
            "VU",
            FontId::proportional(22. * panel.scale),
            INK,
        );
        p.text(
            face.left_bottom() + vec2(7., -20.) * panel.scale,
            Align2::LEFT_CENTER,
            if c == 0 { "LEFT" } else { "RIGHT" },
            FontId::monospace(8. * panel.scale),
            INK,
        );
        let t = dsp_analog::position(
            self.movement.channels[c].position,
            settings.analog_reference_dbfs,
        );
        // Needle travel is circular even when the printed scale is straight.
        let tip = at(needle_offset(t, needle_length(l)));
        let hub = at(Vec2::ZERO);
        for (offset, width, alpha) in [(vec2(2.8, 1.8), 2.8, 20), (vec2(2., 1.4), 1.8, 35)] {
            p.line_segment(
                [hub + offset * panel.scale, tip + offset * panel.scale],
                Stroke::new(width * panel.scale, Color32::from_black_alpha(alpha)),
            );
        }
        p.line_segment(
            [hub, tip],
            Stroke::new(1.7 * panel.scale, Color32::from_rgb(168, 34, 19)),
        );
        p.line_segment(
            [
                hub - vec2(0.35, 0.) * panel.scale,
                tip - vec2(0.35, 0.) * panel.scale,
            ],
            Stroke::new(0.45 * panel.scale, Color32::from_rgb(245, 111, 65)),
        );
        skins::gradient(
            &p,
            Rect::from_min_size(face.min, vec2(face.width(), face.height() * 0.25)),
            Color32::from_white_alpha(10),
            Color32::TRANSPARENT,
        );
    }
}
fn flat_scale_depth(l: Layout) -> f32 {
    l.radius - 43.
}
fn needle_length(l: Layout) -> f32 {
    if l.ivory { 112. } else { l.radius + 11. }
}
/// Intersect the moving needle's ray with a horizontal scale plane.
/// The detector still maps coil current to angle; changing the card never changes measurement.
fn scale_offset(l: Layout, position: f32, offset: f32) -> Vec2 {
    if l.ivory {
        let ray = needle_offset(position, 1.);
        ray * ((flat_scale_depth(l) + offset) / -ray.y)
    } else {
        needle_offset(position, l.radius + offset)
    }
}
fn angle(position: f32) -> f32 {
    (-135. + 90. * position).to_radians()
}
fn needle_offset(position: f32, length: f32) -> Vec2 {
    let a = angle(position);
    vec2(a.cos(), a.sin()) * length
}
#[allow(clippy::too_many_arguments)]
fn marquee(
    p: &Painter,
    rect: Rect,
    value: &str,
    size: f32,
    scale: f32,
    elapsed: f32,
    ink: Color32,
    center: bool,
) {
    let clipped = p.with_clip_rect(rect);
    let font = FontId::new(size * scale, FontFamily::Name("analog-display".into()));
    let galley = p.layout_no_wrap(value.to_owned(), font, ink);
    let width = galley.size().x;
    if width <= rect.width() {
        let offset = if center {
            (rect.width() - width) * 0.5
        } else {
            0.
        };
        clipped.galley(rect.min + vec2(offset, 0.), galley, ink);
    } else {
        let gap = 35. * scale;
        let offset = ((elapsed - 3.).max(0.) * 20. * scale) % (width + gap);
        clipped.galley(rect.min - vec2(offset, 0.), galley.clone(), ink);
        clipped.galley(rect.min + vec2(width + gap - offset, 0.), galley, ink);
    }
}
#[cfg(test)]
mod geometry_tests {
    use super::*;
    #[test]
    fn real_needle_has_constant_length_and_ninety_degree_sweep() {
        for skin in [Skin::AnalogSilver, Skin::AnalogGlass] {
            let length = needle_length(layout(skin));
            for i in 0..=100 {
                assert!((needle_offset(i as f32 / 100., length).length() - length).abs() < 0.0001);
            }
            let sweep = needle_offset(1., length).angle() - needle_offset(0., length).angle();
            assert!((sweep.to_degrees() - 90.).abs() < 0.001);
        }
    }
    #[test]
    fn shared_glass_scale_is_straight_and_each_tick_matches_the_needle_ray() {
        let l = layout(Skin::AnalogGlass);
        let mut previous = f32::NEG_INFINITY;
        for i in 0..=100 {
            let t = i as f32 / 100.;
            let base = scale_offset(l, t, 0.);
            assert!((base.y + flat_scale_depth(l)).abs() < 0.0001);
            assert!(base.x > previous);
            previous = base.x;
            let ray = needle_offset(t, 1.);
            for plane in [4., 27., 45.] {
                let point = scale_offset(l, t, plane);
                assert!((point.y + flat_scale_depth(l) + plane).abs() < 0.0001);
                assert!((ray.x * point.y - ray.y * point.x).abs() < 0.0001);
            }
        }
    }
    #[test]
    fn silver_card_keeps_its_approved_circular_geometry() {
        let l = layout(Skin::AnalogSilver);
        for i in 0..=100 {
            let t = i as f32 / 100.;
            for offset in [0., 10., 24.] {
                assert_eq!(
                    scale_offset(l, t, offset),
                    needle_offset(t, l.radius + offset)
                );
            }
        }
    }
    #[test]
    fn analog_knobs_are_round_and_display_does_not_overlap_instruments() {
        for skin in [Skin::AnalogSilver, Skin::AnalogGlass] {
            let l = layout(skin);
            assert_eq!(l.knob[2], l.knob[3]);
            let panel = Panel::new(Rect::from_min_size(Pos2::ZERO, vec2(1200., 440.)));
            for face in l.faces {
                assert!(!panel.rect(l.display).intersects(panel.rect(face)));
                // Every scale label remains inside its glass aperture at either extreme.
            }
            for (face, pivot) in l.faces.into_iter().zip(l.pivots) {
                for i in 0..=100 {
                    let pt = pos2(pivot[0], pivot[1])
                        + scale_offset(l, i as f32 / 100., if l.ivory { 45. } else { 24. });
                    assert!(panel.rect(face).shrink(5.).contains(pt), "{skin:?}: {pt:?}");
                }
            }
        }
    }
}
