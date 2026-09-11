"""Rerender audited public opacity streams and require exact full native PNG identity.
Usage: SCRIPT EVIDENCE_ROOT RENDERER NEW_OUTPUT
"""
from pathlib import Path
import hashlib
import json
import subprocess
import sys

def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p): return json.loads(p.read_text())
root,binary,out=(Path(p).resolve() for p in sys.argv[1:]);out.mkdir(parents=True,exist_ok=False)
rows=[]
for recording_name,replay_name in [
    ('group-opacity-public-runtime-recording','group-opacity-public-runtime-lifecycle'),
    ('group-opacity-boundary-public-recording-r2','group-opacity-boundary-public-lifecycle-r2'),
    ('group-opacity-overflow-composition-recording','group-opacity-overflow-composition-lifecycle'),
    ('group-opacity-decoration-recording','group-opacity-decoration-lifecycle')]:
    recording=root/recording_name; replay=root/replay_name
    manifest=read(recording/'lifecycle.json'); reference=read(replay/'replay.json')
    audit=read(replay/'public-opacity-audit.json')
    assert audit['status']=='complete-public-opacity-artifact-audit'
    assert reference['lifecycleSha256']==sha(recording/'lifecycle.json')
    assert any(e['path']==str(replay/'replay.json') and e['sha256']==sha(replay/'replay.json') for e in audit['evidence'])
    references={r['name']:r for r in reference['cases']}
    count=0
    for scene in manifest['cases']:
        for view in scene['views']:
            name=f"{scene['name']}-{view['instance']}-step{view['frame']}"
            old=references[name]; stream=Path(view['stream'])
            assert sha(stream)==old['streamSha256']
            assert not old['failures'] and not old['geometryFailures']
            assert sha(Path(old['prefix']+'.native.png'))==old['nativeSha256']
            output=out/(name+'.native.png')
            subprocess.run([str(binary),'--stream',str(stream),'--frame',str(view['frame']),
                '--output',str(output),'--backend','rust-metal','--mode','clockwise-atomic'],check=True,capture_output=True)
            actual=sha(output)
            rows.append(dict(name=name,sourceReplay=str(replay/'replay.json'),streamSha256=sha(stream),
                nativeSha256=actual,expectedSha256=old['nativeSha256'],identical=actual==old['nativeSha256']))
            count+=1
    assert count==len(references)
receipt=dict(rendererSha256=sha(binary),frames=len(rows),changed=sum(not r['identical'] for r in rows),rows=rows)
(out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({k:receipt[k] for k in ['frames','changed','rendererSha256']}))
assert receipt['changed']==0, 'Native output changed; inspect retained PNGs before qualification'
