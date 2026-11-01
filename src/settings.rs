use serde::{Deserialize, Serialize};
use std::path::PathBuf;
fn radio_stream_url() -> String {
    "https://live.radiocrash.net/".into()
}
fn radio_metadata_url() -> String {
    "https://live.radiocrash.net/stats?json=1&sid=1".into()
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    #[serde(skip, default = "radio_stream_url")]
    pub stream_url: String,
    #[serde(skip, default = "radio_metadata_url")]
    pub metadata_url: String,
    pub device: String,
    pub volume: f32,
    pub autoplay: bool,
    pub start_minimized: bool,
    pub minimize_to_tray: bool,
    pub spectrum: bool,
    pub fast_vu: bool,
    pub vu_response: crate::dsp::VuResponse,
    pub vu_reference_dbfs: f32,
    pub vu_calibration_version: u8,
    pub hd_kbps: u32,
    pub window_width: f32,
    pub window_height: f32,
    pub lock_aspect: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            stream_url: radio_stream_url(),
            metadata_url: radio_metadata_url(),
            device: String::new(),
            volume: 0.5,
            autoplay: false,
            start_minimized: false,
            minimize_to_tray: false,
            spectrum: true,
            fast_vu: true,
            vu_response: crate::dsp::VuResponse::FastRms,
            vu_reference_dbfs: crate::dsp::VU_REFERENCE_DBFS,
            vu_calibration_version: 1,
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
            .and_then(|b| Self::decode(&b))
            .unwrap_or_default();
        s.volume = s.volume.clamp(0., 1.);
        s.vu_reference_dbfs = if s.vu_reference_dbfs.is_finite() {
            s.vu_reference_dbfs.clamp(-24., 0.)
        } else {
            crate::dsp::VU_REFERENCE_DBFS
        };
        let size = window_size(s.window_width, s.window_height, s.lock_aspect);
        s.window_width = size[0];
        s.window_height = size[1];
        s
    }
    fn decode(bytes: &[u8]) -> Option<Self> {
        let value: serde_json::Value = serde_json::from_slice(bytes).ok()?;
        let old_calibration = value["vu_calibration_version"].as_u64().unwrap_or(0) < 1;
        let legacy_response = value.get("vu_response").is_none();
        let legacy_fast = value["fast_vu"].as_bool().unwrap_or(false);
        let mut s: Self = serde_json::from_value(value).ok()?;
        if legacy_response {
            s.vu_response = if legacy_fast {
                crate::dsp::VuResponse::FastRms
            } else {
                crate::dsp::VuResponse::SmoothRms
            };
        }
        // One-time correction of 1.1.0's default digital profile. Once saved,
        // an explicit choice of Digital remains Digital on later launches.
        if old_calibration && s.vu_reference_dbfs == 0. {
            s.vu_reference_dbfs = crate::dsp::VU_REFERENCE_DBFS;
        }
        s.vu_calibration_version = s.vu_calibration_version.max(1);
        Some(s)
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
    fn legacy_urls_cannot_reroute_the_radio_crash_product() {
        let settings=Settings::decode(br#"{"stream_url":"http://127.0.0.1/other","metadata_url":"","volume":0.3,"vu_response":"peak"}"#).unwrap();
        assert_eq!(settings.stream_url, "https://live.radiocrash.net/");
        assert_eq!(
            settings.metadata_url,
            "https://live.radiocrash.net/stats?json=1&sid=1"
        );
        assert_eq!(settings.volume, 0.3);
        assert_eq!(settings.vu_response, crate::dsp::VuResponse::Peak);
        let saved = serde_json::to_value(&settings).unwrap();
        assert!(saved.get("stream_url").is_none());
        assert!(saved.get("metadata_url").is_none());
    }
    #[test]
    fn old_settings_get_compact_window_defaults() {
        let s: Settings = serde_json::from_str(r#"{"volume":0.35,"hd_kbps":256}"#).unwrap();
        assert_eq!(s.volume, 0.35);
        assert_eq!(s.window_width, 780.);
        assert!(s.lock_aspect);
    }
    #[test]
    fn measurement_upgrade_preserves_user_choices() {
        let upgraded = Settings::decode(
            br#"{"fast_vu":true,"vu_reference_dbfs":-13,"vu_calibration_version":1}"#,
        )
        .unwrap();
        assert!(upgraded.fast_vu);
        assert_eq!(upgraded.vu_response, crate::dsp::VuResponse::FastRms);
        assert_eq!(upgraded.vu_reference_dbfs, -13.);
        let classic = Settings::decode(br#"{"fast_vu":true,"vu_reference_dbfs":-18}"#).unwrap();
        let saved = serde_json::to_vec(&classic).unwrap();
        let restored = Settings::decode(&saved).unwrap();
        assert!(restored.fast_vu);
        assert_eq!(restored.vu_reference_dbfs, -18.);
    }
    #[test]
    fn response_profiles_persist_and_migrate_legacy_decay_choices() {
        use crate::dsp::VuResponse;
        assert_eq!(Settings::default().vu_response, VuResponse::FastRms);
        let smooth =
            Settings::decode(br#"{"fast_vu":false,"volume":0.3,"auto_updates":false}"#).unwrap();
        assert_eq!(smooth.vu_response, VuResponse::SmoothRms);
        for response in [VuResponse::SmoothRms, VuResponse::FastRms, VuResponse::Peak] {
            let selected = Settings {
                vu_response: response,
                ..smooth.clone()
            };
            let restored = Settings::decode(&serde_json::to_vec(&selected).unwrap()).unwrap();
            assert_eq!(restored.vu_response, response);
            assert_eq!(restored.volume, 0.3);
            assert!(!restored.fast_vu);
        }
        let future = Settings::decode(br#"{"vu_response":"future","volume":0.3}"#).unwrap();
        assert_eq!(future.vu_response, VuResponse::FastRms);
        assert_eq!(future.volume, 0.3);
    }
    #[test]
    fn hifi_migration_is_once_and_preserves_explicit_profiles() {
        let old = Settings::decode(br#"{"vu_reference_dbfs":0,"volume":0.4}"#).unwrap();
        assert_eq!(old.vu_reference_dbfs, -15.);
        assert_eq!(old.volume, 0.4);
        let digital =
            Settings::decode(br#"{"vu_reference_dbfs":0,"vu_calibration_version":1}"#).unwrap();
        assert_eq!(digital.vu_reference_dbfs, 0.);
        let studio = Settings::decode(br#"{"vu_reference_dbfs":-18}"#).unwrap();
        assert_eq!(studio.vu_reference_dbfs, -18.);
        let custom = Settings::decode(br#"{"vu_reference_dbfs":-13}"#).unwrap();
        assert_eq!(custom.vu_reference_dbfs, -13.);
        let saved = serde_json::to_vec(&old).unwrap();
        assert_eq!(Settings::decode(&saved).unwrap().vu_reference_dbfs, -15.);
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
