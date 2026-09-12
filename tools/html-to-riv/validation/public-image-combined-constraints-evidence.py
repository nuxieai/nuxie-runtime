"""Verify combined public image constraints without compilation or rendering.

Transfer 192 exact private measured rows from completed visual review. Only the
new primary case and minimum-only corpus claim fresh sheet inspection.
"""
import hashlib
import json
from pathlib import Path

MODULE = Path(__file__).resolve().parents[1]
OUTPUT = MODULE / 'output'


def verify():
    checked = {}
    def bind(path, expected=None):
        path = Path(path).resolve()
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        if expected is not None:
            assert digest == expected, f'Changed artifact: {path}'
        checked[str(path)] = digest
        return digest
    def read(path):
        bind(path)
        return json.loads(Path(path).read_text())
    def bindings(rows, live=False):
        for row in rows:
            bind(row.get('snapshot', row['path']), row['sha256'])
            if live:
                bind(row['path'], row['sha256'])
    def measured(row):
        return {k:v for k,v in row.items() if k != 'evidence'}
    def key(row):
        return row['name'], row['frame']

    build = OUTPUT / 'public-image-combined-constraints-build-r1'
    summary = read(build / 'summary.json')
    assert summary['status'] == 'public-build-and-transport-pass' and summary['sourceUnchanged']
    assert (summary['rustTests'], summary['nodeTests']) == (346,56)
    source = read(build / 'frozen/source-bindings.json')['files']
    assert len(source) == 274
    bindings(source, live=True)
    bind(build / 'frozen/html-to-riv', summary['compilerSha256'])
    bind(build / 'frozen/compiler.wasm', summary['wasmSha256'])

    def emitted(root, count, files):
        bindings(read(root / 'authoring-bindings.json'), live=True)
        assert read(root / 'frozen/source-bindings.json')['files'] == source
        bind(root / 'frozen/html-to-riv', summary['compilerSha256'])
        rows = read(root / 'compile-receipt.json')
        assert len(rows) == len({r['name'] for r in rows}) == count
        assert sum(r['compiled'] for r in rows) == files
        assert all(r['matchesExpectation'] for r in rows)
        for row in rows:
            folder = root / row['name']
            bind(folder / 'request.json', row['requestSha256'])
            if row['compiled']:
                bind(folder / 'scene.riv', row['rivSha256'])
                bind(folder / 'scene.map.json', row['mapSha256'])
            else:
                assert not (folder / 'scene.riv').exists() and not (folder / 'scene.map.json').exists()
        return {r['name']:r for r in rows}

    def visual(root, expected, fresh_names=None):
        coverage = read(root / 'visual-r1/coverage.json')
        bindings(coverage['bindings'])
        bindings(coverage['artifacts'])
        reps = {key(r) for r in coverage['representatives']}
        transfers = {(r['case'],r['frame']) for r in coverage['transfers']}
        assert len(reps) == len(coverage['representatives'])
        assert len(transfers) == len(coverage['transfers'])
        assert not reps & transfers and len(reps | transfers) == expected
        if fresh_names is None:
            review = read(root / 'visual-r1/review-receipt.json')
            assert review['visualReviewCompleted'] and review['counts']['frames'] == expected
            bindings(review['bindings'])
            inspected = review['inspectedSheets']
            assert {(r['path'],r['sha256']) for r in inspected} == {(r['path'],r['sha256']) for r in coverage['sheets']}
            selected = reps | transfers
        else:
            review = read(root / 'visual-r1/fresh-inspection-notes.json')
            assert review['reviewCompleted'] and review['observations'] and review['limitations']
            inspected = review['inspectedSheets']
            selected = {(name,frame) for name in fresh_names for frame in range(8)}
            assert selected <= reps | transfers
        bindings(inspected)
        sheet_index = {r['path']:r for r in coverage['sheets']}
        direct = set()
        for sheet in inspected:
            assert sheet['sha256'] == sheet_index[sheet['path']]['sha256']
            direct |= {(r['case'],r['frame']) for r in sheet_index[sheet['path']]['members']}
        assert reps & selected <= direct
        if fresh_names is not None:
            assert direct <= selected and len(direct) == 3
        # Independently verify PNG equality for within-case transfers. All
        # selected transfers in these finite corpora use equal-size PNGs.
        for row in coverage['transfers']:
            if (row['case'],row['frame']) not in selected:
                continue
            assert (row['case'],row['sourceFrame']) in reps
            assert row['sameRequestSceneMapReferenceNativeBoxesAndOtherMetadata']
            assert not row['backgroundBoxMetadataDifferences']
            for image in row['images']:
                bindings([image['source'],image['target']])
                assert image['source']['sha256'] == image['target']['sha256']
        return selected

    donor = OUTPUT / 'image-combined-constraints-candidate-r1'
    reviewed = visual(donor,192)
    prior = read(donor / 'combined-receipt.json')
    assert not prior['errors'] and prior['browser'] == '153.0.8010.12'
    prior_rows = {key(r):r for r in prior['rows']}
    assert len(prior_rows) == len(prior['rows']) == 192 and set(prior_rows) == reviewed
    primary = OUTPUT / 'public-image-combined-constraints-r1'
    fresh_name = 'image-responsive-constraints-diagnostic-percentage-minimum'
    fresh_reviewed = visual(primary,200,{fresh_name})
    minimum = OUTPUT / 'public-image-combined-constraints-minimum-r1'
    minimum_reviewed = visual(minimum,16)
    corpora, failures, transferred = [], [], []
    primary_index = None
    for root, request_count, file_count, pixels, fresh, parity in [
        (primary,29,25,190,8,'public-image-combined-constraints-parity-r1'),
        (minimum,2,2,14,16,'public-image-combined-constraints-minimum-parity-r1'),
    ]:
        index = emitted(root,request_count,file_count)
        if root == primary:
            primary_index = index
        native = read(root / 'combined-receipt.json')
        assert native['browser'] == '153.0.8010.12' and not native['errors']
        bindings(native['bindings']); bindings(native['sources']); bindings([native['compilerBuild']])
        for name,digest in [('baseline-probe','7b17829dfdea44709faaff688ad146e659862d66bf284dd9aa4b17696839fe6a'),
                            ('renderer-replay','276a49d810c05de7cfb0f0a4109853d12216da64e5d9d8e131f7400357f8e51f')]:
            matches = [b for b in native['bindings'] if Path(b['path']).name == name]
            assert len(matches) == 1 and matches[0]['sha256'] == digest
        assert native['summary']['freshFrames'] == fresh
        assert native['summary']['transferredFrames'] == file_count*8-fresh
        rows = native['rows']
        expected = {(r['name'],frame) for r in index.values() if r['compiled'] for frame in range(8)}
        assert len(rows) == len({key(r) for r in rows}) == file_count*8 and {key(r) for r in rows} == expected
        assert all(not r['metricFailures'] for r in rows)
        assert sum(not r['pixelFailures'] for r in rows) == pixels
        assert all(v['passed'] for r in rows for v in r['imagePresence'].values())
        for row in rows:
            folder = root / row['name']
            for filename,hashkey in [('request.json','requestSha256'),('scene.riv','rivSha256'),('scene.map.json','mapSha256')]:
                assert row[hashkey] == index[row['name']][hashkey]
                bind(folder / filename,row[hashkey])
            assert (row['instance'],row['step']) == divmod(row['frame'],4)
            assert (row['width'],row['height']) == [(240,240),(390,320),(768,560),(240,240)][row['step']]
            for suffix,hashkey in [('.chrome.png','chromeSha256'),('.native.png','nativeSha256')]:
                bind(row['prefix']+suffix,row[hashkey])
            for pathkey,hashkey in [('geometry','geometrySha256'),('stream','streamSha256')]:
                bind(Path(row['probeDirectory']) / row[pathkey],row[hashkey])
            evidence = row['evidence']
            bind(evidence['receipt'],evidence['receiptSha256'])
            bind(evidence['result'],evidence['resultSha256'])
            assert read(evidence['result']) == measured(row)
            raw = read(evidence['receipt'])
            assert sum(measured(r) == measured(row) for r in raw['rows']) == 1
            if root == primary and row['name'] != fresh_name:
                assert key(row) in reviewed and measured(row) == measured(prior_rows[key(row)])
                assert evidence['kind'] == 'exact-unchanged-transfer'
                for filename,hashkey in [('request.json','requestSha256'),('scene.riv','rivSha256'),('scene.map.json','mapSha256')]:
                    bind(donor / row['name'] / filename,row[hashkey])
                transferred.append({'case':row['name'],'frame':row['frame'],'sourceReview':str(donor / 'visual-r1/review-receipt.json'),
                    'chromeSha256':row['chromeSha256'],'nativeSha256':row['nativeSha256']})
            else:
                assert evidence['kind'] == 'fresh-render'
                assert key(row) in (fresh_reviewed if root == primary else minimum_reviewed)
            if row['pixelFailures']:
                assert row['pixelFailures'] == ['tail: local RGB error']
                failures.append({'case':row['name'],'frame':row['frame'],'failures':row['pixelFailures']})
        transport = read(OUTPUT / parity / 'receipt.json'); bindings(transport['bindings'])
        assert transport['cases'] == request_count and transport['passed'] == transport['observations'] == request_count*2
        corpora.append({'requests':request_count,'files':file_count,'frames':file_count*8,'geometryPass':file_count*8,
                        'pixelPass':pixels,'presencePass':file_count*8,'parityPass':request_count*2})
    expected_failures = {(name,frame) for name,frames in [
        ('image-combined-constraints-height-percent-min-point-max-row',[1,5]),
        ('image-combined-constraints-height-conflicting-percent-row',[2,6]),
        ('image-combined-constraints-width-ordered-percent-column',[0,1,3,4,5,7]),
        ('image-combined-constraints-percentage-minimum-only-height',[2,6]),
    ] for frame in frames}
    assert len(expected_failures) == len(failures) == 12
    assert {(r['case'],r['frame']) for r in failures} == expected_failures
    assert len(transferred) == 192

    # Prior API recheck includes the same converted request already counted in
    # primary; its repeated observation is never an extra unique feature file.
    prior_root = OUTPUT / 'public-image-combined-constraints-prior-r1'
    prior_index = emitted(prior_root,19,13)
    old_index = {r['name']:r for r in read(OUTPUT / 'public-image-responsive-constraints-r1/compile-receipt.json')}
    diagnostic_wording_changes = []
    for name,row in prior_index.items():
        old = old_index[name]
        assert row['requestSha256'] == old['requestSha256']
        if name == fresh_name:
            assert not old['compiled'] and row['compiled']
            assert all(row[k] == primary_index[name][k] for k in ['requestSha256','rivSha256','mapSha256'])
        elif row['compiled']:
            assert old['compiled'] and row['rivSha256'] == old['rivSha256'] and row['mapSha256'] == old['mapSha256']
        else:
            assert not old['compiled']
            before, after = old['diagnostics'], row['diagnostics']
            assert [(d['code'],d['source']) for d in before] == [(d['code'],d['source']) for d in after]
            if before != after:
                assert len(before) == len(after) == 1
                expected = before[0]['message'].replace('an opposite point minimum or point/percentage maximum','opposite point/percentage min/max bounds').replace('preferred-axis and simultaneous bounds','preferred-axis bounds')
                assert after[0]['message'] == expected
                diagnostic_wording_changes.append({'case':name,'before':before,'after':after})
    assert {r['case'] for r in diagnostic_wording_changes} == {
        'image-responsive-constraints-diagnostic-image-padding', 'image-responsive-constraints-diagnostic-automatic-minimum',
        'image-responsive-constraints-diagnostic-cross-stretch', 'image-responsive-constraints-diagnostic-preferred-axis-maximum'}
    transport = read(OUTPUT / 'public-image-combined-constraints-prior-parity-r1/receipt.json'); bindings(transport['bindings'])
    assert transport['cases'] == 19 and transport['passed'] == transport['observations'] == 38
    old = read(OUTPUT / 'public-image-combined-constraints-existing-r1/results/manifest.json')
    assert old['total'] == old['passed'] == 196
    bind(old['compiler'], summary['compilerSha256'])
    bind(old['priorManifest'], old['priorManifestSha256'])
    for row in old['results']:
        assert row['exact']
        bind(row['request'], row['requestSha256'])
        for filename, key, prior_key in [('scene.riv', 'rivSha256', 'priorRiv'), ('scene.map.json', 'mapSha256', 'priorMap')]:
            bind(Path(row['result']) / filename, row[key])
            bind(row[prior_key], row[key])
    history_root = OUTPUT / 'public-transport-malformed-combined-constraints-regression-r1'
    history = read(history_root / 'receipt.json')
    assert history['total'] == history['passed'] == 794 and history['allExact']
    assert history['inputHashes']['html-to-riv'] == summary['compilerSha256']
    for row in history['results']:
        assert row['exact']
        reference = row['reference']
        for filename, digest in reference['hashes'].items():
            bind(Path(reference['reference']) / filename, digest)
        for filename, digest in row['actualHashes'].items():
            assert digest == reference['hashes'][filename]
            bind(history_root / 'results' / str(row['index']) / filename, digest)
    out = OUTPUT / 'public-image-combined-constraints-checkpoint-r1'
    out.mkdir(exist_ok=True)
    proof = out / 'visual-transfer-review.json'
    proof.write_text(json.dumps({'scope':'192 exact donor frame transfers plus 24 freshly reviewed public frames; primary old-case sheets are not claimed as freshly inspected.',
        'exactDonorFrameTransfers':192,'freshPublicFramesReviewed':24,'transfers':transferred},indent=2)+'\n')
    bind(proof)
    bind(Path(__file__).resolve())
    return {'scope':'Partial combined opposite-axis public image constraints; 12 following-tail pixel failures retained.',
        'build':summary,'corpora':corpora,'uniqueRequests':31,'uniqueFiles':27,'nativeFrames':216,'geometryPass':216,'pixelPass':204,'presencePass':216,
        'newParityObservations':62,'priorRecheckRequests':19,'priorParityObservations':38,
        'priorRecheckConvertedCaseAlreadyCounted':fresh_name,'priorDiagnosticWordingChanges':diagnostic_wording_changes,'historicalOutputsExact':794,'existingImageFilesExact':196,
        'visualTransferProof':str(proof),'pixelFailures':failures,
        'bindings':[{'path':path,'sha256':digest} for path,digest in sorted(checked.items())]}


if __name__ == '__main__':
    result = verify()
    out = OUTPUT / 'public-image-combined-constraints-checkpoint-r1'
    (out / 'verification.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({'corpora':result['corpora'],'artifactBindings':len(result['bindings'])}))
