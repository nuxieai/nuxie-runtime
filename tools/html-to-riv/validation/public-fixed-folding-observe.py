"""Closed offline effective rectangle stream comparison. No rendering/compilation."""
from pathlib import Path
import copy,hashlib,importlib.util,json,math,re,struct,sys
M=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('clip_grammar',M/'validation/wrapped-clip-stream-check.py');grammar=importlib.util.module_from_spec(spec);spec.loader.exec_module(grammar)
N=grammar.NUMBER
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
bind=lambda p:dict(path=str(Path(p).resolve()),sha256=sha(p))
def require(v,m):
 if not v:raise ValueError(m)
def f32(x):
 v=struct.unpack('<f',struct.pack('<f',float(x)))[0];require(math.isfinite(v),'nonfinite');return v
def intersect(a,b):return [max(a[0],b[0]),max(a[1],b[1]),min(a[2],b[2]),min(a[3],b[3])]
def observe_one(text,w,h):
 lines=text.rstrip('\n').splitlines();require(lines and lines[0]=='rive-golden-stream-v1','header');state=dict(t=[0.,0.],clip=None);stack=[];draws=[];firstclip=False;frames=clears=samples=finished=0
 paint=r'\{id=\d+,style=fill,color=0x[0-9a-fA-F]{8},thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=0\}'
 def rectangle(path):
  require(re.fullmatch(r'path=\{id=\d+,fillRule=[02],path=\{verbs=\[(?:move,line,line,line,close|move,line,line,line,line,close)\],points=\[.*\]\}\}',path),'unqualified rectangle verbs/fill')
  b=grammar.aabb('clipPath '+path);return [f32(b[0]+state['t'][0]),f32(b[1]+state['t'][1]),f32(b[2]+state['t'][0]),f32(b[3]+state['t'][1])]
 for line in lines[1:]:
  require(not finished,'command after frame terminator')
  token=line.split(' ',1)[0]
  if line=='save':stack.append(copy.deepcopy(state))
  elif line=='restore':require(stack,'stack underflow');state=stack.pop()
  elif token=='transform':
   m=re.fullmatch(r'transform matrix=\[('+N+r'(?:,'+N+r'){5})\]',line);require(m,'matrix grammar');v=[f32(x)for x in m[1].split(',')];require(v[:4]==[1,0,0,1],'nonidentity linear matrix');state['t']=[f32(state['t'][0]+v[4]),f32(state['t'][1]+v[5])]
  elif token=='clipPath':
   b=rectangle(line[len('clipPath '):])
   if not firstclip:require(b==[0.,0.,w,h]and state['clip']is None,'missing artboard clip');firstclip=True
   state['clip']=b if state['clip']is None else intersect(state['clip'],b)
  elif token=='drawPath':
   m=re.fullmatch(r'drawPath (path=\{.*\}) paint=('+paint+r')',line);require(m,'draw grammar');require(state['clip']is not None,'draw outside artboard clip');b=rectangle(m[1]);color=int(re.search(r'color=(0x[0-9a-fA-F]{8})',m[2])[1],16);b=intersect(b,state['clip'])
   if color>>24 and b[2]>b[0]and b[3]>b[1]:draws.append(dict(rect=b,color=color))
  elif token=='makeRenderPaint':require(re.fullmatch('makeRenderPaint '+paint,line),'paint resource grammar')
  elif token=='makeEmptyRenderPath':require(re.fullmatch(r'makeEmptyRenderPath \{id=\d+,fillRule=0,path=\{verbs=\[\],points=\[\]\}\}',line),'path resource grammar')
  elif token=='frameSize':require(line==f'frameSize width={w} height={h}'and not frames,'frame metadata');frames+=1
  elif token=='clearColor':require(line=='clearColor value=0xffffffff'and not clears,'clear metadata');clears+=1
  elif line=='frame':finished+=1
  elif token=='sample':require(re.fullmatch(r'sample seconds='+N,line)and not samples,'sample metadata');samples+=1
  else:raise ValueError('unknown command '+token)
 require(firstclip and frames==clears==samples==finished==1 and not stack and state==dict(t=[0.,0.],clip=None),'incomplete stream')
 return draws

def observe(text,w,h,expected_frames=None):
 lines=text.rstrip('\n').splitlines();require(lines and lines[0]=='rive-golden-stream-v1','header')
 chunk=['rive-golden-stream-v1'];results=[];last_size=None
 for line in lines[1:]:
  if not line:continue
  chunk.append(line)
  if line=='frame':
   sizes=[re.fullmatch(r'frameSize width=(\d+) height=(\d+)',l)for l in chunk if l.startswith('frameSize')]
   require(len(sizes)==1 and sizes[0],'frame size grammar');a,b=map(int,sizes[0].groups())
   results.append(observe_one('\n'.join(chunk),a,b));last_size=(a,b);chunk=['rive-golden-stream-v1']
 require(len(chunk)==1 and results and last_size==(w,h),'incomplete or wrong requested frame')
 if expected_frames is not None:require(len(results)==expected_frames,'cumulative frame count')
 return results[-1]

def controls(text,w,h):
 original=observe(text,w,h);tests=dict(unknown=text+'\nunknown value=1',stack=text+'\nsave',linear=text.replace('transform matrix=[1,0,0,1,0,0]','transform matrix=[2,0,0,1,0,0]',1),curve=text.replace('verbs=[move,line,line,line,close]','verbs=[move,cubic,line,line,close]',1))
 results=[]
 for name,modified in tests.items():
  require(modified!=text,'control did not mutate')
  try:observe(modified,w,h)
  except ValueError as e:results.append(dict(name=name,rejected=str(e)))
  else:raise ValueError('control accepted '+name)
 # Sequence comparison itself must preserve overlapping alpha order and color.
 changed=copy.deepcopy(original);require(changed,'no painted draw');changed[-1]['color']^=1;require(changed!=original,'color comparison control')
 results.append(dict(name='changed-color',rejected='effective sequence differs'))
 return results
if __name__=='__main__':
 out=Path(sys.argv[1]).resolve();rows=[];failures=[];negative=[];inputs=[]
 for c in json.loads((out/'cases.json').read_text()):
  name=c['name'];dirs=[out/(v+'-capture')/name/'probe'for v in ['original','folded']];frames=[json.loads((d/'frames.json').read_text())['frames']for d in dirs];require(len(frames[0])==len(frames[1])==8,'frame count')
  for i in range(8):
   sequences=[];errors=[]
   for d,fs in zip(dirs,frames):
    f=fs[i];p=d/f['stream'];inputs.append(bind(p));text=p.read_text()
    try:
     sequences.append(observe(text,f['width'],f['height'],i+1))
     if i==0:negative.append(dict(name=name,variant=d.parent.parent.name,controls=controls(text,f['width'],f['height'])))
    except ValueError as e:sequences.append(None);errors.append(str(e))
   row=dict(name=name,frame=i,original=sequences[0],folded=sequences[1],errors=errors,equal=not errors and sequences[0]==sequences[1]);rows.append(row)
   if not row['equal']:failures.append(row)
 receipt=dict(scope='Effective clipped axis-aligned solid rectangle draw sequence, in actual back-to-front stream order; no pixel identity inference',frames=len(rows),failedFrames=len(failures),failures=failures,rows=rows,negativeControls=negative,inputs=inputs,script=bind(__file__),grammar=bind(M/'validation/wrapped-clip-stream-check.py'))
 (out/'stream-observation-r3.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(dict(frames=len(rows),failedFrames=len(failures))))
