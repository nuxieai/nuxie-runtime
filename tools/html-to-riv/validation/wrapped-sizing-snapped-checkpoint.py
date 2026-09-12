"""Verify the private snapped sizing checkpoint without rebuilding or rendering."""
import hashlib
import json
import re
from pathlib import Path

MODULE=Path(__file__).resolve().parents[1]
OUTPUT=MODULE/'output'
ROOT=OUTPUT/'wrapped-sizing-snapped-r1'


def verify():
    checked={}
    def bind(path,expected=None):
        path=Path(path).resolve();digest=hashlib.sha256(path.read_bytes()).hexdigest()
        assert expected is None or digest==expected, f'Changed artifact: {path}'
        checked[str(path)]=digest
        return digest
    def read(path):
        bind(path);return json.loads(Path(path).read_text())
    def bindings(rows,live=False):
        for row in rows:
            bind(row.get('snapshot',row['path']),row['sha256'])
            if live:bind(row['path'],row['sha256'])
    def records(path):
        # Count actual ordinary records, including schema Bool fields omitted
        # from the ToC, rather than trusting a cost asserted by the producer.
        data=Path(path).read_bytes();assert data[:7]==b'RIVE\x07\x03\x00';pos=7
        def uint():
            nonlocal pos
            result=shift=0
            while True:
                byte=data[pos];pos+=1;result|=(byte&127)<<shift
                if byte<128:return result
                shift+=7;assert shift<35
        fields=[]
        while field:=uint():fields.append(field)
        kinds={}
        for start in range(0,len(fields),4):
            word=int.from_bytes(data[pos:pos+4],'little');pos+=4
            for offset,key in enumerate(fields[start:start+4]):kinds[key]=(word>>(2*offset))&3
        count=0
        while pos<len(data):
            uint();count+=1
            while key:=uint():
                if key not in kinds:assert data[pos] in [0,1];pos+=1
                elif kinds[key]==0:uint()
                elif kinds[key]==1:
                    length=uint();pos+=length
                else:pos+=4
        assert pos==len(data)
        return count

    evidence=read(ROOT/'sizing-evidence.json')
    assert len(evidence['bindings'])==5400
    for path,digest in evidence['bindings'].items():bind(path,digest)
    collector=MODULE/'validation/wrapped-sizing-snapped-evidence.py'
    assert str(collector) in evidence['bindings']
    assert not evidence['failures']
    expected={'cases':48,'frames':384,'sizingGates':768,'forwardValues':1152,'backwardValues':1152,
        'lineMaximumValues':1152,'offsetValues':1152,'sizingPass':384,'geometryPass':384,
        'pixelPass':384,'clearPass':768,'changedPairs':0,'failureRows':0}
    assert evidence['counts']==expected
    private=read(ROOT/'build-receipt.json')
    assert private['epsilon']==1/64 and private['epsilonIsAdmissionCertificate'] is False
    bindings(private['bindings'],live=True);bindings(private['artifacts'])
    observations=read(ROOT/'sizing-observations.json')
    assert len(observations['rows'])==384 and len(observations['resizes'])==48
    keys={(row['name'],row['frame']) for row in observations['rows']}
    assert len(keys)==384
    for row in observations['rows']:
        assert not row['errors'] and len(row['gates'])==2 and len(row['lines'])==3
        for i,gate in enumerate(row['gates']):
            expected_gate=0 if row['lines'][i]==row['lines'][i+1] else 65536
            assert gate['gate']==gate['expected']==expected_gate
        for field in ['forward','backward','line_maxima','offsets']:assert len(row['values'][field])==3
    assert all(r['changesDuringResize'] and r['cloneMatchesOriginal'] for r in observations['resizes'])
    pixels=read(ROOT/'sizing-pixel-identity.json')
    assert len(pixels)==384 and {(r['name'],r['frame']) for r in pixels}==keys
    for row in pixels:
        assert row['namedGeometryIdentical'] and row['browserMeasurementsIdentical']
        assert len(row['images'])==2 and {i['engine'] for i in row['images']}=={'chrome','native'}
        for image in row['images']:
            assert image['fullPngIdentical'] and image['decodedRgbaIdentical']
            assert image['newSha256']==image['oldSha256']
            bind(image['new'],image['newSha256']);bind(image['old'],image['oldSha256'])
    visual=read(ROOT/'sizing-visual-transfer.json')
    assert visual['allTransferred'] and visual['transferredPairs']==384 and visual['changedPairs']==[]
    assert (visual['priorDirectPairs'],visual['priorExactTransfers'])==(96,288)
    bind(visual['priorReview'])
    native=read(ROOT/'render/receipt.json')
    assert native['browser']=='153.0.8010.12' and native['backend']=='rust-metal'
    assert native['cliModeToken']=='clockwise-atomic' and native['effectiveMode']=='RasterOrdering'
    assert native['toolHashes']['probe']=='2831265baa244ae03a26389fd82daaaa79eb88b77f542f6e8bc617343ca15b53'
    assert native['toolHashes']['renderer']=='276a49d810c05de7cfb0f0a4109853d12216da64e5d9d8e131f7400357f8e51f'
    assert len(native['rows'])==384 and {(r['name'],r['frame']) for r in native['rows']}==keys
    for row in native['rows']:
        assert not row['geometryFailures'] and not row['pixelFailures']
        assert len(row['clearChecks'])==2 and all(c['samePixels'] for c in row['clearChecks'])
    assert len(native['artifacts'])==48
    costs=[]
    for artifact in native['artifacts']:
        folder=ROOT/'render'/artifact['name']
        raw,sized,final=[records(folder/name) for name in ['scene.raw.riv','scene.sized.riv','scene.riv']]
        assert (raw,sized,final)==(43,193,342)
        assert sized-raw==61*3-33==150 and final-sized==149 and final-raw==299
        costs.append({'case':artifact['name'],'raw':raw,'sized':sized,'final':final})

    build=OUTPUT/'wrapped-sizing-snapped-build-r1'
    summary=read(build/'summary.json')
    assert summary['status']=='public-build-and-transport-pass' and summary['sourceUnchanged']
    assert (summary['rustTests'],summary['nodeTests'])==(361,56)
    checks=read(build/'checks.json');assert len(checks)==6
    names={'full-rust','native-build','wasm-build','typescript','full-node','runtime-guard'}
    assert {c['name'] for c in checks}==names
    for check in checks:assert check['exitCode']==0;bind(check['log'],check['logSha256'])
    logs={c['name']:Path(c['log']).read_text() for c in checks}
    rust=re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed',logs['full-rust'])
    assert rust and sum(int(passed) for _,passed,_ in rust)==361
    assert all(status=='ok' and failed=='0' for status,_,failed in rust)
    assert all(s in logs['full-node'] for s in ['ℹ tests 56','ℹ pass 56','ℹ fail 0'])
    guard=json.loads(logs['runtime-guard'])
    assert guard['status']=='pass' and guard['baseline']=='6c7ac16617835b5f581784ff08a9e779bb52faf3'
    assert guard['baselineTree']=='25ccbb131d88dbd0fdde8f4c660919143b252edf'
    assert guard['changesOutsideCompiler']=={'index':[],'worktree':[],'untracked':[]}
    sources=read(build/'frozen/source-bindings.json')['files'];assert len(sources)==288;bindings(sources,live=True)
    prior_build=read(OUTPUT/'public-image-mixed-build-r1/summary.json')
    for file,key in [('html-to-riv','compilerSha256'),('compiler.wasm','wasmSha256')]:
        assert summary[key]==prior_build[key]
        bind(build/'frozen'/file,summary[key]);bind(OUTPUT/'public-image-mixed-build-r1/frozen'/file,summary[key])

    old=read(OUTPUT/'wrapped-sizing-snapped-existing-r1/results/manifest.json')
    assert old['total']==old['passed']==282
    bind(old['compiler'],summary['compilerSha256']);bind(old['priorManifest'],old['priorManifestSha256'])
    refs=read(old['priorManifest']);assert len(refs['results'])==282;bindings(refs['bindings'])
    for row in old['results']:
        assert row['exact'];bind(row['request'],row['requestSha256'])
        for file,key,prior in [('scene.riv','rivSha256','priorRiv'),('scene.map.json','mapSha256','priorMap')]:
            bind(Path(row['result'])/file,row[key]);bind(row[prior],row[key])
    history_root=OUTPUT/'public-transport-malformed-wrapped-sizing-regression-r1'
    history=read(history_root/'receipt.json')
    assert history['total']==history['passed']==794 and history['allExact']
    assert history['inputHashes']['html-to-riv']==summary['compilerSha256']
    for row in history['results']:
        assert row['exact'];reference=row['reference']
        for file,digest in reference['hashes'].items():bind(Path(reference['reference'])/file,digest)
        for file,digest in row['actualHashes'].items():
            assert digest==reference['hashes'][file];bind(history_root/'results'/str(row['index'])/file,digest)
    threshold=read(MODULE/'validation/wrapped-snapped-gate-receipt.json')
    assert threshold['totals']['frames']==88 and threshold['totals']['exactSignals']==80
    assert threshold['totals']['pixelsPassed']==80
    bind(MODULE/'validation/wrapped-sizing-certificate-next.md')
    bind(Path(__file__))
    return {'scope':'Private snapped sizing experiment only; epsilon1/64 is not a public semantic certificate.',
        'counts':expected,'recordCosts':costs,'publicBuild':summary,'sourceBindings':288,
        'unchangedPublicImages':282,'unchangedHistoricalOutputs':794,
        'visualTransfer':visual,'preservedThresholdTransitionFailures':8,
        'bindings':[{'path':path,'sha256':digest} for path,digest in sorted(checked.items())]}


if __name__=='__main__':
    result=verify();folder=OUTPUT/'wrapped-sizing-snapped-checkpoint-r1';folder.mkdir(exist_ok=True);destination=folder/'verification.json'
    destination.write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({'counts':result['counts'],'publicRustTests':361,'publicNodeTests':56,'artifactBindings':len(result['bindings'])}))
