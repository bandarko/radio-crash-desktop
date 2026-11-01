use rustfft::{FftPlanner, num_complex::Complex};
use std::sync::Arc;
pub const BANDS: [f32; 10] = [
    32., 64., 125., 250., 500., 1000., 2000., 4000., 8000., 16000.,
];
pub const N: usize = 4096;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VuResponse {
    SmoothRms,
    Peak,
    #[default]
    #[serde(other)]
    FastRms,
}
impl VuResponse {
    pub fn label(self) -> &'static str {
        match self {
            Self::SmoothRms => "Hi-Fi RMS (smooth)",
            Self::FastRms => "Fast RMS (recommended)",
            Self::Peak => "Peak (fast transients)",
        }
    }
}
#[derive(Clone, Copy)]
pub struct VuReading {
    pub level: [f32; 2],
    pub held: [f32; 2],
}
impl Default for VuReading {
    fn default() -> Self {
        Self {
            level: [-60.; 2],
            held: [-60.; 2],
        }
    }
}
#[derive(Clone)]
pub struct Meters {
    pub rms: [f32; 2],
    pub peaks: [f32; 2],
    pub held_peaks: [f32; 2],
    pub held_rms: [f32; 2],
    pub fast_rms: VuReading,
    pub peak_meter: VuReading,
    pub true_peaks: Option<[f32; 2]>,
    pub momentary_lufs: Option<f64>,
    pub short_term_lufs: Option<f64>,
    pub spectrum: [f32; 10],
    pub available: [bool; 10],
}
impl Default for Meters {
    fn default() -> Self {
        Self {
            rms: [-60.; 2],
            peaks: [-60.; 2],
            held_peaks: [-60.; 2],
            held_rms: [-60.; 2],
            fast_rms: VuReading::default(),
            peak_meter: VuReading::default(),
            true_peaks: None,
            momentary_lufs: None,
            short_term_lufs: None,
            spectrum: [-60.; 10],
            available: [false; 10],
        }
    }
}
impl Meters {
    pub fn vu(&self, response: VuResponse) -> VuReading {
        match response {
            VuResponse::SmoothRms => VuReading {
                level: self.rms,
                held: self.held_rms,
            },
            VuResponse::FastRms => self.fast_rms,
            VuResponse::Peak => self.peak_meter,
        }
    }
}
/// Fast stereo level measurement, independent of the FFT window.
#[cfg(test)]
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
/// Radio Crash Hi-Fi display calibration; never applied to audio gain.
/// A programme RMS of -11 dBFS reads +4 VU, and -7 dBFS reads +8 VU.
pub const VU_REFERENCE_DBFS: f32 = -15.;
pub const VU_SCALE: [f32; 40] = [
    -40., -35., -30., -25., -22., -20., -18., -16., -14., -12., -11., -10., -9., -8., -7., -6.,
    -5., -4., -3., -2., -1., 0., 1., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12., 13., 14., 15.,
    16., 17., 18.,
];
pub fn vu_calibrated(dbfs: f32, reference: f32) -> f32 {
    dbfs - reference
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
/// The bar and its held segment use the same lower threshold, never rounding
/// a held value into a segment the instantaneous meter has not actually reached.
pub fn vu_segment_index(dbfs: f32, reference: f32) -> Option<usize> {
    let relative = vu_calibrated(dbfs, reference);
    if !relative.is_finite() || dbfs <= -60. || relative < VU_SCALE[0] {
        return None;
    }
    VU_SCALE
        .partition_point(|threshold| relative >= *threshold)
        .checked_sub(1)
}

pub struct LevelBlock {
    pub rms: [f32; 2],
    pub max_rms: [f32; 2],
    pub sample_peaks: [f32; 2],
}

/// Continuous 50 ms RMS energy integration, independent of packet/block size.
/// Retain the maximum of this SAME envelope within each block for VU hold.
/// Raw sample peaks remain separate diagnostics, not the RMS meter's marker.
pub struct RmsMeter {
    energy: [f64; 2],
    alpha: f64,
}
impl RmsMeter {
    pub fn new(rate: u32) -> Self {
        Self::with_time_constant(rate, 0.05)
    }
    pub fn with_time_constant(rate: u32, seconds: f64) -> Self {
        Self {
            energy: [0.; 2],
            alpha: 1. - (-1. / (rate.max(1) as f64 * seconds)).exp(),
        }
    }
    pub fn observe(&mut self, frames: &[[f32; 2]]) -> LevelBlock {
        let mut maximum = self.energy;
        let mut sample_peaks = [0f32; 2];
        for f in frames {
            for (c, sample) in f.iter().enumerate() {
                let power = (*sample as f64).powi(2);
                self.energy[c] += self.alpha * (power - self.energy[c]);
                maximum[c] = maximum[c].max(self.energy[c]);
                sample_peaks[c] = sample_peaks[c].max(sample.abs());
            }
        }
        LevelBlock {
            rms: self.energy.map(|v| db(v.max(0.).sqrt() as f32)),
            max_rms: maximum.map(|v| db(v.max(0.).sqrt() as f32)),
            sample_peaks: sample_peaks.map(db),
        }
    }
}

/// Sample-peak envelope: immediate attack and 20 dB / 320 ms release.
/// Sample-clock processing preserves impulses and is independent of chunks.
/// This is a peak-reading option, not a True Peak or hardware-emulation claim.
pub struct PeakMeter {
    amplitude: [f64; 2],
    release: f64,
}
impl PeakMeter {
    pub fn new(rate: u32) -> Self {
        Self {
            amplitude: [0.; 2],
            release: 10f64.powf(-62.5 / (20. * rate.max(1) as f64)),
        }
    }
    pub fn observe(&mut self, frames: &[[f32; 2]]) -> VuReading {
        let mut maximum = self.amplitude;
        for frame in frames {
            for c in 0..2 {
                self.amplitude[c] = (frame[c] as f64)
                    .abs()
                    .max(self.amplitude[c] * self.release);
                maximum[c] = maximum[c].max(self.amplitude[c]);
            }
        }
        VuReading {
            level: self.amplitude.map(|v| db(v as f32)),
            held: maximum.map(|v| db(v as f32)),
        }
    }
}

pub const PEAK_HOLD_SECONDS: f64 = 2.;
/// Audio-clock VU peak memory. New/equal maxima refresh a two-second hold;
/// expiration resets to the current meter level rather than inventing a decay.
pub struct PeakHold {
    value: [f32; 2],
    until: [f64; 2],
}

/// Consumed-PCM broadcast measurements. All filtering runs on the analysis
/// worker, never the audio callback or GUI. S mode keeps only a bounded 3 s
/// ring; no unbounded integrated loudness history or normalization is enabled.
pub struct BroadcastMeter {
    ebu: ebur128::EbuR128,
    rate: u32,
    frames: u64,
    next_loudness: u64,
    momentary: Option<f64>,
    short_term: Option<f64>,
    hold: PeakHold,
}
pub struct BroadcastLevels {
    pub true_peaks: [f32; 2],
    pub held_peaks: [f32; 2],
    pub momentary_lufs: Option<f64>,
    pub short_term_lufs: Option<f64>,
}
impl BroadcastMeter {
    pub fn new(rate: u32) -> Result<Self, ebur128::Error> {
        Ok(Self {
            ebu: ebur128::EbuR128::new(2, rate, ebur128::Mode::S | ebur128::Mode::TRUE_PEAK)?,
            rate,
            frames: 0,
            next_loudness: 0,
            momentary: None,
            short_term: None,
            hold: PeakHold::new(),
        })
    }
    pub fn observe(&mut self, frames: &[[f32; 2]]) -> Result<BroadcastLevels, ebur128::Error> {
        self.ebu.add_frames_f32(frames.as_flattened())?;
        self.frames += frames.len() as u64;
        let true_peaks = [
            db(self.ebu.prev_true_peak(0)? as f32),
            db(self.ebu.prev_true_peak(1)? as f32),
        ];
        let held_peaks = self
            .hold
            .observe(true_peaks, self.frames as f64 / self.rate as f64);
        // The library zero-pads at startup. Publish only complete time windows,
        // then sample LUFS at 10 Hz. Silence remains unavailable, not fake zero.
        if self.frames >= self.next_loudness {
            self.next_loudness = self.frames + (self.rate / 10) as u64;
            if self.frames >= self.rate as u64 * 2 / 5 {
                let value = self.ebu.loudness_momentary()?;
                self.momentary = (value.is_finite() && value >= -100.).then_some(value);
            }
            if self.frames >= self.rate as u64 * 3 {
                let value = self.ebu.loudness_shortterm()?;
                self.short_term = (value.is_finite() && value >= -100.).then_some(value);
            }
        }
        Ok(BroadcastLevels {
            true_peaks,
            held_peaks,
            momentary_lufs: self.momentary,
            short_term_lufs: self.short_term,
        })
    }
}
impl PeakHold {
    pub fn new() -> Self {
        Self {
            value: [-60.; 2],
            until: [0.; 2],
        }
    }
    pub fn observe(&mut self, maxima: [f32; 2], seconds: f64) -> [f32; 2] {
        for (c, peak) in maxima.iter().enumerate() {
            if seconds >= self.until[c] || *peak >= self.value[c] {
                self.value[c] = *peak;
                self.until[c] = seconds + PEAK_HOLD_SECONDS;
            }
        }
        self.value
    }
}
pub struct Analyzer {
    fft: Arc<dyn rustfft::Fft<f32>>,
    buf: Vec<Complex<f32>>,
    scratch: Vec<Complex<f32>>,
    window: Vec<f32>,
    window_energy: f32,
}
fn db(v: f32) -> f32 {
    (20. * v.max(0.000001).log10()).max(-60.)
}
impl Analyzer {
    pub fn new() -> Self {
        let fft = FftPlanner::new().plan_fft_forward(N);
        let scratch = vec![Complex::default(); fft.get_inplace_scratch_len()];
        let window: Vec<f32> = (0..N)
            .map(|i| 0.5 - 0.5 * (std::f32::consts::TAU * i as f32 / (N - 1) as f32).cos())
            .collect();
        let window_energy = window.iter().map(|v| v * v).sum();
        Self {
            fft,
            buf: vec![Complex::default(); N],
            scratch,
            window,
            window_energy,
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
            // One-sided Parseval power, compensated for actual Hann energy.
            // Band units are stereo-average RMS dBFS, not peak amplitude.
            m.spectrum[b] = db((2. * power / (N as f32 * self.window_energy)).sqrt());
        }
        m
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fast_rms_changes_response_without_changing_calibration() {
        let mut slow = RmsMeter::new(48000);
        let mut fast = RmsMeter::with_time_constant(48000, 0.015);
        let step = [[0.5, 0.]; 720];
        let short_slow = slow.observe(&step);
        let short_fast = fast.observe(&step);
        assert!((short_fast.rms[0] + 8.012).abs() < 0.02);
        assert!(short_fast.rms[0] > short_slow.rms[0] + 3.);
        let steady = [[0.5, 0.]; 24000];
        let settled_slow = slow.observe(&steady).rms;
        let settled_fast = fast.observe(&steady).rms;
        assert!((settled_slow[0] - settled_fast[0]).abs() < 0.001);
        assert_eq!(settled_fast[1], -60.);
        let falling_fast = fast.observe(&[[0.; 2]; 720]).rms[0];
        let falling_slow = slow.observe(&[[0.; 2]; 720]).rms[0];
        assert!(falling_slow > falling_fast + 3.);
    }
    #[test]
    fn peak_profile_preserves_impulses_and_has_sample_clock_release() {
        for rate in [32000, 44100, 48000, 96000, 192000] {
            let mut peak = PeakMeter::new(rate);
            let hit = peak.observe(&[[0.5, 0.]]);
            assert!((hit.level[0] + 6.0206).abs() < 0.001);
            assert_eq!(hit.held, hit.level);
            let decayed = peak.observe(&vec![[0.; 2]; (rate as f64 * 0.32) as usize]);
            assert!((decayed.level[0] + 26.0206).abs() < 0.01, "rate {rate}");
            assert_eq!(decayed.level[1], -60.);
        }
        let input = tone(48000, 1000., 0.5, 4800);
        let mut whole = PeakMeter::new(48000);
        let expected = whole.observe(&input);
        let mut split = PeakMeter::new(48000);
        let mut maximum = [-60f32; 2];
        for part in input.chunks(17) {
            let part = split.observe(part);
            for (max, value) in maximum.iter_mut().zip(part.held) {
                *max = max.max(value);
            }
        }
        assert_eq!(expected.level, split.observe(&[]).level);
        assert_eq!(expected.held, maximum);
    }
    #[test]
    fn each_profile_selects_its_own_matching_bar_and_hold() {
        let meters = Meters {
            rms: [-20., -21.],
            held_rms: [-15., -16.],
            fast_rms: VuReading {
                level: [-18., -19.],
                held: [-12., -13.],
            },
            peak_meter: VuReading {
                level: [-5., -6.],
                held: [-1., -2.],
            },
            held_peaks: [3., 4.],
            ..Default::default()
        };
        for (profile, level, held) in [
            (VuResponse::SmoothRms, meters.rms, meters.held_rms),
            (
                VuResponse::FastRms,
                meters.fast_rms.level,
                meters.fast_rms.held,
            ),
            (
                VuResponse::Peak,
                meters.peak_meter.level,
                meters.peak_meter.held,
            ),
        ] {
            let selected = meters.vu(profile);
            assert_eq!(selected.level, level);
            assert_eq!(selected.held, held);
            assert_ne!(selected.held, meters.held_peaks);
        }
    }
    fn tone(rate: u32, frequency: f64, amplitude: f64, count: usize) -> Vec<[f32; 2]> {
        (0..count)
            .map(|i| {
                let sample = (std::f64::consts::TAU * frequency * i as f64 / rate as f64
                    + std::f64::consts::FRAC_PI_4)
                    .sin()
                    * amplitude;
                [sample as f32; 2]
            })
            .collect()
    }
    #[test]
    fn true_peak_finds_intersample_overs_and_keeps_stereo_independent() {
        let mut meter = BroadcastMeter::new(48000).unwrap();
        let mut frames = tone(48000, 12000., 1.1, 48000);
        for f in &mut frames {
            f[1] = 0.;
        }
        let mut last = None;
        for chunk in frames.chunks(256) {
            last = Some(meter.observe(chunk).unwrap());
        }
        let last = last.unwrap();
        let sample_peak = levels(&frames).1;
        assert!(sample_peak[0] < -2.);
        assert!(
            last.true_peaks[0] > 0.5 && last.true_peaks[0] < 1.2,
            "{:?}",
            last.true_peaks
        );
        assert!(last.true_peaks[0] - sample_peak[0] > 2.5);
        assert_eq!(last.true_peaks[1], -60.);
        assert_eq!(last.held_peaks[1], -60.);
    }
    #[test]
    fn broadcast_silence_warmup_and_reset_do_not_publish_fake_values() {
        let mut meter = BroadcastMeter::new(48000).unwrap();
        let short = meter.observe(&tone(48000, 1000., 0.1, 4800)).unwrap();
        assert!(short.momentary_lufs.is_none() && short.short_term_lufs.is_none());
        let mut last = short;
        for _ in 0..800 {
            last = meter.observe(&[[0.; 2]; 256]).unwrap();
        }
        assert_eq!(last.true_peaks, [-60.; 2]);
        assert_eq!(last.held_peaks, [-60.; 2]);
        assert!(last.momentary_lufs.is_none());
        assert!(last.short_term_lufs.is_none());
        let mut restarted = BroadcastMeter::new(48000).unwrap();
        let reset = restarted.observe(&[[0.; 2]; 256]).unwrap();
        assert_eq!(reset.held_peaks, [-60.; 2]);
        assert!(reset.short_term_lufs.is_none());
    }
    #[test]
    fn true_peak_and_loudness_are_independent_of_worker_chunk_size() {
        let frames = tone(44100, 11025., 0.5, 44100);
        let mut one = BroadcastMeter::new(44100).unwrap();
        let mut split = BroadcastMeter::new(44100).unwrap();
        let whole = one.observe(&frames).unwrap();
        let mut highest = [-60f32; 2];
        for part in frames.chunks(127) {
            let part = split.observe(part).unwrap();
            for (highest, value) in highest.iter_mut().zip(part.true_peaks) {
                *highest = highest.max(value);
            }
        }
        for (a, b) in whole.true_peaks.into_iter().zip(highest) {
            assert!((a - b).abs() < 0.001);
        }
        assert!(
            (one.ebu.loudness_momentary().unwrap() - split.ebu.loudness_momentary().unwrap()).abs()
                < 0.00001
        );
    }
    #[test]
    fn known_stereo_loudness_and_gain_at_supported_output_rates() {
        for rate in [32000, 44100, 48000, 96000, 192000] {
            let mut meter = BroadcastMeter::new(rate).unwrap();
            let frames = tone(rate, 1000., 0.1, rate as usize * 4);
            let mut last = None;
            for chunk in frames.chunks(256) {
                last = Some(meter.observe(chunk).unwrap());
            }
            let last = last.unwrap();
            let m = last.momentary_lufs.unwrap();
            let s = last.short_term_lufs.unwrap();
            assert!((m + 20.).abs() < 0.15, "rate {rate}: {m}");
            assert!((s - m).abs() < 0.02);
            assert!((last.true_peaks[0] + 20.).abs() < 0.1);
            let mut quieter = BroadcastMeter::new(rate).unwrap();
            let half: Vec<_> = frames.iter().map(|f| [f[0] * 0.5, f[1] * 0.5]).collect();
            let half = quieter.observe(&half).unwrap();
            assert!((half.momentary_lufs.unwrap() - m + 6.0206).abs() < 0.02);
        }
    }
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
        assert_eq!(vu_calibrated(-15., VU_REFERENCE_DBFS), 0.);
        assert_eq!(vu_calibrated(-11., VU_REFERENCE_DBFS), 4.);
        assert_eq!(vu_calibrated(-7., VU_REFERENCE_DBFS), 8.);
        assert_eq!(vu_calibrated(0., VU_REFERENCE_DBFS), 15.);
        assert_eq!(vu_calibrated(-8., 0.), -8.);
        assert_eq!(vu_calibrated(-18., -18.), 0.);
        assert_eq!(vu_calibrated(0., -18.), 18.);
        assert_eq!(vu_position(-40.), 0.);
        assert_eq!(vu_position(18.), 1.);
        assert_eq!(vu_segment_index(0., 0.), Some(21));
        assert_eq!(vu_segment_index(0., VU_REFERENCE_DBFS), Some(36));
        assert_eq!(vu_segment_index(0., -18.), Some(39));
        assert_eq!(vu_segment_index(-60., VU_REFERENCE_DBFS), None);
        assert_eq!(vu_segment_index(-18., 0.), Some(6));
        for (i, v) in VU_SCALE.iter().enumerate() {
            assert_eq!(vu_segment_index(*v - 18., -18.), Some(i));
        }
        for pair in VU_SCALE.windows(2) {
            assert!(vu_position(pair[0]) < vu_position(pair[1]));
        }
    }
    #[test]
    fn spectrum_has_absolute_rms_calibration() {
        let frames: Vec<_> = (0..N)
            .map(|i| {
                let x = (std::f32::consts::TAU * 1125. * i as f32 / 48000.).sin() * 0.25;
                [x, x]
            })
            .collect();
        let measured = Analyzer::new().analyze(&frames, 48000);
        assert!((measured.spectrum[5] - db(0.25 / 2f32.sqrt())).abs() < 0.02);
        let left_only: Vec<_> = frames.iter().map(|f| [f[0], 0.]).collect();
        let left = Analyzer::new().analyze(&left_only, 48000);
        assert!((left.spectrum[5] - measured.spectrum[5] + 3.0103).abs() < 0.02);
    }
    #[test]
    fn every_frequency_label_tracks_its_actual_band() {
        let mut analyzer = Analyzer::new();
        for (band, frequency) in BANDS.iter().enumerate() {
            let frames: Vec<_> = (0..N)
                .map(|i| {
                    let x = (std::f32::consts::TAU * frequency * i as f32 / 48000.).sin() * 0.2;
                    [x, x]
                })
                .collect();
            let measured = analyzer.analyze(&frames, 48000);
            let strongest = measured
                .spectrum
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.total_cmp(b.1))
                .unwrap()
                .0;
            assert_eq!(strongest, band, "frequency {frequency}");
        }
    }
    #[test]
    fn rms_is_fast_but_does_not_turn_a_transient_into_a_sustained_level() {
        let mut meter = RmsMeter::new(48000);
        let at_50ms = meter.observe(&vec![[0.5, 0.]; 2400]).rms;
        assert!((at_50ms[0] - (-8.012)).abs() < 0.02);
        let settled = meter.observe(&vec![[0.5, 0.]; 21600]).rms;
        assert!((settled[0] + 6.0206).abs() < 0.01);
        assert_eq!(settled[1], -60.);
        let mut impulse = RmsMeter::new(48000);
        let measured = impulse.observe(&[[1., 0.]]);
        let body = measured.rms;
        let peak = measured.sample_peaks;
        assert!(body[0] < -30.);
        assert_eq!(peak[0], 0.);
        let mut chunked = RmsMeter::new(48000);
        for _ in 0..100 {
            chunked.observe(&[[0.5, 0.]; 240]);
        }
        assert!((chunked.observe(&[]).rms[0] - settled[0]).abs() < 0.001);
    }
    #[test]
    fn peak_holds_two_seconds_then_resets_without_synthetic_decay() {
        let mut h = PeakHold::new();
        assert_eq!(h.observe([-2., -20.], 0.01), [-2., -20.]);
        assert_eq!(h.observe([-30., -40.], 2.009), [-2., -20.]);
        assert_eq!(h.observe([-30., -40.], 2.011), [-30., -40.]);
        assert_eq!(h.observe([-60.; 2], 9.), [-60.; 2]);
    }
    #[test]
    fn equal_peaks_refresh_hold() {
        let mut h = PeakHold::new();
        h.observe([-3., -9.], 0.);
        h.observe([-3., -9.], 1.9);
        assert_eq!(h.observe([-40.; 2], 3.8), [-3., -9.]);
        assert_eq!(h.observe([-40.; 2], 3.901), [-40.; 2]);
    }
    #[test]
    fn new_peak_refreshes_only_its_own_channel() {
        let mut h = PeakHold::new();
        h.observe([-8., -9.], 0.);
        h.observe([-3., -20.], 1.9);
        assert_eq!(h.observe([-30., -40.], 2.001), [-3., -40.]);
        assert_eq!(h.observe([-30., -50.], 3.8), [-3., -40.]);
        assert_eq!(h.observe([-30., -50.], 3.901), [-30., -40.]);
    }
    #[test]
    fn vu_hold_keeps_envelope_peak_instead_of_raw_sample_peak() {
        let mut frames = [[0.; 2]; 256];
        frames[0][0] = 1.;
        let mut meter = RmsMeter::new(48000);
        let block = meter.observe(&frames);
        assert_eq!(block.sample_peaks[0], 0.);
        assert!(block.max_rms[0] < -30.);
        assert!(block.max_rms[0] > block.rms[0] + 0.4);
        assert_eq!(block.max_rms[1], -60.);
        let mut hold = PeakHold::new();
        let held = hold.observe(block.max_rms, 256. / 48000.);
        assert_eq!(held, block.max_rms);
        assert_ne!(held[0], block.sample_peaks[0]);
        let mut chunked = RmsMeter::new(48000);
        let mut maximum = [-60f32; 2];
        for part in frames.chunks(17) {
            let block = chunked.observe(part);
            for (value, measured) in maximum.iter_mut().zip(block.max_rms) {
                *value = value.max(measured);
            }
        }
        assert_eq!(maximum, block.max_rms);
        assert_eq!(chunked.observe(&[]).rms, block.rms);
    }
    #[test]
    fn held_segment_uses_the_same_thresholds_as_the_bar() {
        for (i, pair) in VU_SCALE.windows(2).enumerate() {
            let middle = (pair[0] + pair[1]) / 2.;
            let measured = middle + VU_REFERENCE_DBFS;
            assert_eq!(vu_segment_index(measured, VU_REFERENCE_DBFS), Some(i));
            assert_eq!(VU_SCALE.iter().rposition(|v| middle >= *v), Some(i));
        }
        assert_eq!(vu_segment_index(-60., -24.), None);
        assert_eq!(vu_segment_index(f32::NAN, -15.), None);
        assert_eq!(vu_segment_index(10., -15.), Some(39));
    }
}
