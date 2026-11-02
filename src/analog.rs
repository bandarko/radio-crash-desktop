//! Silver analog skin. Only materials are photographed; scales, needles,
//! metadata, control indicators and volume index are painted live by egui.
use crate::{
    audio::Snapshot, clock_face::ClockFace, dsp_analog, panel::Panel, settings::Settings, skins,
};
use eframe::egui::*;

pub const POWER: [f32; 4] = [70., 299., 60., 34.];
pub const PLAY: [f32; 4] = [255., 296., 143., 43.];
pub const MUTE: [f32; 4] = [808., 299., 53., 34.];
pub const SETTINGS: [f32; 4] = [1043., 299., 80., 34.];
pub const KNOB: [f32; 4] = [526., 243., 151., 128.];
pub const GLASS: [f32; 4] = [465., 143., 271., 66.];
const AMBER: Color32 = Color32::from_rgb(239, 178, 84);
const INK: Color32 = Color32::from_rgb(32, 26, 18);
const RED: Color32 = Color32::from_rgb(168, 42, 25);

pub struct AnalogSkin {
    texture: TextureHandle,
    movement: dsp_analog::Reading,
}
impl AnalogSkin {
    pub fn new(ctx: &Context) -> Self {
        let photo = image::load_from_memory(include_bytes!(
            "../assets/receiver-analog-silver-substrate.png"
        ))
        .expect("bundled silver analog fascia")
        .to_rgba8();
        Self {
            texture: ctx.load_texture(
                "analog-silver-materials",
                ColorImage::from_rgba_unmultiplied(
                    [photo.width() as usize, photo.height() as usize],
                    photo.as_raw(),
                ),
                TextureOptions::LINEAR,
            ),
            movement: dsp_analog::Reading::default(),
        }
    }
    pub fn background(&self, p: &Painter, panel: Panel) {
        p.image(
            self.texture.id(),
            panel.rect([0., 0., 1200., 440.]),
            Rect::from_min_max(Pos2::ZERO, pos2(1., 1.)),
            Color32::WHITE,
        );
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
            // Already rectified and mechanically integrated at the audio sample clock.
            // Never apply the digital RMS/Peak response or a second GUI envelope.
            self.movement = snap.meters.analog;
        } else {
            let step = dsp_analog::Transition::new(dt as f64);
            for m in &mut self.movement.channels {
                step.advance(m, 0.);
            }
        }
        for c in 0..2 {
            self.dial(p, panel, c, settings, power);
        }
        let mut display = p.with_clip_rect(panel.rect(GLASS));
        display.multiply_opacity(crate::settings::display_brightness(settings.vfd_brightness));
        let label = |at: Pos2, s: &str, size: f32, align| {
            display.text(
                at,
                align,
                s,
                FontId::proportional(size * panel.scale),
                AMBER,
            );
        };
        if !running && settings.standby_clock {
            label(
                panel.point(600., 164.),
                &clock.time,
                27.,
                Align2::CENTER_CENTER,
            );
            label(
                panel.point(600., 195.),
                &clock.date,
                10.,
                Align2::CENTER_CENTER,
            );
            label(panel.point(724., 147.), "STANDBY", 7.5, Align2::RIGHT_TOP);
        } else if power {
            label(
                panel.point(724., 147.),
                &snap.status,
                7.5,
                Align2::RIGHT_TOP,
            );
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
                panel.rect([490., 157., 235., 18.]),
                artist,
                15.,
                panel.scale,
                track_seconds,
            );
            marquee(
                &display,
                panel.rect([490., 177., 235., 17.]),
                title,
                13.,
                panel.scale,
                track_seconds,
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
                panel.point(490., 198.),
                &format!("{codec}   {bitrate}   {rate}   {channels}"),
                8.,
                Align2::LEFT_TOP,
            );
        }
        if muted {
            label(panel.point(476., 147.), "MUTE", 7.5, Align2::LEFT_TOP);
        }
    }
    fn dial(&self, p: &Painter, panel: Panel, c: usize, settings: &Settings, power: bool) {
        let pivot = vec2(if c == 0 { 255. } else { 948. }, 320.);
        let at = |offset: Vec2| panel.point(pivot.x + offset.x, pivot.y + offset.y);
        // Real circular travel about a shaft hidden below the lower bezel.
        // A rotating needle must have constant length, never elliptical stretching.
        let scale_point = |t: f32, radius: f32| at(needle_offset(t, radius));
        // Linear angular deflection per coil current; dB labels are logarithmically spaced.
        for db in -20..=3 {
            let t = 10f32.powf((db as f32 - 3.) / 20.);
            let color = if db > 0 { RED } else { INK };
            p.line_segment(
                [scale_point(t, 200.), scale_point(t, 210.)],
                Stroke::new(0.9 * panel.scale, color),
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
                scale_point(t, 224.),
                Align2::CENTER_CENTER,
                text,
                FontId::proportional(12. * panel.scale),
                if db > 0. { RED } else { INK },
            );
        }
        for (start, end, color, width) in [
            (0., 10f32.powf(-3. / 20.), INK, 0.8),
            (10f32.powf(-3. / 20.), 1., RED, 2.2),
        ] {
            let points: Vec<_> = (0..=60)
                .map(|i| scale_point(start + (end - start) * i as f32 / 60., 200.))
                .collect();
            p.add(Shape::line(points, Stroke::new(width * panel.scale, color)));
        }
        p.text(
            at(vec2(0., -133.)),
            Align2::CENTER_CENTER,
            "VU",
            FontId::proportional(22. * panel.scale),
            INK,
        );
        p.text(
            at(vec2(-145., -104.)),
            Align2::LEFT_CENTER,
            if c == 0 { "LEFT" } else { "RIGHT" },
            FontId::monospace(8. * panel.scale),
            INK,
        );
        let face = panel.rect([if c == 0 { 80. } else { 771. }, 96., 349., 144.]);
        // Illumination only. No gain or detector settings are changed by Night mode.
        let light = if power { settings.vfd_brightness } else { 0.16 };
        p.rect_filled(
            face,
            0.,
            Color32::from_black_alpha(((1. - light) * 175.) as u8),
        );
        let t = dsp_analog::position(
            self.movement.channels[c].position,
            settings.analog_reference_dbfs,
        );
        let tip = scale_point(t, 211.);
        let hub = at(Vec2::ZERO);
        let clipped = p.with_clip_rect(face);
        for (offset, width, alpha) in [(vec2(2.8, 1.8), 2.8, 20), (vec2(2., 1.4), 1.8, 35)] {
            clipped.line_segment(
                [hub + offset * panel.scale, tip + offset * panel.scale],
                Stroke::new(width * panel.scale, Color32::from_black_alpha(alpha)),
            );
        }
        clipped.line_segment(
            [hub, tip],
            Stroke::new(1.7 * panel.scale, Color32::from_rgb(168, 34, 19)),
        );
        clipped.line_segment(
            [
                hub - vec2(0.35, 0.) * panel.scale,
                tip - vec2(0.35, 0.) * panel.scale,
            ],
            Stroke::new(0.45 * panel.scale, Color32::from_rgb(245, 111, 65)),
        );
        skins::gradient(
            &clipped,
            Rect::from_min_size(face.min, vec2(face.width(), face.height() * 0.25)),
            Color32::from_white_alpha(10),
            Color32::TRANSPARENT,
        );
    }
}
fn angle(position: f32) -> f32 {
    (-135. + 90. * position).to_radians()
}
fn needle_offset(position: f32, length: f32) -> Vec2 {
    let a = angle(position);
    vec2(a.cos(), a.sin()) * length
}

