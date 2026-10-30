use rustfft::{FftPlanner, num_complex::Complex};
use std::sync::Arc;
pub const BANDS: [f32; 10] = [
    32., 64., 125., 250., 500., 1000., 2000., 4000., 8000., 16000.,
];
pub const N: usize = 4096;
#[derive(Clone)]
pub struct Meters {
    pub rms: [f32; 2],
    pub peaks: [f32; 2],
    pub held_peaks: [f32; 2],
    pub spectrum: [f32; 10],
    pub available: [bool; 10],
}
impl Default for Meters {
    fn default() -> Self {
        Self {
            rms: [-60.; 2],
            peaks: [-60.; 2],
            held_peaks: [-60.; 2],
            spectrum: [-60.; 10],
            available: [false; 10],
        }
    }
}
/// Fast stereo level measurement, independent of the FFT window.
pub fn levels(frames: &[[f32; 2]]) -> ([f32; 2], [f32; 2]) {
    let mut rms = [-60.; 2];
    let mut peaks = [-60.; 2];
    if frames.is_empty() {
        return (rms, peaks);
    }
    for c in 0..2 {
        rms[c] = db((frames.iter().map(|f| f[c] * f[c]).sum::<f32>() / frames.len() as f32).sqrt());
        peaks[c] = db(frames.iter().map(|f| f[c].abs()).fold(0., f32::max));
    }
    (rms, peaks)
}
/// Immediate attack and elapsed-time decay, matching the live website's response model.
pub fn follow(current: f32, target: f32, seconds: f32, decay_db_per_second: f32) -> f32 {
    if target >= current {
        target
    } else {
        target.max(current - decay_db_per_second * seconds.clamp(0., 0.1))
    }
}
/// Radio Crash's relative level scale: 0 dB reference is -18 dBFS.
/// This is display calibration, never audio gain; +18 denotes digital full scale.
pub const VU_REFERENCE_DBFS: f32 = -18.;
pub const VU_SCALE: [f32; 40] = [
    -40., -35., -30., -25., -22., -20., -18., -16., -14., -12., -11., -10., -9., -8., -7., -6.,
    -5., -4., -3., -2., -1., 0., 1., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12., 13., 14., 15.,
    16., 17., 18.,
];
pub fn vu_relative(dbfs: f32) -> f32 {
    dbfs - VU_REFERENCE_DBFS
}
pub fn vu_position(relative: f32) -> f32 {
    if relative <= VU_SCALE[0] {
        return 0.;
    }
    for (i, pair) in VU_SCALE.windows(2).enumerate() {
        if relative <= pair[1] {
            return (i as f32 + (relative - pair[0]) / (pair[1] - pair[0]))
                / (VU_SCALE.len() - 1) as f32;
        }
    }
    1.
}
pub fn vu_peak_index(dbfs: f32) -> Option<usize> {
    let relative = vu_relative(dbfs);
    if !relative.is_finite() || relative < VU_SCALE[0] {
        return None;
    }
    Some((vu_position(relative) * (VU_SCALE.len() - 1) as f32).round() as usize)
}
/// Sample-clock peak hold runs before the GUI, preserving even sub-frame transients.
pub struct PeakHold {
    value: [f32; 2],
    until: [f64; 2],
    last: f64,
}
impl PeakHold {
    pub fn new() -> Self {
        Self {
            value: [-60.; 2],
            until: [0.; 2],
            last: 0.,
        }
    }
    pub fn observe(&mut self, peaks: [f32; 2], seconds: f64) -> [f32; 2] {
        for (c, peak) in peaks.iter().enumerate() {
            if *peak >= self.value[c] {
                self.value[c] = *peak;
                self.until[c] = seconds + 1.5;
            } else {
                let elapsed = (seconds - self.last.max(self.until[c])).max(0.) as f32;
                self.value[c] = (self.value[c] - elapsed * 12.).max(*peak).max(-60.);
            }
        }
        self.last = seconds;
        self.value
    }
}
pub struct Analyzer {
    fft: Arc<dyn rustfft::Fft<f32>>,
    buf: Vec<Complex<f32>>,
    scratch: Vec<Complex<f32>>,
    window: Vec<f32>,
}
fn db(v: f32) -> f32 {
    (20. * v.max(0.000001).log10()).max(-60.)
}
impl Analyzer {
    pub fn new() -> Self {
        let fft = FftPlanner::new().plan_fft_forward(N);
        let scratch = vec![Complex::default(); fft.get_inplace_scratch_len()];
        Self {
            fft,
            buf: vec![Complex::default(); N],
            scratch,
            window: (0..N)
                .map(|i| 0.5 - 0.5 * (std::f32::consts::TAU * i as f32 / (N - 1) as f32).cos())
                .collect(),
        }
    }
    pub fn analyze(&mut self, frames: &[[f32; 2]], rate: u32) -> Meters {
        let mut m = Meters::default();
        let len = frames.len().min(N);
        if len == 0 {
            return m;
        }
        for c in 0..2 {
            m.rms[c] =
                db((frames[..len].iter().map(|f| f[c] * f[c]).sum::<f32>() / len as f32).sqrt());
            m.peaks[c] = db(frames[..len].iter().map(|f| f[c].abs()).fold(0., f32::max));
        }
        let mut powers = [0f32; N / 2];
        for channel in [0usize, 1usize] {
            for (i, value) in self.buf.iter_mut().enumerate() {
                *value = Complex::new(
                    if i < len {
                        frames[i][channel] * self.window[i]
                    } else {
                        0.
                    },
                    0.,
                );
            }
            self.fft
                .process_with_scratch(&mut self.buf, &mut self.scratch);
            for (power, bin) in powers.iter_mut().zip(&self.buf) {
                *power += bin.norm_sqr() * 0.5;
            }
        }
        for (b, f) in BANDS.iter().enumerate() {
            m.available[b] = *f < rate as f32 / 2.;
            if !m.available[b] {
                continue;
            }
            let lo = ((*f / 2f32.sqrt() * N as f32 / rate as f32).ceil() as usize).max(1);
            let hi = ((*f * 2f32.sqrt() * N as f32 / rate as f32).floor() as usize)
                .max(lo)
                .min(N / 2 - 1);
            let power = powers[lo..=hi].iter().sum::<f32>();
            m.spectrum[b] = db((power / 1.5).sqrt() * 4. / N as f32);
        }
        m
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn silence() {
        let m = Analyzer::new().analyze(&vec![[0.; 2]; N], 48000);
        assert_eq!(m.rms, [-60.; 2]);
        assert_eq!(m.spectrum, [-60.; 10]);
    }
    #[test]
    fn stereo_and_frequency() {
        let f: Vec<_> = (0..N)
            .map(|i| {
                [
                    (std::f32::consts::TAU * 1000. * i as f32 / 48000.).sin() * 0.5,
                    0.,
                ]
            })
            .collect();
        let m = Analyzer::new().analyze(&f, 48000);
        assert!((m.rms[0] + 9.03).abs() < 0.1);
        assert_eq!(m.rms[1], -60.);
        assert_eq!(
            m.spectrum
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.total_cmp(b.1))
                .unwrap()
                .0,
            5
        );
    }
    #[test]
    fn nyquist() {
        let m = Analyzer::new().analyze(&vec![[0.; 2]; N], 16000);
        assert!(!m.available[9]);
        assert!(!m.available[8]);
        assert!(m.available[7]);
    }
    #[test]
    fn antiphase_stereo_keeps_spectrum() {
        let frames: Vec<_> = (0..N)
            .map(|i| {
                let x = (std::f32::consts::TAU * 1000. * i as f32 / 48000.).sin() * 0.5;
                [x, -x]
            })
            .collect();
        let m = Analyzer::new().analyze(&frames, 48000);
        assert!(m.spectrum[5] > -10.);
        assert!((m.rms[0] - m.rms[1]).abs() < 0.001);
    }
    #[test]
    fn fast_vu_has_no_fft_window_latency() {
        let (rms, peaks) = levels(&[[0.5, 0.]; 256]);
        assert!((rms[0] + 6.0206).abs() < 0.001);
        assert_eq!(rms[1], -60.);
        assert_eq!(rms, peaks);
        assert_eq!(follow(-60., -6., 1. / 60., 120.), -6.);
        assert!((follow(-6., -60., 1. / 60., 120.) + 8.).abs() < 0.001);
        let mut at30 = -6.;
        for _ in 0..6 {
            at30 = follow(at30, -60., 1. / 30., 120.);
        }
        let mut at120 = -6.;
        for _ in 0..24 {
            at120 = follow(at120, -60., 1. / 120., 120.);
        }
        assert!((at30 - at120).abs() < 0.001);
    }
    #[test]
    fn relative_vu_reference_and_endpoints() {
        assert_eq!(vu_relative(-18.), 0.);
        assert_eq!(vu_relative(0.), 18.);
        assert_eq!(vu_position(-40.), 0.);
        assert_eq!(vu_position(18.), 1.);
        assert_eq!(vu_peak_index(0.), Some(39));
        assert_eq!(vu_peak_index(-60.), None);
        assert_eq!(vu_peak_index(-18.), Some(21));
        for (i, v) in VU_SCALE.iter().enumerate() {
            assert_eq!(vu_peak_index(*v - 18.), Some(i));
        }
        for pair in VU_SCALE.windows(2) {
            assert!(vu_position(pair[0]) < vu_position(pair[1]));
        }
    }
    #[test]
    fn dsp_peak_survives_short_transient_and_decays() {
        let mut h = PeakHold::new();
        assert_eq!(h.observe([-2., -20.], 0.01), [-2., -20.]);
        assert_eq!(h.observe([-30., -40.], 1.), [-2., -20.]);
        let v = h.observe([-30., -40.], 2.01);
        assert!((v[0] + 8.).abs() < 0.001);
        assert!((v[1] + 26.).abs() < 0.001);
        assert_eq!(h.observe([-60., -60.], 9.), [-60.; 2]);
    }
    #[test]
    fn equal_peaks_refresh_hold() {
        let mut h = PeakHold::new();
        h.observe([-3., -9.], 0.);
        h.observe([-3., -9.], 1.4);
        assert_eq!(h.observe([-40.; 2], 2.8), [-3., -9.]);
    }
}
