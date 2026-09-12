"""Read-only measurement collector for the private snapped sizing experiment.

Run only after native capture completes. Writes evidence, never scenes or references.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path
from PIL import Image

BASE = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('root', nargs='?', type=Path, default=BASE/'output/wrapped-sizing-snapped-r1')
args = parser.parse_args()
root = args.root.resolve()
old = BASE/'output/wrapped-snapped-layout-r1'
bindings = {}
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def bind(p, expected=None):
    p=Path(p).resolve(); actual=sha(p)
    assert expected is None or actual==expected, ('changed binding',str(p),expected,actual)
    bindings[str(p)]=actual
    return actual
def read(p):
    bind(p);return json.loads(Path(p).read_text())
def write(name,value):
    p=root/name;p.write_text(json.dumps(value,indent=2)+'\n');return bind(p)
def rgba(p):
    with Image.open(p) as im:
        im=im.convert('RGBA');return im.size,im.tobytes()
def finite(value):
    if isinstance(value,float):assert math.isfinite(value)
    elif isinstance(value,dict):
        for v in value.values():finite(v)
    elif isinstance(value,list):
        for v in value:finite(v)

build=read(root/'build-receipt.json')
assert build['epsilon']==1/64 and build['epsilonIsAdmissionCertificate'] is False
assert build['bindings']
for b in build['bindings']:
    bind(b['path'],b['sha256'])
    if 'snapshot' in b:bind(b['snapshot'],b['sha256'])
observer=read(BASE/'validation/wrapped-snapped-gate-receipt.json')['observer']
for file,key in [('node-probe','binarySha256'),('node-probe.rs','sourceSha256')]:bind(BASE/'output/wrapped-snapped-gate-r1'/file,observer[key])
bind(BASE/'output/immutable-baseline-toolchain-r2/baseline-probe.rs',observer['baselineObserverSha256'])
assert observer['exactlyTwoObservationSubstitutionsVerified']
for file,expected in observer['immutableDependencyIdentity'].items():bind(BASE/'output/immutable-baseline-toolchain-r2'/file,expected)
native=read(root/'render/receipt.json'); prior=read(old/'render/receipt.json')
finite(native);finite(prior)
cases=read(root/'pairs.json');old_cases=read(old/'pairs.json');assert cases==old_cases
assert read(root/'chrome-cases.json')==read(old/'chrome-cases.json')
assert len(cases)==48 and len({c['name'] for c in cases})==48
names={c['name'] for c in cases};by_case={c['name']:c for c in cases}
for receipt in [native,prior]:
    assert receipt['browser']=='153.0.8010.12'
    assert receipt['backend']=='rust-metal' and receipt['cliModeToken']=='clockwise-atomic'
    assert receipt['effectiveMode']=='RasterOrdering'
    for tool,expected in [('probe','2831265baa244ae03a26389fd82daaaa79eb88b77f542f6e8bc617343ca15b53'),('renderer','276a49d810c05de7cfb0f0a4109853d12216da64e5d9d8e131f7400357f8e51f')]:
        assert receipt['toolHashes'][tool]==expected;bind(receipt['tools'][tool],expected)
    bind(receipt['tools']['compiler'],receipt['toolHashes']['compiler'])
    assert len(receipt['rows'])==384
    assert {(r['name'],r['frame']) for r in receipt['rows']}=={(n,f) for n in names for f in range(8)}
    assert {a['name'] for a in receipt['artifacts']}==names and len(receipt['artifacts'])==48
for folder,receipt in [(root,native),(old,prior)]:
    for b in receipt['sourceBindings']:bind(folder/'render'/b['snapshot'],b['sha256'])
    for artifact in receipt['artifacts']:
        d=folder/'render'/artifact['name']
        for file,key in [('request.json','requestSha256'),('scene.riv','rivSha256'),('scene.map.json','mapSha256'),('probe/frames.json','probeManifestSha256')]:bind(d/file,artifact[key])

# Snapshot/build source and dependency manifests belong to the new construction,
# separate from the retained old visual qualification. Bind every non-render file;
# recursive manifest bindings below verify explicit external identities too.
for p in sorted(root.rglob('*')):
    if p.is_file() and 'render' not in p.relative_to(root).parts and p.name not in ['sizing-observations.json','sizing-pixel-identity.json','sizing-visual-transfer.json','sizing-evidence.json']:
        bind(p)
def external(document):
    if isinstance(document,list):
        for v in document:external(v)
    elif isinstance(document,dict):
        for key in ['path','source','snapshot']:
            if isinstance(document.get(key),str) and isinstance(document.get('sha256'),str):
                p=Path(document[key]);p=p if p.is_absolute() else root/p
                if p.exists():bind(p,document['sha256'])
        for key,value in document.items():
            if isinstance(value,str) and len(value)==64 and all(c in '0123456789abcdef' for c in value):
                p=Path(key)
                if p.is_absolute() and p.exists():bind(p,value)
            external(value)
for p in root.glob('*.json'):
    if p.name not in ['sizing-observations.json','sizing-pixel-identity.json','sizing-visual-transfer.json','sizing-evidence.json']:external(read(p))
bind(Path(__file__))

reproductions=read(root/'reproductions.json')
assert len(reproductions)==48 and {r['name'] for r in reproductions}==names
for reproduction in reproductions:
    assert reproduction['exactRiv'] and reproduction['exactMap'] and reproduction['exactTrace']
    for key in ['request','originalRiv','resultRiv','originalMap','resultMap']:
        b=reproduction[key];bind(b['path'],b['sha256'])
    assert Path(reproduction['originalRiv']['path']).read_bytes()==Path(reproduction['resultRiv']['path']).read_bytes()
    assert read(Path(reproduction['originalMap']['path']))==read(Path(reproduction['resultMap']['path']))
    first_trace=Path(reproduction['originalRiv']['path']).with_suffix('.sizing-trace.json')
    second_trace=Path(reproduction['resultRiv']['path']).with_suffix('.sizing-trace.json')
    assert read(first_trace)==read(second_trace)

old_rows={(r['name'],r['frame']):r for r in prior['rows']}
observations=[];pixels=[];failures=[]
def check(condition,message,record):
    if not condition:record.append(message)
def near(actual,expected,label,errors):check(abs(actual-expected)<=0.1,f'{label}: {actual} vs {expected}',errors)
for r in native['rows']:
    name=r['name'];case=by_case[name];f=r['frame'];before=old_rows[name,f];d=root/'render'/name;old_d=old/'render'/name
    assert r['instance']==f//4 and r['step']==f%4
    assert [r['width'],r['height']]==case['viewports'][f%4]
    assert r['requestSha256']==before['requestSha256'];bind(d/'request.json',r['requestSha256'])
    for rawfile in ['scene.raw.riv','scene.raw.map.json']:
        assert bind(d/rawfile)==bind(old_d/rawfile),('raw seed changed',name,rawfile)
    bind(d/'scene.sized.riv')
    trace=read(d/'scene.sizing-trace.json');mapping={n['id']:n['object_id'] for n in read(d/'scene.raw.map.json')}
    geometry={n['objectId']:n for n in r['geometry']};assert len(geometry)==len(r['geometry'])
    bind(d/'probe'/f'frame-{f}.geometry.json',r['geometrySha256']);bind(d/'probe'/r['stream'],r['streamSha256'])
    assert read(d/'probe'/f'frame-{f}.geometry.json')==r['geometry']
    cross=5 if case['axis']=='row' else 4;extent='height' if case['axis']=='row' else 'width';main='width' if case['axis']=='row' else 'height'
    scalar=lambda node:geometry[node]['worldMatrix'][cross]
    ids=['b','a','c'];slots=['sb','sa','sc'];boxes=r['boxes'];errors=[]
    assert set(boxes)=={'p','a','b','c','oa','ob','oc','footer'}
    assert len(trace['boundaries'])==2
    for key in ['tops','heights','anchors','forward','backward','line_maxima','offsets','targets']:assert len(trace[key])==3
    # Independent browser packing oracle: all three slots have fixed main size,
    # no margins/gap/padding; logical order b,a,c survives reversed main flow.
    lines=[];line=0;used=0.
    for identifier in ids:
        size=boxes[identifier][main]
        if used and used+size>boxes['p'][main]+0.00001:line+=1;used=0.
        lines.append(line);used+=size
    heights=[boxes[n][extent] for n in ids]
    expected_forward=[max(heights[j] for j in range(i+1) if lines[j]==lines[i]) for i in range(3)]
    expected_backward=[max(heights[j] for j in range(i,3) if lines[j]==lines[i]) for i in range(3)]
    values={key:[scalar(node) for node in trace[key]] for key in ['tops','heights','anchors','forward','backward','line_maxima','offsets','targets']}
    fraction=case['level']/2
    if case['wrapValue']==2:fraction=1-fraction
    for i,identifier in enumerate(ids):
        slot=geometry[mapping[slots[i]]];top=slot['worldMatrix'][cross]
        near(values['tops'][i],top,f'top{i}',errors);near(values['heights'][i],slot[extent],f'slot height{i}',errors)
        near(values['heights'][i],heights[i],f'Chrome height{i}',errors)
        near(values['anchors'][i],top+fraction*slot[extent],f'anchor{i}',errors)
        near(values['forward'][i],expected_forward[i],f'forward{i}',errors)
        near(values['backward'][i],expected_backward[i],f'backward{i}',errors)
        maximum=max(expected_forward[i],expected_backward[i]);near(values['line_maxima'][i],maximum,f'maximum{i}',errors)
        alignment=[1.,.5,0.][i]
        if case['wrapValue']==2:alignment=1-alignment
        offset=(maximum-heights[i])*(alignment-fraction)
        near(values['offsets'][i],offset,f'offset{i}',errors)
        near(values['targets'][i],top+offset,f'target{i}',errors)
        near(geometry[mapping[identifier]]['worldMatrix'][cross],top+offset,f'visible target{i}',errors)
    gates=[]
    for i,boundary in enumerate(trace['boundaries']):
        measured={key:scalar(node) for key,node in boundary.items()};expected=65536 if lines[i]!=lines[i+1] else 0
        check(measured['gate']==expected,f'gate{i}: {measured["gate"]} vs {expected}',errors)
        gates.append({'expected':expected,**measured})
    current_map={n['id']:n['object_id'] for n in read(d/'scene.map.json')};old_map={n['id']:n['object_id'] for n in read(old_d/'scene.map.json')};old_geometry={g['objectId']:g for g in before['geometry']}
    assert set(current_map)==set(old_map)==set(boxes)
    named=lambda m,g:{key:{k:v for k,v in g[node].items() if k!='objectId'} for key,node in m.items()}
    named_same=named(current_map,geometry)==named(old_map,old_geometry)
    browser_same=all(r[key]==before[key] for key in ['boxes','computedStyles','swatchObservations'])
    comparisons=[]
    for engine in ['chrome','native']:
        new=Path(r['prefix']+'.'+engine+'.png');previous=Path(before['prefix']+'.'+engine+'.png')
        a=bind(new,r[engine+'Sha256']);b=bind(previous,before[engine+'Sha256'])
        comparisons.append({'engine':engine,'new':str(new),'old':str(previous),'newSha256':a,'oldSha256':b,'fullPngIdentical':a==b,'decodedRgbaIdentical':rgba(new)==rgba(previous)})
    assert len(r['clearChecks'])==2 and {c['name'] for c in r['clearChecks']}=={'cyan','transparent'}
    for clear in r['clearChecks']:
        bind(clear['path'],clear['sha256']);check(clear['samePixels'] and rgba(clear['path'])==rgba(Path(r['prefix']+'.native.png')),'clear '+clear['name'],errors)
    pixels.append({'name':name,'frame':f,'images':comparisons,'namedGeometryIdentical':named_same,'browserMeasurementsIdentical':browser_same})
    observations.append({'name':name,'frame':f,'lines':lines,'expectedForward':expected_forward,'expectedBackward':expected_backward,'values':values,'gates':gates,'errors':errors})
    if errors or r['geometryFailures'] or r['pixelFailures']:failures.append({'name':name,'frame':f,'sizingFailures':errors,'geometryFailures':r['geometryFailures'],'pixelFailures':r['pixelFailures']})

flips=[]
for name in sorted(names):
    seq=sorted((r for r in observations if r['name']==name),key=lambda r:r['frame'])
    assert [r['frame'] for r in seq]==list(range(8))
    for i,j in [(0,3),(0,4),(1,5),(2,6),(0,7)]:assert seq[i]['values']==seq[j]['values'] and seq[i]['gates']==seq[j]['gates'],('clone/repeat sizing changed',name,i,j)
    signals=[[g['gate'] for g in r['gates']] for r in seq]
    flips.append({'name':name,'sequence':signals,'changesDuringResize':any(v!=signals[0] for v in signals[1:4]),'cloneMatchesOriginal':signals[:4]==signals[4:]})
assert all(f['changesDuringResize'] for f in flips)
changed=[{'name':r['name'],'frame':r['frame']} for r in pixels if not(all(i['decodedRgbaIdentical'] for i in r['images']) and r['namedGeometryIdentical'] and r['browserMeasurementsIdentical'])]
# Preserve the complete reviewed chain, including its original direct/transfer
# ledger. Historical paths are repository-relative, unlike the newer bindings.
for p in [BASE/'validation/wrapped-snapped-layout-receipt.json',BASE/'validation/wrapped-combined-slots-receipt.json',old/'visual-review-transfer.json',old/'prior-pixel-identity.json',BASE/'output/wrapped-combined-optimized-r1/visual-review-transfer.json']:
    doc=read(p)
    for key,value in (doc.get('hashes',{}) if isinstance(doc,dict) else {}).items():
        q=Path(key);q=q if q.is_absolute() else (BASE.parents[1]/q if key.startswith('tools/') else Path(doc.get('outputRoot',p.parent))/q)
        bind(q,value)
    # Historical build-library paths may now refer to newer builds; retained
    # scene, reference, and review artifacts are the visual identity authority.
# Independently reconnect every old snapped pair to the original reviewed
# pixels, instead of trusting only a previous transfer's boolean summary.
original_root=BASE/'output/wrapped-combined-slots-r1'
original_native=read(original_root/'render/receipt.json')
original_rows={(r['name'],r['frame']):r for r in original_native['rows']}
direct=read(original_root/'visual-direct.json');transferred=read(original_root/'visual-transfer-verified.json')
assert len(direct)==96 and len(transferred)==288
assert {(r['name'],r['frame']) for r in direct+transferred}==set(old_rows)
for row in direct:
    for engine in ['chrome','native']:bind(row['prefix']+'.'+engine+'.png',row[engine+'Sha256'])
for row in transferred:
    donor=direct[row['directIndex']];target=original_rows[row['name'],row['frame']]
    for i,engine in enumerate(['chrome','native']):
        a=Path(target['prefix']+'.'+engine+'.png');b=Path(donor['prefix']+'.'+engine+'.png')
        bind(a,target[engine+'Sha256']);size,data=rgba(a)
        assert [list(size),hashlib.sha256(data).hexdigest()]==row['rgba'][i]
        assert (size,data)==rgba(b)
for key,row in old_rows.items():
    original=original_rows[key]
    for engine in ['chrome','native']:
        p=Path(original['prefix']+'.'+engine+'.png');bind(p,original[engine+'Sha256'])
        assert rgba(p)==rgba(Path(row['prefix']+'.'+engine+'.png'))
visual={'scope':'Exact prior reviewed visual transfer only; no new direct inspection claim. Changed pairs require fresh direct review.','allTransferred':not changed,'transferredPairs':384-len(changed),'changedPairs':changed,'priorDirectPairs':96,'priorExactTransfers':288,'priorReview':str(old/'visual-review-transfer.json')}
counts={'cases':48,'frames':384,'sizingGates':sum(len(r['gates']) for r in observations),'forwardValues':1152,'backwardValues':1152,'lineMaximumValues':1152,'offsetValues':1152,'sizingPass':sum(not r['errors'] for r in observations),'geometryPass':sum(not r['geometryFailures'] for r in native['rows']),'pixelPass':sum(not r['pixelFailures'] for r in native['rows']),'clearPass':sum(c['samePixels'] for r in native['rows'] for c in r['clearChecks']),'changedPairs':len(changed),'failureRows':len(failures)}
write('sizing-observations.json',{'rows':observations,'resizes':flips})
write('sizing-pixel-identity.json',pixels)
write('sizing-visual-transfer.json',visual)
write('sizing-evidence.json',{'scope':'Private experimental epsilon1/64 sizing discriminator, no public admission or general threshold certificate','counts':counts,'failures':failures,'bindings':bindings})
print(json.dumps(counts))