fn marquee(p: &Painter, rect: Rect, value: &str, size: f32, scale: f32, elapsed: f32) {
    let clipped = p.with_clip_rect(rect);
    let font = FontId::new(size * scale, FontFamily::Name("analog-display".into()));
    let galley = p.layout_no_wrap(value.to_owned(), font, AMBER);
    let width = galley.size().x;
    if width <= rect.width() {
        clipped.galley(rect.min, galley, AMBER);
    } else {
        let gap = 35. * scale;
        let offset = ((elapsed - 3.).max(0.) * 20. * scale) % (width + gap);
        clipped.galley(rect.min - vec2(offset, 0.), galley.clone(), AMBER);
        clipped.galley(rect.min + vec2(width + gap - offset, 0.), galley, AMBER);
    }
}

#[cfg(test)]
mod geometry_tests {
    use super::*;
    #[test]
    fn real_needle_has_constant_length_and_ninety_degree_sweep() {
        let length = 211.;
        for i in 0..=100 {
            let p = needle_offset(i as f32 / 100., length);
            assert!((p.length() - length).abs() < 0.0001);
        }
        let start = needle_offset(0., length).angle();
        let end = needle_offset(1., length).angle();
        assert!(((end - start).to_degrees() - 90.).abs() < 0.001);
    }
}
