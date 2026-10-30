//! Small deterministic tray images. All levels come from the consumed-PCM meter snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    MenuBar,
    NotificationArea,
}
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}
impl Frame {
    fn rect(&mut self, x: u32, y: u32, w: u32, h: u32, c: [u8; 4]) {
        for yy in y..(y + h).min(self.height) {
            for xx in x..(x + w).min(self.width) {
                let i = ((yy * self.width + xx) * 4) as usize;
                self.rgba[i..i + 4].copy_from_slice(&c);
            }
        }
    }
    fn letter(&mut self, ch: usize, x: u32, y: u32, scale: u32) {
        let rows = if ch == 0 {
            [4, 4, 4, 4, 7]
        } else {
            [6, 5, 6, 5, 5]
        };
        for (row, bits) in rows.into_iter().enumerate() {
            for col in 0..3 {
                if bits & (1 << (2 - col)) != 0 {
                    self.rect(
                        x + col * scale,
                        y + row as u32 * scale,
                        scale,
                        scale,
                        [225, 244, 246, 255],
                    );
                }
            }
        }
    }
    pub fn render(levels: [f32; 2], peaks: [f32; 2], layout: Layout) -> Self {
        let (width, height) = match layout {
            Layout::MenuBar => (80, 40),
            Layout::NotificationArea => (32, 32),
        };
        let mut f = Self {
            width,
            height,
            rgba: vec![0; (width * height * 4) as usize],
        };
        for c in 0..2 {
            let position = crate::dsp::vu_position(crate::dsp::vu_relative(levels[c]));
            let peak = crate::dsp::vu_peak_index(peaks[c]).map(|i| i as f32 / 39.);
            match layout {
                Layout::MenuBar => {
                    let y = 3 + c as u32 * 20;
                    f.rect(0, y - 1, 79, 18, [7, 17, 19, 175]);
                    f.letter(c, 3, y, 3);
                    for i in 0..12 {
                        let on = position * 12. > i as f32 && levels[c] > -59.;
                        let color = color(on, i, 12);
                        f.rect(17 + i * 5, y + 4, 4, 8, color);
                    }
                    if let Some(v) = peak {
                        let i = (v * 11.).round() as u32;
                        f.rect(17 + i * 5, y + 1, 4, 2, [255, 237, 151, 255]);
                    }
                }
                Layout::NotificationArea => {
                    let x = 3 + c as u32 * 16;
                    f.rect(x - 1, 2, 12, 29, [7, 17, 19, 175]);
                    f.letter(c, x + 3, 26, 1);
                    for i in 0..10 {
                        let on = position * 10. > i as f32 && levels[c] > -59.;
                        f.rect(x + 1, 23 - i * 2, 8, 1, color(on, i, 10));
                    }
                    if let Some(v) = peak {
                        let i = (v * 9.).round() as u32;
                        f.rect(x + 1, 22 - i * 2, 8, 1, [255, 237, 151, 255]);
                    }
                }
            }
        }
        f
    }
}
fn color(on: bool, i: u32, n: u32) -> [u8; 4] {
    if !on {
        [33, 59, 62, 210]
    } else if i >= n - 1 {
        [255, 80, 45, 255]
    } else if i >= n * 3 / 4 {
        [255, 209, 72, 255]
    } else {
        [68, 231, 241, 255]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn left_and_right_are_independent() {
        for layout in [Layout::MenuBar, Layout::NotificationArea] {
            let silence = Frame::render([-60.; 2], [-60.; 2], layout);
            let (rms, peaks) = crate::dsp::levels(&[[0.5, 0.]; 256]);
            let signal = Frame::render(rms, peaks, layout);
            let mut changed = 0;
            for y in 0..signal.height {
                for x in 0..signal.width {
                    let i = ((y * signal.width + x) * 4) as usize;
                    if signal.rgba[i..i + 4] != silence.rgba[i..i + 4] {
                        changed += 1;
                        match layout {
                            Layout::MenuBar => assert!(y < 20),
                            Layout::NotificationArea => assert!(x < 16),
                        }
                    }
                }
            }
            assert!(changed > 0);
        }
    }
    #[test]
    fn finite_and_silent_frames() {
        for layout in [Layout::MenuBar, Layout::NotificationArea] {
            let f = Frame::render([-60.; 2], [-60.; 2], layout);
            assert_eq!(f.rgba.len(), (f.width * f.height * 4) as usize);
            let loud = Frame::render([0.; 2], [0.; 2], layout);
            assert_ne!(f.rgba, loud.rgba);
        }
    }
}
