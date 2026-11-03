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

## Since 1986 inscription

The original v4 fascia and HD Radio lens remain unchanged. Built-in imagegen edited only the header year in `assets/receiver-header-since-1986-source.png`; its exact prompt is saved alongside it. `Materials` composites only the small year region from that image. The rest of the full generated image is not used at runtime. The engraving experiment was abandoned at the owner's request and is not included in the app or source assets.

## Optional silver analog skin (1.4.0)

- Material substrate: `assets/receiver-analog-silver-substrate.png`, built-in imagegen precise edits from the owner-approved concept. Exact prompt set and generated-source identifiers are in `assets/receiver-analog-silver-substrate.prompt.txt`.
- Natural silver aluminium, transparent exterior, blank amber ivory dial cards, small empty smoked-glass window, left PLAY/STOP, large central volume knob, right MUTE and SETTINGS without an LED.
- The revised substrate hides the needle shafts below the lower bezel. It contains no needle, shadow, scale, metadata, knob index or indicator. Rust paints every live instrument element.
- The native needle has constant length and an actual circular 90-degree sweep. The earlier elliptical first preview is superseded after owner feedback.
- Rajdhani-SemiBold (already bundled and SIL OFL licensed) draws the continuous amber artist/title lettering, with fallback for Croatian characters. No new font dependency.


## Analog material refinements (1.4.1 local preview)

- Tool: built-in imagegen, precise-object-edit, transparent background. All project assets are embedded locally.
- Silver refinement: `assets/receiver-analog-silver-v2-substrate.png`, from the prior production silver substrate. Prompt: `assets/receiver-analog-silver-v2-substrate.prompt.txt`. Generated source: `exec-1828c46f-05f0-4328-a96c-7ea9b6f96e17.png`.
- Third skin: `assets/receiver-analog-ivory-glass-substrate.png`, from the owner's approved shared-glass concept with continuous ivory centre. Prompt: `assets/receiver-analog-ivory-glass-substrate.prompt.txt`. Generated source: `exec-8ae07484-602d-4282-b6f8-3a130eb5703f.png`.
- Both source images use approximately 30:11 proportions matching the native panel. Circular knob targets and live indices share the same uniform panel transform. Lossless two-tile texture upload avoids exceeding egui's initial 2048 texture limit.
- Only materials and fixed exterior labels are photographic. Every scale, needle, metadata field, standby clock, button icon and volume index is painted live. There is no baked track, meter reading or simulated animation.
- The third skin's now-playing text is dark ink directly on the illuminated shared backplate, without a separate black rectangle. Enhanced optics cover the shared aperture rather than the VFD fascia shader.

Owner refinement: both silver SETTINGS caps remain bare metal. Their functional hit targets and exterior labels are unchanged; no gear or LED is drawn. VFD controls retain their existing symbols.


## Exact station lettering on silver skins

Both analog skins draw the unchanged owner-supplied `radio-crash-logo-user.png`
using the same crop/coverage as the black VFD skin, with dark ink tint. No font
substitute or generated letter shapes are used. The original letters were removed
from the photographic substrates using built-in imagegen. No rectangular cover,
mesh patch, label background or separate material is drawn underneath the logo.

- Silver runtime material: `assets/receiver-analog-silver-v3-bare-metal.png`.
  Exact edit prompt: `assets/receiver-analog-silver-v3-bare-metal.prompt.txt`.
  Generated source: `exec-64fccf2b-eb8a-44bf-8477-f942574e11b3.png`.
- Ivory Glass runtime material: `assets/receiver-analog-ivory-glass-v2-bare-metal.png`.
  Exact edit prompt: `assets/receiver-analog-ivory-glass-v2-bare-metal.prompt.txt`.
  Generated source: `exec-ee090682-6264-4d45-b10c-c98deb4eadd6.png`.

Only the native exact logo pixels appear on the continuous photographic metal.
The earlier sampled header patch was rejected by the owner and removed entirely.
