"""Offline closed command reader for direct fixed_paint experiment (no source proof)."""
import hashlib,json,re,sys
from pathlib import Path
out=Path(sys.argv[1]).resolve();sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
number=r'-?(?:\d+(?:\.\d*)?|\.\d+)(?:e[+-]?\d+)?'
quad=re.compile(r'path=\{id=\d+,fillRule=([02]),path=\{verbs=\[(move,line,line,line(?:,line)?,close)\],points=\[(.*?)\]\}\}')
paint=re.compile(r'paint=\{id=\d+,style=fill,color=0x([0-9a-f]{8}),thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=0\}')
def check(v,message):
 if not v:raise ValueError(message)
# Artboard path is reused by the white fill: initial clip rule2 becomes rule0
# on later frames. Both rules have identical coverage for this strict simple quad.
def path_rect(text,rule):
 match=quad.fullmatch(text);check(match is not None and int(match[1]) in rule,'closed rectangle grammar')
 pairs=re.findall(r'\(('+number+'),('+number+r')\)',match[3]);check(','.join(f'({x},{y})'for x,y in pairs)==match[3],'point grammar');points=[(float(x),float(y))for x,y in pairs]
 check(len(points) in [4,5],'point count');l,t=points[0];r,b=points[2]
 check(points==[(l,t),(r,t),(r,b),(l,b)]+([(l,t)]if len(points)==5 else []),'axis-aligned path');return [l,t,r,b]
def observe(lines,c,f):
 expected=[([0,0,f['width'],f['height']],0xffffffff)]+[([r[k]for k in ['left','top','right','bottom']],r['color'])for r in c['rects']if r['left']<r['right'] and r['top']<r['bottom'] and r['color']>>24]
 stack=[];clip=False;draws=[];clip_count=0
 for line in lines:
  if line=='save':stack.append(clip)
  elif line=='restore':check(bool(stack),'stack underflow');clip=stack.pop()
  elif line=='transform matrix=[1,0,0,1,0,0]':pass
  elif line.startswith('clipPath '):
   check(path_rect(line[9:],(0,2))==[0,0,f['width'],f['height']],'artboard clip');check(not clip,'extra clip');clip=True;clip_count+=1
  elif line.startswith('drawPath '):
   split=line[9:].split(' paint=');check(len(split)==2,'draw grammar');rect=path_rect(split[0],(0,));p=paint.fullmatch('paint='+split[1]);check(p is not None,'paint grammar');check(clip,'draw outside artboard clip');draws.append((rect,int(p[1],16)))
  elif re.fullmatch(r'makeRenderPaint \{id=\d+,style=fill,color=0xff000000,thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=0\}',line):pass
  elif re.fullmatch(r'makeEmptyRenderPath \{id=\d+,fillRule=0,path=\{verbs=\[\],points=\[\]\}\}',line):pass
  elif line==f'frameSize width={f["width"]} height={f["height"]}':pass
  elif line=='clearColor value=0xffffffff':pass
  elif line==f'sample seconds={f["frame"]}':pass
  else:raise ValueError('unknown command: '+line)
 check(not stack and not clip and clip_count==1,'unbalanced stack or clip count');check(draws==expected,'ordered rectangle/color mismatch');return draws
cases=json.loads((out/'cases.json').read_text());reports=[];controls=[]
for c in cases:
 p=out/'cases'/c['name'];manifest=p/'probe/frames.json'
 for f in json.loads(manifest.read_text())['frames']:
  source=p/'probe'/f['stream'];lines=source.read_text().splitlines();check(lines.pop(0)=='rive-golden-stream-v1','stream version');frames=[];current=[]
  for line in lines:
   if line=='frame':frames.append(current);current=[]
   else:current.append(line)
  check(not current and len(frames)==f['frame']+1,'cumulative frame count');lines=frames[f['frame']];draws=observe(lines,c,f)
  reports.append(dict(name=c['name'],frame=f['frame'],stream=str(source),sha256=sha(source),draws=draws))
  if not controls:
   indices=[i for i,s in enumerate(lines)if s.startswith('drawPath ')];mutations={'unknown-command':lines+['policy enabled=1'],'missing-paint':[s for i,s in enumerate(lines)if i!=indices[-1]],'wrong-color':[s.replace('color=0xffffcc22','color=0xff000000')for s in lines],'wrong-transform':[s.replace('matrix=[1,0,0,1,0,0]','matrix=[1,0,0,1,1,0]')for s in lines]}
   changed=lines.copy();changed[indices[-1]],changed[indices[-2]]=changed[indices[-2]],changed[indices[-1]];mutations['paint-order']=changed
   for name,changed in mutations.items():
    try:observe(changed,c,f)
    except ValueError as e:controls.append(dict(name=name,rejected=str(e)))
    else:raise ValueError('negative control accepted: '+name)
receipt=dict(scope='Strict actual native rectangle/color order for direct fixed_paint descriptors only; no source/layout proof',scriptSha256=sha(Path(__file__)),casesSha256=sha(out/'cases.json'),frames=reports,negativeControls=controls)
(out/'stream-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(dict(frames=len(reports),controls=len(controls))))
