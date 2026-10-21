#!/usr/bin/env python3
"""Exercise install.sh offline with real archives and a mocked HTTPS transport."""
import hashlib
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class InstallerTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='downtube-installer-test-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.tools = self.root / 'tools'
        self.tools.mkdir()
        self.assets = self.root / 'assets'
        self.assets.mkdir()
        self.destination = self.root / 'install with spaces'
        self.env = dict(os.environ, PATH=f'{self.tools}{os.pathsep}{os.environ["PATH"]}',
                        DOWNTUBE_TEST_ASSETS=str(self.assets), DOWNTUBE_TEST_OS='Linux',
                        DOWNTUBE_TEST_ARCH='x86_64', TMPDIR=str(self.root),
                        DOWNTUBE_INSTALL_DIR=str(self.destination), DOWNTUBE_VERSION='latest')
        self.executable('uname', '#!/bin/sh\ncase "$1" in -s) echo "$DOWNTUBE_TEST_OS";; -m) echo "$DOWNTUBE_TEST_ARCH";; esac\n')
        self.executable('sysctl', '#!/bin/sh\necho 0\n')
        self.executable('curl', f'#!{sys.executable}\n' + '''import os, pathlib, shutil, sys
args = sys.argv[1:]
url = next(a for a in args if a.startswith('https://'))
if '%{url_effective}' in args:
    print('https://github.com/Nightmare99/downtube/releases/tag/v9.8.7', end='')
else:
    shutil.copyfile(pathlib.Path(os.environ['DOWNTUBE_TEST_ASSETS']) / url.rsplit('/', 1)[1], args[args.index('-o') + 1])
''')

    def executable(self, name, body):
        path = self.tools / name
        path.write_text(body)
        path.chmod(0o755)

    def archive(self, target='x86_64-unknown-linux-musl', checksum=True, binary_version='9.8.7'):
        binary = self.root / 'binary'
        binary.write_text(f'#!/bin/sh\necho "downtube {binary_version}"\n')
        binary.chmod(0o755)
        archive = self.assets / f'downtube-v9.8.7-{target}.tar.gz'
        with tarfile.open(archive, 'w:gz') as bundle:
            for name in ('downtube', 'dtube'):
                bundle.add(binary, arcname=name)
        digest = hashlib.sha256(archive.read_bytes()).hexdigest() if checksum else '0' * 64
        (self.assets / 'SHA256SUMS').write_text(f'{digest}  {archive.name}\n')

    def run_installer(self, *args):
        return subprocess.run(['sh', str(ROOT / 'install.sh'), *args], env=self.env,
                              capture_output=True, text=True, timeout=15)

    def test_latest_install_upgrade_and_pinned_version(self):
        self.archive()
        for args in ((), ('--version', '9.8.7')):
            result = self.run_installer(*args)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            for name in ('downtube', 'dtube'):
                version = subprocess.check_output([str(self.destination / name), '--version'], text=True)
                self.assertEqual(version.strip(), 'downtube 9.8.7')
        self.assertFalse(list(self.root.glob('downtube-install.*')))

    def test_arm64_macos(self):
        self.env.update(DOWNTUBE_TEST_OS='Darwin', DOWNTUBE_TEST_ARCH='arm64')
        self.archive('aarch64-apple-darwin')
        result = self.run_installer()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_bad_checksum_preserves_existing_install(self):
        self.archive(checksum=False)
        self.destination.mkdir()
        original = self.destination / 'downtube'
        original.write_text('existing installation')
        result = self.run_installer()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('checksum mismatch', result.stderr)
        self.assertEqual(original.read_text(), 'existing installation')
        self.assertFalse(list(self.root.glob('downtube-install.*')))

    def test_wrong_binary_version_is_rejected(self):
        self.archive(binary_version='1.0.0')
        result = self.run_installer()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('binary version does not match', result.stderr)
        self.assertFalse(self.destination.exists())

    def test_unsupported_architecture_and_invalid_options(self):
        self.env['DOWNTUBE_TEST_ARCH'] = 'i686'
        result = self.run_installer()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('unsupported CPU', result.stderr)
        self.assertNotEqual(self.run_installer('--version').returncode, 0)
        self.assertNotEqual(self.run_installer('--unknown').returncode, 0)
        self.assertEqual(self.run_installer('--help').returncode, 0)


if __name__ == '__main__':
    unittest.main()
