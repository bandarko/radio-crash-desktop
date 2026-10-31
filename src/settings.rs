use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub stream_url: String,
    pub metadata_url: String,
    pub device: String,
    pub volume: f32,
    pub autoplay: bool,
    pub start_minimized: bool,
    pub minimize_to_tray: bool,
    pub spectrum: bool,
    pub fast_vu: bool,
    pub vu_reference_dbfs: f32,
    pub auto_updates: bool,
    pub hd_kbps: u32,
    pub window_width: f32,
    pub window_height: f32,
    pub lock_aspect: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            stream_url: "https://live.radiocrash.net/".into(),
            metadata_url: "https://live.radiocrash.net/stats?json=1&sid=1".into(),
            device: String::new(),
            volume: 0.5,
            autoplay: false,
            start_minimized: false,
            minimize_to_tray: false,
            spectrum: true,
            fast_vu: false,
            vu_reference_dbfs: 0.,
            auto_updates: true,
            hd_kbps: 192,
            window_width: 780.,
            window_height: 286.,
            lock_aspect: true,
        }
    }
}
impl Settings {
    pub fn path() -> PathBuf {
        // Isolated configuration for QA/portable launches, never affects the
        // normal installed app unless explicitly set by the launching process.
        if let Some(path) = std::env::var_os("RC_CONFIG_DIR")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
        {
            return path.join("settings.json");
        }
        directories::ProjectDirs::from("net", "RadioCrash", "RadioCrashDesktop")
            .expect("configuration directory")
            .config_dir()
            .join("settings.json")
    }
    pub fn load() -> Self {
        let mut s: Self = std::fs::read(Self::path())
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        s.volume = s.volume.clamp(0., 1.);
        s.vu_reference_dbfs = if s.vu_reference_dbfs.is_finite() {
            s.vu_reference_dbfs.clamp(-24., 0.)
        } else {
            0.
        };
        let size = window_size(s.window_width, s.window_height, s.lock_aspect);
        s.window_width = size[0];
        s.window_height = size[1];
        s
    }
    pub fn save(&self) -> anyhow::Result<()> {
        let p = Self::path();
        std::fs::create_dir_all(p.parent().unwrap())?;
        let tmp = p.with_extension("tmp");
        std::fs::write(&tmp, serde_json::to_vec_pretty(self)?)?;
        if cfg!(windows) && p.exists() {
            std::fs::remove_file(&p)?;
        }
        std::fs::rename(tmp, p)?;
        Ok(())
    }
}

/// Logical desktop points. Material geometry always preserves the physical front-panel ratio.
pub const PANEL_ASPECT: f32 = 1200. / 440.;
pub fn window_size(width: f32, height: f32, lock: bool) -> [f32; 2] {
    let w = if width.is_finite() {
        width.clamp(560., 1920.)
    } else {
        780.
    };
    let h = if height.is_finite() {
        height.clamp(206., 1080.)
    } else {
        286.
    };
    if lock { [w, w / PANEL_ASPECT] } else { [w, h] }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn old_settings_get_compact_window_defaults() {
        let s: Settings = serde_json::from_str(r#"{"volume":0.35,"hd_kbps":256}"#).unwrap();
        assert_eq!(s.volume, 0.35);
        assert_eq!(s.window_width, 780.);
        assert!(s.lock_aspect);
    }
    #[test]
    fn custom_dimensions_and_ratio() {
        assert_eq!(window_size(640., 300., false), [640., 300.]);
        let s = window_size(640., 300., true);
        assert!((s[0] / s[1] - PANEL_ASPECT).abs() < 0.0001);
        assert_eq!(window_size(0., 0., false), [560., 206.]);
        assert_eq!(window_size(f32::NAN, f32::INFINITY, false), [780., 286.]);
        let s = window_size(10000., 1., true);
        assert_eq!(s[0], 1920.);
        assert!(s[1] > 700.);
    }
}
