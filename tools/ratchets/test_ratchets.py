import hashlib
import importlib.util
import io
import json
from contextlib import redirect_stdout
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

spec=importlib.util.spec_from_file_location('ratchets',Path(__file__).with_name('ratchets.py'))
ratchets=importlib.util.module_from_spec(spec);spec.loader.exec_module(ratchets)


class ClippyDiagnosticTests(unittest.TestCase):
    def test_local_bazel_paths_remain_usable(self):
        with tempfile.TemporaryDirectory() as directory,patch.object(ratchets,'ROOT',Path(directory)):
            paths=ratchets.clippy_diagnostics('Target //crates/probe:probe up-to-date:\n  bazel-bin/crates/probe/probe.clippy.diagnostics\n')
            self.assertEqual(paths,{'crates/probe/probe':(Path(directory)/'bazel-bin/crates/probe/probe.clippy.diagnostics').resolve()})

    def test_remote_verified_diagnostic_is_read_from_the_returned_products(self):
        with tempfile.TemporaryDirectory() as directory,patch.object(ratchets,'ROOT',Path(directory)):
            root=Path(directory);state=root/'.nuxie/homelab/job';evidence=state/'evidence';evidence.mkdir(parents=True)
            name='products/crates/probe/probe.clippy.diagnostics';product=state/name;product.parent.mkdir(parents=True)
            product.write_text('{"$message_type":"diagnostic","code":{"code":"clippy::panic"}}\n')
            report={'exitCode':0,'outputs':{name:hashlib.sha256(product.read_bytes()).hexdigest()}}
            (evidence/'report.json').write_text(json.dumps(report))
            stderr='Pool build result: 0; evidence: '+str(evidence)+'\n'
            paths=ratchets.clippy_diagnostics(stderr)
            self.assertEqual(paths,{'crates/probe/probe':product.resolve()})
            product.write_text('tampered')
            with self.assertRaisesRegex(ValueError,'digest mismatch'):ratchets.clippy_diagnostics(stderr)

    def test_foreign_pool_evidence_is_refused(self):
        with tempfile.TemporaryDirectory() as directory,patch.object(ratchets,'ROOT',Path(directory)):
            with self.assertRaisesRegex(ValueError,'outside this checkout'):
                ratchets.clippy_diagnostics('Pool build result: 0; evidence: /foreign/evidence\n')

    def test_remote_diagnostic_counts_still_fail_when_a_panic_site_increases(self):
        with tempfile.TemporaryDirectory() as directory, patch.object(ratchets, 'ROOT', Path(directory).resolve()):
            root = ratchets.ROOT
            crate = root / 'crates/probe'
            crate.mkdir(parents=True)
            (crate / 'Cargo.toml').write_text('[package]\nname = "probe"\n')
            baseline = root / 'baseline.txt'
            baseline.write_text('crates/probe clippy::panic 0\n')
            state = root / '.nuxie/homelab/job'
            evidence = state / 'evidence'
            evidence.mkdir(parents=True)
            name = 'products/crates/probe/probe.clippy.diagnostics'
            product = state / name
            product.parent.mkdir(parents=True)
            product.write_text('{"$message_type":"diagnostic","code":{"code":"clippy::panic"}}\n')
            (evidence / 'report.json').write_text(json.dumps({
                'exitCode': 0, 'outputs': {name: hashlib.sha256(product.read_bytes()).hexdigest()}}))
            result = SimpleNamespace(returncode=0, stderr=f'Pool build result: 0; evidence: {evidence}\n')
            args = SimpleNamespace(baseline=baseline, patterns=['crates/probe'], update=False)
            output = io.StringIO()
            with patch.object(ratchets, 'bazel_command', return_value=['bazel']), \
                    patch.object(ratchets.subprocess, 'run', return_value=result), redirect_stdout(output):
                self.assertEqual(ratchets.panic_lints_command(args), 1)
            self.assertIn('1 forced panic-lint sites', output.getvalue())

    def test_a_named_variant_is_linted_on_its_platform_as_its_own_row(self):
        with tempfile.TemporaryDirectory() as directory, patch.object(ratchets, 'ROOT', Path(directory).resolve()):
            root = ratchets.ROOT
            crate = root / 'crates/probe'
            crate.mkdir(parents=True)
            (crate / 'Cargo.toml').write_text('[package]\nname = "probe"\n')
            for target, lints in [('probe', ['clippy::panic']), ('probe__web', ['clippy::panic', 'clippy::unwrap_used'])]:
                diagnostics = root / f'bazel-out/cfg-{target}/bin/crates/probe/{target}.clippy.diagnostics'
                diagnostics.parent.mkdir(parents=True)
                diagnostics.write_text(''.join(json.dumps({'$message_type': 'diagnostic', 'code': {'code': lint}}) + '\n' for lint in lints))
            baseline = root / 'baseline.txt'
            baseline.write_text('crates/probe clippy::panic 1\ncrates/probe:probe__web@wasm clippy::panic 1\n'
                                'crates/probe:probe__web@wasm clippy::unwrap_used 0\n')
            commands = []

            def build(command, **_):
                commands.append(command)
                target = 'probe__web' if '//crates/probe:probe__web' in command else 'probe'
                return SimpleNamespace(returncode=0, stderr=f'  bazel-out/cfg-{target}/bin/crates/probe/{target}.clippy.diagnostics\n')

            args = SimpleNamespace(baseline=baseline, patterns=[], update=False)
            output = io.StringIO()
            with patch.object(ratchets, 'bazel_command', return_value=['bazel']), \
                    patch.object(ratchets.subprocess, 'run', side_effect=build), redirect_stdout(output):
                self.assertEqual(ratchets.panic_lints_command(args), 1)
            self.assertEqual(len(commands), 2)
            native, wasm = commands
            self.assertIn('//crates/probe:probe', native)
            self.assertNotIn('--platforms=@rules_rust//rust/platform:wasm', native)
            self.assertIn('//crates/probe:probe__web', wasm)
            self.assertIn('--platforms=@rules_rust//rust/platform:wasm', wasm)
            self.assertRegex(output.getvalue(), r'crates/probe:probe__web@wasm clippy::unwrap_used +1 +0 +\+1  UP')
            self.assertRegex(output.getvalue(), r'crates/probe clippy::panic +1 +1 +\+0  ok')
            with self.assertRaisesRegex(ValueError, 'unknown platform'):
                ratchets.lint_target('crates/probe:probe__web@mars')


if __name__=='__main__':unittest.main()
