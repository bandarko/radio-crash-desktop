# Analog VU - Silver Aluminium

Optional skin in Settings > Receiver skin. VFD Digital - Black Aluminium is
still the default for fresh and upgraded configurations without a saved skin.
Switching skins does not restart playback. Skin selection persists.

The photographed substrate contains no needles, scales, display contents,
control icons or knob index. `src/analog.rs` paints these natively from current
state. Metadata supports Croatian characters and clipped long-text scrolling.
The glass uses the existing guarded enhanced optics pass when supported; the
VFD-specific full-fascia shader is intentionally not applied to silver metal.

## Fixed analog measurement

`src/dsp_analog.rs` rectifies each consumed output PCM channel independently,
then models a damped moving-coil instrument at the output sample clock. This
runs on the existing analysis worker, not in the CPAL callback or GUI thread.
There is no RMS-to-needle conversion, peak detector, peak-hold needle, automatic
gain adjustment or random movement. Volume/mute affect listening gain only.

The full-wave absolute input is sine calibrated by pi/(2*sqrt(2)); a steady
sine's RMS level is the indicated reference. For nonsinusoidal material the
indication intentionally differs from an RMS meter. Exact state transitions of
x'' + 2*zeta*w*x' + w²*x = w²*u use zeta=0.8127169864286775 and
w=13.51191256253271 rad/s: 99% deflection at 300 ms, 1.25% overshoot. Natural
return uses the same damping; there is no fast-decay option for this skin.

The angular scale is proportional to coil current, with logarithmically placed
dB labels ending at +3 VU. No display inflation or mapping +8 onto a +3 label.
The instrument stops at its visible endpoints. Default analog reference is
-8 dBFS sine RMS; Settings exposes only the separate Analog VU level slider.
This changes display sensitivity, never PCM gain. It does not overwrite the
digital skin's response, decay or reference. Tray readings use the selected
skin's detector too; tray artwork remains the existing compact L/R bars.

Design sources (not a certification claim):
- https://michaelfidler.com/articles/practical-vu-meter-circuits/
- https://hoytmeter.com/analog-panel-meters/vu-meters.html

Deterministic tests cover sine calibration at 44.1/48/96 kHz, 300 ms rise and
return, overshoot, stereo independence, pulse response, packet boundaries,
nonfinite sample safety and current-linear +3 scale placement. Separate egui
pointer tests cover both layouts at compact/full sizes and 100/125/150/200% DPI.
Native Windows pointer/GPU behavior still requires physical owner testing.

## Local owner preview

Build is 1.4.0, local review only, not a GitHub release. A Mac preview bundle
with `Contents/Resources/LOCAL-PREVIEW` uses separate `analog-preview` settings
under the normal config directory. It does not overwrite `/Applications` or
normal settings. `RC_CONFIG_DIR` remains available for isolated runtime QA.
Public packaging must omit this local marker. No public release upload, tag or
asset replacement until the owner explicitly approves the locally tested build.

## Calibration revision after owner review

The first local preview used -12 dBFS and a 110-degree elliptical projection;
that combination held hot programme material too high and visibly changed the
needle length across the sweep. It is superseded. The revised skin uses a
true circular 90-degree movement, a fixed-length shaft below the lower bezel,
and -8 dBFS sine-RMS reference (4 dB less display sensitivity). Audio is not
normalized or altered, and no automatic meter gain hides real level changes.
Owner must judge the revised movement locally before publication.
Visual comparison viewed: https://www.youtube.com/watch?v=kPy2RFPY0Yw
(Shinyas Studio, Sifam R22 vs AL-19). This is a visual reference, not a
calibration recording of our radio stream or a compliance certificate.
