"""Construct every authored recipe twice with the frozen Derived constructor."""
import hashlib,json,shutil,subprocess,sys
from pathlib import Path
root=Path(sys.argv[1]).resolve();recipes=Path(sys.argv[2]).resolve()
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
assert not (root/'cases').exists()
shutil.copy2(recipes,root/'recipes.json');shutil.copy2(__file__,root/'construct.py')
rows=[]
for name,recipe in json.loads((root/'recipes.json').read_text()).items():
 assert name and set(name)<=set('abcdefghijklmnopqrstuvwxyz0123456789-')
 d=root/'cases'/name;d.mkdir(parents=True)
 (d/'recipe.json').write_text(json.dumps(recipe,indent=2)+'\n')
 command=[str(root/'candidate'),str(d/'recipe.json'),str(d/'first')]
 result=subprocess.run(command,capture_output=True,text=True)
 (d/'construct.log').write_text(result.stdout+result.stderr)
 row=dict(name=name,command=command,exitCode=result.returncode,logSha256=sha(d/'construct.log'),recipeSha256=sha(d/'recipe.json'))
 if result.returncode==0:
  again=subprocess.run([str(root/'candidate'),str(d/'recipe.json'),str(d/'repeat')],capture_output=True,text=True)
  (d/'repeat.log').write_text(again.stdout+again.stderr);assert again.returncode==0
  row['artifacts']={}
  for name in ['base.riv','scene.riv','scene.map.json','trace.json','proof.json']:
   assert sha(d/'first'/name)==sha(d/'repeat'/name)
   shutil.copy2(d/'first'/name,d/name);row['artifacts'][name]=sha(d/name)
  row['deterministic']=True
 rows.append(row)
(root/'construction-receipt.json').write_text(json.dumps(dict(constructorSha256=sha(root/'candidate'),recipesSha256=sha(root/'recipes.json'),scriptSha256=sha(root/'construct.py'),cases=rows),indent=2)+'\n')
print(json.dumps(dict(cases=len(rows),accepted=sum(r['exitCode']==0 for r in rows),rejected=sum(r['exitCode']!=0 for r in rows))))
