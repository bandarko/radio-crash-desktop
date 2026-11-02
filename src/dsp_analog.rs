//! Full-wave average, sine-RMS calibrated electromechanical VU model.
//! Fixed 300 ms to 99% travel and 1.25% overshoot, NOT an RMS/peak detector.
//! Reference: Classic Audio, Practical VU Meter Circuits (ANSI VU meter section).
//! Runs on consumed PCM in the analysis worker. No GUI-frame-clock filtering.
use crate::dsp::db;

pub const DEFAULT_REFERENCE_DBFS: f32 = -8.;

const DAMPING: f64 = 0.8127169864286775;
const OMEGA: f64 = 13.51191256253271;
const SINE_CALIBRATION: f64 = std::f64::consts::PI / (2. * std::f64::consts::SQRT_2);

#[derive(Clone, Copy, Default)]
pub struct Movement {
    pub position: f64,
    pub velocity: f64,
}
#[derive(Clone, Copy, Default)]
pub struct Reading {
    pub channels: [Movement; 2],
}
impl Reading {
    pub fn levels(self) -> [f32; 2] {
        self.channels.map(|m| db(m.position.max(0.) as f32))
    }
}
/// Exact zero-order-hold transition of x'' + 2*zeta*w*x' + w²*x = w²*u.
/// Precomputed at the sample rate; no per-sample trig, allocations or locks.
pub struct Transition {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
}
impl Transition {
    pub fn new(seconds: f64) -> Self {
        let t = if seconds.is_finite() {
            seconds.max(0.)
        } else {
            0.
        };
        let decay = DAMPING * OMEGA;
        let frequency = OMEGA * (1. - DAMPING * DAMPING).sqrt();
        let e = (-decay * t).exp();
        let (s, c) = (frequency * t).sin_cos();
        let b = e * s / frequency;
        Self {
            a: e * c + decay * b,
            b,
            c: -OMEGA * OMEGA * b,
            d: e * c - decay * b,
        }
    }
    pub fn advance(&self, movement: &mut Movement, target: f64) {
        let error = movement.position - target;
        let position = target + self.a * error + self.b * movement.velocity;
        let velocity = self.c * error + self.d * movement.velocity;
        *movement = Movement { position, velocity };
    }
}
pub struct AnalogMeter {
    reading: Reading,
    step: Transition,
}
impl AnalogMeter {
    pub fn new(rate: u32) -> Self {
        Self {
            reading: Reading::default(),
            step: Transition::new(1. / rate.max(1) as f64),
        }
    }
    pub fn observe(&mut self, frames: &[[f32; 2]]) -> Reading {
        for frame in frames {
            for (c, sample) in frame.iter().enumerate() {
                let input = if sample.is_finite() {
                    sample.abs() as f64
                } else {
                    0.
                };
                self.step
                    .advance(&mut self.reading.channels[c], input * SINE_CALIBRATION);
            }
        }
        self.reading
    }
}
/// Moving-coil deflection is proportional to rectified current, not dB.
/// The printed logarithmic labels end at +3 VU; zero input rests before -20.
pub fn position(amplitude: f64, reference_dbfs: f32) -> f32 {
    if !amplitude.is_finite() || !reference_dbfs.is_finite() {
        return 0.;
    }
    (amplitude.max(0.) / 10f64.powf((reference_dbfs as f64 + 3.) / 20.)).clamp(0., 1.) as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sine(rate: u32, seconds: f32, rms: f32) -> Vec<[f32; 2]> {
        (0..(rate as f32 * seconds) as usize)
            .map(|n| {
                let v = (std::f32::consts::TAU * 1000. * n as f32 / rate as f32).sin()
                    * rms
                    * 2f32.sqrt();
                [v, 0.]
            })
            .collect()
    }
    #[test]
    fn reference_sine_is_zero_vu_and_channels_are_independent() {
        for rate in [44100, 48000, 96000] {
            let reference = -12.;
            let mut meter = AnalogMeter::new(rate);
            let r = meter.observe(&sine(rate, 1.5, 10f32.powf(reference / 20.)));
            assert!(
                (r.levels()[0] - reference).abs() < 0.03,
                "rate={rate}: {:?}",
                r.levels()
            );
            assert_eq!(r.levels()[1], -60.);
            assert!(
                (position(r.channels[0].position, reference) - 10f32.powf(-3. / 20.)).abs() < 0.003
            );
        }
    }
    #[test]
    fn authentic_step_timing_overshoot_and_symmetric_return() {
        for rate in [44100, 48000, 96000] {
            let mut meter = AnalogMeter::new(rate);
            let mut crossed = None;
            let mut peak = 0f64;
            for n in 1..=rate {
                let p = meter
                    .observe(&[[1. / SINE_CALIBRATION as f32, 0.]])
                    .channels[0]
                    .position;
                if crossed.is_none() && p >= 0.99 {
                    crossed = Some(n as f64 / rate as f64);
                }
                peak = peak.max(p);
            }
            assert!((crossed.unwrap() - 0.3).abs() < 0.001);
            assert!((1.01..=1.015).contains(&peak), "peak {peak}");
            let returned = meter.observe(&vec![[0.; 2]; (rate as f64 * 0.3) as usize]);
            assert!(returned.channels[0].position.abs() < 0.011);
        }
    }
    #[test]
    fn average_rectified_is_distinct_from_rms_for_short_pulses() {
        let mut meter = AnalogMeter::new(48000);
        let frames: Vec<_> = (0..96000)
            .map(|i| if i % 10 == 0 { [1., 0.] } else { [0.; 2] })
            .collect();
        let actual = meter.observe(&frames).levels()[0];
        let expected = 20. * (0.1 * SINE_CALIBRATION as f32).log10();
        assert!((actual - expected).abs() < 0.03);
        assert!(actual < -18.9); // RMS of this waveform is -10 dBFS.
    }
    #[test]
    fn ten_millisecond_hits_cannot_whip_the_needle_to_full_scale() {
        for rate in [44100, 48000, 96000] {
            let frames = sine(rate, 0.01, 0.25);
            let mut meter = AnalogMeter::new(rate);
            let mut peak = meter.observe(&frames).channels[0].position;
            for _ in 0..rate {
                peak = peak.max(meter.observe(&[[0.; 2]]).channels[0].position);
            }
            // A peak-reading meter would register the full 0.354 amplitude.
            // The mechanical movement integrates the brief burst instead.
            assert!(peak < 0.25 * 0.10, "rate={rate}, peak={peak}");
        }
    }
    #[test]
    fn packet_boundaries_do_not_change_movement_and_nonfinite_input_is_safe() {
        let frames = sine(48000, 1., 0.2);
        let full = AnalogMeter::new(48000).observe(&frames);
        let mut split = AnalogMeter::new(48000);
        let mut last = Reading::default();
        for block in frames.chunks(137) {
            last = split.observe(block);
        }
        assert_eq!(full.channels[0].position, last.channels[0].position);
        assert_eq!(full.channels[0].velocity, last.channels[0].velocity);
        let last = split.observe(&[[f32::NAN, f32::INFINITY]; 512]);
        assert!(last.levels().iter().all(|v| v.is_finite()));
    }
    #[test]
    fn scale_is_current_linear_with_plus_three_endpoint() {
        assert_eq!(position(0., -12.), 0.);
        for db in [-20., -10., -7., -5., -3., 0., 1., 2., 3.] {
            let a = 10f64.powf((-12. + db) / 20.);
            assert!((position(a, -12.) - 10f32.powf((db as f32 - 3.) / 20.)).abs() < 0.0001);
        }
        assert_eq!(position(1., -12.), 1.);
    }
}
