"""Exact red native replay/minimization; exits 1 while original/folded differ."""
from pathlib import Path
import hashlib,json,shutil,subprocess,sys
from PIL import Image
M=Path(__file__).resolve().parents[1];O=M/'output/playwright/public-fixed-folding-debug-r1';O.mkdir(exist_ok=False)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
bind=lambda p:dict(path=str(Path(p).resolve()),sha256=sha(p))
def write(p,j):p.write_text(json.dumps(j,indent=2)+'\n')
renderer=M/'output/immutable-baseline-toolchain-r2/renderer-replay';assert sha(renderer)=='276a49d810c05de7cfb0f0a4109853d12216da64e5d9d8e131f7400357f8e51f';shutil.copy2(renderer,O/'renderer-replay');shutil.copy2(__file__,O/'repro.py')
commands=[]
def render(lines,label,frame):
 p=O/(label+'.stream');p.write_text('\n'.join(lines)+'\n');png=O/(label+'.png');cmd=[str(O/'renderer-replay'),'--stream',str(p),'--output',str(png),'--backend','rust-metal','--mode','clockwise-atomic','--frame',str(frame)]
 r=subprocess.run(cmd,capture_output=True);log=O/(label+'.log');log.write_bytes(r.stdout+r.stderr);commands.append(dict(command=cmd,exitCode=r.returncode,stream=bind(p),log=bind(log)))
 if r.returncode:return None
 return Image.open(png).convert('RGBA').tobytes()
streams={};references={};sources=[]
for variant in ['original','folded']:
 p=M/'output/playwright/public-fixed-folding-r1'/(variant+'-capture')/'fixed-fold-row-r1-w2/probe';f=json.loads((p/'frames.json').read_text())['frames'][1];source=p/f['stream'];sources.append(bind(source));lines=source.read_text().rstrip().splitlines();streams[variant]=lines
 a=render(lines,variant+'-exact-0',1);b=render(lines,variant+'-exact-1',1);assert a is not None and a==b,'transient/replay failure';references[variant]=a
assert references['original']!=references['folded'],'retained mismatch did not reproduce'
write(O/'reproduction.json',dict(inputs=sources,script=bind(O/'repro.py'),renderer=bind(O/'renderer-replay'),deterministicEachSide=True,exactMismatch=True,commands=commands))
retained=[];attempt=0
for variant in ['original','folded']:
 lines=streams[variant];last=next(i for i in range(len(lines)-1,-1,-1)if lines[i].startswith('frameSize'))
 # Delete the earlier complete frame, preserving the observed resource setup.
 resources=[l for l in lines[:last]if l.startswith(('makeRenderPaint ','makeEmptyRenderPath '))]
 candidate=[lines[0],*resources,*lines[last:]];label=variant+'-single-frame';data=render(candidate,label,0)
 if data==references[variant]:lines=candidate;retained.append(dict(variant=variant,action='remove earlier complete frame',stream=bind(O/(label+'.stream'))))
 frame=0 if lines==candidate else 1
 # Greedy legal command/block removal. Retain only reductions that reproduce
 # this side's complete decoded RGBA exactly, preventing a fake new mismatch.
 changed=True
 while changed:
  changed=False;i=1
  while i<len(lines):
   line=lines[i];remove=[]
   if line=='save':
    depth=1
    for j in range(i+1,len(lines)):
     depth+=int(lines[j]=='save')-int(lines[j]=='restore')
     if depth==0:remove=[i,j];break
   elif line.startswith(('makeRenderPaint ','makeEmptyRenderPath ','transform ','clipPath ','drawPath ')):remove=[i]
   if not remove:i+=1;continue
   candidate=[l for j,l in enumerate(lines)if j not in remove];attempt+=1;label=f'{variant}-attempt-{attempt:04}';data=render(candidate,label,frame)
   if data==references[variant]:
    retained.append(dict(variant=variant,action='remove commands',removed=[lines[j]for j in remove],stream=bind(O/(label+'.stream'))));lines=candidate;changed=True
   else:i+=1
  # Minimal against this command/pair deletion grammar, not arbitrary rewriting.
 streams[variant]=lines;data=render(lines,variant+'-minimal',frame);assert data==references[variant]
write(O/'reductions.json',retained)
final={v:bind(O/(v+'-minimal.stream'))for v in streams};a=Image.open(O/'original-minimal.png').convert('RGBA');b=Image.open(O/'folded-minimal.png').convert('RGBA');delta=[]
for i,(x,y) in enumerate(zip(a.getdata(),b.getdata())):
 if x!=y:delta.append(dict(x=i%a.width,y=i//a.width,original=x,folded=y))
write(O/'receipt.json',dict(scope='Exact deterministic native red pair, minimized by output-preserving command deletion',inputs=sources,renderer=bind(O/'renderer-replay'),script=bind(O/'repro.py'),attempts=attempt,retainedReductions=len(retained),reductions=bind(O/'reductions.json'),final=final,finalLines={v:len(s)for v,s in streams.items()},decodedDifferences=delta,commands=commands,stillRed=bool(delta)))
print(json.dumps(dict(attempts=attempt,retained=len(retained),lines={v:len(s)for v,s in streams.items()},differences=delta)),flush=True)
raise SystemExit(1 if delta else 0)
