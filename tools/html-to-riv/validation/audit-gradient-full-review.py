"""Transfer a terminal native full gallery from audited baseline and gradient reviews.
Usage: SCRIPT TARGET BASELINE INITIAL_PUBLIC_REPLAY COMPOSITION_REPLAY
No visual inspections are performed or invented. Changed pairs remain outstanding.
"""
import hashlib
import json
from pathlib import Path
import re
import runpy
import subprocess
import sys
import tempfile

read = lambda p: json.loads(Path(p).read_text())
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()
canonical = lambda value: hashlib.sha256(json.dumps(value,sort_keys=True,separators=(',',':')).encode()).hexdigest()


def validate_gallery(report, expected_keys):
    assert report['status']=='passed' and not report['errors']
    assert len(report['checks'])==report['expected']
    assert all(c['status']=='passed' and not c['errors'] and c['project']=='native' for c in report['checks'])
    keys=[(c['name'],c['width']) for c in report['cases']]
    assert len(keys)==len(set(keys)) and set(keys)==expected_keys
    assert all(c['status']=='passed' and not c['errors'] and c['metrics']['backend']=='rust-metal' for c in report['cases'])


def verify_bindings(bindings):
    for binding in bindings:
        assert sha(binding['path'])==binding['sha256'], binding['path']


def verify_images(row):
    for kind in ['browser','native']:
        assert sha(row['prefix']+'.'+kind+'.png')==row[kind+'Sha256']


def verify_transport(reference_prefix, target_prefix):
    assert Path(str(reference_prefix)+'.riv').read_bytes()==Path(str(target_prefix)+'.riv').read_bytes()
    for suffix in ['.map.json','.requirements.json']:
        assert read(str(reference_prefix)+suffix)==read(str(target_prefix)+suffix)


