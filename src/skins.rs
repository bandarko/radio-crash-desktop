//! Original photographic Radio Crash fascia, with native live VFD painting.
use eframe::egui::*;
pub const CYAN: Color32 = Color32::from_rgb(71, 237, 249);
pub const DIM: Color32 = Color32::from_rgb(20, 63, 66);
pub const SILVER: Color32 = Color32::from_rgb(190, 189, 181);
pub struct Materials {
    left: TextureHandle,
    right: TextureHandle,
    logo: TextureHandle,
}
impl Materials {
    pub fn new(ctx: &Context) -> Self {
        let photo = image::load_from_memory(include_bytes!(
            "../assets/receiver-photographic-v4-brand.png"
        ))
        .expect("bundled receiver fascia")
        .to_rgba8();
        let w = photo.width() as usize;
        let h = photo.height() as usize;
        let split = w / 2;
        let tile = |start: usize, end: usize| {
            let mut pixels = Vec::with_capacity((end - start) * h);
            for y in 0..h {
                for x in start..end {
                    let p = photo.get_pixel(x as u32, y as u32);
                    pixels.push(Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]));
                }
            }
            ColorImage::new([end - start, h], pixels)
        };
        // Preserve the original Retina source without exceeding egui's initialization texture cap.
        let left = ctx.load_texture(
            "photographic-fascia-left",
            tile(0, split),
            TextureOptions::LINEAR,
        );
        let right = ctx.load_texture(
            "photographic-fascia-right",
            tile(split, w),
            TextureOptions::LINEAR,
        );
        // Exact station artwork, not a font substitute or AI-redrawn wordmark.
        // White ink uses source luminance as coverage; black source pixels are transparent.
        let logo_image =
            image::load_from_memory(include_bytes!("../assets/radio-crash-logo-user.png"))
                .expect("official station logo")
                .to_rgb8();
        let pixels = logo_image
            .pixels()
            .enumerate()
            .map(|(index, pixel)| {
                let x = index % logo_image.width() as usize;
                let y = index / logo_image.width() as usize;
                let antenna = (380..512).contains(&x) && y < 112;
                let coverage = (0.2126 * pixel[0] as f32
                    + 0.7152 * pixel[1] as f32
                    + 0.0722 * pixel[2] as f32)
                    .round() as u8;
                Color32::from_rgba_unmultiplied(
                    255,
                    255,
                    255,
                    if antenna || coverage < 8 { 0 } else { coverage },
                )
            })
            .collect();
        let logo = ctx.load_texture(
            "official-radio-crash-white-ink",
            ColorImage::new(
                [logo_image.width() as usize, logo_image.height() as usize],
                pixels,
            ),
            TextureOptions::LINEAR,
        );
        Self { left, right, logo }
    }
    pub fn logo(&self, p: &Painter, r: Rect) {
        let uv = Rect::from_min_max(pos2(25. / 960., 97. / 264.), pos2(925. / 960., 164. / 264.));
        p.image(
            self.logo.id(),
            r.translate(vec2(0., 0.4)),
            uv,
            Color32::from_black_alpha(75),
        );
        p.image(self.logo.id(), r, uv, Color32::WHITE);
    }
    pub fn paint(&self, p: &Painter, r: Rect) {
        let uv = Rect::from_min_max(Pos2::ZERO, pos2(1., 1.));
        let split =
            self.left.size()[0] as f32 / (self.left.size()[0] + self.right.size()[0]) as f32;
        let x = r.left() + r.width() * split;
        p.image(
            self.left.id(),
            Rect::from_min_max(r.min, pos2(x, r.bottom())),
            uv,
            Color32::WHITE,
        );
        p.image(
            self.right.id(),
            Rect::from_min_max(pos2(x, r.top()), r.max),
            uv,
            Color32::WHITE,
        );
    }
}
pub fn text(p: &Painter, at: Pos2, s: &str, size: f32, c: Color32) {
    let f = FontId::monospace(size);
    for d in [vec2(-0.45, 0.), vec2(0.45, 0.)] {
        p.text(
            at + d,
            Align2::LEFT_TOP,
            s,
            f.clone(),
            Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 25),
        );
    }
    p.text(at, Align2::LEFT_TOP, s, f, c);
}
pub fn glow(p: &Painter, at: Pos2, s: &str, size: f32) {
    let font = FontId::new(size, FontFamily::Name("vfd".into()));
    let scale = (size / 23.).max(0.4);
    for (radius, alpha) in [(2.4, 5), (1.3, 10), (0.55, 20)] {
        for (x, y) in [
            (-1., 0.),
            (1., 0.),
            (0., -1.),
            (0., 1.),
            (-0.7, -0.7),
            (0.7, 0.7),
        ] {
            p.text(
                at + vec2(x, y) * radius * scale,
                Align2::LEFT_TOP,
                s,
                font.clone(),
                Color32::from_rgba_unmultiplied(0, 222, 247, alpha),
            );
        }
    }
    p.text(
        at,
        Align2::LEFT_TOP,
        s,
        font,
        Color32::from_rgb(107, 248, 255),
    );
}
pub fn segment(p: &Painter, r: Rect, on: bool, index: usize, total: usize) {
    let c = if index >= total * 9 / 10 {
        Color32::from_rgb(255, 64, 27)
    } else if index >= total * 7 / 10 {
        Color32::from_rgb(255, 205, 52)
    } else {
        CYAN
    };
    segment_with_color(p, r, on, c);
}
pub fn vu_color(db: f32) -> Color32 {
    if db >= 8. {
        Color32::from_rgb(255, 61, 27)
    } else if db >= 4. {
        Color32::from_rgb(255, 157, 29)
    } else if db >= 0. {
        Color32::from_rgb(255, 219, 65)
    } else {
        CYAN
    }
}
pub fn segment_with_color(p: &Painter, r: Rect, on: bool, c: Color32) {
    if on {
        for (spread, alpha) in [(3.2, 3), (2., 6), (1., 14), (0.4, 36)] {
            p.rect_filled(
                r.expand(spread),
                0.7,
                Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), alpha),
            );
        }
    }
    p.rect_filled(r, 0.15, if on { c } else { Color32::from_rgb(2, 12, 13) });
    if on {
        let core = Rect::from_min_max(
            pos2(r.left() + 0.25, r.top() + r.height() * 0.35),
            pos2(r.right() - 0.25, r.top() + r.height() * 0.65),
        );
        p.rect_filled(core, 0., Color32::from_rgba_unmultiplied(220, 255, 250, 85));
    }
}
pub fn gradient(p: &Painter, r: Rect, top: Color32, bottom: Color32) {
    let mut m = Mesh::default();
    for (at, c) in [
        (r.left_top(), top),
        (r.right_top(), top),
        (r.right_bottom(), bottom),
        (r.left_bottom(), bottom),
    ] {
        m.colored_vertex(at, c);
    }
    m.add_triangle(0, 1, 2);
    m.add_triangle(0, 2, 3);
    p.add(Shape::mesh(m));
}

