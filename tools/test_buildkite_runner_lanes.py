"""Execute composed recipes with recording tools and preserve check ownership."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from tools.bazel.pipeline import pipeline


class LaneTests(unittest.TestCase):
    def execute(self, step):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            (root / 'tools/bazel').mkdir(parents=True)
            (root / 'tools/bazel/ci-cache.sh').write_text('export NUXIE_BAZEL_BATCH=1\n')
            (root / 'rive-runtime').mkdir()
            recorder = root / 'record.py'
            recorder.write_text('import json, os, sys\nif sys.argv[1:3] == ["make", "nux-capi-android-contract-test"] and os.environ.get("CARGO_INCREMENTAL"): raise SystemExit(41)\nwith open(os.environ["CALLS"], "a") as log: log.write(json.dumps(sys.argv[1:]) + "\\n")\n')
            calls = root / 'calls.jsonl'
            functions = 'record() { "$REAL_PYTHON" "$RECORDER" "$@"; }\n'
            for tool in ('make', 'rustup', 'git', 'sudo', 'python3', 'tools/bazel/install.sh',
                         'tools/bazel/runtime.py', 'build/build_rive.sh'):
                functions += 'function ' + tool + ' { record ' + tool + ' "$@"; }\n'
            subprocess.run(['/bin/bash', '-c', functions + step['command']], cwd=root,
                           env={**os.environ, 'REAL_PYTHON': sys.executable, 'RECORDER': str(recorder),
                                'CALLS': str(calls), 'CARGO_INCREMENTAL': '0', 'RIVE_PREMAKE_TAG': 'fixture-tag', 'RIVE_RUNTIME_REF': 'fixture-ref'},
                           check=True, timeout=10)
            return [[argument.replace(str(root), '@checkout') for argument in json.loads(line)]
                    for line in calls.read_text().splitlines()]

    def test_activated_graph_retains_every_existing_recipe_and_adds_android(self):
        legacy = pipeline({})
        graph = pipeline({'NUXIE_CI_RUNNER_POLICY_VERSION': '2', 'NUXIE_SDK_SPLIT_LANES': '1'})
        self.assertEqual(len(legacy['steps']), 6)
        self.assertEqual(len(graph['steps']), 7)
        for old, new in zip(legacy['steps'], graph['steps']):
            self.assertEqual(new['command'], old['command'])
            self.assertEqual(self.execute(new), self.execute(old))
            platform = new['agents']['nuxie-platform']
            self.assertEqual(new['concurrency'], 1)
            self.assertEqual(new['concurrency_method'], 'eager')
            self.assertEqual(new['concurrency_group'], 'nuxie/physical/mac-m2' if platform == 'macos' else 'nuxie/physical/proxmox-heavy')
            self.assertEqual(new['agents']['nuxie-resources'], 'mac-64g-v2' if platform == 'macos' else 'linux-14g-v2')
            self.assertEqual(new['env']['NUXIE_CI_RUNNER_PROFILE'], 'macos-apple' if platform == 'macos' else 'linux-native')
            self.assertEqual(new['env']['NUXIE_BAZEL_REMOTE_CACHE_MODE'], 'off')
        android = graph['steps'][-1]
        self.assertEqual(android['agents']['nuxie-platform'], 'linux')
        self.assertEqual(android['if'], 'build.source == "schedule"')
        self.assertIn('target/nux-capi-android/NuxieRuntimeAndroid*', android['artifact_paths'])
        self.assertEqual(self.execute(android), [
            ['tools/bazel/install.sh'], ['make', 'nux-capi-android-contract-test', 'nux-capi-android'],
        ])

    def test_original_platform_inventory_has_no_hidden_opposite_platform_recipe(self):
        graph = pipeline({'NUXIE_CI_RUNNER_POLICY_VERSION': '2'})
        linux, apple = [], []
        for step in graph['steps']:
            (apple if step['agents']['nuxie-platform'] == 'macos' else linux).extend(self.execute(step))
        linux_make = {argument for call in linux if call[0] == 'make' for argument in call[1:]}
        apple_make = {argument for call in apple if call[0] == 'make' for argument in call[1:]}
        self.assertIn('nux-capi-pr-gate', linux_make)
        self.assertNotIn('feature-compile-gate-apple', linux_make)
        self.assertNotIn('microbench-gate', linux_make)
        self.assertIn('feature-compile-gate-apple', apple_make)
        self.assertIn('microbench-gate', apple_make)
        self.assertIn('crate-seams-browser-check', apple_make)

    def test_half_activation_and_unknown_settings_fail_before_dispatch(self):
        for environment in ({'NUXIE_SDK_SPLIT_LANES': '1'}, {'NUXIE_CI_RUNNER_POLICY_VERSION': '3'},
                            {'NUXIE_SDK_SPLIT_LANES': 'yes'}, {'NUXIE_CI_RUNNER_TOPOLOGY': 'unknown'},
                            {'NUXIE_CI_RUNNER_POLICY_VERSION': '2', 'NUXIE_CI_RUNNER_TOPOLOGY': 'fallback'}):
            with self.subTest(environment=environment), self.assertRaises(ValueError):
                pipeline(environment)


if __name__ == '__main__':
    unittest.main()