def main(target, baseline, initial, composition):
    scripts = Path(__file__).parent
    report = read(target/'review.json')
    baseline_keys={(c['name'],c['width']) for c in read(baseline/'review.json')['cases']}
    initial_names={c['name'] for c in read(scripts.parent/'tests/assets/linear-gradient-initial-oracle.json')['cases']}
    composition_names={c['name'] for c in read(scripts/'linear-gradient-composition-cases.json') if c['expectedCompile']['status']=='accepted'}
    gradient_keys={(name,width) for name in initial_names|composition_names for width in [240,390,768]}
    assert len(baseline_keys)==7950 and len(gradient_keys)==102 and not baseline_keys & gradient_keys
    validate_gallery(report,baseline_keys|gradient_keys)
    subprocess.run([sys.executable,str(scripts/'audit-gallery-transfer.py'),str(target),str(baseline)],check=True)
    baseline_proof=read(target/'baseline-comparison.json')
    assert baseline_proof['reviewSha256']==sha(target/'review.json')
    # Replay the existing diagnostic-to-public review audit, including its public
    # compiler provenance audit, rather than treating its stored status as proof.
    initial_proof=read(initial/'visual-public-gradient-transfer.json')
    verify_bindings(initial_proof['evidence'])
    bindings=[Path(e['path']) for e in initial_proof['evidence']]
    recording=next(p.parent for p in bindings if p.name=='lifecycle.json')
    toolchain=next(p.parent for p in bindings if p.name=='manifest.json')
    compiler=toolchain/'html-to-riv'
    transfer=runpy.run_path(str(scripts/'transfer-public-gradient-review.py'))['transfer']
    transfer(initial_proof['sourceReplay'],initial,recording,compiler,toolchain,audit=True)
    composition_audit=runpy.run_path(str(scripts/'audit-gradient-composition.py'))['audit'](composition)
    assert composition_audit==read(composition/'composition-artifact-audit.json')
    known={}
    def add(name,row,source,request):
        verify_images(row)
        assert row['html']==request['html'] and row['css']==request['css']
        identity=(name,row['width'],canonical(request),row['browserSha256'],row['nativeSha256'])
        known[identity]=source
    for row in read(initial/'replay.json')['cases']:
        match=re.fullmatch(r'(linear-gradient-.+)-[01]-step[0-7]',row['name']);assert match
        assert row['qualification']=='public-compiler-lifecycle' and not row['geometryFailures'] and not row['failures']
        add(match[1],row,str(initial/'visual-public-gradient-transfer.json'),dict(html=row['html'],css=row['css'],width=390,height=320))
    for row in read(composition/'replay.json')['cases']:
        request=read(composition/(row['name']+'.json'))
        add(row['name'],row,str(composition/'composition-artifact-audit.json'),request)
    assert len({(k[0],k[1]) for k in known})==102
    gradient_rows=[]
    with tempfile.TemporaryDirectory(prefix='gradient-full-review-') as temp:
        temp=Path(temp)
        for row in report['cases']:
            if not row['name'].startswith('linear-gradient-'):continue
            prefix=Path(row['artifactPrefix'])
            request=read(str(prefix)+'.json')
            assert request['html']==row['html'] and request['css']==row['css']
            hashes={kind+'Sha256':sha(str(prefix)+'.'+kind+'.png') for kind in ['browser','native']}
            identity=(row['name'],row['width'],canonical(request),hashes['browserSha256'],hashes['nativeSha256'])
            source=known.get(identity)
            # Recompile each public request and compare actual target transport,
            # preventing image identity from masking a missing/wrong requirement.
            (temp/'request.json').write_text(json.dumps(request))
            subprocess.run([str(compiler),str(temp/'request.json'),str(temp/'scene.riv')],check=True,capture_output=True)
            verify_transport(temp/'scene',prefix)
            gradient_rows.append(dict(name=row['name'],width=row['width'],compilerInputSha256=canonical(request),**hashes,reviewTransferred=source is not None,sourceReceipt=source))
    assert len(gradient_rows)==102
    component=dict(scenePairs=len(report['cases']),reviewedPairs=sum(r['reviewTransferred'] for r in gradient_rows),
        remaining=[r for r in gradient_rows if not r['reviewTransferred']],rows=gradient_rows,reviewSha256=sha(target/'review.json'),
        sourceReceipts=[dict(path=str(p),sha256=sha(p)) for p in [initial/'visual-public-gradient-transfer.json',initial/'gradient-public-artifact-audit.json',composition/'composition-artifact-audit.json',composition/'composition-visual-inspection.json']],
        method='Audited public provenance, exact compiler request and complete Chrome/native image pairs. No new visual inspections.')
    save(target/'gradient-visual-comparison.json',component)
    combined_rows={}
    for filename,proof in [('baseline-comparison.json',baseline_proof),('gradient-visual-comparison.json',component)]:
        for row in proof['rows']:
            k=row['name'],row['width']
            if row['reviewTransferred']:
                assert k not in combined_rows,'Overlapping review provenance'
                combined_rows[k]={**row,'sourceComparison':filename}
    remaining=[dict(name=c['name'],width=c['width']) for c in report['cases'] if (c['name'],c['width']) not in combined_rows]
    combined=dict(status='partial' if remaining else 'complete',scenePairs=len(report['cases']),reviewedPairs=len(combined_rows),remaining=remaining,
        reviewSha256=sha(target/'review.json'),newVisualInspections=0,
        sourceComparisons=[dict(path=str(target/f),sha256=sha(target/f)) for f in ['baseline-comparison.json','gradient-visual-comparison.json']],rows=list(combined_rows.values()))
    save(target/'combined-visual-comparison.json',combined)
    return {k:combined[k] for k in ['status','scenePairs','reviewedPairs','remaining','newVisualInspections']}


def save(path,value):
    if path.exists():assert read(path)==value,'Refusing to overwrite changed evidence'
    else:path.write_text(json.dumps(value,indent=2)+'\n')

if __name__=='__main__':
    assert len(sys.argv)==5,__doc__
    print(json.dumps(main(*(Path(p).resolve() for p in sys.argv[1:]))))
