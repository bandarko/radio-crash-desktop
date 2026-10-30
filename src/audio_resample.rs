//! Streaming 64-tap Blackman-windowed sinc converter. Never runs on the audio callback.
use std::collections::VecDeque;
const TAPS: usize = 64;
const PHASES: usize = 1024;
pub struct Resampler {
    input: u32,
    output: u32,
    frames: VecDeque<[f32; 2]>,
    position: f64,
    table: Vec<[f32; TAPS]>,
}
impl Resampler {
    pub fn new(input: u32, output: u32) -> Self {
        let cutoff = (output as f64 / input as f64).min(1.) * 0.94;
        let table = (0..PHASES)
            .map(|phase| {
                let frac = phase as f64 / PHASES as f64;
                let mut taps = [0.; TAPS];
                let mut sum = 0.;
                for (i, v) in taps.iter_mut().enumerate() {
                    let x = i as f64 - 31. - frac;
                    let sinc = if x.abs() < 1e-9 {
                        cutoff
                    } else {
                        (std::f64::consts::PI * x * cutoff).sin() / (std::f64::consts::PI * x)
                    };
                    let w = 0.42 - 0.5 * (std::f64::consts::TAU * i as f64 / 63.).cos()
                        + 0.08 * (2. * std::f64::consts::TAU * i as f64 / 63.).cos();
                    *v = (sinc * w) as f32;
                    sum += *v;
                }
                for v in &mut taps {
                    *v /= sum;
                }
                taps
            })
            .collect();
        Self {
            input,
            output,
            frames: std::iter::repeat_n([0.; 2], if input == output { 0 } else { 32 }).collect(),
            position: 31.,
            table,
        }
    }
    pub fn push(&mut self, f: [f32; 2]) {
        self.frames.push_back(f);
    }
    pub fn pop(&mut self) -> Option<[f32; 2]> {
        if self.input == self.output {
            return self.frames.pop_front();
        }
        let at = self.position.floor() as usize;
        if at + 32 >= self.frames.len() {
            return None;
        }
        let taps = &self.table[((self.position.fract() * PHASES as f64) as usize).min(PHASES - 1)];
        let mut out = [0.; 2];
        for (i, t) in taps.iter().enumerate() {
            let f = self.frames[at - 31 + i];
            out[0] += f[0] * t;
            out[1] += f[1] * t;
        }
        self.position += self.input as f64 / self.output as f64;
        let discard = (self.position.floor() as usize).saturating_sub(31);
        for _ in 0..discard {
            self.frames.pop_front();
        }
        self.position -= discard as f64;
        Some(out)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rate_and_dc() {
        let mut r = Resampler::new(44100, 48000);
        let mut out = vec![];
        for _ in 0..44100 {
            r.push([0.5, -0.5]);
            while let Some(v) = r.pop() {
                out.push(v);
            }
        }
        assert!((out.len() as i32 - 48000).abs() < 40);
        assert!((out[1000][0] - 0.5).abs() < 0.0001);
        assert!((out[1000][1] + 0.5).abs() < 0.0001);
    }
    #[test]
    fn rejects_aliases() {
        let mut r = Resampler::new(48000, 16000);
        let mut energy = 0.;
        let mut n = 0;
        for i in 0..48000 {
            let x = (std::f32::consts::TAU * 12000. * i as f32 / 48000.).sin();
            r.push([x, x]);
            while let Some(v) = r.pop() {
                if n > 100 {
                    energy += v[0] * v[0];
                }
                n += 1;
            }
        }
        assert!(energy / (n as f32) < 0.0001);
    }
}
