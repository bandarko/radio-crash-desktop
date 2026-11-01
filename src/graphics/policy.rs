use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Quality {
    Classic,
    Enhanced,
    #[default]
    #[serde(other)]
    Auto,
}
impl Quality {
    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Automatic (recommended)",
            Self::Classic => "Classic · original appearance",
            Self::Enhanced => "Enhanced · GPU glass and lighting",
        }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Capabilities {
    pub vendor: String,
    pub renderer: String,
    pub version: String,
    pub max_texture_size: i32,
    pub texture_units: i32,
    pub supported: bool,
    pub automatic_candidate: bool,
    pub reason: String,
}
impl Capabilities {
    pub fn classify(
        vendor: String,
        renderer: String,
        version: String,
        max_texture_size: i32,
        texture_units: i32,
    ) -> Self {
        let name = format!("{vendor} {renderer}").to_ascii_lowercase();
        let software = [
            "software",
            "llvmpipe",
            "softpipe",
            "swiftshader",
            "microsoft",
            "gdi generic",
            "virtualbox",
            "vmware",
            "virgl",
            "parallels",
        ]
        .iter()
        .any(|v| name.contains(v));
        let gl_version = version.split_whitespace().next().unwrap_or("");
        let mut numbers = gl_version.split('.').filter_map(|v| v.parse::<u32>().ok());
        let desktop_gl = (numbers.next().unwrap_or(0), numbers.next().unwrap_or(0)) >= (4, 1);
        let supported = !software && desktop_gl && max_texture_size >= 8192 && texture_units >= 2;
        // Conservative automatic policy. Older/unknown devices retain 1.2.0;
        // supported but unclassified devices can be explicitly opted in.
        let modern_nvidia = name.contains("nvidia")
            && (name.contains("rtx")
                || name.contains("quadro p")
                || name.contains("quadro a")
                || name
                    .split("gtx")
                    .nth(1)
                    .and_then(|s| s.split_whitespace().next())
                    .and_then(|s| s.parse::<u32>().ok())
                    .is_some_and(|n| n >= 1000));
        let modern = name.contains("apple m")
            || modern_nvidia
            || name.contains("radeon rx")
            || name.contains("radeon pro")
            || name.contains("intel") && (name.contains("arc") || name.contains("iris xe"));
        let reason = if software {
            "Software or virtual graphics: original appearance retained."
        } else if !supported {
            "Required GPU capabilities are unavailable: original appearance retained."
        } else if !modern {
            "Older or unclassified GPU: automatic mode retains the original appearance."
        } else {
            "GPU capabilities passed. Enhanced optics will be checked during rendering."
        }
        .to_owned();
        Self {
            vendor,
            renderer,
            version,
            max_texture_size,
            texture_units,
            supported,
            automatic_candidate: supported && modern,
            reason,
        }
    }
    pub fn requested(&self, quality: Quality) -> bool {
        self.supported
            && match quality {
                Quality::Classic => false,
                Quality::Auto => self.automatic_candidate,
                Quality::Enhanced => true,
            }
    }
    pub fn fingerprint(&self) -> String {
        format!("{}|{}|{}", self.vendor, self.renderer, self.version)
    }
}

pub const MAX_CAPTURE_PIXELS: u64 = 8_500_000;
pub fn within_budget(width: u32, height: u32, max_texture: i32) -> bool {
    width > 0
        && height > 0
        && width <= max_texture.max(0) as u32
        && height <= max_texture.max(0) as u32
        && width as u64 * height as u64 <= MAX_CAPTURE_PIXELS
}

#[derive(Default)]
pub struct Performance {
    samples: u32,
    slow: u32,
}
impl Performance {
    pub fn observe(&mut self, milliseconds: f32) -> bool {
        if !milliseconds.is_finite() || milliseconds < 0. {
            return false;
        }
        self.samples += 1;
        if self.samples <= 8 {
            return false;
        }
        self.slow = if milliseconds > 7.5 {
            self.slow + 1
        } else {
            self.slow.saturating_sub(2)
        };
        self.slow >= 12
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn caps(vendor: &str, name: &str, version: &str) -> Capabilities {
        Capabilities::classify(vendor.into(), name.into(), version.into(), 16384, 16)
    }
    #[test]
    fn modern_devices_are_candidates_and_legacy_devices_stay_classic() {
        for (vendor, name) in [
            ("Apple", "Apple M2 Pro"),
            ("NVIDIA Corporation", "NVIDIA GeForce RTX 4070"),
            ("NVIDIA Corporation", "GeForce GTX 1060"),
            ("ATI Technologies Inc.", "AMD Radeon RX 6800"),
            ("Intel", "Intel Arc A770"),
        ] {
            assert!(
                caps(vendor, name, "4.6 Core").requested(Quality::Auto),
                "{name}"
            );
        }
        for (vendor, name) in [
            ("Intel", "Intel HD Graphics 4000"),
            ("NVIDIA Corporation", "GeForce GTX 750 Ti"),
            ("Unknown", "GPU 1000"),
        ] {
            let c = caps(vendor, name, "4.6 Core");
            assert!(!c.requested(Quality::Auto));
            assert!(c.requested(Quality::Enhanced));
        }
        assert!(!caps("Apple", "Apple M2 Pro", "4.1 Metal").requested(Quality::Classic));
    }
    #[test]
    fn software_low_limits_and_shader_versions_cannot_be_opted_in() {
        for name in [
            "llvmpipe",
            "Microsoft Basic Render Driver",
            "VMware SVGA",
            "Apple Software Renderer",
        ] {
            assert!(!caps("test", name, "4.6").requested(Quality::Enhanced));
        }
        assert!(!caps("NVIDIA", "GeForce RTX 4070", "3.3").supported);
        assert!(!caps("NVIDIA", "GeForce RTX 4070", "OpenGL ES 3.2").supported);
        assert!(
            !Capabilities::classify("Apple".into(), "Apple M2".into(), "4.1".into(), 4096, 16)
                .supported
        );
    }
    #[test]
    fn allocation_and_sustained_gpu_budget_protect_the_original_renderer() {
        assert!(within_budget(2400, 880, 16384));
        assert!(within_budget(3840, 1408, 16384));
        assert!(!within_budget(7680, 2816, 16384));
        assert!(!within_budget(u32::MAX, u32::MAX, 16384));
        let mut p = Performance::default();
        for _ in 0..100 {
            assert!(!p.observe(1.5));
        }
        for _ in 0..11 {
            assert!(!p.observe(10.));
        }
        assert!(p.observe(10.));
    }
}
