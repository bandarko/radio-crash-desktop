# Peak-hold reference and Radio Crash semantics

The hardware comparison was requested by the owner. It is a technical reference, not product branding or a claim that the Radio Crash player reproduces a cassette mechanism.

## Original sources

- [Technics RS-B100 operating instructions, printed page 15](https://manuals.plus/m/4f6b9732f0ec362a8bb33f46bb3972c81b477eafb6a294cd3d818315056c2f5b.pdf): the manufacturer describes a peak-reading fluorescent display, a roughly two-second auto-reset hold and three indication colors. The scale spans -40 to +18 dB. These are analogue instrument reference levels, not digital dBFS.
- [Original hardware display photograph by Retired electrician, CC0](https://commons.wikimedia.org/wiki/File:Technics_RS-B100_level_display.jpg): the display uses filament segments in the L/R rows, with the extended range associated with the unit's dbx operation. The photo is a comparison source; it is not bundled into the app.
- [Howard A. Roberson, Audio, March 1985, pages 97-98](https://manuals.plus/m/89adc14b417557b97bef180f9b8c8da019fe1d1da62e5c4f9549b8dfae610de3.pdf): measured full indication took about 30 ms and a 20 dB bar decay took 320 ms. The tested unit's hold varied, often nearer one second than the claimed two. These measurements concern one physical sample; they do not establish a universal digital timing algorithm.

## Version 1.1.5 RMS hold (retained as Classic RMS hold)

Radio Crash retains its approved 50 ms RMS VU and -15 dBFS reference. A programme RMS of -11 dBFS still reads +4 VU. It does not silently replace those bars with a faster analogue peak-reading detector or add gain to make an indicator reach a target.

Previously, the bar showed RMS while a floating marker held a raw sample maximum for 1.5 seconds and then decayed at 12 dB/s. That is a valid kind of hybrid RMS/sample-peak display, but it does not represent the maximum of the same detector as the bar, nor the documented auto-reset behavior.

Version 1.1.5 holds the maximum of the continuously integrated RMS envelope, including maxima between GUI frames. Each channel holds a new/equal maximum for two seconds of consumed-audio time and then resets to its current meter level. The marker is an ordinary lit segment in its own row and zone color. It uses the same lower threshold as the bar, including when a value lies between segment marks. The miniature tray layout uses the same held data with consistent coarse quantization.

Raw sample peaks remain separately available in smoke diagnostics. They are not used to move the RMS hold marker. The result reproduces the documented hold/reset principle while preserving Radio Crash's RMS instrument; it does not claim a component-level or metrological emulation of the entire original peak meter. No dbx, tape, calibration oscillator or third-party model controls are added.

## Version 1.2.0 advanced measurements, approved RMS front panel

The main instrument and tray retain the 1.1.5 RMS hold. The front panel keeps the same 50 ms RMS bars, maximum-RMS two-second hold, full in-row two-filament segment, zone colors, lower-threshold quantization and PEAK HOLD caption. There is no external pointer, dot or thin lane above the bar. No detector gain or artificial peak target is used.

True Peak and momentary/short-term LUFS are independent measurements in Settings > Live signal measurements. They use the pure-Rust [ebur128 0.1.11 implementation](https://docs.rs/ebur128/0.1.11/ebur128/), a port of libebur128. Its authors document EBU TECH 3341/3342 tests; this product does not claim independent certification. See [ITU-R BS.1770](https://www.itu.int/rec/R-REC-BS.1770-5-202311-I/en) and [EBU loudness specifications](https://tech.ebu.ch/loudness/). The crate oversamples 4x below 96 kHz, 2x below 192 kHz and uses native samples above that. The stream is analyzed at the output-device rate after resampling, before volume; device-rate changes reconnect and reset the meter state.

Absolute True Peak is in dBTP and has its own two-second held value, independent of the visible RMS hold. RMS -11 dBFS and True Peak 0 dBTP can both be correct. At the default -15 reference they would be +4 and +15 on a common scale, so a large gap is expected when comparing these different detectors. The app keeps them in distinct views rather than altering their values. Red segments on the relative Hi-Fi RMS scale do not prove clipping or audible distortion. True Peak estimates source headroom; playback gain may provide further headroom.

Only momentary/short-term modes are enabled: a fixed 3 s history, no indefinitely growing integrated history and no normalization. LUFS warm-up is 400 ms / 3 s, queried at about 10 Hz. Silence and filter-tail readings below -100 LUFS are unavailable. True Peak updates per 256 consumed frames on the analysis worker. The original analogue peak-meter comparison remains historical context, not a claim of circuitry emulation.
