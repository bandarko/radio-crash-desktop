#!/bin/sh
# macOS/Linux cross-build; install MinGW-w64 and the Windows GNU Rust target first.
set -eu
cd "$(dirname "$0")/.."
export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
cargo build --release --locked --target x86_64-pc-windows-gnu
_rc_build_dir="${CARGO_TARGET_DIR:-target}"
mkdir -p dist/windows-x64
cp "$_rc_build_dir/x86_64-pc-windows-gnu/release/radio-crash-desktop.exe" dist/windows-x64/RadioCrashDesktop.exe

cp assets/*OFL.txt assets/*.prompt.txt UI-ASSETS.md DEPENDENCIES.md THIRD_PARTY_NOTICES.md README.md VERIFICATION.md RELEASE-NOTES.md INSTALL-Windows.md LICENSE dist/windows-x64/
cp -R licenses dist/windows-x64/
cp RELEASE-NOTES.md dist/windows-x64/RELEASE-NOTES.txt
cp INSTALL-Windows.md dist/windows-x64/INSTALL.txt
