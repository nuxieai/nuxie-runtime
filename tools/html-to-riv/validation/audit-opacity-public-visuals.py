"""Transfer visual inspection only from opacity diagnostics to identical public images.
This proves image coverage, not public artifact correctness; run the artifact audit too.
Usage: SCRIPT PUBLIC_REVIEW [--audit]
"""
import hashlib
import json
from pathlib import Path
import runpy
import sys

def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def read(p): return json.loads(Path(p).read_text())
def audit(out, verify=True):
    out=Path(out).resolve(); root=out.parent
    record=runpy.run_path(str(Path(__file__).with_name('record-visual-review.py')))['record']
    sources={}; evidence=[]
    for kind in ['initial','stacking','composition']:
        folder=root/f'group-opacity-{kind}-corrected-static-review'
        assert record(folder,audit=True)['remaining']==0
        for row in read(folder/'replay.json')['cases']:
            assert row['qualification']=='runtime-experiment-only'
            key=(row['name'],row['width']); assert key not in sources
            sources[key]=row
        evidence.extend(dict(path=str(p),sha256=sha(p)) for p in [folder/'replay.json',folder/'visual-inspection.json'])
    rows=read(out/'replay.json')['cases']; seen=set(); transfers=[]
    for row in rows:
        key=(row['name'],row['width']); assert key not in seen; seen.add(key)
        assert row['qualification']=='public-compiler-lifecycle' and not row['failures'] and not row['geometryFailures']
        source=sources[key]
        for field in ['html','css','browserSha256','nativeSha256']: assert row[field]==source[field],(key,field)
        for kind in ['browser','native']: assert sha(row['prefix']+'.'+kind+'.png')==row[kind+'Sha256']
        transfers.append(dict(name=row['name'],width=row['width'],browserSha256=row['browserSha256'],nativeSha256=row['nativeSha256']))
    assert seen==set(sources)
    result=dict(status='complete',reviewed=len(rows),remaining=0,scope='Authored HTML/CSS and full-image identity only. Compiler artifact provenance is separately audited.',
        targetReplaySha256=sha(out/'replay.json'),sourceEvidence=evidence,transfers=transfers)
    path=out/'visual-transfer.json'
    if verify: assert read(path)==result
    else: path.write_text(json.dumps(result,indent=2)+'\n')
    return result
if __name__=='__main__':
    result=audit(sys.argv[1],verify='--audit' in sys.argv[2:])
    print(json.dumps(dict(reviewed=result['reviewed'],remaining=result['remaining'])))
