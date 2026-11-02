#!/bin/sh
# macOS/Linux cross-build; install MinGW-w64 and the Windows GNU Rust target first.
set -eu
cd "$(dirname "$0")/.."
export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
cargo build --release --locked --target x86_64-pc-windows-gnu
_rc_build_dir="${CARGO_TARGET_DIR:-target}"
python3 scripts/download_payload.py windows "$_rc_build_dir/x86_64-pc-windows-gnu/release/radio-crash-desktop.exe"
