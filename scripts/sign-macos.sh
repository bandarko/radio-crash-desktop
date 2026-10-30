#!/bin/sh
# Development packages default to ad-hoc. Public releases require Developer ID.
set -eu
APP="${1:?Usage: sign-macos.sh /path/to/Application.app}"
if [ -n "${RC_SIGN_IDENTITY:-}" ]; then
    codesign --force --options runtime --timestamp --sign "$RC_SIGN_IDENTITY" "$APP"
else
    codesign --force --sign - "$APP"
fi
codesign --verify --deep --strict --verbose=2 "$APP"
