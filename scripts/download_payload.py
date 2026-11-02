#!/usr/bin/env python3
"""User-facing ZIP payloads. Standard-library only; no runtime Python dependency."""
import argparse
import pathlib
import plistlib
import subprocess
import shutil
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]


def legal_sources(root):
    files = [root / name for name in ('LICENSE', 'THIRD_PARTY_NOTICES.md', 'DEPENDENCIES.md')]
    files += sorted((root / 'assets').glob('*OFL.txt'))
    files += sorted(p for p in (root / 'licenses').rglob('*') if p.is_file())
    return [(p.relative_to(root).as_posix(), p.read_bytes()) for p in files]


def legal_bundle(sources):
    header = (
        'Radio Crash Desktop - copyright, license and source notices\n\n'
        'All original notices are included below, without removing their text.\n'
        'Paths in the notices identify sections in this file and files in the source repository.\n'
        'Application source and exact tagged Cargo.lock: https://github.com/bandarko/radio-crash-desktop\n'
        'Unmodified dependency versions and upstream source links are in the DEPENDENCIES.md section.\n'
        'Symphonia and its codecs are unmodified MPL-2.0 components; source is available from the\n'
        'listed upstream/crates.io versions and the repository lockfile.\n\n'
    ).encode('utf-8')
    return header + b''.join(
        b'\n\n' + b'=' * 72 + b'\nSOURCE SECTION: ' + name.encode('utf-8') + b'\n' + b'=' * 72 + b'\n' + data + b'\n'
        for name, data in sources
    )


def current_notes(text):
    lines = text.splitlines(keepends=True)
    end = next((i for i, line in enumerate(lines[1:], 1) if line.startswith('# ')), len(lines))
    return ''.join(lines[:end]).strip()


def readme(platform, version, notes, testing=''):
    start = ('Raspakiraj ZIP i premjesti Radio Crash Desktop.app u Applications. Pokreni aplikaciju.'
             if platform == 'macos' else
             'Zatvori stari player preko POWER > EXIT. Raspakiraj cijeli ZIP u vlastitu mapu i pokreni RadioCrashDesktop.exe.')
    license_location = ('Licence su unutar aplikacije: Contents/Resources. Potpis i notarizacija ostaju očuvani.'
                        if platform == 'macos' else
                        'LICENSES.txt sadrži obavezne licence i autorska prava svih uključenih komponenti.')
    return (
        f'RADIO CRASH DESKTOP {version}\n\n{start}\n\n'
        'POWER > EXIT zatvara aplikaciju. Postavke se čuvaju između nadogradnji.\n'
        'Gornji dio kućišta služi za pomicanje; lijevi gornji vijak za minimiziranje.\n'
        f'{license_location}\n\n'
        'Službena izdanja i izvorni kod: https://github.com/bandarko/radio-crash-desktop\n\n'
        + ('TEST BUILD / NIJE STABILNO IZDANJE\n' + testing.strip() + '\n\n' if testing else '')
        + 'RELEASE NOTES\n\n' + current_notes(notes) + '\n'
    )


def prepare_windows(binary):
    destination = ROOT / 'dist/windows-x64'
    destination.parent.mkdir(exist_ok=True)
    # A fresh staging directory prevents development files from older builds leaking in.
    with tempfile.TemporaryDirectory(prefix='.windows-payload-', dir=destination.parent) as temp:
        stage = pathlib.Path(temp)
        shutil.copy2(binary, stage / 'RadioCrashDesktop.exe')
        version = next(line.split('"')[1] for line in (ROOT / 'Cargo.toml').read_text().splitlines() if line.startswith('version = '))
        (stage / 'PROCITAJ-ME.txt').write_text(readme('windows', version, (ROOT / 'RELEASE-NOTES.md').read_text()), encoding='utf-8')
        (stage / 'LICENSES.txt').write_bytes(legal_bundle(legal_sources(ROOT)))
        if destination.exists():
            shutil.rmtree(destination)  # Generated dist payload only, never user configuration.
        shutil.copytree(stage, destination)
    return destination


def prepare_mac_resources(resources):
    # Called only before signing. Existing signed/notarized apps are never edited.
    if not resources.resolve().is_relative_to((ROOT / 'dist').resolve()):
        raise ValueError('Mac resources must be inside this checkout generated dist directory')
    for pattern in ('*.prompt.txt', '*OFL.txt'):
        for file in resources.glob(pattern):
            file.unlink()
    for name in ('UI-ASSETS.md', 'GRAPHICS.md', 'DEPENDENCIES.md', 'THIRD_PARTY_NOTICES.md', 'RELEASE-NOTES.md', 'LICENSE'):
        (resources / name).unlink(missing_ok=True)
    if (resources / 'licenses').exists():
        shutil.rmtree(resources / 'licenses')
    (resources / 'LICENSES.txt').write_bytes(legal_bundle(legal_sources(ROOT)))
    (resources / 'RELEASE-NOTES.txt').write_text(current_notes((ROOT / 'RELEASE-NOTES.md').read_text()), encoding='utf-8')


def prepare_mac_download(app, destination):
    if not destination.resolve().is_relative_to((ROOT / 'dist').resolve()):
        raise ValueError('Mac download staging must be inside generated dist')
    version = plistlib.loads((app / 'Contents/Info.plist').read_bytes())['CFBundleShortVersionString']
    if destination.exists():
        shutil.rmtree(destination)
    destination.mkdir(parents=True)
    subprocess.run(['ditto', app, destination / app.name], check=True)
    (destination / 'PROCITAJ-ME.txt').write_text(readme('macos', version, (ROOT / 'RELEASE-NOTES.md').read_text()), encoding='utf-8')


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest='platform', required=True)
    sub.add_parser('windows').add_argument('binary', type=pathlib.Path)
    sub.add_parser('macos').add_argument('resources', type=pathlib.Path)
    mac_download = sub.add_parser('macos-download')
    mac_download.add_argument('app', type=pathlib.Path)
    mac_download.add_argument('destination', type=pathlib.Path)
    args = parser.parse_args()
    if args.platform == 'windows':
        prepare_windows(args.binary.resolve())
    elif args.platform == 'macos':
        prepare_mac_resources(args.resources.resolve())
    else:
        prepare_mac_download(args.app.resolve(), args.destination.resolve())
