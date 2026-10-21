#!/usr/bin/env python3
"""Check tag/version consistency or bump Cargo's version for a stable release."""
import argparse
import re
from pathlib import Path
import tomllib

parser = argparse.ArgumentParser(description=__doc__)
action = parser.add_mutually_exclusive_group(required=True)
action.add_argument('--check-tag', metavar='vX.Y.Z')
action.add_argument('--bump', metavar='X.Y.Z')
args = parser.parse_args()
manifest = Path('Cargo.toml')
lockfile = Path('Cargo.lock')
current = tomllib.loads(manifest.read_text())['package']['version']
locked = next(p['version'] for p in tomllib.loads(lockfile.read_text())['package'] if p['name'] == 'downtube')
if current != locked:
    parser.error(f'Cargo.toml ({current}) and Cargo.lock ({locked}) disagree')
if args.check_tag:
    if args.check_tag != f'v{current}':
        parser.error(f'tag {args.check_tag!r} must match Cargo version v{current}')
    changelog = Path('CHANGELOG.md').read_text()
    if not re.search(rf'^## \[{re.escape(current)}\] - \d{{4}}-\d{{2}}-\d{{2}}$', changelog, re.M):
        parser.error(f'CHANGELOG.md needs a dated [{current}] section')
    print(f'Version verified: v{current}')
else:
    new = args.bump.removeprefix('v')
    if not re.fullmatch(r'(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)', new):
        parser.error('use a stable semantic version such as 0.3.0')
    if tuple(map(int, new.split('.'))) <= tuple(map(int, current.split('.'))):
        parser.error(f'new version must be greater than {current}')
    text, count = re.subn(r'(?m)^(version\s*=\s*")' + re.escape(current) + r'("\s*)$', lambda m: m[1] + new + m[2], manifest.read_text(), count=1)
    if count != 1:
        parser.error('could not locate the package version in Cargo.toml')
    lock, count = re.subn(r'(\[\[package\]\]\nname = "downtube"\nversion = ")' + re.escape(current) + '"', lambda m: m[1] + new + '"', lockfile.read_text(), count=1)
    if count != 1:
        parser.error('could not locate downtube in Cargo.lock')
    manifest.write_text(text)
    lockfile.write_text(lock)
    print(f'Bumped {current} → {new}. Add a dated [{new}] CHANGELOG.md entry, then commit and tag v{new}.')
