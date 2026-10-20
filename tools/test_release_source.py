"""Exercise both real publisher source guards without building or publishing."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class ReleaseSourceTest(unittest.TestCase):
    def command(self, *args, cwd=None):
        return subprocess.check_output(args, cwd=cwd, text=True, stderr=subprocess.DEVNULL).strip()

    def publisher(self, platform, *, landed=True, dirty=False):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / 'runtime'
            root.mkdir()
            (root / 'tools').mkdir()
            (root / 'crates/nux-capi').mkdir(parents=True)
            (root / 'crates/nux-capi/Cargo.toml').write_text('version = "1.2.3"\n')
            (root / '.gitignore').write_text('/target/\n')
            (root / 'bin').mkdir()
            python = root / 'bin/python3'
            python.write_text(r'''#!/bin/sh
case "$1 $2" in
  *android_runtime_contract.py\ release-tag) echo android-runtime-v1.2.3 ;;
  *json-scalar.py*) echo "$TEST_SOURCE" ;;
  *) echo passed-source-guards; exit 70 ;;
esac
''')
            python.chmod(0o755)
            name = 'publish-nux-capi-release.sh' if platform == 'apple' else 'publish-nux-capi-android-release.sh'
            shutil.copy2(ROOT / 'tools' / name, root / 'tools' / name)
            self.command('git', 'init', '-b', 'main', cwd=root)
            self.command('git', 'config', 'user.name', 'Release tests', cwd=root)
            self.command('git', 'config', 'user.email', 'release-tests@example.invalid', cwd=root)
            self.command('git', 'add', '.', cwd=root)
            self.command('git', 'commit', '-m', 'Initial', cwd=root)
            if not landed:
                self.command('git', 'checkout', '-b', 'unmerged', cwd=root)
                (root / 'side').write_text('unmerged change\n')
                self.command('git', 'add', '.', cwd=root)
                self.command('git', 'commit', '-m', 'Side branch', cwd=root)
            source = self.command('git', 'rev-parse', 'HEAD', cwd=root)
            tag = f'{platform}-runtime-v1.2.3'
            self.command('git', 'tag', tag, cwd=root)
            self.command('git', 'checkout', 'main', cwd=root)
            (root / 'later').write_text('main advances\n')
            self.command('git', 'add', '.', cwd=root)
            self.command('git', 'commit', '-m', 'Later main', cwd=root)
            remote = Path(directory) / 'remote.git'
            self.command('git', 'clone', '--bare', str(root), str(remote))
            self.command('git', 'remote', 'add', 'origin', str(remote), cwd=root)
            self.command('git', 'checkout', '--detach', source, cwd=root)
            artifacts = root / 'target/release'
            artifacts.mkdir(parents=True)
            names = ['NuxieRuntime.xcframework.zip', 'NuxieRuntime-iOS.xcframework.zip', 'artifact-set.json', 'SIZE_REPORT.json'] if platform == 'apple' else ['NuxieRuntimeAndroid.zip', 'NuxieRuntimeAndroid.json', 'NuxieRuntimeAndroid-BUILD_INPUTS.json', 'NuxieRuntimeAndroid-SIZE_REPORT.json']
            for asset in names:
                (artifacts / asset).write_text('{}')
            if dirty:
                (root / 'untracked').write_text('dirty\n')
            return subprocess.run(
                ['bash', str(root / 'tools' / name), tag, str(artifacts)], cwd=root,
                env={**os.environ, 'PATH': str(root / 'bin') + os.pathsep + os.environ['PATH'], 'TEST_SOURCE': source},
                text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            )

    def test_landed_commit_can_publish_after_main_advances(self):
        for platform in ['apple', 'android']:
            with self.subTest(platform=platform):
                result = self.publisher(platform)
                self.assertEqual(result.returncode, 70, result.stderr)
                self.assertIn('passed-source-guards', result.stdout)

    def test_unmerged_source_is_rejected(self):
        for platform in ['apple', 'android']:
            with self.subTest(platform=platform):
                result = self.publisher(platform, landed=False)
                self.assertEqual(result.returncode, 4, result.stderr)
                self.assertIn('landed on origin/main', result.stderr)

    def test_dirty_source_is_rejected(self):
        for platform in ['apple', 'android']:
            with self.subTest(platform=platform):
                result = self.publisher(platform, dirty=True)
                self.assertEqual(result.returncode, 4, result.stderr)
                self.assertIn('dirty', result.stderr)


if __name__ == '__main__':
    unittest.main()
