"""Revalidate reviewed circular-corner lifecycle evidence against a full gallery.

Usage: SCRIPT EVIDENCE_ROOT [COMPLETED_GALLERY]
Source-only mode freshly reproduces the 24 public artifacts and audits all 192
resize/clone frames. Gallery mode additionally compares the 72 initial views.
"""
import json
from pathlib import Path
import runpy
import sys


def references(root):
    here = Path(__file__).resolve().parent
    verify = runpy.run_path(str(here / 'audit-border-composition-review.py'))['audit']
    replay_dir = root / 'corner-radii-proportional-lifecycle'
    recording = root / 'corner-radii-proportional-recording'
    fresh = verify(root / 'corner-radii-proportional-static-review', recording,
                   replay_dir, root / 'corner-radii-proportional-toolchain', expected_scenes=24)
    saved = json.loads((replay_dir / 'public-corner-audit.json').read_text())
    # These two labels describe the corpus, rather than artifact identity.
    fresh['qualification'] = 'public-circular-corner-focused-lifecycle'
    fresh['limitation'] = 'Initial24-scene corner corpus only; broader P03 qualification remains open.'
    assert fresh == saved, 'Changed corner lifecycle evidence'
    fixtures = {r['name']: r for r in json.loads((recording / 'lifecycle.json').read_text())['cases']}
    refs = {}
    for row in json.loads((replay_dir / 'replay.json').read_text())['cases']:
        if row['instance'] != 0 or row['frame'] > 2:
            continue
        name = row['name'].rsplit('-0-step', 1)[0]
        fixture = fixtures[name]
        assert not fixture['font'] and not fixture['image']
        key = name, row['width']
        assert key not in refs
        refs[key] = dict(row=row, recording=recording, fixture=fixture)
    assert len(refs) == 72
    return refs


if __name__ == '__main__':
    root = Path(sys.argv[1]).resolve()
    refs = references(root)
    if len(sys.argv) == 2:
        print(json.dumps(dict(sourceViewsVerified=len(refs))))
    else:
        gallery = Path(sys.argv[2]).resolve()
        audit = runpy.run_path(str(Path(__file__).with_name('border-gallery-reference.py')))['audit']
        result = audit(root, gallery, refs=refs)
        output = gallery / 'corner-lifecycle-comparison.json'
        if output.exists():
            assert json.loads(output.read_text()) == result
        else:
            output.write_text(json.dumps(result, indent=2) + '\n')
        print(json.dumps(dict(reviewedPairs=result['reviewedPairs'], remaining=len(result['remaining']))))
