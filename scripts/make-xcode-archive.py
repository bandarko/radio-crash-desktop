#!/usr/bin/env python3
"""Wrap the Rust-built app for Xcode Organizer's account-based notarization."""
import datetime
import pathlib
import plistlib
import subprocess
import sys

if len(sys.argv) != 3:
    raise SystemExit('Usage: make-xcode-archive.py Application.app Destination.xcarchive')
app, archive = map(pathlib.Path, sys.argv[1:])
if archive.exists():
    raise SystemExit('Choose a new archive destination; existing archives are preserved.')
info = plistlib.loads((app / 'Contents/Info.plist').read_bytes())
signature = subprocess.run(['codesign', '-dv', '--verbose=4', str(app)],
                           capture_output=True, text=True, check=True).stderr
fields = dict(line.split('=', 1) for line in signature.splitlines() if '=' in line)
authority = next((line.removeprefix('Authority=') for line in signature.splitlines()
                  if line.startswith('Authority=Developer ID Application:')), None)
if not authority or fields.get('TeamIdentifier') in (None, 'not set'):
    raise SystemExit('Sign the app with Developer ID Application first.')
product = archive / 'Products/Applications' / app.name
product.parent.mkdir(parents=True)
subprocess.run(['ditto', str(app), str(product)], check=True)
properties = {
    'ApplicationPath': f'Applications/{app.name}',
    'CFBundleIdentifier': info['CFBundleIdentifier'],
    'CFBundleShortVersionString': info['CFBundleShortVersionString'],
    'CFBundleVersion': info['CFBundleVersion'],
    'SigningIdentity': authority,
    'Team': fields['TeamIdentifier'],
    'Architectures': subprocess.check_output(
        ['lipo', '-archs', str(app / 'Contents/MacOS' / info['CFBundleExecutable'])],
        text=True).split(),
}
(archive / 'Info.plist').write_bytes(plistlib.dumps({
    'ArchiveVersion': 2,
    'CreationDate': datetime.datetime.now(datetime.timezone.utc).replace(tzinfo=None),
    'Name': 'Radio Crash Desktop',
    'SchemeName': 'Radio Crash Desktop',
    'ApplicationProperties': properties,
}))
print(archive.resolve())
