"""Audit realistic public lifecycle transport, profile and exact reviewed pixels.
Usage: SCRIPT RECORDING REPLAY STATIC_REPLAY TOOLCHAIN
"""
import hashlib,json,re,runpy,subprocess,sys,tempfile,struct,copy
from pathlib import Path
read=lambda p:json.loads(Path(p).read_text())
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
recording,replay,static,toolchain=(Path(p).resolve() for p in sys.argv[1:])
scripts=Path(__file__).parent;module=scripts.parent
source=read(scripts/'linear-gradient-realistic-cases.json');life=read(recording/'lifecycle.json');report=read(replay/'replay.json');manifest=read(toolchain/'manifest.json')
assert life['kind']=='gradient-public' and life['corpus']=='linear-gradient-realistic' and life['nativeGlyphs'] is True
assert report['browser']==life['browser']=='153.0.8010.12'
assert report['lifecycleSha256']==sha(recording/'lifecycle.json')
assert report['rendererSha256']==manifest['files']['renderer-replay']['sha256']
for name,entry in manifest['files'].items():assert sha(toolchain/name)==entry['sha256']
runpy.run_path(str(scripts/'record-visual-review.py'))['record'](static,audit=True)
old={(r['name'],r['width']):r for r in read(static/'replay.json')['cases']};current={r['name']:r for r in report['cases']}
assert len(current)==len(report['cases'])==48
assert len(life['cases'])==len(source)==6
known={c['name']:c for c in source};rows=[]
with tempfile.TemporaryDirectory() as tmp:
 tmp=Path(tmp)
 for c in life['cases']:
  name=c['name'];authored=known[name]
  for key in ['html','css','font','image']:assert c[key]==authored[key]
  assert c['font'] and c['image'] and c['nativeGlyphs']
  request=read(recording/(name+'.request.json'));assert request['html']==c['html'] and request['css']==c['css']
  assert request['width']==390 and request['height']==320
  assert bytes(request['assets']['inter']['bytes'])==(module/'tests/assets/Inter-Regular.ttf').read_bytes()
  assert bytes(request['assets']['photo']['bytes'])==(module/'tests/assets/quadrants.png').read_bytes()
  (tmp/'request.json').write_text(json.dumps(request));subprocess.run([str(toolchain/'html-to-riv'),str(tmp/'request.json'),str(tmp/'scene.riv')],check=True,capture_output=True)
  assert (tmp/'scene.riv').read_bytes()==(recording/(name+'.riv')).read_bytes()
  published=read(tmp/'scene.requirements.json');recorded=copy.deepcopy(c['runtimeRequirements'])
  # serde_json::to_value expands f32 opacity to f64; CLI emits shortest f32 JSON.
  # Compare exact float32 bits for this typed field, never a visual tolerance.
  for left,right in zip(published.get('layout_group_opacity',[]),recorded.get('layout_group_opacity',[])):
   assert struct.pack('<f',left['opacity'])==struct.pack('<f',right['opacity'])
   right['opacity']=left['opacity']
  assert published==recorded
  assert read(tmp/'scene.map.json')==c['sourceMap']
  assert len(c['views'])==8
  for i,v in enumerate(c['views']):
   assert (v['frame'],v['instance'],v['width'],v['height'])==(i,i//4,[240,390,768,240][i%4],320)
   r=current[f'{name}-{v["instance"]}-step{i}'];assert not r['failures'] and not r['geometryFailures']
   assert r['qualification']=='public-compiler-lifecycle'
   assert r['runtimeRequirements']==c['runtimeRequirements']
   assert r['pixelProfile']=={'font':True,'name':'font','nativeGlyphs':True}
   assert r['streamSha256']==sha(v['stream'])
   o=old[name,v['width']]
   assert r['html']==o['html'] and r['css']==o['css']
   for kind in ['browser','native']:
    p=Path(r['prefix']+'.'+kind+'.png');q=Path(o['prefix']+'.'+kind+'.png')
    assert sha(p)==r[kind+'Sha256'] and sha(q)==o[kind+'Sha256'] and p.read_bytes()==q.read_bytes()
   rows.append({'name':r['name'],'width':r['width'],'sourceName':name,'browserSha256':r['browserSha256'],'nativeSha256':r['nativeSha256'],'reviewTransferred':True})
paths=[recording/'lifecycle.json',replay/'replay.json',static/'replay.json',static/'visual-inspection.json',scripts/'linear-gradient-realistic-cases.json',scripts/'clip-margin-lifecycle.mjs',scripts/'lifecycle-pixel-profile.mjs',module/'tests/gradient_realistic_runtime.rs',module/'tests/assets/Inter-Regular.ttf',module/'tests/assets/quadrants.png',toolchain/'manifest.json']
receipt={'status':'passed','scenes':6,'frames':48,'reviewTransferred':48,'pixelProfile':'font','nativeGlyphs':True,'newVisualInspections':0,'scope':'Public compiled transport, original/clone sequential viewport states, explicit native-glyph profile and exact full image transfer from reviewed static views. Vector glyph failures remain separate.','evidence':[{'path':str(p),'sha256':sha(p)} for p in paths],'rows':rows}
p=replay/'realistic-public-audit.json';assert not p.exists();p.write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({k:receipt[k] for k in ['status','scenes','frames','reviewTransferred']}))
