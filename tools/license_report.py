"""Validate cargo-about coverage and render notices, including pinned supplements.

Run cargo-about with --workspace --all-features --locked --offline --fail first.
This complements, rather than replaces, the full release source/notice inventory.
"""
import argparse
import hashlib
import html
import json
from pathlib import Path
import re
import subprocess
import tomllib

from development_package import supplemental_notices

ROOT = Path(__file__).resolve().parents[1]


def package_key(package):
    return package['name'], package['version']


def verify_coverage(report, required):
    packages = {package_key(c['package']) for c in report['crates']}
    if missing := required - packages:
        raise ValueError(f'Packages missing from cargo-about: {sorted(missing)}')
    covered = set()
    for license in report['licenses']:
        if not license['text'].strip():
            raise ValueError(f'Empty license text: {license["id"]}')
        for user in license['used_by']:
            key = package_key(user['crate'])
            if key not in packages:
                raise ValueError(f'Unknown license owner: {key}')
            covered.add(key)
    if covered != packages:
        raise ValueError(f'Packages without license text: {sorted(packages - covered)}')
    return packages


def production_and_test_packages(targets):
    packages = set()
    for target in targets:
        graph = subprocess.check_output([
            'cargo', 'tree', '--workspace', '--all-features', '--locked', '--offline',
            '--target', target, '--edges', 'normal,build,dev', '--prefix', 'none',
            '--format', '{p}',
        ], cwd=ROOT, text=True, encoding='utf-8')
        packages.update(re.findall(r'^(\S+) v(\S+)', graph, re.MULTILINE))
    return packages


def verify_clarifications(report, config):
    packages = {c['package']['name']: c['package'] for c in report['crates']}
    for name, settings in config.items():
        if not isinstance(settings, dict) or 'clarify' not in settings:
            continue
        if name not in packages:
            raise ValueError(f'Stale clarification: {name}')
        root = Path(packages[name]['manifest_path']).parent
        for source in settings['clarify'].get('files', []):
            path = root / source['path']
            if hashlib.sha256(path.read_bytes()).hexdigest() != source['checksum']:
                raise ValueError(f'Changed clarification source: {path}')
            text = path.read_text(encoding='utf-8').strip()
            if not any(
                text == license['text'].replace('\r\n', '\n').strip()
                and any(owner['crate']['name'] == name for owner in license['used_by'])
                for license in report['licenses']
            ):
                raise ValueError(f'Clarified license text missing from report: {name}: {path}')


def render(report, supplements):
    esc = html.escape
    parts = ['<!doctype html><html lang="en"><meta charset="utf-8">',
             '<title>Polished Map NG — license report</title>',
             '<style>body{font:16px system-ui;max-width:70rem;margin:2rem auto;padding:0 1rem}'
             'pre{white-space:pre-wrap}summary{cursor:pointer}section{margin:2rem 0}</style>',
             '<h1>Polished Map NG — license report</h1>',
             '<p>All workspace features, including build and test dependencies, for Windows, '
             'Linux and both macOS targets. The GUI entry includes separately licensed bundled '
             'artwork/fonts and adapted code; this does not relicense those components. '
             'License exceptions are retained in the full texts below.</p>',
             '<p>Original UI artwork provenance limitations remain in NOTICE.md and docs/assets.md. '
             'This report does not establish additional third-party artwork rights.</p>']
    for license in report['licenses']:
        owners = sorted({f'{u["crate"]["name"]} {u["crate"]["version"]}' for u in license['used_by']})
        parts += [f'<section><h2>{esc(license["name"])}</h2>',
                  f'<p>{esc(", ".join(owners))}</p>',
                  f'<details><summary>License text</summary><pre>{esc(license["text"])}</pre></details></section>']
    parts.append('<h1>Supplemental upstream notices</h1><p>These pinned files preserve '
                 'authorship and terms omitted by published crates. They supplement automatic '
                 'collection; permitted alternative license choices can differ between the '
                 'collector and the release inventory. Both retained sets of terms remain available.</p>')
    for (name, version), record in sorted(supplements.items()):
        parts.append(f'<section><h2>{esc(name)} {esc(version)}</h2>')
        for source in record['files']:
            text = (ROOT / source['path']).read_text(encoding='utf-8')
            parts.append(f'<details><summary>{esc(source["path"])}</summary><pre>{esc(text)}</pre></details>')
        parts.append('</section>')
    parts.append('<h1>Project and asset attribution</h1>')
    for path in [ROOT / 'NOTICE.md', *sorted((ROOT / 'crates/gui/assets').rglob('NOTICE.md'))]:
        parts.append(f'<h2>{esc(path.relative_to(ROOT).as_posix())}</h2><pre>{esc(path.read_text(encoding="utf-8"))}</pre>')
    return '\n'.join(parts) + '</html>\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('report', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    report = json.loads(args.report.read_text(encoding='utf-8'))
    config = tomllib.loads((ROOT / 'about.toml').read_text(encoding='utf-8'))
    packages = verify_coverage(report, production_and_test_packages(config['targets']))
    verify_clarifications(report, config)
    supplements = supplemental_notices(ROOT)
    lock = tomllib.loads((ROOT / 'Cargo.lock').read_text(encoding='utf-8'))
    locked = {package_key(p) for p in lock['package']}
    if stale := supplements.keys() - locked:
        raise ValueError(f'Stale supplemental notice: {sorted(stale)}')
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(render(report, supplements), encoding='utf-8')
    print(f'Verified {len(packages)} target-selected crates, all local clarifications and '
          f'{len(supplements)} pinned supplemental notice records; wrote {args.output}')


if __name__ == '__main__':
    main()
