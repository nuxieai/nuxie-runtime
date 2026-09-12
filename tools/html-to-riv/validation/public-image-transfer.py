"""Join changed public image renders with verified unchanged native/Chrome evidence."""
import hashlib
import json
import re
from pathlib import Path
import sys

root, previous_path = map(lambda p: Path(p).resolve(), sys.argv[1:3])
fresh_path = root / 'native-receipt.json'
output = root / 'combined-receipt.json'
assert not output.exists()


def read(p):
    return json.loads(p.read_text())


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


# Combined receipts add provenance to unchanged measured rows. Trace that
# provenance to a real raw capture; outputLabel belongs to that capture, never
# to the later combined receipt or destination run.
class EvidenceSources:
    def __init__(self, fresh_path):
        self.documents = {}
        self.hashes = {}
        self.expected = None
        fresh = self.load(fresh_path)
        self.expected = self.tool_bindings(fresh)

    @staticmethod
    def tool_bindings(document):
        result = []
        for name in ['baseline-probe', 'renderer-replay', 'reset.css']:
            matches = [b for b in document['bindings'] if Path(b['path']).name == name]
            assert len(matches) == 1, ('Ambiguous native binding', name)
            binding = matches[0]
            assert sha(Path(binding['path'])) == binding['sha256'], ('Changed native binding', name)
            result.append(binding)
        return result

    def load(self, path, expected_hash=None):
        path = Path(path).resolve()
        actual = sha(path)
        if expected_hash is not None:
            assert actual == expected_hash, ('Receipt binding mismatch', path)
        if path not in self.documents:
            document = read(path)
            assert document['browser'] == '153.0.8010.12'
            assert not document['errors'], ('Source capture has errors', path)
            actual_tools = self.tool_bindings(document)
            if self.expected is not None:
                assert [b['sha256'] for b in actual_tools] == [b['sha256'] for b in self.expected], ('Native source tools/reset differ', path)
            self.documents[path] = document
            self.hashes[path] = actual
        assert self.hashes[path] == actual, ('Source changed during transfer', path)
        return self.documents[path]

    @staticmethod
    def measured(row):
        return {key: value for key, value in row.items() if key != 'evidence'}

    def resolve(self, path, row, seen=()):
        path = Path(path).resolve()
        assert path not in seen, ('Cyclic receipt provenance', path)
        document = self.load(path)
        matches = [r for r in document['rows'] if (r['name'], r['frame']) == (row['name'], row['frame'])]
        assert len(matches) == 1 and matches[0] == row, ('Source row identity mismatch', path, row['name'], row['frame'])
        evidence = row.get('evidence')
        if evidence is None:
            label = document['outputLabel']
            assert isinstance(label, str) and re.fullmatch(r'[a-z0-9-]+', label), 'Invalid raw output label'
            return path, document, row
        assert evidence['kind'] in ['fresh-render', 'exact-unchanged-transfer'], 'Unknown provenance kind'
        result_path = Path(str(row['prefix']) + '.result.json').resolve()
        assert Path(evidence['result']).resolve() == result_path, 'Result does not belong to the recorded prefix'
        assert sha(result_path) == evidence['resultSha256'], ('Result binding mismatch', result_path)
        assert read(result_path) == self.measured(row), ('Result measurement mismatch', result_path)
        nested_path = Path(evidence['receipt']).resolve()
        nested = self.load(nested_path, evidence['receiptSha256'])
        matches = [r for r in nested['rows'] if (r['name'], r['frame']) == (row['name'], row['frame'])]
        assert len(matches) == 1, ('Missing or duplicated nested source row', nested_path)
        assert self.measured(matches[0]) == self.measured(row), ('Nested measurement differs', nested_path)
        return self.resolve(nested_path, matches[0], (*seen, path))

    def assert_unchanged(self):
        for path, digest in self.hashes.items():
            assert sha(path) == digest, ('Source changed during transfer', path)
            self.tool_bindings(self.documents[path])


