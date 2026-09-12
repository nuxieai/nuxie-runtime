"""One-variable replay probes of the retained exact native red pair."""
from pathlib import Path
import hashlib,json,re,shutil,subprocess
from PIL import Image
M=Path(__file__).resolve().parents[1];I=M/'output/playwright/public-fixed-folding-debug-r1';O=M/'output/playwright/public-fixed-folding-debug-r2';O.mkdir(exist_ok=False)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
bind=lambda p:dict(path=str(Path(p).resolve()),sha256=sha(p))
renderer=M/'output/immutable-baseline-toolchain-r2/renderer-replay';assert sha(renderer)=='276a49d810c05de7cfb0f0a4109853d12216da64e5d9d8e131f7400357f8e51f';shutil.copy2(renderer,O/'renderer-replay');shutil.copy2(__file__,O/'probe.py')
inputs={};texts={};references={}
for variant in ['original','folded']:
 for extension in ['stream','png']:
  p=I/f'{variant}-minimal.{extension}';shutil.copy2(p,O/p.name);inputs[p.name]=bind(p)
 texts[variant]=(I/f'{variant}-minimal.stream').read_text();references[variant]=list(Image.open(I/f'{variant}-minimal.png').convert('RGBA').getdata())
fold=texts['folded'];original=texts['original'];clip=next(l for l in fold.splitlines()if l.startswith('clipPath'))
def extents(w,h):
 old='points=[(0,0),(20,0),(20,50),(0,50),(0,0)]';new=f'points=[(0,0),({w},0),({w},{h}),(0,{h}),(0,0)]';assert fold.count(old)==1;return fold.replace(old,new)
probes=[('original-control','original',original,'none'),('folded-control','folded',fold,'none'),('folded-both-extents','folded',extents(32768,32768),'rectangle size pair'),('folded-width','folded',extents(32768,50),'rectangle width'),('folded-height','folded',extents(20,32768),'rectangle height'),('original-add-clip','original',original.replace('drawPath',clip+'\ndrawPath',1),'add exact folded clip'),('folded-remove-clip','folded',fold.replace(clip+'\n',''),'remove exact folded clip'),('folded-path-id','folded',fold.replace('drawPath path={id=2,','drawPath path={id=32,'),'draw path resource ID'),('folded-paint-id','folded',fold.replace('paint={id=12,','paint={id=19,'),'paint resource ID')]
rows=[]
for name,base,text,variable in probes:
 p=O/(name+'.stream');p.write_text(text);trials=[];arrays=[]
 for trial in [0,1]:
  image=O/f'{name}-{trial}.png';cmd=[str(O/'renderer-replay'),'--stream',str(p),'--output',str(image),'--backend','rust-metal','--mode','clockwise-atomic','--frame','0'];r=subprocess.run(cmd,capture_output=True);log=O/f'{name}-{trial}.log';log.write_bytes(r.stdout+r.stderr);assert r.returncode==0,(name,r.stderr)
  im=Image.open(image).convert('RGBA');assert im.size==(19,31);pixels=list(im.getdata());arrays.append(pixels);comparisons={}
  for ref,expected in references.items():
   delta=[dict(x=i%19,y=i//19,expected=a,actual=b)for i,(a,b)in enumerate(zip(expected,pixels))if a!=b];comparisons[ref]=dict(exact=not delta,changedPixels=len(delta),differences=delta)
  trials.append(dict(command=cmd,exitCode=r.returncode,image=bind(image),log=bind(log),comparisons=comparisons))
 rows.append(dict(name=name,base=base,variable=variable,stream=bind(p),deterministic=arrays[0]==arrays[1],trials=trials))
receipt=dict(scope='One-variable native coverage probes; no compiler/runtime changes',inputs=inputs,renderer=bind(O/'renderer-replay'),script=bind(O/'probe.py'),rows=rows)
(O/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps([dict(name=r['name'],deterministic=r['deterministic'],matches={k:v['exact']for k,v in r['trials'][0]['comparisons'].items()},changedPixels={k:v['changedPixels']for k,v in r['trials'][0]['comparisons'].items()})for r in rows],indent=2))
