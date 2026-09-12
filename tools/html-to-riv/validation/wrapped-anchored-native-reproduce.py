"""Reemit frozen recipes and compare raw bytes/trace; no new native capture."""
from pathlib import Path
import hashlib,json,subprocess,sys
root=Path(sys.argv[1]).resolve();out=Path(sys.argv[2]).resolve();out.mkdir(parents=True,exist_ok=False)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
r=json.loads((root/'receipt.json').read_text());results=[]
for case in r['cases']:
 dest=out/case['name'];dest.mkdir()
 run=subprocess.run([str(root/'candidate'),str(dest),*case['arguments']],capture_output=True,text=True);(dest/'candidate.log').write_text(run.stdout+run.stderr);assert run.returncode==0,run.stderr
 for name in ['base.riv','scene.riv','trace.json']:
  p=dest/name;original=root/'cases'/case['name']/name
  assert p.read_bytes()==original.read_bytes()
  results.append({'path':str(p),'original':str(original),'sha256':sha(p)})
result={'scope':'Frozen candidate executable reemission; no recaptured native evidence','cases':len(r['cases']),'exactArtifacts':len(results),'candidate':{'path':str(root/'candidate'),'sha256':sha(root/'candidate')},'sourceReceipt':{'path':str(root/'receipt.json'),'sha256':sha(root/'receipt.json')},'driver':{'path':str(Path(__file__).resolve()),'sha256':sha(__file__)},'results':results}
(out/'receipt.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'cases':len(r['cases']),'exactArtifacts':len(results)}))
