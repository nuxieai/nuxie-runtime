"""Replay each reviewed stream once, reusing a terminal predecessor's old-renderer proof.
Usage: SCRIPT TERMINAL_PREDECESSOR FINAL_TOOLCHAIN NEW_OUTPUT
Exactly 8052 source pairs are required. No browser/compiler/layout requalification.
"""
import hashlib,json,subprocess,sys,tempfile
from pathlib import Path
from PIL import Image

read=lambda p:json.loads(Path(p).read_text())
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
SUFFIXES={'stream','native.png','browser.png','json','riv','map.json','requirements.json','bounds.json'}

def frozen_manifest(path,expected_hash,renderer_hash):
    path=Path(path)
    assert sha(path)==expected_hash,'Changed renderer manifest'
    assert read(path)['files']['renderer-replay']['sha256']==renderer_hash,'Manifest renderer mismatch'
    binary=path.parent/'renderer-replay'
    assert sha(binary)==renderer_hash,'Changed renderer binary'
    return binary

def verify_predecessor(prior,expected=8052):
    """Read-only complete identity audit; production always uses 8052.
    The optional count exists for isolated negative tests, never a CLI override.
    """
    receipt=read(prior/'receipt.json')
    assert receipt['status']=='complete' and receipt['expected']==expected
    assert receipt['completed']==expected and receipt['reviewedPairs']==expected and receipt['remaining']==[]
    assert receipt['backend']=='rust-metal' and receipt['mode']=='clockwise-atomic'
    assert sha(prior/'rows.jsonl')==receipt['rowsSha256'],'Changed predecessor journal'
    assert sha(prior/'source-bindings.json')==receipt['sourceBindingsSha256'],'Changed predecessor bindings'
    source=Path(receipt['sourceGallery'])
    assert sha(source/'review.json')==receipt['sourceReviewSha256'],'Changed source review'
    assert sha(source/'combined-visual-comparison.json')==receipt['sourceVisualProofSha256'],'Changed visual proof'
    report=read(source/'review.json');proof=read(source/'combined-visual-comparison.json')
    assert report['status']=='passed' and report['errors']==[] and len(report['checks'])==report['expected']
    assert all(c['status']=='passed' and c['errors']==[] and c['project']=='native' for c in report['checks'])
    assert proof['status']=='complete' and proof['remaining']==[] and proof['reviewSha256']==receipt['sourceReviewSha256']
    assert proof['reviewedPairs']==proof['scenePairs']==len(report['cases'])==expected
    assert len({(c['name'],c['width']) for c in report['cases']})==expected
    for label in ['old','new']:
        frozen_manifest(receipt[label+'Manifest'],receipt[label+'ManifestSha256'],receipt[label+'RendererSha256'])
    bindings=read(prior/'source-bindings.json');rows=[json.loads(line) for line in (prior/'rows.jsonl').read_text().splitlines()]
    assert len(bindings)==len(rows)==expected
    for index,(binding,row,case) in enumerate(zip(bindings,rows,report['cases'])):
        assert binding['index']==row['index']==index,'Wrong row index'
        assert binding['name']==row['name']==case['name'],'Wrong source identity'
        assert binding['width']==row['width']==case['width'] and binding['height']==case['height']
        assert case['status']=='passed' and case['errors']==[]
        assert case['metrics']['backend']=='rust-metal' and case['metrics']['browserVersion']=='153.0.8010.12'
        assert row['reviewTransferred'] is True and 'failure' not in row and row.get('exitCode',0)==0
        assert set(binding['files'])==SUFFIXES
        for suffix,file in binding['files'].items():
            assert file['path']==case['artifactPrefix']+'.'+suffix,'Wrong artifact identity'
            assert sha(file['path'])==file['sha256'],'Changed source artifact'
        expected_png=binding['files']['native.png']['sha256']
        assert row['streamSha256']==binding['files']['stream']['sha256']
        assert row['expectedNativeSha256']==row['source-controlSha256']==row['new-rendererSha256']==expected_png,'Unproven source reproduction'
        output=Path(row['output'])
        assert output.resolve()==(prior/f'{index:05d}-new-renderer.native.png').resolve(),'Wrong predecessor output'
        assert sha(output)==expected_png,'Changed predecessor PNG'
        with Image.open(output) as image:assert image.size==(binding['width'],binding['height'])
    return receipt,bindings,rows

def audit_source_review(source):
    with tempfile.TemporaryDirectory(prefix='verified-migration-review-') as tmp:
        temp=Path(tmp);(temp/'review.json').write_bytes((source/'review.json').read_bytes())
        command=[sys.executable,str(Path(__file__).with_name('audit-gallery-transfer.py')),str(temp),str(source)]
        result=subprocess.run(command,capture_output=True,text=True)
        assert result.returncode==0,result.stdout+result.stderr
        proof=read(temp/'baseline-comparison.json')
        assert proof['remaining']==[] and proof['reviewedPairs']==8052
        return dict(command=command,result=proof)

