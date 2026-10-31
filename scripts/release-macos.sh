#!/bin/sh
# Requires an installed Developer ID Application identity and an existing
# notarytool Keychain profile. No account passwords belong in this repository.
set -eu
cd "$(dirname "$0")/.."
: "${RC_SIGN_IDENTITY:?Set RC_SIGN_IDENTITY to your Developer ID Application identity}"
: "${RC_NOTARY_PROFILE:?Set RC_NOTARY_PROFILE to an existing notarytool Keychain profile}"
case "$RC_SIGN_IDENTITY" in
    'Developer ID Application:'*) ;;
    *) echo 'A Developer ID Application identity is required for direct downloads.' >&2; exit 1 ;;
esac
export RC_SIGN_IDENTITY
VERSION=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)
scripts/package-universal-macos.sh
APP='dist/universal/Radio Crash Desktop.app'
STAGING='dist/notarization-upload.zip'
RELEASE_DIR="dist/release/$VERSION"
mkdir -p "$RELEASE_DIR"
ditto -c -k --keepParent "$APP" "$STAGING"
xcrun notarytool submit "$STAGING" --keychain-profile "$RC_NOTARY_PROFILE" --wait --output-format json > "$RELEASE_DIR/notarization.json"
# notarytool can return successfully for a completed but rejected submission.
/usr/bin/plutil -extract status raw -o - "$RELEASE_DIR/notarization.json" | /usr/bin/grep -qx Accepted
xcrun stapler staple "$APP"
xcrun stapler validate "$APP"
spctl --assess --type execute --verbose=2 "$APP"
ZIP="$RELEASE_DIR/Radio-Crash-Desktop-$VERSION-macOS-universal.zip"
PAYLOAD="dist/macOS-download-$VERSION/Radio Crash Desktop"
mkdir -p "$PAYLOAD"
ditto "$APP" "$PAYLOAD/Radio Crash Desktop.app"
cp RELEASE-NOTES.md INSTALL-macOS.md LICENSE THIRD_PARTY_NOTICES.md DEPENDENCIES.md "$PAYLOAD/"
cp -R licenses "$PAYLOAD/"
cp RELEASE-NOTES.md "$PAYLOAD/RELEASE-NOTES.txt"
cp INSTALL-macOS.md "$PAYLOAD/INSTALL.txt"
ditto -c -k --keepParent "$PAYLOAD" "$ZIP"
cp "$ZIP" "$RELEASE_DIR/Radio-Crash-Desktop-macOS-universal.zip"
(cd "$RELEASE_DIR" && shasum -a 256 "Radio-Crash-Desktop-$VERSION-macOS-universal.zip" > SHA256SUMS-macOS.txt)
echo "$ZIP"
