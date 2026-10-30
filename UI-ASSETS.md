# Photographic front-panel asset

- Tool: built-in `image_gen.imagegen`, product-mockup generation mode, followed by background-extraction edit mode.
- Runtime asset: `assets/receiver-photographic-v3-cutout.png` (2071 × 759 RGBA pixels). The opaque generation source is retained as `assets/receiver-photographic-v3.png`.
- Prompts: `assets/receiver-photographic-v3.prompt.txt` and `assets/receiver-photographic-v3-cutout.prompt.txt`.
- Original generation result was copied into the project. It is not loaded from the Codex cache at runtime.
- The image is the unlit physical fascia: metal, glass, printed branding, button caps and knob surfaces. The display, PCM-driven meters, peak markers, transport symbol, power LED, mute state and volume pointer are painted live by egui. No measured values or lit meter animations are baked into the asset.
- The native renderer uploads two lossless horizontal tiles, preserving the source pixels while staying below egui's initial 2048-pixel per-texture limit. The generated source asset remains intact.
- Phosphor bloom and a subtle optical reflection layer are painted in the native GUI; source samples, RMS measurement, FFT and peak hold are unchanged.

The supplied receiver image informed the material direction. The finished fascia uses only the five functional controls for this application and original Radio Crash branding.

The edited asset has alpha outside the chassis and feet. The native root window and framebuffer are transparent, so the surrounding desktop remains visible; the separate Settings viewport stays opaque.

## Station branding and illuminated quality emblem

- Current runtime fascia: `assets/receiver-photographic-v4-brand.png` (2070 × 760 RGBA pixels). It supersedes v3 at runtime; earlier sources remain for provenance.
- Tool: built-in imagegen, precise-object-edit mode. Exact edit prompt: `assets/receiver-photographic-v4-brand.prompt.txt`.
- The photographic edit clears only the former invented wordmark and adds an empty, unlit recessed badge housing between Play and Mute. All lettering in these areas is painted natively.
- Exact station logo: `assets/radio-crash-logo-user.png`, the owner's horizontal 960 × 264 PNG with antenna, supplied in this chat. The matching public identity was verified on the current Radio Crash website; a reference download is retained as `assets/radio-crash-official-logo.jpg` from https://www.radiocrash.net/wp-content/uploads/2024/10/radio-crash-white-on-black-960x264-960-1.jpg .
- The supplied logo is embedded unchanged. The renderer interprets black as transparent coverage and draws a compact crop of the original white letterforms directly onto the metal, excluding the antenna and tagline. It neither substitutes a font nor asks imagegen to recreate the logo.
- HD Radio uses an original native text treatment in its photographed lens. It has a soft steady cold-white/cyan backlight while the receiver is powered on, full white/cyan illumination when the configured bitrate threshold is met, and an unlit appearance when powered off. It is a Radio Crash stream-quality emblem, not a control or an HD Radio/IBOC broadcast-decoder claim.

The HD RADIO emblem uses one consistent stylized Audiowide face, with its SIL OFL license in `assets/Audiowide-OFL.txt`. It is not a mixture of two typefaces. The supplied logo bitmap remains unchanged on disk; native coverage excludes the antenna, and black becomes transparent so only small white station lettering appears on aluminium.

The L/R tray icon is generated deterministically in native Rust code (`tray_meter.rs`), not imagegen. Its live bars and peak caps are driven exclusively by the output-PCM DSP snapshot.
