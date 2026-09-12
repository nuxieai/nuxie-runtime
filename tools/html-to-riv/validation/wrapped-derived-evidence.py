"""Verify the private actual Derived rendering checkpoint without compiling or rendering."""
import hashlib
import json
from pathlib import Path

MODULE = Path(__file__).resolve().parents[1]
BUILD = MODULE / 'output/wrapped-derived-background-build-r2'
IMAGES = MODULE / 'output/wrapped-paint-mask-existing-r1/manifest.json'
HISTORY = MODULE / 'output/public-transport-malformed-wrapped-paint-mask-regression-r1/receipt.json'
bindings = {}


def bind(path, expected=None):
    path = Path(path)
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    if expected is not None:
        assert digest == expected, f'Changed artifact: {path}'
    bindings[str(path)] = digest
    return digest


def read(path):
    bind(path)
    return json.loads(Path(path).read_text())


summary = read(BUILD / 'summary.json')
assert summary['sourceUnchanged'] and summary['rustTests'] == 404 and summary['nodeTests'] == 56
for row in read(BUILD / 'frozen/source-bindings.json')['files']:
    bind(row['path'], row['sha256'])
    if 'snapshot' in row:
        bind(row['snapshot'], row['sha256'])
checks = read(BUILD / 'checks.json')
assert [row['name'] for row in checks] == [
    'full-rust', 'native-build', 'wasm-build', 'typescript', 'full-node', 'runtime-guard']
for row in checks:
    assert row['exitCode'] == 0
    bind(row['log'], row['logSha256'])
cli = summary['compilerSha256']
bind(BUILD / 'frozen/html-to-riv', cli)
bind(BUILD / 'frozen/compiler.wasm', summary['wasmSha256'])

images = read(IMAGES)
assert images['total'] == images['passed'] == len(images['results']) == 282
bind(images['compiler'], cli)
bind(images['priorManifest'], images['priorManifestSha256'])
bind(MODULE / 'validation/check-output-regression.py', images['scriptSha256'])
for row in images['results']:
    assert row['exitCode'] == 0 and row['exact']
    out = Path(row['result'])
    for prior_key, hash_key, filename in [('request', 'requestSha256', 'request.json'),
        ('priorRiv', 'rivSha256', 'scene.riv'), ('priorMap', 'mapSha256', 'scene.map.json')]:
        bind(row[prior_key], row[hash_key])
        bind(out / filename, row[hash_key])

history = read(HISTORY)
assert history['total'] == history['passed'] == len(history['results']) == 794
assert history['allExact']
for row in history['results']:
    assert row['exitCode'] == 0 and row['exact'] and row['termination'] is None
    bind(row['command'][0], cli)
    reference = row['reference']
    for filename, digest in reference['hashes'].items():
        bind(Path(reference['reference']) / filename, digest)
    bind(row['command'][1], reference['hashes']['request.json'])
    out = Path(row['command'][2]).parent
    ROOT = MODULE / 'output/playwright/wrapped-derived-r2'
CONSTRUCTOR = MODULE / 'output/wrapped-derived-constructor-r2'
construction = read(CONSTRUCTOR / 'construction-receipt.json')
bind(CONSTRUCTOR / 'candidate', construction['constructorSha256'])
bind(CONSTRUCTOR / 'recipes.json', construction['recipesSha256'])
bind(CONSTRUCTOR / 'construct.py', construction['scriptSha256'])
for b in read(CONSTRUCTOR / 'source-bindings.json'):
    bind(b['snapshot'], b['sha256'])
    if Path(b['source']) == MODULE/'src/wrapping_composition.rs':
        # Constructor predates a test-only strengthening. Compare full reviewed
        # delta and require byte-identical non-test module source.
        import difflib
        old=Path(b['snapshot']).read_text();current=Path(b['source']).read_text()
        assert old.split('#[cfg(test)]')[0] == current.split('#[cfg(test)]')[0]
        delta=''.join(difflib.unified_diff(old.splitlines(True),current.splitlines(True),fromfile='constructor-frozen/wrapping_composition.rs',tofile='current/wrapping_composition.rs'))
        patch=MODULE/'validation/wrapped-derived-test-only.patch'
        assert delta==patch.read_text();bind(patch);bind(b['source'])
    else:bind(b['source'], b['sha256'])
