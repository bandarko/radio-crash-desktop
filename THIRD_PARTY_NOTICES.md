# Third-party notices

Audiowide font: Copyright (c) 2012 Brian J. Bonislawsky DBA Astigmatic, SIL Open Font License 1.1, Reserved Font Name Audiowide. Full notice: `assets/Audiowide-OFL.txt`.

Rajdhani font: Copyright (c) 2014 Indian Type Foundry, SIL Open Font License 1.1. Full terms: `assets/Rajdhani-OFL.txt`.

Michroma font: Copyright 2011 The Michroma Project Authors, SIL Open Font License 1.1. Full notice and terms: `assets/Michroma-OFL.txt`.

Doto font: Copyright 2024 The Doto Project Authors, SIL Open Font License 1.1. Full notice and terms: `assets/Doto-OFL.txt`.

This project builds third-party Rust libraries from the pinned Cargo.lock; no external codec binaries are bundled. Their license declarations and source locations are available with:

```sh
cargo metadata --locked --format-version 1 > dependency-metadata.json
```

Major libraries: eframe/egui (MIT OR Apache-2.0), CPAL (Apache-2.0), Symphonia (MPL-2.0), rustfft (MIT OR Apache-2.0), reqwest (MIT OR Apache-2.0), Tokio (MIT), serde (MIT OR Apache-2.0), tray-icon/muda (MIT OR Apache-2.0), rustls (Apache-2.0 OR ISC OR MIT).

The resolved dependency inventory is in `DEPENDENCIES.md`; available upstream license and notice files are copied under `licenses/`. The lockfile and inventory include target-specific dependencies as well as the dependencies used by each shipped executable. Codec patent/licensing obligations depend on jurisdiction and distribution context.

Source code for this application and the exact dependency lockfile are available at https://github.com/bandarko/radio-crash-desktop . Symphonia and its codec crates are used unmodified under MPL-2.0; their corresponding source locations and versions are listed in `DEPENDENCIES.md` and are retrievable from crates.io using `Cargo.lock`. The MPL-2.0 license is included in `licenses/`.
