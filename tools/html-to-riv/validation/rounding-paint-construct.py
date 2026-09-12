"""Construct authored live rounded paint recipes twice; freeze identities."""
from pathlib import Path
import hashlib,json,shutil,subprocess,sys
M=Path(__file__).resolve().parents[1];root=Path(sys.argv[1]).resolve()
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
assert not(root/'cases').exists()
for suffix in ['cases','recipes']:shutil.copy2(M/f'validation/rounding-paint-{suffix}.json',root/f'{suffix}.json')
shutil.copy2(__file__,root/'construct.py');rows=[]
for name,recipe in json.loads((root/'recipes.json').read_text()).items():
 p=root/'cases'/name;p.mkdir(parents=True);q=p/'recipe.json';q.write_text(json.dumps(recipe,indent=2)+'\n')
 row=dict(name=name,recipeSha256=sha(q),artifacts={})
 for run in ['first','repeat']:
  cmd=[str(root/'candidate'),str(q),str(p/run)];r=subprocess.run(cmd,capture_output=True,text=True);(p/(run+'.log')).write_text(r.stdout+r.stderr);assert r.returncode==0,r.stderr
  for filename in ['scene.riv','scene.map.json','construction.json']:
   h=sha(p/run/filename)
   if run=='first':row['artifacts'][filename]=h;shutil.copy2(p/run/filename,p/filename)
   else:assert row['artifacts'][filename]==h
 row['deterministic']=True;row['exitCode']=0;rows.append(row)
(root/'construction-receipt.json').write_text(json.dumps(dict(cases=rows,constructorSha256=sha(root/'candidate'),recipesSha256=sha(root/'recipes.json'),casesSha256=sha(root/'cases.json'),scriptSha256=sha(root/'construct.py')),indent=2)+'\n');print(json.dumps(dict(cases=len(rows))))