def main(prior,toolchain,out):
    assert not out.exists(),'Refusing to overwrite evidence'
    old,bindings,_=verify_predecessor(prior)
    source_proof=audit_source_review(Path(old['sourceGallery']))
    manifest=toolchain/'manifest.json';digest=read(manifest)['files']['renderer-replay']['sha256'];manifest_hash=sha(manifest)
    binary=frozen_manifest(manifest,manifest_hash,digest)
    assert digest!=old['newRendererSha256'],'Expected final renderer to differ from predecessor'
    prior_hashes={name:sha(prior/name) for name in ['receipt.json','rows.jsonl','source-bindings.json']}
    fingerprint=lambda p:(p.stat().st_ino,p.stat().st_size,p.stat().st_mtime_ns)
    binary_stat=fingerprint(binary)
    out.mkdir(parents=True);(out/'source-bindings.json').write_bytes((prior/'source-bindings.json').read_bytes())
    (out/'source-review-audit.json').write_text(json.dumps(source_proof,indent=2)+'\n')
    base=dict(scope='Renderer-only exact-image migration; old-stream reproduction reused from terminal predecessor, all final streams actually replayed once; no new visual inspections.',
        predecessor=str(prior),predecessorHashes=prior_hashes,sourceGallery=old['sourceGallery'],sourceReviewSha256=old['sourceReviewSha256'],
        sourceVisualProofSha256=old['sourceVisualProofSha256'],sourceBindingsSha256=sha(out/'source-bindings.json'),
        sourceReviewAuditSha256=sha(out/'source-review-audit.json'),newManifest=str(manifest),newManifestSha256=manifest_hash,newRendererSha256=digest,
        backend='rust-metal',mode='clockwise-atomic',expected=8052,newVisualInspections=0,predecessorReplayReused=True)
    (out/'receipt.json').write_text(json.dumps({**base,'status':'running'},indent=2)+'\n')
    rows=[]
    with (out/'rows.jsonl').open('w') as journal:
        for binding in bindings:
            i=binding['index'];stream=Path(binding['files']['stream']['path']);expected=Path(binding['files']['native.png']['path'])
            assert sha(stream)==binding['files']['stream']['sha256'] and sha(expected)==binding['files']['native.png']['sha256']
            assert fingerprint(binary)==binary_stat,'Renderer changed during migration'
            png=out/f'{i:05d}-new-renderer.native.png'
            row=dict(index=i,name=binding['name'],width=binding['width'],height=binding['height'],streamSha256=sha(stream),expectedNativeSha256=sha(expected),reviewTransferred=False,output=str(png))
            try:
                result=subprocess.run([str(binary),'--stream',str(stream),'--output',str(png),'--backend','rust-metal','--mode','clockwise-atomic'],capture_output=True,text=True,timeout=120)
                if result.returncode or not png.exists():
                    (out/f'{i:05d}.log').write_text(result.stdout+result.stderr);row['failure']='Final renderer failed';row['exitCode']=result.returncode
                else:
                    with Image.open(png) as image:assert image.size==(binding['width'],binding['height'])
                    row['new-rendererSha256']=sha(png);row['reviewTransferred']=png.read_bytes()==expected.read_bytes()
                    if not row['reviewTransferred']:row['failure']='Final pixels changed; review outstanding'
            except subprocess.TimeoutExpired:row['failure']='Final renderer timed out'
            assert sha(stream)==binding['files']['stream']['sha256'],'Stream changed during replay'
            rows.append(row);journal.write(json.dumps(row)+'\n');journal.flush()
            if (i+1)%100==0:print(json.dumps(dict(completed=i+1,expected=8052,changed=sum(not r['reviewTransferred'] for r in rows))),flush=True)
    # Revalidate every actual source/old reproduction binding after the run.
    verify_predecessor(prior)
    final_source_audit=audit_source_review(Path(old['sourceGallery']))
    (out/'source-review-audit-final.json').write_text(json.dumps(final_source_audit,indent=2)+'\n')
    assert all(sha(prior/name)==value for name,value in prior_hashes.items()),'Predecessor changed during replay'
    frozen_manifest(manifest,manifest_hash,digest)
    for row in rows:
        if row['reviewTransferred']:assert sha(row['output'])==row['new-rendererSha256'],'Final image changed'
    remaining=[dict(name=r['name'],width=r['width'],reason=r.get('failure')) for r in rows if not r['reviewTransferred']]
    receipt={**base,'finalSourceReviewAuditSha256':sha(out/'source-review-audit-final.json'),'status':'changed-or-failed' if remaining else 'complete','completed':len(rows),'reviewedPairs':len(rows)-len(remaining),'remaining':remaining,'rowsSha256':sha(out/'rows.jsonl')}
    (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
    print(json.dumps({k:receipt[k] for k in ['status','completed','reviewedPairs']}));return bool(remaining)

if __name__=='__main__':
    assert len(sys.argv)==4,__doc__
    sys.exit(main(*(Path(p).resolve() for p in sys.argv[1:])))
