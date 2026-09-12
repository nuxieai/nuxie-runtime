"""Repeat every private wrapped file construction and bind exact artifacts."""
from pathlib import Path
import json,hashlib,subprocess,sys
root=Path(sys.argv[1]).resolve();out=root/'reproduction';out.mkdir(exist_ok=False)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def bind(p):return {'path':str(p),'sha256':sha(p)}
rows=[]
for c in json.loads((root/'pairs.json').read_text()):
 old=root/'render'/c['name'];target=out/c['name'];target.mkdir()
 command=[str(root/'adapter.py'),str(old/'request.json'),str(target/'scene.riv')]
 r=subprocess.run(command,capture_output=True,text=True);(target/'compile.log').write_text(r.stdout+r.stderr);assert r.returncode==0,r.stderr
 row={'name':c['name'],'command':command,'request':bind(old/'request.json'),'originalRiv':bind(old/'scene.riv'),'resultRiv':bind(target/'scene.riv'),'originalMap':bind(old/'scene.map.json'),'resultMap':bind(target/'scene.map.json'),'originalTrace':bind(old/'scene.sizing-trace.json'),'resultTrace':bind(target/'scene.sizing-trace.json'),'log':bind(target/'compile.log')}
 for label,name in [('Riv','scene.riv'),('Map','scene.map.json'),('Trace','scene.sizing-trace.json')]:
  row['exact'+label]=(old/name).read_bytes()==(target/name).read_bytes();assert row['exact'+label]
 assert (old/'scene.sized.riv').read_bytes()==(target/'scene.sized.riv').read_bytes()
 rows.append(row)
assert len(rows)==48
(root/'reproductions.json').write_text(json.dumps(rows,indent=2)+'\n')
(root/'reproduction-driver.json').write_text(json.dumps(bind(Path(__file__).resolve()),indent=2)+'\n')
print(json.dumps({'reproduced':len(rows),'filesMapsTracesExact':True}))
