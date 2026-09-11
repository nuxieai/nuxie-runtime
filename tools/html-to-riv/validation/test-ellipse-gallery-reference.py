"""Exercise gallery transfer acceptance and rejection with audited reference data.

Usage: SCRIPT EVIDENCE_ROOT. Synthetic reports test the verifier only; they do
not qualify a runtime or claim a completed full regression.
"""
import importlib.util
import json
from pathlib import Path
import sys
import tempfile

spec = importlib.util.spec_from_file_location('ellipse_audit', Path(__file__).with_name('ellipse-gallery-reference.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
root = Path(sys.argv[1]).resolve()
refs, receipts = module.references(root)
module.references = lambda _: (refs, receipts)  # Source proof was revalidated above.
assets_dir = Path(__file__).resolve().parent.parent / 'tests/assets'
with tempfile.TemporaryDirectory(prefix='ellipse-audit-test-') as folder:
    folder = Path(folder)
    cases = []
    asset_request = None
    for index, ((name, width), ref) in enumerate(refs.items()):
        prefix = folder / str(index)
        fixture = ref['fixture']
        request = dict(html=fixture['html'], css=fixture['css'], width=390, height=320)
        assets = {}
        if fixture['font']:
            assets['inter'] = dict(kind='font', family='Inter', weight=400, bytes=list((assets_dir / 'Inter-Regular.ttf').read_bytes()))
        if fixture['image']:
            assets['photo'] = dict(kind='image', bytes=list((assets_dir / 'quadrants.png').read_bytes()))
        if assets:
            request['assets'] = assets
            asset_request = Path(str(prefix) + '.json')
        Path(str(prefix) + '.json').write_text(json.dumps(request))
        for ext in ['.riv', '.requirements.json']:
            Path(str(prefix) + ext).symlink_to(ref['recording'] / (name + ext))
        for kind in ['browser', 'native']:
            Path(str(prefix) + '.' + kind + '.png').symlink_to(ref['row']['prefix'] + '.' + kind + '.png')
        cases.append(dict(name=name, width=width, html=fixture['html'], css=fixture['css'], artifactPrefix=str(prefix), status='passed', errors=[], metrics=dict(backend='rust-metal')))
    report = dict(status='passed', errors=[], expected=len(cases), checks=[dict(status='passed', errors=[], project='native') for _ in cases], cases=cases)
    path = folder / 'review.json'
    path.write_text(json.dumps(report))
    result = module.audit(root, folder)
    assert result['reviewedPairs'] == 252 and not result['remaining']

    original = asset_request.read_text()
    changed = json.loads(original)
    next(iter(changed['assets'].values()))['bytes'][0] ^= 1
    asset_request.write_text(json.dumps(changed))
    try:
        module.audit(root, folder)
    except AssertionError:
        pass
    else:
        raise AssertionError('Changed embedded asset inherited review')
    asset_request.write_text(original)

    target = Path(cases[0]['artifactPrefix'] + '.native.png')
    original_target = target.readlink()
    target.unlink()
    target.symlink_to(cases[-1]['artifactPrefix'] + '.native.png')
    result = module.audit(root, folder)
    assert result['reviewedPairs'] == 251 and len(result['remaining']) == 1
    target.unlink()
    target.symlink_to(original_target)

    report['checks'].pop()
    path.write_text(json.dumps(report))
    try:
        module.audit(root, folder)
    except AssertionError:
        pass
    else:
        raise AssertionError('Incomplete gallery inherited review')
print(json.dumps(dict(status='verifier-controls-pass', exactPairs=252, changedAssetRejected=True, changedImageUnreviewed=True, incompleteGalleryRejected=True)))
