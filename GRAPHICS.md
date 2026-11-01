# Adaptive receiver optics

The stable eframe 0.37 glow renderer paints the complete original receiver first. Optional `egui_glow::CallbackFn` passes then capture the already-rendered VFD and chassis. No audio analysis is performed on the graphics thread and no synthesized levels drive the effects. CPAL, decoded PCM, FFT, stereo detectors, holds and all input geometry retain their existing paths.

## Optical pipeline

1. A bounded RGBA8 capture contains the actual VFD text, bars, holds and clock at native device-pixel resolution.
2. Colored emission is extracted in linear light to quarter-resolution RGBA16F targets. A separable Gaussian blur creates optical scattering.
3. A glass composite adds restrained bloom, a small internal reflection and recessed-edge attenuation. The Day/Night brightness is part of emission extraction.
4. A separate full-panel pass adds fine directional metal sheen, faint real-signal spill at the glass lip and machined-knob highlights. Masks preserve the exact white logo, printed legend, existing HD badge, glass, button faces and transparent perimeter.

These are SDR optics. Internal floating-point textures are not an HDR swapchain. The application does not claim EDR/HDR presentation or change OS brightness, HDR, color profiles or GPU preferences. The original photograph remains the material base; there is no 3D scene, camera tracking or environment capture.

## Compatibility and recovery

- Automatic is conservative: recognized modern Apple M-series, NVIDIA GTX 10-series and newer/RTX, selected modern Quadro, Radeon RX/Pro and Intel Arc/Iris Xe hardware can be candidates.
- All candidates still require desktop OpenGL 4.1+, texture size at least 8192, two texture units, loaded core timer read functions, successful GLSL compilation and complete RGBA16F framebuffers. A name alone never guarantees initialization.
- Older/unclassified GPUs default to Classic. Software and virtual renderers cannot opt in. Hardware that meets hard limits but is not in the automatic list can explicitly select Enhanced.
- Classic schedules no custom callbacks. The baseline panel paint code and assets are unchanged from 1.2.0.
- Each capture is capped at 8.5 million pixels and the driver texture-size limit. The two captures plus bloom buffers have bounded memory; buffers are reused and replaced when physical dimensions change.
- Four asynchronous timer queries measure shader work without blocking on results. Sustained combined glass/material cost above 7.5 ms triggers Classic after warm-up. Stalled timing also triggers fallback. No timer read is allowed to block the audio callback.
- A local GPU-fingerprinted `graphics-state.json` marks an enhanced session in progress. Clean shutdown or a deliberate return to Classic clears the marker. A subsequent launch with an incomplete marker or recorded graphics failure retains Classic until the user selects Retry in Graphics details. The record is local and contains no telemetry destination.
- Shader/texture/framebuffer errors disable further enhanced passes. The next frame uses the original renderer. GL resources are released with a valid context during exit; no GL calls happen from a destructor after context teardown.
- The native Windows title bar, DPI manifest and logical-point input transform are unchanged.

The core OpenGL query read addresses are explicitly checked and loaded from eframe's current context. This avoids glow 0.17's incorrect GL 4.4 query-buffer capability test for GL 3.3 timer reads on macOS OpenGL 4.1. Only the documented core timer ABI is used; no third-party native library is bundled.

## Verification and recovery flags

`--classic-graphics` forces only the original renderer for troubleshooting. It cannot bypass compatibility checks. `--graphics-report /absolute/report.json` exports the actual capabilities, active/fallback state, completed passes and asynchronous shader time after six seconds. `--qa-exit-after 8` requests a normal clean shutdown after eight seconds for a capture run. Existing `RC_CONFIG_DIR` isolates QA settings/recovery records. `--capture` saves a native rendered frame. These flags are developer diagnostics, not controls on the receiver fascia.

Native runtime evidence and Classic pixel comparisons are recorded with release artifacts. Deterministic tests cover conservative device selection, refusal of software/low-limit devices, allocation bounds, time-budget hysteresis and settings migration. A successful Windows compile is not proof of NVIDIA/AMD driver behavior; physical Windows and Intel Mac runtime remain separate checks.