/// A steady cold-white phosphor emblem; all lettering uses one equipment-style face.
pub fn hd_radio(p: &Painter, r: Rect, powered: bool, qualified: bool) {
    let s = r.width() / 164.;
    let intensity = if qualified {
        1.
    } else if powered {
        0.65
    } else {
        0.
    };
    let font = FontId::new(24. * s, FontFamily::Name("badge".into()));
    let label = "HD RADIO";
    let size = p
        .layout_no_wrap(label.into(), font.clone(), Color32::WHITE)
        .size();
    let at = r.center() - size * 0.5;
    if intensity > 0. {
        for (radius, alpha) in [(3.5, 3), (1.8, 7), (0.65, 17)] {
            for (x, y) in [(-1., 0.), (1., 0.), (0., -1.), (0., 1.)] {
                p.text(
                    at + vec2(x, y) * radius * s,
                    Align2::LEFT_TOP,
                    label,
                    font.clone(),
                    Color32::from_rgba_unmultiplied(44, 216, 241, (alpha as f32 * intensity) as u8),
                );
            }
        }
    }
    let core = if qualified {
        Color32::from_rgb(222, 252, 255)
    } else if powered {
        Color32::from_rgb(170, 227, 237)
    } else {
        Color32::from_rgb(48, 62, 65)
    };
    p.text(at, Align2::LEFT_TOP, label, font, core);
    gradient(
        p,
        r.shrink(3. * s),
        Color32::from_white_alpha(6),
        Color32::TRANSPARENT,
    );
}
