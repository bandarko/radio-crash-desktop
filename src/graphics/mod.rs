//! Enhanced SDR optics, layered over the unchanged original renderer.
mod gpu;
pub mod policy;
use eframe::{
    egui,
    glow::{self, HasContext},
};
pub use policy::Quality;
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Default, Serialize, Deserialize)]
struct SafetyRecord {
    fingerprint: String,
    probing: bool,
    blocked: Option<String>,
}
impl SafetyRecord {
    fn failure_for(self, fingerprint: &str) -> Option<String> {
        if self.fingerprint != fingerprint {
            return None;
        }
        self.blocked.or_else(||self.probing.then(||"The previous enhanced graphics session did not shut down cleanly. Original appearance retained.".into()))
    }
}
struct State {
    caps: policy::Capabilities,
    timer_api: Option<gpu::TimerApi>,
    renderer: Option<gpu::Renderer>,
    blocked: Option<String>,
    guard_marked: bool,
    guard_cleared: bool,
    active: bool,
}
pub struct Graphics {
    state: Arc<Mutex<State>>,
    safety: PathBuf,
    report: Option<PathBuf>,
    started: Instant,
    force_classic: bool,
}
impl Graphics {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let timer_api = gpu::TimerApi::load(cc);
        let caps = cc
            .gl
            .as_ref()
            .map(|gl| unsafe {
                policy::Capabilities::classify(
                    gl.get_parameter_string(glow::VENDOR),
                    gl.get_parameter_string(glow::RENDERER),
                    gl.get_parameter_string(glow::VERSION),
                    gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE),
                    gl.get_parameter_i32(glow::MAX_TEXTURE_IMAGE_UNITS),
                )
            })
            .unwrap_or_default();
        let safety = crate::settings::Settings::path().with_file_name("graphics-state.json");
        let record: SafetyRecord = std::fs::read(&safety)
            .ok()
            .and_then(|v| serde_json::from_slice(&v).ok())
            .unwrap_or_default();
        let blocked = record.failure_for(&caps.fingerprint());
        log::info!(
            "Graphics: {} / {} / {}. {}",
            caps.vendor,
            caps.renderer,
            caps.version,
            caps.reason
        );
        Self {
            state: Arc::new(Mutex::new(State {
                caps,
                timer_api,
                renderer: None,
                blocked,
                guard_marked: false,
                guard_cleared: false,
                active: false,
            })),
            safety,
            report: std::env::args()
                .skip_while(|a| a != "--graphics-report")
                .nth(1)
                .map(Into::into),
            started: Instant::now(),
            force_classic: std::env::args().any(|a| a == "--classic-graphics"),
        }
    }
    fn write_guard(&self, state: &State, probing: bool) -> Result<(), String> {
        let record = SafetyRecord {
            fingerprint: state.caps.fingerprint(),
            probing,
            blocked: state.blocked.clone(),
        };
        let parent = self
            .safety
            .parent()
            .ok_or("Graphics safety path is unavailable")?;
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        let tmp = self.safety.with_extension("tmp");
        std::fs::write(
            &tmp,
            serde_json::to_vec_pretty(&record).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        if cfg!(windows) && self.safety.exists() {
            std::fs::remove_file(&self.safety).map_err(|e| e.to_string())?;
        }
        std::fs::rename(tmp, &self.safety).map_err(|e| e.to_string())
    }
    pub fn tick(&mut self, quality: Quality) {
        let mut state = self.state.lock().unwrap();
        state.active =
            !self.force_classic && state.caps.requested(quality) && state.blocked.is_none();
        if state.active && !state.guard_marked {
            if let Err(e) = self.write_guard(&state, true) {
                state.blocked = Some(format!("Graphics recovery record could not be saved: {e}"));
                state.active = false;
            }
            state.guard_marked = true;
            state.guard_cleared = false;
        }
        if state.guard_marked && !state.guard_cleared && (state.blocked.is_some() || !state.active)
        {
            if let Err(e) = self.write_guard(&state, false) {
                log::warn!("Graphics recovery: {e}");
            }
            state.guard_cleared = true;
            if state.blocked.is_none() {
                state.guard_marked = false;
            }
        }
        if self.started.elapsed() > Duration::from_secs(6)
            && let Some(path) = self.report.take()
        {
            let report = serde_json::json!({"version":env!("CARGO_PKG_VERSION"),"capabilities":state.caps,"selection":quality,"active":state.active,"fallback":state.blocked,"passes":state.renderer.as_ref().map(|r|r.completed),"gpu_ms":state.renderer.as_ref().and_then(|r|r.gpu_ms),"output":"SDR","force_classic":self.force_classic});
            if let Err(e) = std::fs::write(path, report.to_string()) {
                log::warn!("Graphics report: {e}");
            }
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn paint(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        glass: bool,
        brightness: f32,
        signal: f32,
        volume: f32,
    ) {
        if !self.state.lock().unwrap().active {
            return;
        }
        let shared = self.state.clone();
        painter.add(egui::PaintCallback {
            rect,
            callback: Arc::new(eframe::egui_glow::CallbackFn::new(move |info, painter| {
                let mut state = shared.lock().unwrap();
                if !state.active {
                    return;
                }
                let gl = painter.gl();
                let viewport = info.viewport_in_pixels();
                let result = (|| {
                    if state.renderer.is_none() {
                        state.renderer = Some(gpu::Renderer::new(
                            gl,
                            state.timer_api.ok_or(
                                "GPU timer functions unavailable. Original appearance retained.",
                            )?,
                        )?);
                        log::info!("Enhanced GPU optics initialized");
                    }
                    let max_texture = state.caps.max_texture_size;
                    state.renderer.as_mut().unwrap().paint(
                        gl,
                        painter.intermediate_fbo(),
                        [
                            viewport.left_px,
                            viewport.from_bottom_px,
                            viewport.width_px,
                            viewport.height_px,
                        ],
                        glass,
                        brightness,
                        signal,
                        volume,
                        max_texture,
                    )
                })();
                if let Err(e) = result {
                    log::warn!("Enhanced graphics: {e}");
                    state.blocked = Some(e);
                    state.active = false;
                    state.guard_cleared = false;
                    if let Some(mut renderer) = state.renderer.take() {
                        renderer.destroy(gl);
                    }
                    // The callback restores the output FBO even on allocation failures.
                    unsafe {
                        gl.bind_framebuffer(glow::FRAMEBUFFER, painter.intermediate_fbo());
                    }
                }
            })),
        });
    }
    pub fn diagnostics(&self, quality: Quality) -> serde_json::Value {
        let state = self.state.lock().unwrap();
        serde_json::json!({"version":env!("CARGO_PKG_VERSION"),"backend":"OpenGL (eframe/glow)","minimum_desktop_opengl":"3.3","capabilities":state.caps,"selection":quality,"active":state.active,"fallback":state.blocked,"timer_functions_loaded":state.timer_api.is_some(),"gpu_ms":state.renderer.as_ref().and_then(|r|r.gpu_ms)})
    }
    pub fn settings(&mut self, ui: &mut egui::Ui, quality: &mut Quality) {
        let mut state = self.state.lock().unwrap();
        egui::ComboBox::from_label("Receiver graphics")
            .selected_text(quality.label())
            .show_ui(ui, |ui| {
                for q in [Quality::Auto, Quality::Classic, Quality::Enhanced] {
                    ui.add_enabled_ui(q != Quality::Enhanced || state.caps.supported, |ui| {
                        ui.selectable_value(quality, q, q.label());
                    });
                }
            });
        ui.small(if state.active {
            "Enhanced optics active · GPU glass, phosphor bloom and metal lighting"
        } else {
            "Classic appearance active · original receiver rendering"
        });
        egui::CollapsingHeader::new("Graphics details").show(ui,|ui|{
            ui.label(format!("{} · {}",state.caps.vendor,state.caps.renderer));
            ui.small(format!("OpenGL context: {} (required: desktop 3.3+)",state.caps.version));
            if let Some(reason)=&state.blocked{ui.label(reason);}else{ui.small(&state.caps.reason);}
            if let Some(ms)=state.renderer.as_ref().and_then(|r|r.gpu_ms){ui.small(format!("GPU optics: {ms:.2} ms per frame (asynchronous shader measurement)"));}
            ui.small("Standard dynamic range output. No HDR display mode is requested. Older or unsupported graphics keep the original appearance automatically.");
            if state.blocked.is_some()&&state.caps.supported&&ui.button("Retry enhanced graphics").clicked(){state.blocked=None;state.guard_marked=false;state.guard_cleared=false;}
        });
    }
    pub fn destroy(&mut self, gl: Option<&glow::Context>) {
        let mut state = self.state.lock().unwrap();
        if let (Some(gl), Some(mut renderer)) = (gl, state.renderer.take()) {
            renderer.destroy(gl);
        }
        if state.guard_marked {
            let _ = self.write_guard(&state, false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interrupted_probe_recovers_only_the_matching_gpu() {
        let interrupted = || SafetyRecord {
            fingerprint: "Apple M2".into(),
            probing: true,
            blocked: None,
        };
        assert!(interrupted().failure_for("Apple M2").is_some());
        assert!(interrupted().failure_for("NVIDIA RTX").is_none());
        assert!(
            SafetyRecord {
                fingerprint: "Apple M2".into(),
                probing: false,
                blocked: None
            }
            .failure_for("Apple M2")
            .is_none()
        );
        assert_eq!(
            SafetyRecord {
                fingerprint: "Apple M2".into(),
                probing: false,
                blocked: Some("allocation failed".into())
            }
            .failure_for("Apple M2"),
            Some("allocation failed".into())
        );
    }
}
