#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
scripts/package-macos.sh aarch64-apple-darwin
scripts/package-macos.sh x86_64-apple-darwin
_rc_build_dir="${CARGO_TARGET_DIR:-target}"
APP='dist/universal/Radio Crash Desktop.app'
mkdir -p "$APP"
cp -R 'dist/aarch64-apple-darwin/Radio Crash Desktop.app/Contents' "$APP/"
lipo -create "$_rc_build_dir/aarch64-apple-darwin/release/radio-crash-desktop" "$_rc_build_dir/x86_64-apple-darwin/release/radio-crash-desktop" -output "$APP/Contents/MacOS/RadioCrashDesktop"
sh scripts/sign-macos.sh "$APP"
echo "$APP"