sources = EvidenceSources(fresh_path)
fresh, previous = sources.load(fresh_path), sources.load(previous_path)
bindings = sources.expected
reset = (root / 'frozen/inputs/src/reset.css').read_text()
compiled = read(root / 'compile-receipt.json')
rows, transfers = [], []
for case in (c for c in compiled if c['compiled']):
    name = case['name']
    request = read(root / name / 'request.json')
    assert sha(root / name / 'scene.riv') == case['rivSha256']
    assert sha(root / name / 'scene.map.json') == case['mapSha256']
    current = [r for r in fresh['rows'] if r['name'] == name]
    transferred = not current
    source_receipt = previous_path if transferred else fresh_path
    source = previous if transferred else fresh
    chosen = [r for r in source['rows'] if r['name'] == name]
    assert len(chosen) == 8, name
    assert [(r['instance'], r['step'], r['width'], r['height']) for r in chosen] == [
        (instance, step, w, h) for instance in [0, 1]
        for step, (w, h) in enumerate([(240, 240), (390, 320), (768, 560), (240, 240)])]
    for selected in chosen:
        raw_receipt, raw_source, row = sources.resolve(source_receipt, selected)
        prefix = Path(row['prefix'])
        old_dir = prefix.parent.parent
        assert read(old_dir / 'request.json') == request, name
        assert row['rivSha256'] == case['rivSha256'] == sha(old_dir / 'scene.riv')
        assert row['mapSha256'] == case['mapSha256'] == sha(old_dir / 'scene.map.json')
        assert row['requestSha256'] == sha(old_dir / 'request.json')
        # This also rejects the original contaminated r1 references whose base
        # lacked the per-run/per-case namespace required by the corrected driver.
        base = f"http://html-to-riv.invalid/{raw_source['outputLabel']}/{name}/"
        expected_html = f'<!doctype html><meta charset="utf-8"><base href="{base}"><style>{reset}\n{request["css"]}</style>{request["html"]}'
        html = prefix.parent / 'reference.html'
        assert html.read_text() == expected_html and sha(html) == row['htmlSha256']
        observed = Path(row['probeDirectory'])
        assert sha(observed / 'scene.riv') == case['rivSha256']
        for key, path in [('chromeSha256', Path(str(prefix) + '.chrome.png')),
                          ('nativeSha256', Path(str(prefix) + '.native.png')),
                          ('geometrySha256', observed / row['geometry']),
                          ('streamSha256', observed / row['stream'])]:
            assert sha(path) == row[key], (name, key)
        result_path = Path(str(prefix) + '.result.json')
        assert read(result_path) == row
        joined = dict(row, evidence=dict(kind='exact-unchanged-transfer' if transferred else 'fresh-render',
                     receipt=str(raw_receipt), receiptSha256=sha(raw_receipt),
                     result=str(result_path), resultSha256=sha(result_path)))
        rows.append(joined)
    if transferred:
        transfers.append(name)
counts = [dict(name=case['name'], frames=len(r := [r for r in rows if r['name'] == case['name']]),
               geometryPass=sum(not x['metricFailures'] for x in r), pixelPass=sum(not x['pixelFailures'] for x in r),
               presencePass=sum(all(v['passed'] for v in x['imagePresence'].values()) for x in r))
          for case in compiled if case['compiled']]
summary = dict(frames=len(rows), freshFrames=sum(r['evidence']['kind'] == 'fresh-render' for r in rows),
               transferredFrames=sum(r['evidence']['kind'] == 'exact-unchanged-transfer' for r in rows),
               geometryPass=sum(c['geometryPass'] for c in counts), pixelPass=sum(c['pixelPass'] for c in counts),
               presencePass=sum(c['presencePass'] for c in counts))
sources.assert_unchanged()
# Freeze the exact consumer of this evidence only after all input checks pass.
# Historical joins can then survive later validation-helper edits.
helper = Path(__file__).resolve()
helper_snapshot = root / 'validation-inputs-transfer' / helper.name
helper_snapshot.parent.mkdir(exist_ok=False)
helper_snapshot.write_bytes(helper.read_bytes())
assert sha(helper_snapshot) == sha(helper)
source_bindings = []
for path in dict.fromkeys([fresh_path, previous_path, root / 'compile-receipt.json', helper, *sources.documents]):
    binding = dict(path=str(path), sha256=sha(path))
    if path == helper:
        binding['snapshot'] = str(helper_snapshot)
    source_bindings.append(binding)
receipt = dict(scope='Public image candidate evidence join; full public integration and visual qualification separate',
               browser=fresh['browser'], bindings=bindings, compilerBuild=dict(path=str(root / 'build-receipt.json'),
               sha256=sha(root / 'build-receipt.json')), sources=source_bindings,
               rows=rows, counts=counts, summary=summary, transferredCases=transfers, errors=[])
output.write_text(json.dumps(receipt, indent=2) + '\n')
print(json.dumps(summary))
