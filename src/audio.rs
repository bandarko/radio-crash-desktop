use crate::{
    dsp::{Analyzer, Meters, N},
    metadata,
    settings::Settings,
};
use anyhow::{Result, anyhow};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_queue::ArrayQueue;
use std::{
    io::{self, Read, Seek, SeekFrom},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
#[derive(Clone, Default)]
pub struct Snapshot {
    pub status: String,
    pub error: String,
    pub artist: String,
    pub title: String,
    pub codec: String,
    pub rate: u32,
    pub channels: usize,
    pub bitrate: Option<u32>,
    pub meters: Meters,
}
pub struct Engine {
    pub state: Arc<Mutex<Snapshot>>,
    pub gain: Arc<AtomicU32>,
    cancel: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
    retired: Vec<JoinHandle<()>>,
}
impl Engine {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(Snapshot {
                status: "OFFLINE".into(),
                ..Default::default()
            })),
            gain: Arc::new(AtomicU32::new(0.5f32.to_bits())),
            cancel: Arc::new(AtomicBool::new(false)),
            worker: None,
            retired: vec![],
        }
    }
    pub fn start(&mut self, s: Settings) {
        self.stop();
        self.cancel = Arc::new(AtomicBool::new(false));
        let stop = self.cancel.clone();
        let state = self.state.clone();
        let gain = self.gain.clone();
        self.worker = Some(thread::spawn(move || {
            let mut attempt = 0;
            while !stop.load(Ordering::Relaxed) {
                *state.lock().unwrap() = Snapshot {
                    status: "CONNECTING".into(),
                    ..Default::default()
                };
                let began = Instant::now();
                if let Err(e) = session(&s, &stop, &state, gain.clone()) {
                    log::warn!("stream: {e:#}");
                    let mut st = state.lock().unwrap();
                    st.error = format!("{e:#}");
                    st.status = "RECONNECTING".into();
                    st.meters = Meters::default();
                }
                if began.elapsed() > Duration::from_secs(30) {
                    attempt = 0;
                }
                if !wait(&stop, backoff(attempt)) {
                    break;
                }
                attempt += 1;
            }
        }));
    }
    pub fn stop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
        if let Some(w) = self.worker.take() {
            self.retired.push(w);
        }
        self.retired.retain(|w| !w.is_finished());
        self.state = Arc::new(Mutex::new(Snapshot {
            status: "OFFLINE".into(),
            ..Default::default()
        }));
    }
    pub fn running(&self) -> bool {
        self.worker.is_some()
    }
    pub fn volume(&self, v: f32) {
        self.gain
            .store(v.clamp(0., 1.).to_bits(), Ordering::Relaxed);
    }
}
impl Drop for Engine {
    fn drop(&mut self) {
        self.stop();
        for w in self.retired.drain(..) {
            let _ = w.join();
        }
    }
}
pub fn backoff(n: u32) -> Duration {
    Duration::from_secs(1u64 << n.min(5))
}
fn wait(stop: &AtomicBool, d: Duration) -> bool {
    let start = Instant::now();
    while start.elapsed() < d {
        if stop.load(Ordering::Relaxed) {
            return false;
        }
        thread::sleep(Duration::from_millis(10));
    }
    true
}
struct Source {
    rx: Mutex<mpsc::Receiver<Vec<u8>>>,
    chunk: io::Cursor<Vec<u8>>,
    stop: Arc<AtomicBool>,
}
impl Read for Source {
    fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
        loop {
            if self.stop.load(Ordering::Relaxed) {
                return Err(io::ErrorKind::ConnectionAborted.into());
            }
            let n = self.chunk.read(b)?;
            if n > 0 {
                return Ok(n);
            }
            match self
                .rx
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_millis(50))
            {
                Ok(v) => self.chunk = io::Cursor::new(v),
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(_) => return Ok(0),
            }
        }
    }
}
impl Seek for Source {
    fn seek(&mut self, _: SeekFrom) -> io::Result<u64> {
        Err(io::ErrorKind::Unsupported.into())
    }
}
impl symphonia::core::io::MediaSource for Source {
    fn is_seekable(&self) -> bool {
        false
    }
    fn byte_len(&self) -> Option<u64> {
        None
    }
}
// Abort-on-drop ensures all nested workers join even on a decoder/device error.
struct Workers {
    stop: Arc<AtomicBool>,
    handles: Vec<JoinHandle<()>>,
}
impl Drop for Workers {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        for h in self.handles.drain(..) {
            let _ = h.join();
        }
    }
}
fn session(
    s: &Settings,
    stop: &Arc<AtomicBool>,
    state: &Arc<Mutex<Snapshot>>,
    gain: Arc<AtomicU32>,
) -> Result<()> {
    let local = Arc::new(AtomicBool::new(false));
    let mut workers = Workers {
        stop: local.clone(),
        handles: vec![],
    };
    let (tx, rx) = mpsc::sync_channel(8);
    let net_stop = local.clone();
    let global = stop.clone();
    let url = s.stream_url.clone();
    let net_state = state.clone();
    workers.handles.push(thread::spawn(move||{let rt=tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();rt.block_on(async move{let client=reqwest::Client::builder().connect_timeout(Duration::from_secs(5)).read_timeout(Duration::from_secs(8)).build().unwrap();let transfer=async{let mut r=client.get(url).header("Icy-MetaData","0").send().await?.error_for_status()?;while let Some(b)=r.chunk().await?{let mut b=b.to_vec();loop{match tx.try_send(b){Ok(())=>break,Err(mpsc::TrySendError::Full(v))=>{b=v;tokio::time::sleep(Duration::from_millis(5)).await;},Err(_)=>return Ok::<(),reqwest::Error>(())}}}Ok(())};tokio::pin!(transfer);loop{tokio::select!{r=&mut transfer=>{if let Err(e)=r{net_state.lock().unwrap().error=e.to_string();}break;},_=tokio::time::sleep(Duration::from_millis(20))=>{if net_stop.load(Ordering::Relaxed)||global.load(Ordering::Relaxed){break;}}}}});}));
    let source = Source {
        rx: Mutex::new(rx),
        chunk: io::Cursor::new(vec![]),
        stop: stop.clone(),
    };
    use symphonia::core::{
        formats::{TrackType, probe::Hint},
        io::MediaSourceStream,
    };
    let mut format = symphonia::default::get_probe().probe(
        &Hint::new(),
        MediaSourceStream::new(Box::new(source), Default::default()),
        Default::default(),
        Default::default(),
    )?;
    let track = format
        .default_track(TrackType::Audio)
        .ok_or_else(|| anyhow!("No audio track"))?;
    let id = track.id;
    let params = track
        .codec_params
        .as_ref()
        .and_then(|p| p.audio())
        .ok_or_else(|| anyhow!("Missing audio parameters"))?;
    let mut decoder =
        symphonia::default::get_codecs().make_audio_decoder(params, &Default::default())?;
    let codec = match params.codec {
        symphonia::core::codecs::audio::well_known::CODEC_ID_AAC => "AAC".to_string(),
        symphonia::core::codecs::audio::well_known::CODEC_ID_MP3 => "MP3".to_string(),
        _ => format!("{:?}", params.codec),
    };
    let host = cpal::default_host();
    let device = if s.device.is_empty() {
        host.default_output_device()
    } else {
        host.output_devices()?
            .find(|d| d.description().is_ok_and(|v| v.name() == s.device))
    }
    .ok_or_else(|| anyhow!("Output device unavailable"))?;
    let default_config = device.default_output_config()?;
    let native_rate = params.sample_rate.unwrap_or(default_config.sample_rate());
    let config = device
        .supported_output_configs()?
        .find(|r| {
            r.channels() == 2
                && r.sample_format() == default_config.sample_format()
                && r.min_sample_rate() <= native_rate
                && r.max_sample_rate() >= native_rate
        })
        .map(|r| r.with_sample_rate(native_rate))
        .unwrap_or(default_config);
    let output_rate = config.sample_rate();
    let output_channels = config.channels() as usize;
    let queue = Arc::new(ArrayQueue::<[f32; 2]>::new(output_rate as usize));
    let analysis = Arc::new(ArrayQueue::<[f32; 2]>::new(N * 4));
    let underflows = Arc::new(AtomicU64::new(0));
    let frames_played = Arc::new(AtomicU64::new(0));
    let output_error = Arc::new(AtomicBool::new(false));
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => output::<f32>(
            &device,
            config.into(),
            queue.clone(),
            analysis.clone(),
            gain,
            underflows.clone(),
            frames_played.clone(),
            output_error.clone(),
        )?,
        cpal::SampleFormat::I16 => output::<i16>(
            &device,
            config.into(),
            queue.clone(),
            analysis.clone(),
            gain,
            underflows.clone(),
            frames_played.clone(),
            output_error.clone(),
        )?,
        cpal::SampleFormat::U16 => output::<u16>(
            &device,
            config.into(),
            queue.clone(),
            analysis.clone(),
            gain,
            underflows.clone(),
            frames_played.clone(),
            output_error.clone(),
        )?,
        f => return Err(anyhow!("Unsupported device sample format: {f}")),
    };
    let a = analysis.clone();
    let st = state.clone();
    let ls = local.clone();
    let gs = stop.clone();
    let metadata_url = s.metadata_url.clone();
    workers.handles.push(thread::spawn(move || {
        let mut analyzer = Analyzer::new();
        let mut block = Vec::with_capacity(N);
        let mut vu = [[0.; 2]; 256];
        let mut vu_index = 0;
        let mut rms_meter = crate::dsp::RmsMeter::new(output_rate);
        let mut fast_rms_meter = crate::dsp::RmsMeter::with_time_constant(output_rate, 0.015);
        let mut peak_meter = crate::dsp::PeakMeter::new(output_rate);
        let mut peak_hold = crate::dsp::PeakHold::new();
        let mut fast_hold = crate::dsp::PeakHold::new();
        let mut envelope_hold = crate::dsp::PeakHold::new();
        let mut broadcast = crate::dsp::BroadcastMeter::new(output_rate)
            .map_err(|e| log::warn!("Broadcast meter unavailable: {e}"))
            .ok();
        let mut analyzed_frames = 0u64;
        while !ls.load(Ordering::Relaxed) && !gs.load(Ordering::Relaxed) {
            while let Some(f) = a.pop() {
                vu[vu_index] = f;
                vu_index += 1;
                if vu_index == vu.len() {
                    let measured = rms_meter.observe(&vu);
                    let fast = fast_rms_meter.observe(&vu);
                    let peak = peak_meter.observe(&vu);
                    analyzed_frames += vu.len() as u64;
                    let seconds = analyzed_frames as f64 / output_rate as f64;
                    let held = peak_hold.observe(
                        measured.max_rms,
                        analyzed_frames as f64 / output_rate as f64,
                    );
                    let fast = crate::dsp::VuReading {
                        level: fast.rms,
                        held: fast_hold.observe(fast.max_rms, seconds),
                    };
                    let peak = crate::dsp::VuReading {
                        level: peak.level,
                        held: envelope_hold.observe(peak.held, seconds),
                    };
                    let quality = match broadcast.as_mut().map(|meter| meter.observe(&vu)) {
                        Some(Ok(measured)) => Some(measured),
                        Some(Err(e)) => {
                            log::warn!("Broadcast meter disabled for this session: {e}");
                            broadcast = None;
                            None
                        }
                        None => None,
                    };
                    let mut snapshot = st.lock().unwrap();
                    snapshot.meters.held_rms = held;
                    snapshot.meters.fast_rms = fast;
                    snapshot.meters.peak_meter = peak;
                    snapshot.meters.rms = measured.rms;
                    snapshot.meters.peaks = measured.sample_peaks;
                    if let Some(quality) = quality {
                        snapshot.meters.true_peaks = Some(quality.true_peaks);
                        snapshot.meters.held_peaks = quality.held_peaks;
                        snapshot.meters.momentary_lufs = quality.momentary_lufs;
                        snapshot.meters.short_term_lufs = quality.short_term_lufs;
                    } else {
                        snapshot.meters.true_peaks = None;
                        snapshot.meters.held_peaks = [-60.; 2];
                        snapshot.meters.momentary_lufs = None;
                        snapshot.meters.short_term_lufs = None;
                    }
                    vu_index = 0;
                }
                block.push(f);
                if block.len() == N {
                    let measured = analyzer.analyze(&block, output_rate);
                    {
                        let mut snapshot = st.lock().unwrap();
                        snapshot.meters.spectrum = measured.spectrum;
                        snapshot.meters.available = measured.available;
                    }
                    block.copy_within(N / 4..N, 0);
                    block.truncate(N * 3 / 4);
                }
            }
            thread::sleep(Duration::from_millis(2));
        }
    }));
    let st = state.clone();
    let ls = local.clone();
    let gs = stop.clone();
    workers.handles.push(thread::spawn(move || {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .unwrap();
        while !ls.load(Ordering::Relaxed) && !gs.load(Ordering::Relaxed) {
            if !metadata_url.is_empty() {
                match client
                    .get(&metadata_url)
                    .send()
                    .and_then(|r| r.error_for_status())
                    .and_then(|r| r.json::<serde_json::Value>())
                {
                    Ok(v) => {
                        let parsed = metadata::parse(&v);
                        let mut st = st.lock().unwrap();
                        if let Some((artist, title)) = parsed {
                            st.artist = artist;
                            st.title = title;
                        } else {
                            st.artist.clear();
                            st.title.clear();
                        }
                    }
                    Err(e) => {
                        log::debug!("metadata: {e}");
                        let mut st = st.lock().unwrap();
                        st.artist.clear();
                        st.title.clear();
                    }
                }
            }
            for _ in 0..120 {
                if ls.load(Ordering::Relaxed) || gs.load(Ordering::Relaxed) {
                    return;
                }
                thread::sleep(Duration::from_millis(100));
            }
        }
    }));
    let mut samples: Vec<f32> = vec![];
    let mut started = false;
    let mut bytes = 0usize;
    let mut seconds = 0f64;
    let mut resampler: Option<crate::audio_resample::Resampler> = None;
    let mut source_rate = 0;
    let mut underrun_last = 0;
    while !stop.load(Ordering::Relaxed) {
        if output_error.load(Ordering::Relaxed) {
            return Err(anyhow!("Audio output device failed"));
        }
        let Some(packet) = format.next_packet()? else {
            return Err(anyhow!("Stream ended"));
        };
        if packet.track_id != id {
            continue;
        }
        bytes += packet.data.len();
        let decoded = decoder.decode(&packet)?;
        let spec = decoded.spec();
        let rate = spec.rate();
        let channels = spec.channels().count();
        if channels == 0 || channels > 2 {
            return Err(anyhow!("Only mono/stereo streams supported"));
        }
        samples.resize(decoded.samples_interleaved(), 0.);
        decoded.copy_to_slice_interleaved(&mut samples);
        seconds += samples.len() as f64 / channels as f64 / rate as f64;
        {
            let mut st = state.lock().unwrap();
            st.rate = rate;
            st.channels = channels;
            st.codec = if codec.to_lowercase().contains("aac") {
                "AAC".into()
            } else if codec.to_lowercase().contains("mp3") {
                "MP3".into()
            } else {
                codec.clone()
            };
            st.bitrate = if seconds > 1. {
                Some((bytes as f64 * 8. / seconds / 1000.).round() as u32)
            } else {
                None
            };
            st.status = if started { "ON AIR" } else { "BUFFERING" }.into();
            st.error.clear();
        }
        if source_rate != rate {
            resampler = Some(crate::audio_resample::Resampler::new(rate, output_rate));
            source_rate = rate;
        }
        let converter = resampler.as_mut().unwrap();
        for frame in samples.chunks_exact(channels) {
            converter.push([frame[0], frame[channels - 1]]);
            while let Some(out) = converter.pop() {
                let mut pending = out;
                loop {
                    match queue.push(pending) {
                        Ok(()) => break,
                        Err(v) => {
                            pending = v;
                            if output_error.load(Ordering::Relaxed) {
                                return Err(anyhow!("Output device disconnected"));
                            }
                            if !wait(stop, Duration::from_millis(3)) {
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }
        if !started && queue.len() > output_rate as usize / 5 {
            stream.play()?;
            started = true;
        }
        let u = underflows.load(Ordering::Relaxed);
        if u > underrun_last {
            state.lock().unwrap().status = "BUFFERING".into();
            underrun_last = u;
        }
    }
    log::info!(
        "Output: {} frames, {} underrun frames, {} channels",
        frames_played.load(Ordering::Relaxed),
        underflows.load(Ordering::Relaxed),
        output_channels
    );
    drop(stream);
    Ok(())
}
#[allow(clippy::too_many_arguments)]
fn output<T: cpal::SizedSample + cpal::FromSample<f32>>(
    d: &cpal::Device,
    c: cpal::StreamConfig,
    q: Arc<ArrayQueue<[f32; 2]>>,
    a: Arc<ArrayQueue<[f32; 2]>>,
    gain: Arc<AtomicU32>,
    underruns: Arc<AtomicU64>,
    played: Arc<AtomicU64>,
    error: Arc<AtomicBool>,
) -> Result<cpal::Stream> {
    let ch = c.channels as usize;
    let mut smooth = 0.;
    Ok(d.build_output_stream(
        c,
        move |data: &mut [T], _| {
            let target = f32::from_bits(gain.load(Ordering::Relaxed));
            for frame in data.chunks_mut(ch) {
                let raw = q.pop().unwrap_or_else(|| {
                    underruns.fetch_add(1, Ordering::Relaxed);
                    [0.; 2]
                });
                smooth += (target - smooth) * 0.002;
                let _ = a.push(raw);
                for (i, v) in frame.iter_mut().enumerate() {
                    let x = if ch == 1 {
                        (raw[0] + raw[1]) * 0.5
                    } else if i < 2 {
                        raw[i]
                    } else {
                        0.
                    };
                    *v = T::from_sample((x * smooth).clamp(-1., 1.));
                }
                played.fetch_add(1, Ordering::Relaxed);
            }
        },
        move |e| handle_output_error(e, &error),
        None,
    )?)
}
fn handle_output_error(e: cpal::Error, restart: &AtomicBool) {
    match e.kind() {
        // CPAL keeps the stream alive for these notifications. Restarting it
        // would turn a short scheduling glitch into an unnecessary reconnect.
        cpal::ErrorKind::Xrun | cpal::ErrorKind::RealtimeDenied => {
            log::warn!("audio output continues: {e}");
        }
        cpal::ErrorKind::DeviceChanged => log::info!("audio route: {e}"),
        _ => {
            log::error!("audio device: {e}");
            restart.store(true, Ordering::Relaxed);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn output_notifications_do_not_disconnect_stream() {
        let restart = AtomicBool::new(false);
        for kind in [
            cpal::ErrorKind::Xrun,
            cpal::ErrorKind::RealtimeDenied,
            cpal::ErrorKind::DeviceChanged,
        ] {
            handle_output_error(cpal::Error::new(kind), &restart);
            assert!(!restart.load(Ordering::Relaxed));
        }
        for kind in [
            cpal::ErrorKind::DeviceNotAvailable,
            cpal::ErrorKind::StreamInvalidated,
            cpal::ErrorKind::BackendError,
        ] {
            restart.store(false, Ordering::Relaxed);
            handle_output_error(cpal::Error::new(kind), &restart);
            assert!(restart.load(Ordering::Relaxed));
            handle_output_error(cpal::Error::new(cpal::ErrorKind::Xrun), &restart);
            assert!(restart.load(Ordering::Relaxed));
        }
    }
    #[test]
    fn bounded_backoff() {
        assert_eq!(
            (0..8).map(|i| backoff(i).as_secs()).collect::<Vec<_>>(),
            vec![1, 2, 4, 8, 16, 32, 32, 32]
        );
    }
    #[test]
    fn cancellation_while_server_stalls() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let mut e = Engine::new();
        let settings = Settings {
            stream_url: format!("http://{address}/"),
            metadata_url: String::new(),
            ..Default::default()
        };
        e.start(settings);
        std::thread::sleep(Duration::from_millis(60));
        let began = Instant::now();
        e.stop();
        assert_eq!(e.state.lock().unwrap().status, "OFFLINE");
        drop(e);
        assert!(began.elapsed() < Duration::from_secs(2));
    }
}
