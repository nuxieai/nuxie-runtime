"""Negative controls for public opacity evidence. Originals remain read-only.
Usage: SCRIPT EVIDENCE_ROOT COMPILER NEW_RECEIPT
"""
import copy
import hashlib
import json
from pathlib import Path
import runpy
import shutil
import sys
import tempfile

HERE=Path(__file__).resolve().parent
visual=runpy.run_path(str(HERE/'audit-opacity-public-visuals.py'))['audit']
artifact=runpy.run_path(str(HERE/'audit-opacity-diagnostic.py'))['audit']
def read(p): return json.loads(p.read_text())
def write(p,d): p.write_text(json.dumps(d,indent=2)+'\n')
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()

def main(root,compiler,receipt):
    assert not receipt.exists(), 'Refusing to overwrite evidence'
    results=[]
    with tempfile.TemporaryDirectory(prefix='opacity-audit-controls-') as temporary:
        tmp=Path(temporary)
        for kind in ['initial','stacking','composition']:
            name=f'group-opacity-{kind}-corrected-static-review'
            (tmp/name).symlink_to(root/name,target_is_directory=True)
        review=tmp/'review'; replay=tmp/'replay'; recording=tmp/'recording'
        for p in [review,replay,recording]: p.mkdir()
        for p in (root/'group-opacity-public-runtime-recording').glob('*.riv'): shutil.copyfile(p,recording/p.name)
        original_source=read(root/'group-opacity-public-runtime-recording/lifecycle.json')
        original_replay=read(root/'group-opacity-public-runtime-lifecycle/replay.json')
        original_projection=read(root/'group-opacity-public-static-review/replay.json')
        def reset(source=None,frames=None):
            write(recording/'lifecycle.json',source or original_source)
            data=copy.deepcopy(frames or original_replay)
            data['lifecycleSha256']=sha(recording/'lifecycle.json')
            write(replay/'replay.json',data)
            projection=copy.deepcopy(original_projection)
            projection.update(sourceReplay=str((replay/'replay.json').resolve()),sourceReplaySha256=sha(replay/'replay.json'))
            write(review/'replay.json',projection)
            visual(review,verify=False)
        def reject(name,operation):
            try: operation()
            except (AssertionError,ValueError,KeyError): results.append(dict(name=name,rejected=True))
            else: raise AssertionError('Corrupted evidence accepted: '+name)
        reset(); assert artifact(recording,replay,review,compiler)['frames']==240
        for field,value in [('css','div{opacity:0}'),('html','<div></div>'),('nativeSha256','0'*64),
            ('qualification','runtime-experiment-only'),('geometryFailures',[{'mismatch':True}]),('failures',['bad pixels'])]:
            reset(); projection=read(review/'replay.json');projection['cases'][0][field]=value;write(review/'replay.json',projection)
            reject('visual-'+field,lambda:visual(review,verify=False))
        for duplicate in [False,True]:
            reset(); projection=read(review/'replay.json')
            if duplicate: projection['cases'].append(copy.deepcopy(projection['cases'][0]))
            else: projection['cases'].pop()
            write(review/'replay.json',projection)
            reject('duplicate-view' if duplicate else 'missing-view',lambda:visual(review,verify=False))
        for kind in ['browser','native']:
            reset(); projection=read(review/'replay.json');row=projection['cases'][0]
            prefix=tmp/f'corrupt-{kind}'
            for image_kind in ['browser','native']:
                data=Path(row['prefix']+'.'+image_kind+'.png').read_bytes()
                if image_kind==kind: data=data[:-1]+bytes([data[-1]^1])
                Path(str(prefix)+'.'+image_kind+'.png').write_bytes(data)
            row['prefix']=str(prefix);write(review/'replay.json',projection)
            reject('image-bytes-'+kind,lambda:visual(review,verify=False))
        reset()
        riv=recording/(original_source['cases'][0]['name']+'.riv')
        data=riv.read_bytes();riv.write_bytes(data+b'corrupt')
        reject('rive-bytes',lambda:artifact(recording,replay,review,compiler))
        riv.write_bytes(data)
        for field,value in [('compilerRequestCss','div{opacity:0}'),('injectedOpacity',{}),('runtimeRequirements',{})]:
            source=copy.deepcopy(original_source);source['cases'][0][field]=value;reset(source=source)
            reject('artifact-'+field,lambda:artifact(recording,replay,review,compiler))
        for field,value in [('streamSha256','0'*64),('frame',999),('qualification','runtime-experiment-only')]:
            frames=copy.deepcopy(original_replay);frames['cases'][0][field]=value;reset(frames=frames)
            reject('frame-'+field,lambda:artifact(recording,replay,review,compiler))
        frames=copy.deepcopy(original_replay);frames['cases'].pop();reset(frames=frames)
        reject('missing-frame',lambda:artifact(recording,replay,review,compiler))
        reset(); assert artifact(recording,replay,review,compiler)['frames']==240
    write(receipt,dict(status='passed',negativeControls=results,positiveControls=2,
        sourceReplaySha256=sha(root/'group-opacity-public-runtime-lifecycle/replay.json'),compilerSha256=sha(compiler)))
    print(json.dumps(dict(negativeControls=len(results),positiveControls=2,status='passed')))
if __name__=='__main__': main(*(Path(p).resolve() for p in sys.argv[1:]))
