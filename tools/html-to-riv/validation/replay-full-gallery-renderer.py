"""Exact native-image migration of an already reviewed terminal full gallery.
Usage: SCRIPT SOURCE_GALLERY OLD_TOOLCHAIN NEW_TOOLCHAIN NEW_OUTPUT
Replays retained streams only: no Chrome, compiler, or layout requalification.
Old renderer replay proves each stream still produces its reviewed source image.
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
from PIL import Image

read=lambda p:json.loads(Path(p).read_text())
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()


def frozen(directory):
    manifest_path=directory/'manifest.json';manifest=read(manifest_path)
    binary=directory/'renderer-replay';digest=manifest['files']['renderer-replay']['sha256']
    assert sha(binary)==digest,'Stale frozen renderer'
    return binary,digest,manifest_path,sha(manifest_path)


def main(source,old_toolchain,new_toolchain,out):
    assert not out.exists(),'Refusing to overwrite migration evidence'
    report=read(source/'review.json');proof=read(source/'combined-visual-comparison.json')
    assert report['status']=='passed' and not report['errors']
    assert len(report['checks'])==report['expected']
    assert all(c['status']=='passed' and not c['errors'] and c['project']=='native' for c in report['checks'])
    assert proof['status']=='complete' and not proof['remaining']
    assert proof['reviewSha256']==sha(source/'review.json')
    assert proof['reviewedPairs']==proof['scenePairs']==len(report['cases'])
    assert len({(c['name'],c['width']) for c in report['cases']})==len(report['cases'])
    old,old_sha,old_manifest,old_manifest_sha=frozen(old_toolchain)
    new,new_sha,new_manifest,new_manifest_sha=frozen(new_toolchain)
    assert old_sha!=new_sha,'Expected a changed renderer'
    fingerprints={str(binary):(binary.stat().st_ino,binary.stat().st_size,binary.stat().st_mtime_ns) for binary in [old,new]}
    # Reuse full-gallery provenance verifier, with the source itself as target.
    # It verifies successful union component receipts, actual requests and PNGs.
    with tempfile.TemporaryDirectory(prefix='full-gallery-review-check-') as tmp:
        temp=Path(tmp);(temp/'review.json').write_bytes((source/'review.json').read_bytes())
        check=subprocess.run([sys.executable,str(Path(__file__).with_name('audit-gallery-transfer.py')),str(temp),str(source)],capture_output=True,text=True)
        assert check.returncode==0,check.stdout+check.stderr
        verified=read(temp/'baseline-comparison.json')
        assert not verified['remaining'] and verified['reviewedPairs']==len(report['cases'])
    out.mkdir(parents=True)
    bindings=[]
    for index,row in enumerate(report['cases']):
        assert row['status']=='passed' and not row['errors'] and row['metrics']['backend']=='rust-metal'
        assert row['metrics']['browserVersion']=='153.0.8010.12'
        prefix=row['artifactPrefix']
        files={suffix:dict(path=prefix+'.'+suffix,sha256=sha(prefix+'.'+suffix)) for suffix in ['stream','native.png','browser.png','json','riv','map.json','requirements.json','bounds.json']}
        with Image.open(files['native.png']['path']) as image:assert image.size==(row['width'],row['height'])
        bindings.append(dict(index=index,name=row['name'],width=row['width'],height=row['height'],files=files))
    (out/'source-bindings.json').write_text(json.dumps(bindings,indent=2)+'\n')
    base=dict(scope='Renderer-only exact-image migration of retained reviewed streams. No new Chrome capture, public compiler qualification, layout or visual inspection.',
        sourceGallery=str(source),sourceReviewSha256=sha(source/'review.json'),sourceVisualProofSha256=sha(source/'combined-visual-comparison.json'),
        sourceBindingsSha256=sha(out/'source-bindings.json'),oldRendererSha256=old_sha,newRendererSha256=new_sha,
        oldManifest=str(old_manifest),oldManifestSha256=old_manifest_sha,newManifest=str(new_manifest),newManifestSha256=new_manifest_sha,
        backend='rust-metal',mode='clockwise-atomic',expected=len(bindings),newVisualInspections=0)
    (out/'receipt.json').write_text(json.dumps({**base,'status':'running'},indent=2)+'\n')
    rows=[]
    with (out/'rows.jsonl').open('w') as journal:
        for binding in bindings:
            index=binding['index'];stream=Path(binding['files']['stream']['path'])
            assert sha(stream)==binding['files']['stream']['sha256'],'Stale retained stream'
            expected=Path(binding['files']['native.png']['path'])
            assert sha(expected)==binding['files']['native.png']['sha256'],'Changed source native PNG'
            result=dict(index=index,name=binding['name'],width=binding['width'],streamSha256=sha(stream),expectedNativeSha256=sha(expected),reviewTransferred=False)
            for label,binary,digest in [('source-control',old,old_sha),('new-renderer',new,new_sha)]:
                stat=binary.stat()
                assert (stat.st_ino,stat.st_size,stat.st_mtime_ns)==fingerprints[str(binary)],'Renderer changed during migration'
                png=out/f'{index:05d}-{label}.native.png'
                args=[str(binary),'--stream',str(stream),'--output',str(png),'--backend','rust-metal','--mode','clockwise-atomic']
                try:run=subprocess.run(args,capture_output=True,text=True,timeout=120)
                except subprocess.TimeoutExpired:
                    result['failure']=label+' timed out';break
                assert sha(stream)==binding['files']['stream']['sha256'],'Stream changed during replay'
                if run.returncode!=0 or not png.exists():
                    (out/f'{index:05d}-{label}.log').write_text(run.stdout+run.stderr)
                    result['failure']=label+' failed';result['exitCode']=run.returncode;break
                with Image.open(png) as image:assert image.size==(binding['width'],binding['height']),'Wrong replay dimensions'
                result[label+'Sha256']=sha(png)
                identical=png.read_bytes()==expected.read_bytes()
                if label=='source-control':
                    if not identical:result['failure']='Retained stream does not reproduce reviewed source PNG';break
                    png.unlink()  # Successful controls are byte-identical to the preserved source.
                else:
                    result['output']=str(png);result['reviewTransferred']=identical
                    if not identical:result['failure']='Native pixels changed; visual review remains outstanding'
            rows.append(result);journal.write(json.dumps(result)+'\n');journal.flush()
            if (index+1)%100==0:print(json.dumps(dict(completed=index+1,expected=len(bindings),changed=sum(not r['reviewTransferred'] for r in rows))),flush=True)
    # Check immutable evidence again; new hashes never overwrite expected bindings.
    assert sha(source/'review.json')==base['sourceReviewSha256']
    assert sha(source/'combined-visual-comparison.json')==base['sourceVisualProofSha256']
    assert sha(old_manifest)==old_manifest_sha and sha(new_manifest)==new_manifest_sha
    assert sha(old)==old_sha and sha(new)==new_sha
    for binding in bindings:
        for file in binding['files'].values():assert sha(file['path'])==file['sha256'],'Source evidence changed during migration'
    remaining=[dict(name=r['name'],width=r['width'],reason=r.get('failure')) for r in rows if not r['reviewTransferred']]
    receipt={**base,'status':'changed-or-failed' if remaining else 'complete','completed':len(rows),'reviewedPairs':len(rows)-len(remaining),'remaining':remaining,'rowsSha256':sha(out/'rows.jsonl')}
    (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
    print(json.dumps({k:receipt[k] for k in ['status','completed','reviewedPairs']}))
    return bool(remaining)

if __name__=='__main__':
    assert len(sys.argv)==5,__doc__
    sys.exit(main(*(Path(arg).resolve() for arg in sys.argv[1:])))
