#!/usr/bin/env python3
"""Refresh notices from cargo metadata; never modify upstream license text."""
import json
import pathlib
import shutil
import sys

metadata=json.loads(pathlib.Path(sys.argv[1]).read_text())
rows=[]
for package in sorted(metadata['packages'],key=lambda p:(p['name'],p['version'])):
    if package['name']=='radio-crash-desktop': continue
    source=pathlib.Path(package['manifest_path']).parent
    destination=pathlib.Path('licenses')/f"{package['name']}-{package['version']}"
    candidates=[p for p in source.iterdir() if p.is_file() and p.name.lower().startswith(('license','licence','copying','notice','copyright'))]
    if package.get('license_file'):
        candidates.append(source/package['license_file'])
    for p in set(candidates):
        if p.is_file():
            destination.mkdir(parents=True,exist_ok=True)
            shutil.copy2(p,destination/p.name)
    link=package.get('repository') or f"https://crates.io/crates/{package['name']}/{package['version']}"
    rows.append(f"| {package['name']} | {package['version']} | {package.get('license') or 'See upstream license file'} | [upstream]({link}) |")
pathlib.Path('DEPENDENCIES.md').write_text('# Resolved dependency license inventory\n\nGenerated from pinned Cargo.lock and cargo metadata. Includes target-specific and build dependencies. Unmodified upstream notices are copied under `licenses/`.\n\n| Package | Version | Declared license | Source |\n|---|---|---|---|\n'+'\n'.join(rows)+'\n')
print(f'Refreshed {len(rows)} dependency inventory entries.')
