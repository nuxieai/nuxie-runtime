"""Negative controls for runtime-gradient visual review identity; uses temporary evidence."""
import copy
import hashlib
import json
from pathlib import Path
import runpy
import tempfile
import unittest

canonical = runpy.run_path(str(Path(__file__).with_name('transfer-visual-review.py')))['canonical_input']


class GradientReviewIdentity(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.folder = Path(self.temp.name)
        self.case = dict(name='gradient-0-step1', instance=0, frame=1,
                         html='<div/>', css='background:linear-gradient(red,blue)',
                         compilerRequestCss='', qualification='runtime-experiment-only',
                         injectedGradient={'colors': [0xffff0000, 0xff0000ff]},
                         injectedPixelBounds=[1], runtimeRequirements={'version': 25})
        (self.folder / 'replay.json').write_text(json.dumps({'cases': [self.case]}))
        self.proof = dict(status='passed', evidence=[dict(
            path=str(self.folder / 'replay.json'),
            sha256=hashlib.sha256((self.folder / 'replay.json').read_bytes()).hexdigest())],
            artifacts=[dict(name='gradient', rivSha256='a' * 64)])
        self.write_proof()

    def write_proof(self):
        (self.folder / 'gradient-artifact-audit.json').write_text(json.dumps(self.proof))

    def identity(self, case=None):
        return canonical(self.folder, self.case['name'], self.case if case is None else case)

    def test_each_semantic_input_changes_identity_even_with_identical_images(self):
        baseline = self.identity()
        for field, value in dict(html='<span/>', css='background:none', compilerRequestCss='color:red',
                                 qualification='public', injectedGradient={'colors': [0, 1]},
                                 injectedPixelBounds=[2], runtimeRequirements={'version': 26}).items():
            with self.subTest(field=field):
                changed = copy.deepcopy(self.case)
                changed[field] = value
                self.assertNotEqual(baseline, self.identity(changed))
        self.proof['artifacts'][0]['rivSha256'] = 'b' * 64
        self.write_proof()
        self.assertNotEqual(baseline, self.identity())

    def test_stale_replay_rejected(self):
        (self.folder / 'replay.json').write_text('{}')
        with self.assertRaises(AssertionError):
            self.identity()

    def test_wrong_evidence_path_rejected(self):
        self.proof['evidence'][0]['path'] = str(self.folder / 'other.json')
        self.write_proof()
        with self.assertRaises(AssertionError):
            self.identity()

    def test_failed_audit_rejected(self):
        self.proof['status'] = 'failed'
        self.write_proof()
        with self.assertRaises(AssertionError):
            self.identity()

    def test_original_public_request_route_preserved(self):
        request = {'css': 'color:red', 'html': '<div/>', 'viewport': [390, 320]}
        (self.folder / 'public.json').write_text(json.dumps(request))
        self.assertEqual(canonical(self.folder, 'public', {}),
                         json.dumps(request, sort_keys=True, separators=(',', ':')).encode())


if __name__ == '__main__':
    unittest.main()
