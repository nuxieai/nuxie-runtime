"""Authored Chrome references for integral resource scenes and mixed paint routes.
No browser measurements enter construction. Existing resource artifacts remain intact.
"""
from pathlib import Path
import copy,hashlib,json,shutil,subprocess
M=Path(__file__).resolve().parents[1];R=M/'output/wrapped-integral-resource-r1';O=M/'output/wrapped-integral-resource-visual-constructor-r1'
assert not O.exists();O.mkdir();sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest();bind=lambda p:dict(path=str(Path(p).resolve()),sha256=sha(p))
summary=json.loads((M/'output/wrapped-integral-product-build-r1/summary.json').read_text());assert summary['sourceUnchanged']and summary['rustTests']==442 and summary['nodeTests']==56
receipt=json.loads((R/'construction-receipt.json').read_text());candidate=Path(receipt['constructor']['path']);assert sha(candidate)==receipt['constructor']['sha256'];shutil.copy2(candidate,O/'candidate');shutil.copy2(__file__,O/'generator.py')
fixtures=[];constructed=[]
def fixture(name,recipe,folder,reference):
 assert recipe['initialViewport']==[320,240]and recipe['wrap']==1 and not recipe['reverseMain']and recipe['lineFraction']==recipe['mainFraction']==0
 row=recipe['row'];css='#p{width:100%;height:100%;flex-direction:'+('row'if row else'column')+';flex-wrap:wrap;align-content:flex-start;justify-content:flex-start;align-items:flex-start;}'
 html='<div id="p">'
 for i,s in enumerate(recipe['slots']):
  assert s['slotAlignment']==0 and s['alignment']==0
  html+=f'<div class="s{i}"><div id="{s["name"]}"></div></div>'
  dims=[s['main'],s['cross']]if row else[s['cross'],s['main']];vdims=[s['visibleMain'],s['visibleCross']]if row else[s['visibleCross'],s['visibleMain']]
  assert all(d['unit']=='px' and d.get('min')is None and d.get('max')is None for d in dims+vdims)
  css+=f'.s{i}{{width:{dims[0]["source"]}px;height:{dims[1]["source"]}px;flex-direction:column;align-items:flex-start;justify-content:flex-start;}}'
  color=s['color'];red=(color>>16)&255;green=(color>>8)&255;blue=color&255;alpha=(color>>24)/255
  css+=f'#{s["name"]}{{width:{vdims[0]["source"]}px;height:{vdims[1]["source"]}px;background:rgba({red},{green},{blue},{alpha:.17g});}}'
 html+='</div>'
 artifacts={n:bind(folder/n)for n in ['base.riv','scene.riv','scene.map.json','trace.json','proof.json']}
 fixtures.append(dict(name=name,html=html,css=css,compileViewport=[320,240],viewports=[[320,200],[160,320],[96,240],[320,200]],features=['ordinary integral foreground or mixed rounded paint','same original/clone resize','authored fixed sizes and alpha overlap'],construction=dict(recipe=bind(folder/'recipe.json'),constructor=bind(O/'candidate'),reference=reference,artifacts=artifacts)))
for n in [1,3,8,34]:
 expected=next(c for c in receipt['cases']if c['owners']==n);assert expected['deterministic'];folder=R/'cases'/f'owners-{n}'
 assert sha(folder/'recipe.json')==expected['recipe']['sha256']
 for name,b in expected['artifacts'].items():assert sha(folder/name)==b['sha256']
 fixture(f'integral-resource-{n}',json.loads((folder/'recipe.json').read_text()),folder,bind(R/'construction-receipt.json'))
for row in [True,False]:
 recipe=json.loads((R/'cases/owners-3/recipe.json').read_text());recipe['row']=row
 for slot,value in zip(recipe['slots'],[20,19.5,30]):slot['visibleMain']['value']=value;slot['visibleMain']['source']=str(value)
 name='integral-mixed-'+('row'if row else'column');folder=O/name;folder.mkdir();(folder/'recipe.json').write_text(json.dumps(recipe,indent=2)+'\n');runs=[]
 for run in ['first','repeat']:
  command=[str(O/'candidate'),str(folder/'recipe.json'),str(folder/run)];p=subprocess.run(command,capture_output=True);(folder/f'{run}.log').write_bytes(p.stdout+p.stderr);runs.append(dict(command=command,exitCode=p.returncode,log=bind(folder/f'{run}.log')))
  assert p.returncode==0,p.stderr
 for name in ['base.riv','scene.riv','scene.map.json','trace.json','proof.json']:
  assert sha(folder/'first'/name)==sha(folder/'repeat'/name);shutil.copy2(folder/'first'/name,folder/name)
 constructed.append(dict(name=folder.name,recipe=bind(folder/'recipe.json'),runs=runs,artifacts={name:bind(folder/name)for name in ['base.riv','scene.riv','scene.map.json','trace.json','proof.json']}))
# Write mixed reference before fixtures reference its digest.
(O/'construction-receipt.json').write_text(json.dumps(dict(scope='Two mixed ordinary paint compositions constructed twice; no browser/native execution',constructor=bind(O/'candidate'),parentResourceReceipt=bind(R/'construction-receipt.json'),generator=bind(O/'generator.py'),cases=constructed),indent=2)+'\n')
for c in constructed:
 folder=O/c['name'];fixture(c['name'],json.loads((folder/'recipe.json').read_text()),folder,bind(O/'construction-receipt.json'))
p=M/'validation/wrapped-integral-resource-visual-cases.json';p.write_text(json.dumps(fixtures,indent=2)+'\n');print(json.dumps(dict(cases=len(fixtures),fixtures=str(p),mixedConstructor=str(O))))