build = read(CONSTRUCTOR / 'receipt.json')
for key,filename in [('patchesSha256','patches.json'),('sourceBindingsSha256','source-bindings.json'),('bridgeSha256','bridge.rs'),('buildLogSha256','build.log')]:
    bind(CONSTRUCTOR / filename, build[key])
for b in build['effectiveCrateInputs']: bind(b['path'], b['sha256'])
assert len(construction['cases']) == 48
for c in construction['cases']:
    assert c['exitCode'] == 0 and c['deterministic']
    folder = CONSTRUCTOR / 'cases' / c['name']
    bind(folder / 'recipe.json',c['recipeSha256']); bind(folder / 'construct.log',c['logSha256'])
    for name,digest in c['artifacts'].items():
        for sub in ['', 'first', 'repeat']: bind(folder / sub / name,digest)
baseline = MODULE/'output/immutable-baseline-toolchain-r2'
baseline_manifest=read(baseline/'manifest.json')
assert baseline_manifest['baseline']=='6c7ac16617835b5f581784ff08a9e779bb52faf3'
for name,digest in baseline_manifest['files'].items():bind(baseline/name,digest)
bind(MODULE.parents[1]/'Cargo.toml',baseline_manifest['rootCargoTomlSha256'])
bind(MODULE.parents[1]/'Cargo.lock',baseline_manifest['rootCargoLockSha256'])
r = read(ROOT / 'receipt.json')
assert r['status'] == 'failed-private-derived-baseline'
assert r['browser'] == '153.0.8010.12' and r['effectiveMode'] == 'RasterOrdering'
for k,digest in r['toolHashes'].items(): bind(r['tools'][k],digest)
assert r['toolHashes']['probe'] == '2831265baa244ae03a26389fd82daaaa79eb88b77f542f6e8bc617343ca15b53'
assert r['toolHashes']['renderer'] == '276a49d810c05de7cfb0f0a4109853d12216da64e5d9d8e131f7400357f8e51f'
for b in r['sourceBindings']:
    bind(b['source'],b['sha256']);bind(ROOT / b['snapshot'],b['sha256'])
assert len(r['artifacts']) == 48 and len(r['rows']) == 384
for a in r['artifacts']:
    for key,filename in [('requestSha256','request.json'),('rivSha256','scene.riv'),('mapSha256','scene.map.json'),('probeManifestSha256','probe/frames.json')]:bind(ROOT / a['name'] / filename,a[key])
    expected=next(c for c in construction['cases'] if c['name']==a['name'])
    assert a['rivSha256']==expected['artifacts']['scene.riv'] and a['mapSha256']==expected['artifacts']['scene.map.json']
    for name,digest in expected['artifacts'].items():bind(ROOT / a['name'] / 'derived-construction' / name,digest)
for row in r['rows']:
    prefix=Path(row['prefix'])
    for kind in ['chrome','native']:bind(str(prefix)+'.'+kind+'.png',row[kind+'Sha256'])
    bind(str(prefix)+'.diff.png')
    frame=json.loads((prefix.parent/'probe/frames.json').read_text())['frames'][row['frame']]
    for kind in ['geometry','stream']:bind(prefix.parent/'probe'/frame[kind],row[kind+'Sha256'])
    assert not row['geometryFailures']
    if row['pixelFailures']:
        assert row['pixelFailures']==['mismatch ratio'] and row['step']==2 and 'responsive-visible' in row['name']
    for c in row['clearChecks']:bind(c['path'],c['sha256']);assert c['samePixels']
