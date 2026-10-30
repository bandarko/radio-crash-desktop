#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod audio;
mod audio_resample;
mod dsp;
mod metadata;
mod platform;
mod settings;
mod skins;
mod tray_meter;
mod ui;
fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    if let Some(seconds) = std::env::args().skip_while(|a| a != "--smoke").nth(1) {
        let seconds = seconds.parse::<u64>()?;
        let mut engine = audio::Engine::new();
        let mut s = settings::Settings::default();
        if let Ok(url) = std::env::var("RC_STREAM_URL") {
            s.stream_url = url;
        }
        engine.volume(0.1);
        engine.start(s);
        for _ in 0..seconds {
            std::thread::sleep(std::time::Duration::from_secs(1));
            let s = engine.state.lock().unwrap();
            println!(
                "{} {} {:?} {} Hz {} ch | RMS {:?} | FFT {:?} | {} / {} | {}",
                s.status,
                s.codec,
                s.bitrate,
                s.rate,
                s.channels,
                s.meters.rms,
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
            .with_title("Radio Crash Desktop")
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
