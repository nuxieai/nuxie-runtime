"""Integrity checks for review bookkeeping; these do not perform visual review."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('review', Path(__file__).with_name('record-visual-review.py'))
review = importlib.util.module_from_spec(spec)
spec.loader.exec_module(review)


class ReviewIntegrity(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.out = Path(self.temp.name)
        (self.out / 'review-sheets').mkdir()
        self.cases = []
        for name in ('one', 'same_pixels'):
            prefix = str(self.out / name)
            row = dict(name=name, width=240, prefix=prefix, geometryFailures=[], failures=[])
            for kind in ('browser', 'native'):
                path = Path(prefix + '.' + kind + '.png')
                path.write_bytes(kind.encode())
                row[kind + 'Sha256'] = review.sha(path)
            (self.out / 'review-sheets' / (name + '.png')).write_bytes(name.encode())
            self.cases.append(row)
        self.save_cases()

    def save_cases(self):
        (self.out / 'replay.json').write_text(json.dumps({'cases': self.cases}))

    def test_exact_pixels_transfer_and_audit(self):
        self.assertEqual(review.record(self.out, ['one'], 'Viewed fixture'), {'reviewed': 2, 'remaining': 0})
        self.assertEqual(review.record(self.out, audit=True)['reviewed'], 2)

    def test_changed_previously_reviewed_sheet_cannot_gain_more_reviews(self):
        review.record(self.out, ['one'], 'Viewed fixture')
        receipt = (self.out / 'visual-inspection.json').read_bytes()
        (self.out / 'review-sheets' / 'one.png').write_bytes(b'changed')
        with self.assertRaisesRegex(ValueError, 'Changed reviewed sheet'):
            review.record(self.out, ['same_pixels'], 'Viewed another fixture')
        self.assertEqual((self.out / 'visual-inspection.json').read_bytes(), receipt)

    def test_changed_image_blocks_transfer(self):
        review.record(self.out, ['one'], 'Viewed fixture')
        (self.out / 'same_pixels.native.png').write_bytes(b'changed')
        with self.assertRaisesRegex(ValueError, 'Changed native image'):
            review.record(self.out, audit=True)

    def test_unknown_names_and_failures_are_rejected(self):
        with self.assertRaisesRegex(ValueError, 'Unknown review cases'):
            review.record(self.out, ['typo'], 'Viewed fixture')
        self.cases[0]['failures'] = ['local RGB error']
        self.save_cases()
        with self.assertRaisesRegex(ValueError, 'failure-review'):
            review.record(self.out, ['one'], 'Viewed fixture')

    def test_forged_review_count_fails_audit(self):
        review.record(self.out, ['one'], 'Viewed fixture')
        path = self.out / 'visual-inspection.json'
        receipt = json.loads(path.read_text())
        receipt['reviewed'] = 999
        path.write_text(json.dumps(receipt))
        with self.assertRaisesRegex(ValueError, 'counts or transfers'):
            review.record(self.out, audit=True)


if __name__ == '__main__':
    unittest.main()
