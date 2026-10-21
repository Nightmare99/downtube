#!/usr/bin/env python3
"""Package already-built native binaries and their documentation."""
import argparse
import json
from pathlib import Path
import subprocess
import tarfile
import zipfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--target', required=True)
args = parser.parse_args()
metadata = json.loads(subprocess.check_output(['cargo', 'metadata', '--locked', '--no-deps', '--format-version', '1'], text=True))
package = next(item for item in metadata['packages'] if item['name'] == 'downtube')
version = package['version']
windows = 'windows' in args.target
suffix = '.exe' if windows else ''
binary_dir = Path(metadata['target_directory']) / args.target / 'release'
files = [(binary_dir / (name + suffix), name + suffix) for name in ('downtube', 'dtube')]
files += [(Path(name), name) for name in ('README.md', 'LICENSE', 'CONTRIBUTING.md', 'CHANGELOG.md')]
files += [(path, path.as_posix()) for path in sorted(Path('docs').glob('*.md'))]
for source, _ in files:
    if not source.is_file():
        parser.error(f'missing release file: {source}')
destination = Path('dist')
destination.mkdir(exist_ok=True)
archive = destination / f'downtube-v{version}-{args.target}{".zip" if windows else ".tar.gz"}'
if windows:
    with zipfile.ZipFile(archive, 'w', compression=zipfile.ZIP_DEFLATED) as bundle:
        for source, name in files:
            bundle.write(source, name)
else:
    with tarfile.open(archive, 'w:gz') as bundle:
        for source, name in files:
            bundle.add(source, arcname=name)
print(archive)
