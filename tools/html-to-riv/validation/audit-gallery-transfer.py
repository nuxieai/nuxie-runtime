"""Audit a completed native gallery against completed full/replay reviews.

Usage: python3 audit-gallery-transfer.py TARGET SOURCE [SOURCE ...]
No new visual inspection is claimed: source inputs and full PNG bytes must match.
"""
import hashlib
import importlib.util
import json
import runpy
import sys
from pathlib import Path


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def read(path):
    return json.loads(Path(path).read_text())


def input_sha(directory, case):
    if 'artifactPrefix' in case:
        path = Path(case['artifactPrefix'] + '.json')
    else:
        path = directory / (case['name'] + '.json')
    data = read(path)
    assert data['html'] == case['html'] and data['css'] == case['css']
    return hashlib.sha256(json.dumps(data, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def images(case):
    prefix = case.get('artifactPrefix', case.get('prefix'))
    return {kind + 'Sha256': sha(prefix + '.' + kind + '.png') for kind in ['browser', 'native']}


def complete_gallery(report):
    assert report['status'] == 'passed' and not report['errors']
    assert len(report['checks']) == report['expected']
    assert all(c['status'] == 'passed' and not c['errors'] and c['project'] == 'native' for c in report['checks'])
    assert len({(c['name'], c['width']) for c in report['cases']}) == len(report['cases'])


def verify_combined_comparison(path, report):
    """Require every union row to retain an unchanged successful component proof."""
    combined = read(path)
    assert combined['status'] == 'complete'
    assert combined['reviewSha256'] == sha(path.parent / 'review.json')
    assert not combined['remaining']
    assert combined['scenePairs'] == combined['reviewedPairs'] == len(report['cases'])
    components = {}
    for source in combined['sourceComparisons']:
        source_path = Path(source['path'])
        assert sha(source_path) == source['sha256'], 'Changed component comparison'
        assert source_path.name not in components
        component = read(source_path)
        assert component['reviewSha256'] == combined['reviewSha256']
        rows = {(r['name'], r['width']): r for r in component['rows']}
        assert len(rows) == len(component['rows'])
        assert component['reviewedPairs'] == sum(r['reviewTransferred'] for r in rows.values())
        for receipt in component.get('sourceReceipts', []):
            assert sha(receipt['path']) == receipt['sha256']
        components[source_path.name] = rows
    seen = set()
    for row in combined['rows']:
        key = row['name'], row['width']
        assert key not in seen
        seen.add(key)
        original = components[row['sourceComparison']][key]
        assert original['reviewTransferred'] and row['reviewTransferred']
        assert all(row.get(k) == v for k, v in original.items()), 'Changed union row'
    assert seen == {(c['name'], c['width']) for c in report['cases']}
    return combined


target, *sources = map(Path, sys.argv[1:])
assert sources, 'Expected TARGET SOURCE [SOURCE ...]'
known = {}
evidence = []
for directory in sources:
    if (directory / 'replay.json').exists():
        result_path = directory / 'replay.json'
        receipt_path = directory / 'visual-inspection.json'
        if not receipt_path.exists() and (directory / 'visual-promotion.json').exists():
            # Re-prove diagnostic-to-public policy equivalence before this
            # public replay can supply an ordinary exact-input/image transfer.
            receipt_path = directory / 'visual-promotion.json'
            receipt = read(receipt_path)
            promote = runpy.run_path(str(Path(__file__).with_name('promote-clip-margin-review.py')))['promote']
            promote(receipt['source'], directory, audit=True)
        else:
            spec = importlib.util.spec_from_file_location('visual_review', Path(__file__).with_name('record-visual-review.py'))
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            module.record(directory, audit=True)
            receipt = read(receipt_path)
        assert receipt['status'] == 'complete'
        report = read(result_path)
        expected_hashes = {(c['name'], c['width']): c for c in report['cases']}
    else:
        result_path = directory / 'review.json'
        receipt_path = directory / 'receipt.json'
        # Newer frozen galleries retain the audited comparison itself, rather
        # than a wrapper receipt. Revalidate that proof and all source receipts.
        if not receipt_path.exists():
            receipt_path = directory / 'baseline-comparison.json'
            report, comparison = read(result_path), read(receipt_path)
            complete_gallery(report)
            if (directory / 'combined-visual-comparison.json').exists():
                receipt_path = directory / 'combined-visual-comparison.json'
                comparison = verify_combined_comparison(receipt_path, report)
            assert comparison['reviewSha256'] == sha(result_path)
            assert not comparison['remaining'] and comparison['reviewedPairs'] == len(report['cases'])
            assert len(comparison['rows']) == len(report['cases'])
            assert all(row['reviewTransferred'] for row in comparison['rows'])
            for source in comparison.get('sourceReceipts', []):
                assert sha(source['path']) == source['sha256']
            expected_hashes = {(c['name'], c['width']): c for c in comparison['rows']}
            assert len(expected_hashes) == len(report['cases'])
            for case in report['cases']:
                assert input_sha(directory, case) == expected_hashes[(case['name'], case['width'])]['compilerInputSha256']
            evidence.append({'path': str(receipt_path), 'sha256': sha(receipt_path), 'resultsSha256': sha(result_path)})
            for case in report['cases']:
                key = case['name'], case['width']
                hashes = images(case)
                assert all(expected_hashes[key][k] == v for k, v in hashes.items()), ('Changed source image', key)
                known[(key, input_sha(directory, case), tuple(hashes.values()))] = str(receipt_path)
            continue
        report, receipt = read(result_path), read(receipt_path)
        complete_gallery(report)
        assert receipt['status'] == 'passed' and receipt['exitCode'] == 0
        assert sha(result_path) == receipt['reviewSha256']
        assert sha(directory / 'visual-inspection.json') == receipt['visualInspectionSha256']
        visual = read(directory / 'visual-inspection.json')
        comparison_path = directory / visual['comparison']
        assert sha(comparison_path) == visual['comparisonSha256']
        comparison = read(comparison_path)
        assert not comparison['remaining'] and comparison['reviewedPairs'] == len(report['cases'])
        assert all(row['reviewTransferred'] for row in comparison['rows'])
        for source in visual.get('sourceReceipts', []):
            assert sha(source['path']) == source['sha256']
        expected_hashes = {(c['name'], c['width']): c for c in comparison['rows']}
    evidence.append({'path': str(receipt_path), 'sha256': sha(receipt_path), 'resultsSha256': sha(result_path)})
    for c in report['cases']:
        key = c['name'], c['width']
        hashes = images(c)
        assert all(expected_hashes[key][k] == v for k, v in hashes.items()), ('Changed source image', key)
        identity = (key, input_sha(directory, c), tuple(hashes.values()))
        known[identity] = str(receipt_path)

report = read(target / 'review.json')
complete_gallery(report)
rows = []
for c in report['cases']:
    assert c['status'] == 'passed' and not c['errors'] and c['metrics']['backend'] == 'rust-metal'
    hashes = images(c)
    compiler_input = input_sha(target, c)
    source = known.get(((c['name'], c['width']), compiler_input, tuple(hashes.values())))
    rows.append(dict(name=c['name'], width=c['width'], compilerInputSha256=compiler_input,
                     **hashes, reviewTransferred=source is not None, sourceReceipt=source))
result = dict(scenePairs=len(rows), reviewedPairs=sum(r['reviewTransferred'] for r in rows),
              remaining=[r for r in rows if not r['reviewTransferred']], rows=rows,
              sourceReceipts=evidence, reviewSha256=sha(target / 'review.json'),
              method='Exact complete compiler input and full browser/native PNG bytes, matching name and viewport; completed source reviews audited.')
output = target / 'baseline-comparison.json'
if output.exists():
    assert read(output) == result, 'Refusing to overwrite changed audit evidence'
else:
    output.write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({k: result[k] for k in ['scenePairs', 'reviewedPairs']}))
print('Remaining:', len(result['remaining']))
