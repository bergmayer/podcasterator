import os
from pathlib import Path
import subprocess
import tempfile
import unittest
import zipfile


class PackagingTests(unittest.TestCase):
    def test_replaces_archive_and_preserves_it_on_failure(self):
        source = (Path(__file__).parents[1] / 'build.sh').read_text()
        start = source.index('zip_to_releases() (')
        helper = source[start:source.index('\n)', start) + 2]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            releases = root / 'releases'
            releases.mkdir()
            bundle = root / 'App.app'
            bundle.mkdir()
            (bundle / 'deleted').write_text('first')
            env = {**os.environ, 'RELEASES_DIR': str(releases)}
            def package(path):
                return subprocess.run(['bash', '-c', helper + '\nzip_to_releases "$1" current.zip', 'test', str(path)], env=env, capture_output=True)
            self.assertEqual(package(bundle).returncode, 0)
            bundle.mkdir()
            (bundle / 'current').write_text('second')
            self.assertEqual(package(bundle).returncode, 0)
            with zipfile.ZipFile(releases / 'current.zip') as archive:
                self.assertEqual(archive.namelist(), ['App.app/', 'App.app/current'])
            before = (releases / 'current.zip').read_bytes()
            self.assertNotEqual(package(root / 'missing').returncode, 0)
            self.assertEqual((releases / 'current.zip').read_bytes(), before)
            self.assertEqual(list(releases.glob('.package.*')), [])
            for name in ('old.dmg', 'new.dmg'):
                path = root / name
                path.write_text(name)
                self.assertEqual(package(path).returncode, 0)
            with zipfile.ZipFile(releases / 'current.zip') as archive:
                self.assertEqual(archive.namelist(), ['new.dmg'])


if __name__ == '__main__':
    unittest.main()
