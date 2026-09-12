#!/usr/bin/env python3
"""Private authored-recipe adapter for the unchanged baseline comparison driver.

This is not the public HTML/CSS compiler. It matches only exact authored requests
and invokes the frozen actual Derived constructor without browser measurements.
"""
import hashlib,json,shutil,subprocess,sys
from pathlib import Path
module=Path(__file__).resolve().parents[1]
root=module/'output/wrapped-original-constructor-r1'
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
request_path,output=map(Path,sys.argv[1:])
request=json.loads(request_path.read_text())
fixtures=json.loads((module/'validation/wrapped-boundary-cases.json').read_text())+json.loads((module/'validation/wrapped-derived-cases.json').read_text())
recipes=json.loads((root/'recipes.json').read_text())
receipt=json.loads((root/'construction-receipt.json').read_text())
assert sha(root/'candidate')==receipt['constructorSha256']
assert sha(root/'recipes.json')==receipt['recipesSha256']
matches=[f for f in fixtures if request==dict(html=f['html'],css=f['css'],width=f['compileViewport'][0],height=f['compileViewport'][1])]
assert len(matches)==1,'Only exact authored private campaign requests accepted'
f=matches[0];recipe=recipes[f['name']]
assert recipe['initialViewport']==[request['width'],request['height']]
folder=output.parent/'derived-construction';assert not folder.exists()
recipe_path=output.parent/'recipe.json';recipe_path.write_text(json.dumps(recipe,indent=2)+'\n')
subprocess.run([str(root/'candidate'),str(recipe_path),str(folder)],check=True)
expected=next(c for c in receipt['cases'] if c['name']==f['name'])
for name,digest in expected['artifacts'].items():
 assert sha(folder/name)==digest,'Fresh construction differs from reviewed frozen recipe'
shutil.copy2(folder/'scene.riv',output)
shutil.copy2(folder/'scene.map.json',output.with_suffix('.map.json'))
(output.parent/'private-construction.json').write_text(json.dumps(dict(scope='Private actual Derived recipe; not public compiler admission',name=f['name'],constructorSha256=sha(root/'candidate'),recipeSha256=sha(recipe_path),artifacts=expected['artifacts']),indent=2)+'\n')
