#!/usr/bin/env python3
"""Exact authored fixture adapter; no browser or host layout input."""
from pathlib import Path
import hashlib,json,shutil,subprocess,sys
M=Path(__file__).resolve().parents[1];sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
request_path,output=map(Path,sys.argv[1:]);request=json.loads(request_path.read_text())
fixtures=json.loads((M/'validation/wrapped-original-resource-visual-cases.json').read_text())
match=[f for f in fixtures if request==dict(html=f['html'],css=f['css'],width=f['compileViewport'][0],height=f['compileViewport'][1])];assert len(match)==1,'Only exact authored private requests accepted'
f=match[0];c=f['construction']
for b in [c['recipe'],c['constructor'],c['reference'],*c['artifacts'].values()]:assert sha(b['path'])==b['sha256']
recipe=json.loads(Path(c['recipe']['path']).read_text());assert recipe['initialViewport']==[request['width'],request['height']]
folder=output.parent/'derived-construction';assert not folder.exists();recipe_path=output.parent/'recipe.json';shutil.copy2(c['recipe']['path'],recipe_path)
p=subprocess.run([c['constructor']['path'],str(recipe_path),str(folder)],capture_output=True);assert p.returncode==0,(p.stdout,p.stderr)
for name,b in c['artifacts'].items():assert sha(folder/name)==b['sha256'],'Fresh ordinary construction differs from bound recipe'
shutil.copy2(folder/'scene.riv',output);shutil.copy2(folder/'scene.map.json',output.with_suffix('.map.json'))
(output.parent/'private-construction.json').write_text(json.dumps(dict(scope='Exact authored resource/mixed recipe; ordinary fresh construction, no browser data',name=f['name'],construction=c,recipeSha256=sha(recipe_path)),indent=2)+'\n')
