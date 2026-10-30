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