assert sum(not row['pixelFailures'] for row in r['rows'])==336
assert sum(len(row['clearChecks']) for row in r['rows'])==768
assert len(r['repeated'])==240 and all(x['nativeIdentical'] and x['chromeIdentical'] for x in r['repeated'])
# Recheck command observer outputs rather than trusting the stored success count.
import importlib.util
clip=read(ROOT/'clip-receipt.json');bind(clip['observer']['path'],clip['observer']['sha256'])
spec=importlib.util.spec_from_file_location('clip_observer',clip['observer']['path']);observer=importlib.util.module_from_spec(spec);spec.loader.exec_module(observer)
assert clip['frameCount']==384 and clip['clipCount']==2688 and clip['emptyDrawCount']==1152
for c,row in zip(clip['frames'],r['rows'],strict=True):
    assert (c['name'],c['frame'])==(row['name'],row['frame'])
    for k in ['stream','manifest']:bind(c[k]['path'],c[k]['sha256'])
    lines=observer.extract(Path(c['stream']['path']),row['frame'])
    observed=observer.observe(lines,row['width'],row['height'])
    assert observed['clips']==c['clips'] and observed['draws']==c['draws']
first=r['rows'][0]
assert observer.controls(observer.extract(Path(clip['frames'][0]['stream']['path']),0),first['width'],first['height'])==clip['negativeControls']
coverage=read(ROOT/'visual/coverage.json');sheets={s['path']:s for s in coverage['sheets']};inspected={}
for filename in ['inspection-agent-a.json','inspection-agent-b.json','inspection-root.json']:
    review=read(ROOT/'visual'/filename)
    for s in review['inspectedSheets']:
        assert s['path'] not in inspected and s['observation'].strip()
        assert s['sha256']==sheets[s['path']]['sha256'];bind(s['path'],s['sha256']);inspected[s['path']]=s
assert set(inspected)==set(sheets) and len(sheets)==36
for s in sheets.values():
    for member in s['members']:
        for b in member['files']:bind(b['path'],b['sha256'])
assert coverage['counts']==dict(frames=384,representatives=144,transfers=240,sheets=36,geometryPass=384,pixelPass=336)
for t in coverage['transfers']:
    row=next(x for x in r['rows'] if x['name']==t['name'] and x['frame']==t['frame'])
    donor=next(x for x in r['rows'] if x['name']==t['name'] and x['frame']==t['donorFrame'])
    for k in ['chrome','native']:
        bind(t[k]['path'],t[k]['sha256']);assert row[k+'Sha256']==donor[k+'Sha256']==t[k]['sha256']
    assert row['geometry']==donor['geometry'] and row['boxes']==donor['boxes']
visual=dict(scope='Direct inspection of 144 full Chrome/native pairs on 36 sheets; 240 exact same-case repeat/original-clone transfers. All 48 pixel failures retained; not public admission.',visualReviewCompleted=True,coverageSha256=bind(ROOT/'visual/coverage.json'),inspectedSheets=list(inspected.values()),counts=coverage['counts'])
(ROOT/'visual/review-receipt.json').write_text(json.dumps(visual,indent=2)+'\n');bind(ROOT/'visual/review-receipt.json')
for name in ['wrapped-derived-adapter.py','wrapped-derived-review.py','wrapped-derived-cases.py','wrapped-derived-cases.json','wrapped-derived-recipes.json','wrapped-derived-build.py','wrapped-derived-bridge.rs','wrapped-derived-construct.py','check-wrapped-derived-baseline.mjs','wrapped-derived-constructor-review.md','wrapped-derived-edge-audit.md','wrapped-derived-review.md']:
    bind(MODULE/'validation'/name)
result=dict(status='verified-private-campaign-with-retained-pixel-failures',publicAdmission=False,counts=coverage['counts'],rustTests=404,nodeTests=56,bindings=[dict(path=p,sha256=h)for p,h in sorted(bindings.items())])
out=MODULE/'output/wrapped-derived-verification-r1.json';out.write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(dict(status=result['status'],bindings=len(bindings),**result['counts'])))
