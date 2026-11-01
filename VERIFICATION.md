# Verification - 1.1.2 window controls

- Egui interaction tests exercise all button centers and four inset corners at 560/780/1200 widths and letterboxed wide/tall custom sizes, with 100/125/150/200% native pixels per point. The same context also changes DPI and client size between clicks.
- The actual EXIT widget emits Close for the root viewport and does not emit Minimized/Visible(false). Clicking blank metal between controls does not activate a button. Existing screw interaction tests remain.
- Windows creates a decorated window from startup, avoiding the borderless OpenGL drawable/client offset reported in [egui issue 2770](https://github.com/emilk/egui/issues/2770). The symptoms match the owner's Windows 10 Pro/100% report, but this is not proof of the specific HP GPU cause. No arbitrary pointer offset is introduced.
- An embedded Windows manifest sets Per-Monitor V2 with Per-Monitor fallback before window creation. The Windows executable's manifest is inspected separately during packaging.
- Native Windows hardware/driver interaction remains unverified locally. Ask the owner to confirm centered clicks and EXIT/X on the affected HP. CI build/test results and Mac notarization evidence are recorded in the release manifest.

# Verification - 1.1.1 calibration patch

- Tests cover -11 dBFS -> +4 VU, -7 dBFS -> +8 VU, preserved scale/peak headroom, and one-time settings migration that preserves later explicit Digital and legacy Studio/custom choices.
- A 25-second muted CoreAudio run used actual decoded/consumed Radio Crash PCM. One-second snapshots gave median RMS L -11.28 / R -11.01 dBFS and sampled maxima L -7.59 / R -7.76 dBFS. The chosen fixed -15 reference maps those medians to +3.72 / +3.99 VU and sampled maxima to +7.41 / +7.24. Programme content and transients naturally vary. Audio output gain was zero only for this isolated smoke run.
- The main panel and both tray layouts receive the same reference. RMS integration and FFT normalization are unchanged from 1.1.0.
- Mac signing/notarization, package hashes and native CI results are recorded in the release manifest. Windows playback remains manually unverified.

# Verification - 1.1.0

- 24 deterministic tests pass, including calibrated RMS band levels, independent stereo, 50 ms RMS step/impulse response, all ten FFT tone bands, ETag/304 update checks and stable-version/URL handling.
- The upper-left screw receives primary-button events at widths 560, 640, 780 and 1200 in the actual egui interaction system, including a neighboring drag region. The earlier header context-menu target did not cover the screws.
- Native Apple Silicon receiver capture confirms SINCE 1986, digital VU labeling, corrected spectrum scale and the unchanged HD Radio illuminator.
- A simulated future 1.2.0 cache in an isolated RC_CONFIG_DIR produced the actual native update-notice window. This is a QA fixture, not a published 1.2.0 release. The real GitHub API probe reported current=1.1.0, latest=1.0.0 before publication.
- Native computer-use mouse injection into the borderless receiver returned noWindowsAvailable. Do not describe the egui pointer test as physical Mac pointer automation. Native keyboard close worked. The temporary QA app was closed; a later stale activation attempt produced a macOS not-open-anymore alert, not an application crash.
- Native 1.1 tray lifecycle passed: 54 hidden logic ticks and 29 changed icon uploads in six seconds; ON AIR, tray_only=true, independent RMS -9.47 / -8.91 dBFS; then restore and clean exit.
- New signing/notarization and CI are validated separately in the release manifest. Windows playback remains manually unverified.

The following section preserves the 1.0 release evidence and limits.

# Verification — current local build

Version 1.0.0, Mac build 101, released on 2026-10-30. The owner approved the appearance and requested the first release. The evidence below does not claim native Windows playback verification.

| Check | Observed evidence |
|---|---|
| Production integration | Existing Radio Crash source plus read-only website/STATS inspection; AAC-LC / 44.1 kHz / stereo independently confirmed |
| Actual playback | Network → Symphonia → CPAL/CoreAudio smoke runs; signed build 101 completed 45 seconds with 45 ON AIR observations, AAC / 44.1 kHz / stereo / about 254 kbps, independent L/R RMS, real FFT and artist/title. No output-device failure reconnects or output notifications occurred in that run |
| Reconnect | Local dropping HTTP server: AAC and MP3 each made three connections, recovered to ON AIR and exited cleanly |
| Deterministic tests | 18 passing tests cover stereo RMS, silence, 1 kHz FFT, antiphase stereo, Nyquist limits, resampling rate/DC, anti-alias rejection, retry bounds, stalled-server cancellation, fast VU response, relative scale/+18 endpoint, DSP peak retention/refresh, compact-config migration, window-size constraints and recoverable versus fatal output-device notifications |
| Static checks | cargo fmt and strict cargo clippy with warnings denied |
| Photographic native UI | Live renderer captures at 1000 × 367 and 640 × 235 logical points, including live metadata, actual meter levels and the new fascia/control surfaces |
| Saved window dimensions | The app wrote 640 / 234.66667 / aspect-lock=true to settings. Relaunched without an override; native app screenshot showed the restored 1280 × 470 Retina viewport, corresponding to the 640-point compact window |
| macOS package | Optimized universal arm64 + x86_64 app, build 101. Developer ID Application signature, hardened runtime and secure timestamp verified. Apple notarization succeeded; stapled ticket validation passed; Gatekeeper assessment returned accepted / Notarized Developer ID |
| Windows package | Windows x64 GNU cross-build, icon/version resources and imported system DLLs inspected. Native Windows MSVC CI is configured in `.github/workflows/build.yml`; CI results are separate from manual playback/hardware verification |

The current default is 780 × 286 logical points. The GUI contains actual width/height inputs, aspect preference, four size presets, header-menu presets and native egui drag regions on the right/bottom edges and lower-right corner. Dimensions are saved after a short idle period and at exit. A size-only Apply preserves the existing audio connection.

The receiver's metal, glass, controls and knob are an embedded photographic material. The final 780 × 286 native capture is 1560 × 572 Retina pixels with an alpha range of 0–255, verifying the transparent exterior. Resize hit zones follow the visible metal rim instead of the transparent canvas. Live phosphor, status, track text, measured properties, VU, peak markers, spectrum, power LED, mute/transport state and volume pointer remain native egui painting. No placeholder meter animation is used. Original generation mode, source asset and exact prompt are documented in UI-ASSETS.md.

## Remaining manual / release gates

- Appearance accepted by the owner; extended listening on additional output hardware remains useful.
- Full pointer exercise of resize drag, controls, exact dimension entry, presets, free-aspect mode, output switching and tray restore. Native screenshot inspection and size persistence were verified; computer-use mouse injection returned `noWindowsAvailable`, so this run does not claim end-to-end pointer automation.
- Native Windows 10/11 playback, DPI/resizing, WASAPI device changes, tray and shutdown tests; physical Intel Mac runtime verification.
- Long-duration playback, genuine network outages, DAC unplug/replug and additional mono/low-rate/malformed-metadata stations.
- Mac direct-download signing and notarization are complete. Windows Authenticode signing is not configured; the Windows download is unsigned.

Early development smoke playback encountered a recoverable CoreAudio underrun event. A release-validation run exposed that all CPAL notifications incorrectly requested a device reconnect. Build 101 fixes this: Xrun, RealtimeDenied and an automatically followed DeviceChanged keep the stream alive; fatal device/stream errors still request a restart. A regression test also ensures a later harmless notification cannot clear an already pending fatal restart. Short smoke runs do not prove long-duration glitch-free playback.

## Meter semantics and scope

The VU is a relative display: 0 dB reference is −18 dBFS, and full-scale sample peaks map to +18. The spectrum remains dBFS. Meters measure pre-volume consumed output PCM. A 256-frame DSP path holds peaks for 1.5 seconds and then decays 12 dB/s, retaining transients between GUI frames. RMS attack remains immediate. No audio gain is added by this calibration.

AAC-LC and MP3 are supported. HE-AAC/SBR, HLS, authentication and ICY metadata-block stripping are outside this version. Radio Crash's verified AAC-LC source uses the configured documented JSON endpoint for track metadata. No production SAM/PAL configuration was modified.

## Official logo and HD Radio emblem

The owner-supplied horizontal PNG is embedded unchanged; the renderer draws only its compact white letterforms, excluding the antenna and tagline, with transparent black coverage directly on the aluminium. The invented serif branding was removed from the photographic fascia. A new unlit photographed lens between Play and Mute hosts cold-white HD RADIO lettering in the stylized Audiowide face; soft illumination follows receiver power and full illumination follows the existing quality threshold. No audio, DSP, peak-hold or window-size code was changed for this branding update.

## Live tray stereo meters

A native macOS runtime test hid the receiver at two seconds, switched NSApplication to Accessory activation policy (no Dock icon), continued audio/metadata/DSP and updated the real status-item image while no receiver UI was painted. At eight seconds it recorded 53 hidden logic ticks, 49 changed native icon uploads, independent RMS values −8.62 / −9.67 dBFS, ON AIR status and tray_only=true. The test then restored the normal activation policy and receiver, produced a live receiver capture and exited cleanly. Native pointer click automation remains unavailable; the restore click/menu code uses the documented tray-icon event API.

Two additional deterministic tests check silent/loud frame generation and verify that left-only synthetic PCM alters only the L region in the wide Mac icon and only the left bar region in the square Windows icon. Windows code is cross-built but not run on Windows. No random tray animation, new worker thread or fake level generator was introduced.
