#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod analog;
mod audio;
mod audio_resample;
mod clock_face;
mod dsp;
mod dsp_analog;
mod graphics;
mod metadata;
mod panel;
mod platform;
mod settings;
mod skins;
mod tray_meter;
mod ui;
mod updates;
fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    if let Some(seconds) = std::env::args().skip_while(|a| a != "--audit-vu").nth(1) {
        let seconds = seconds.parse::<u64>()?.clamp(1, 120);
        let mut engine = audio::Engine::new();
        engine.volume(0.);
        engine.start(settings::Settings::default());
        let start = std::time::Instant::now();
        let mut readings = Vec::with_capacity(seconds as usize * 50);
        while start.elapsed().as_secs() < seconds {
            std::thread::sleep(std::time::Duration::from_millis(20));
            let s = engine.state.lock().unwrap();
            if s.status == "ON AIR" {
                readings.push(serde_json::json!({"seconds":start.elapsed().as_secs_f32(),"dbfs":s.meters.analog.levels()}));
            }
        }
        engine.stop();
        println!("{}", serde_json::to_string(&readings)?);
        return Ok(());
    }
    if std::env::args().any(|a| a == "--check-update") {
        println!("{}", updates::check_now()?);
        return Ok(());
    }
    if let Some(seconds) = std::env::args().skip_while(|a| a != "--smoke").nth(1) {
        let seconds = seconds.parse::<u64>()?;
        let mut engine = audio::Engine::new();
        let mut s = settings::Settings::default();
        if let Ok(url) = std::env::var("RC_STREAM_URL") {
            s.stream_url = url;
        }
        engine.volume(if std::env::args().any(|a| a == "--muted") {
            0.
        } else {
            0.1
        });
        engine.start(s);
        for _ in 0..seconds {
            std::thread::sleep(std::time::Duration::from_secs(1));
            let s = engine.state.lock().unwrap();
            println!(
                "{} {} {:?} {} Hz {} ch | ANALOG AVG {:?} | RMS {:?} | RMS HOLD {:?} | FAST RMS {:?} HOLD {:?} | PEAK METER {:?} HOLD {:?} | TP HOLD {:?} | TRUE PEAK {:?} | SAMPLE PEAK {:?} | LUFS M {:?} S {:?} | FFT {:?} | {} / {} | {}",
                s.status,
                s.codec,
                s.bitrate,
                s.rate,
                s.channels,
                s.meters.analog.levels(),
                s.meters.rms,
                s.meters.held_rms,
                s.meters.fast_rms.level,
                s.meters.fast_rms.held,
                s.meters.peak_meter.level,
                s.meters.peak_meter.held,
                s.meters.held_peaks,
                s.meters.true_peaks,
                s.meters.peaks,
                s.meters.momentary_lufs,
                s.meters.short_term_lufs,
                s.meters.spectrum,
                s.artist,
                s.title,
                s.error
            );
        }
        engine.stop();
        return Ok(());
    }
    let mut settings = settings::Settings::load();
    if let Some(size) = std::env::args().skip_while(|a| a != "--window-size").nth(1)
        && let Some((w, h)) = size.split_once('x')
        && let (Ok(w), Ok(h)) = (w.parse(), h.parse())
    {
        settings.window_width = w;
        settings.window_height = h;
    }
    let size = settings::window_size(
        settings.window_width,
        settings.window_height,
        settings.lock_aspect,
    );
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size(size)
            .with_min_inner_size([560., 200.])
            .with_resizable(true)
            .with_title(if settings::Settings::local_preview() {
                "Radio Crash Desktop · Analog preview"
            } else if std::env::args().any(|a| a == "--qa-exit-after") {
                "Radio Crash Desktop · temporary graphics test"
            } else {
                "Radio Crash Desktop"
            })
            // The receiver itself is the window frame on both desktop platforms.
            // Windows' extra undecorated shadow/inset is disabled at creation.
            .with_decorations(false)
            .with_transparent(true)
            .with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!("../assets/RadioCrash.png"))
                    .expect("bundled icon"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "Radio Crash Desktop",
        options,
        Box::new(|cc| Ok(Box::new(ui::RadioApp::new(cc)))),
    )
    .map_err(|e| anyhow::anyhow!("{e}"))
}
