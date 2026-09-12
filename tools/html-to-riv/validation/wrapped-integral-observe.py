#!/usr/bin/env python3
"""Offline mixed integral/rounded Derived paint observation; never renders or compiles.

Requires the completed canonical campaign receipt. Exact native f32 observations
are separate from Chrome LayoutUnit edge comparisons and pixel qualification.
Raw edge differences are retained. Saturated edges are independently intersected
with the artboard interval before determining visible interval equivalence.
"""
from pathlib import Path
import collections,copy,hashlib,importlib.util,json,math,re,struct,sys
from fractions import Fraction
from PIL import Image,ImageDraw
M=Path(__file__).resolve().parents[1]
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
bind=lambda p:dict(path=str(Path(p).resolve()),sha256=sha(p))
f32=lambda x:struct.unpack('<f',struct.pack('<f',float(x)))[0]
bits=lambda x:struct.unpack('<I',struct.pack('<f',float(x)))[0]
def require(value,message):
 if not value:raise ValueError(message)
def rounded(x):
 q=Fraction(x)+Fraction(1,2);return q.numerator//q.denominator
spec=importlib.util.spec_from_file_location('clips',M/'validation/wrapped-clip-stream-check.py');clips=importlib.util.module_from_spec(spec);spec.loader.exec_module(clips)
N=clips.NUMBER

def stream_observe(lines,w,h,replicas):
 art=[0.,0.,f32(w),f32(h)];state={'translation':[0.,0.],'clips':[]};stack=[];observed=[];draws=[]
 allowed={tuple(art)}|{tuple(b)for r in replicas for b in r['masks']}
 for index,line in enumerate(lines):
  token=line.split(' ',1)[0]
  if line=='save':stack.append(copy.deepcopy(state))
  elif line=='restore':
   require(stack,'restore underflow');state=stack.pop()
  elif token=='transform':
   match=re.fullmatch(r'transform matrix=\[('+N+r'(?:,'+N+r'){5})\]',line);require(match,'invalid transform')
   matrix=[f32(v)for v in match[1].split(',')];require(matrix[:4]==[1,0,0,1],'unsupported linear matrix')
   state['translation']=[f32(a+b)for a,b in zip(state['translation'],matrix[4:])]
   require(all(math.isfinite(v)for v in state['translation']),'nonfinite translation')
  elif token=='clipPath':
   box=clips.aabb(line);require(tuple(box)in allowed,'unknown mask AABB');require(state['translation']==[0,0],'nonidentity clip')
   if not observed:require(box==art and not state['clips'],'incorrect initial artboard clip')
   else:require(state['clips'] and state['clips'][0]==art,'mask escaped artboard clip')
   state['clips'].append(box);intersection=list(art)
   for b in state['clips'][1:]:intersection=[max(intersection[0],b[0]),max(intersection[1],b[1]),min(intersection[2],b[2]),min(intersection[3],b[3])]
   observed.append(dict(command=index,bounds=box,intersection=intersection))
  elif token=='drawPath':
   match=re.fullmatch(r'drawPath (path=\{.*\}) paint=\{id=\d+,style=fill,color=(0x[0-9a-fA-F]{8}),thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=0\}',line)
   require(match,'unrecognized paint serialization')
   bounds=clips.aabb('clipPath '+match[1]);color=int(match[2],16);i=len(draws)
   if i==0:
    require(color==0xffffffff and bounds==art and state['clips']==[art] and state['translation']==[0,0],'incorrect file-owned background')
    expected=[];owner=None
   else:
    require(i<=len(replicas),'extra drawable');r=replicas[i-1];expected=r['masks'];owner=r['owner']
    require(color==r['color'],'replica color/draw order mismatch')
    if r['integral']:
     box=r['bounds'];local=[0,0,f32(box[2]-box[0]),f32(box[3]-box[1])]
     require((state['translation']==[0,0]and bounds==box)or(state['translation']==box[:2]and bounds==local),'incorrect integral foreground rectangle/transform')
    else:require(bounds==[0,0,32768,32768]and state['translation']==[0,0],'incorrect rounded replica rectangle/transform')
    require(state['clips']==[art]+expected,'replica box quad or line clips mismatch')
   intersection=list(art)
   for b in expected:intersection=[max(intersection[0],b[0]),max(intersection[1],b[1]),min(intersection[2],b[2]),min(intersection[3],b[3])]
   if i>0 and replicas[i-1]['integral']:
    b=replicas[i-1]['bounds'];intersection=[max(intersection[0],b[0]),max(intersection[1],b[1]),min(intersection[2],b[2]),min(intersection[3],b[3])]
   require(intersection[0]>=0 and intersection[1]>=0 and intersection[2]<=w and intersection[3]<=h,'intersection escaped artboard')
   draws.append(dict(command=index,owner=owner,color=color,bounds=bounds,translation=list(state['translation']),masks=copy.deepcopy(state['clips']),intersection=intersection,empty=intersection[0]>=intersection[2]or intersection[1]>=intersection[3]))
  elif token in ['makeRenderPaint','makeEmptyRenderPath']:require(re.fullmatch(token+r' \{.*\}',line),'bad resource')
  elif token=='sample':require(re.fullmatch(r'sample seconds='+N,line),'bad sample')
  elif token=='frameSize':require(line==f'frameSize width={w} height={h}','wrong frame size')
  elif token=='clearColor':require(line=='clearColor value=0xffffffff','unexpected clear color')
  else:raise ValueError('unsupported command '+token)
 require(not stack and state=={'translation':[0.,0.],'clips':[]},'incomplete stack/state restoration')
 require(len(draws)==1+len(replicas),'missing replicas');require(len(observed)==1+sum(len(r['masks'])for r in replicas),'missing or extra mask applications')
 return dict(clips=observed,draws=draws)

def controls(lines,w,h,replicas):
 first=next(i for i,l in enumerate(lines)if l.startswith('clipPath'));tests={
 'matrix':lines[:first]+['transform matrix=[1,0,0,1,1,0]']+lines[first:],
 'verb':lines[:first]+[lines[first].replace('verbs=[move,line','verbs=[move,cubic',1)]+lines[first+1:],
 'stack':lines+['save'],'unknown':lines+['unknownClipPolicy value=1']}
 mask=next((i for i,l in enumerate(lines)if i>first and l.startswith('clipPath')),None)
 if mask is not None:tests['missing-mask']=lines[:mask]+lines[mask+1:]
 paint=next(i for i,l in enumerate(lines)if l.startswith('drawPath')and 'color=0xffffffff'not in l)
 tests['wrong-paint-color']=lines[:paint]+[re.sub(r'color=0x[0-9a-fA-F]{8}','color=0x01020304',lines[paint])]+lines[paint+1:]
 tests['missing-replica']=lines[:paint]+lines[paint+1:]
 for name,modified in tests.items():
  try:stream_observe(modified,w,h,replicas)
  except ValueError as e:yield dict(name=name,rejected=str(e))
  else:raise ValueError('negative control accepted '+name)

def clipped_interval(edge,extent):
 # Empty intervals have a canonical representation; off-artboard endpoint
 # differences cannot create visible coverage. No tolerance or browser geometry
 # affects native file construction or the independent scalar/stream checks.
 lo=max(0.,edge[0]);hi=min(float(extent),edge[1])
 return None if lo>=hi else [lo,hi]

def observe(row,case):
 proof=json.loads((case/'proof.json').read_text());recipe=json.loads((case.parent/'recipe.json').read_text());trace=json.loads((case/'trace.json').read_text())
 objects={o['objectId']:o for o in row['geometry']};checks=[];edge_comparisons=[];quads={};zero_sign=0;integral_checks=[]
 integral=set(proof['integralPaintOwners']);require(len(integral)==len(proof['integralPaintOwners']),'duplicate integral owners')
 rounded_owners=[b['geometry']for b in proof['paintBoxes']];require(len(set(rounded_owners))==len(rounded_owners),'duplicate rounded owners')
 require(not integral.intersection(rounded_owners)and integral.union(rounded_owners)==set(trace['visible']),'integral/rounded owner partition')
 def get(id,axis):
  o=objects[id];matrix=[f32(v)for v in o['worldMatrix']];require(matrix[:4]==[1,0,0,1],'nonidentity scalar linear transform');return matrix[4+axis]
 def check(label,id,axis,expected):
  nonlocal zero_sign
  actual=get(id,axis);expected=f32(expected);ok=actual==expected
  zero_sign+=int(ok and bits(actual)!=bits(expected));checks.append(dict(label=label,object=id,axis=axis,expected=expected,actual=actual,pass_=ok))
 for box in proof['paintBoxes']:
  i=trace['visible'].index(box['geometry'])
  owner=objects[box['geometry']];matrix=[f32(v)for v in owner['worldMatrix']];require(matrix[:4]==[1,0,0,1],'nonidentity visible owner')
  endpoints=[]
  for axis,a in enumerate(box['axes']):
   start=matrix[4+axis];extent=f32(owner['width'if axis==0 else'height']);end=f32(start+extent)
   for n,value in zip(box['corners'],[start,end]):check('raw-corner',n,axis,value)
   sat=[max(-16384.,min(16384.,v))for v in[start,end]];rs=[rounded(v)for v in sat]
   for n,v in zip(a['saturated'],sat):check('saturated',n,axis,v);check('saturated-orthogonal',n,1-axis,0)
   for n,v in zip(a['rounded'],rs):check('rounded',n,axis,v);check('rounded-orthogonal',n,1-axis,0)
   size=f32(end-start);threshold=f32(size-1/16);check('raw-size',a['size'],axis,size);check('thin-threshold',a['threshold'],axis,threshold)
   stage=max(0.,min(1.,threshold))
   for k,n in enumerate(a['stages']):
    if k:stage=max(0.,min(1.,f32(stage*2**64)))
    check('thin-stage-'+str(k),n,axis,stage)
   flag=int(size>1/16);require(stage==flag,'independent predicate reference disagreement');minimum=rs[0]+flag;last=max(rs[1],minimum)
   check('minimum-end',a['minimum'],axis,minimum);check('final-end',a['finalEnd'],axis,last);check('final-end-orthogonal',a['finalEnd'],1-axis,0)
   endpoints.append([rs[0],last])
   chrome=row['boxes'][recipe['slots'][i]['name']];cstart=chrome['x'if axis==0 else'y'];csize=chrome['width'if axis==0 else'height'];cs=rounded(cstart);ce=max(rounded(cstart+csize),cs+int(csize>1/16))
   native_edge=[rs[0],last];chrome_edge=[cs,ce]
   raw_start=rounded(start);raw_native=[raw_start,max(rounded(end),raw_start+flag)]
   viewport=row['width'if axis==0 else'height']
   native_clip=clipped_interval(native_edge,viewport);chrome_clip=clipped_interval(chrome_edge,viewport);raw_clip=clipped_interval(raw_native,viewport)
   edge_comparisons.append(dict(owner=box['geometry'],axis=axis,native=native_edge,nativeRaw=raw_native,chrome=chrome_edge,same=native_edge==chrome_edge,nativeSize=size,chromeSize=csize,viewport=viewport,nativeClipped=native_clip,chromeClipped=chrome_clip,rawNativeClipped=raw_clip,clippedSame=native_clip==chrome_clip,saturationPreservesNativeIntersection=native_clip==raw_clip))
  (l,r),(t,b)=endpoints;quad=[[l,0,l+32768,32768],[r-32768,0,r,32768],[0,t,32768,t+32768],[0,b-32768,32768,b]];quads[box['geometry']]=quad
  for k,n in enumerate(box['masks']):
   axis=k//2;value=endpoints[axis][k%2];check('mask-axis',n,axis,value);check('mask-orthogonal',n,1-axis,0)
 for owner_id in integral:
  owner=objects[owner_id];matrix=[f32(v)for v in owner['worldMatrix']];require(matrix[:4]==[1,0,0,1],'integral owner linear transform')
  i=trace['visible'].index(owner_id);chrome=row['boxes'][recipe['slots'][i]['name']]
  for axis in [0,1]:
   start=matrix[4+axis];extent=f32(owner['width'if axis==0 else'height']);end=f32(start+extent)
   valid=all(math.isfinite(v)and v==math.trunc(v)for v in [start,extent,end])and extent>=0
   integral_checks.append(dict(owner=owner_id,axis=axis,start=start,extent=extent,end=end,pass_=valid))
   require(valid,'certified integral owner has nonintegral native edge/extent')
   cs=chrome['x'if axis==0 else'y'];size=chrome['width'if axis==0 else'height'];lo=rounded(cs);ce=[lo,max(rounded(cs+size),lo+int(size>1/16))];ne=[start,end];viewport=row['width'if axis==0 else'height'];nc=clipped_interval(ne,viewport);cc=clipped_interval(ce,viewport)
   edge_comparisons.append(dict(owner=owner_id,axis=axis,native=ne,nativeRaw=ne,chrome=ce,same=ne==ce,nativeSize=extent,chromeSize=size,viewport=viewport,nativeClipped=nc,chromeClipped=cc,rawNativeClipped=nc,clippedSame=nc==cc,saturationPreservesNativeIntersection=True))
  quads[owner_id]=[]
 cross=1 if recipe['row']else 0;f=1-recipe['lineFraction']if recipe['wrap']==2 else recipe['lineFraction'];anchors=[]
 for n in trace['slots']:
  o=objects[n];size=f32(o['height'if cross else'width']);anchors.append(f32(get(n,cross)+f32(size*f)))
 def mask(active):
  q=[0,0,32768,32768]
  if not active:q[cross]=65536;q[cross+2]=98304
  return q
 def same(i,j):return abs(f32(anchors[j]-anchors[i]))<=proof['epsilon']
 groups=list(range(len(trace['visible'])));members=groups.copy()
 if recipe['wrap']==2:groups.reverse()
 if recipe['reverseMain']:members.reverse()
 replicas=[]
 for i in groups:
  for j in members:
   if j<i:continue
   owner_id=trace['visible'][j];masks=copy.deepcopy(quads[owner_id])
   if i:masks.append(mask(not same(i-1,i)))
   if j!=i:masks.append(mask(same(i,j)))
   o=objects[owner_id];x,y=map(f32,o['worldMatrix'][4:]);bounds=[x,y,f32(x+f32(o['width'])),f32(y+f32(o['height']))]
   replicas.append(dict(owner=owner_id,integral=owner_id in integral,bounds=bounds,color=recipe['slots'][j]['color'],masks=masks,group=i,member=j))
 return dict(integralChecks=integral_checks,checks=checks,failures=[x for x in checks if not x['pass_']],zeroSignDifferences=zero_sign,edges=edge_comparisons,replicas=replicas,proof=bind(case/'proof.json'),recipe=bind(case.parent/'recipe.json'),trace=bind(case/'trace.json'))

def main():
 root=Path(sys.argv[1]).resolve();receipt=json.loads((root/'receipt.json').read_text());out=root/'integral-observation';require(not out.exists(),'fresh output required');out.mkdir();visual=root/'visual';require(not visual.exists(),'fresh visual output required');visual.mkdir()
 observations=[];representatives=[];transfers=[];seen={};negative={}
 for row in receipt['rows']:
  prefix=Path(row['prefix']);case=prefix.parent/'derived-construction';o=observe(row,case);stream=prefix.parent/'probe'/row['stream'];require(sha(stream)==row['streamSha256'],'stream hash mismatch')
  lines=clips.extract(stream,row['frame'])
  try:s=stream_observe(lines,row['width'],row['height'],o['replicas']);error=None
  except ValueError as e:s=None;error=str(e)
  kind='mixed'if any(x['integral']for x in o['replicas'])and any(not x['integral']for x in o['replicas'])else'integral'if all(x['integral']for x in o['replicas'])else'rounded'
  if kind not in negative and s is not None:negative[kind]=list(controls(lines,row['width'],row['height'],o['replicas']))
  observations.append(dict(name=row['name'],frame=row['frame'],stream=bind(stream),scalar=o,streamObservation=s,streamError=error))
  for kind in ['chrome','native']:require(sha(str(prefix)+'.'+kind+'.png')==row[kind+'Sha256'],'PNG binding mismatch')
  key=(row['name'],row['width'],row['height'])
  if key in seen:
   donor=seen[key];require(all(row[k+'Sha256']==donor[k+'Sha256']for k in ['chrome','native','riv']),'repeat bytes differ');require(row['boxes']==donor['boxes'] and row['geometry']==donor['geometry'],'repeat geometry differs');require(row['pixelFailures']==donor['pixelFailures'] and row['geometryFailures']==donor['geometryFailures'],'repeat gates differ');transfers.append(dict(name=row['name'],frame=row['frame'],donorFrame=donor['frame'],chrome=bind(str(prefix)+'.chrome.png'),native=bind(str(prefix)+'.native.png')))
  else:seen[key]=row;representatives.append(row)
 sheets=[];groups=collections.defaultdict(list)
 for row in representatives:groups[(row['width'],row['height'])].append(row)
 for (w,h),group in groups.items():
  for start in range(0,len(group),4):
   selected=group[start:start+4];canvas=Image.new('RGB',(3*(w+8)+8,len(selected)*(h+50)+26),'#ddd');draw=ImageDraw.Draw(canvas);draw.text((8,5),'Chrome / immutable native / pixel diff - full-size, no scaling',fill='black');members=[]
   for i,row in enumerate(selected):
    top=26+i*(h+50);draw.text((8,top),row['name'],fill='black');draw.text((8,top+14),f"frame {row['frame']} {w}x{h} | geometry {row['geometryFailures']} | pixels {row['pixelFailures']}",fill='black');files=[]
    for j,kind in enumerate(['chrome','native','diff']):
     p=Path(row['prefix']+'.'+kind+'.png');im=Image.open(p).convert('RGB');require(im.size==(w,h),'image dimensions');canvas.paste(im,(8+j*(w+8),top+32));files.append(bind(p))
    members.append(dict(name=row['name'],frame=row['frame'],files=files,pixelFailures=row['pixelFailures']))
   p=visual/f'sheet-{len(sheets):02}.png';canvas.save(p);sheets.append(dict(**bind(p),members=members))
 result=dict(scope='Actual private Derived scalar and strict command observation, not direct GPU-state or public qualification',source=bind(root/'receipt.json'),script=bind(__file__),parser=bind(M/'validation/wrapped-clip-stream-check.py'),negativeControls=negative,frames=observations,counts=dict(frames=len(observations),integralChecks=sum(len(o['scalar']['integralChecks'])for o in observations),integralFailures=sum(not c['pass_']for o in observations for c in o['scalar']['integralChecks']),scalarChecks=sum(len(o['scalar']['checks'])for o in observations),scalarFailures=sum(len(o['scalar']['failures'])for o in observations),zeroSignDifferences=sum(o['scalar']['zeroSignDifferences']for o in observations),streamFailures=sum(o['streamError']is not None for o in observations),chromeEdgeComparisons=sum(len(o['scalar']['edges'])for o in observations),chromeEdgeDifferences=sum(not e['same']for o in observations for e in o['scalar']['edges']),chromeClippedEdgeDifferences=sum(not e['clippedSame']for o in observations for e in o['scalar']['edges']),saturationIntersectionFailures=sum(not e['saturationPreservesNativeIntersection']for o in observations for e in o['scalar']['edges'])))
 (out/'receipt.json').write_text(json.dumps(result,indent=2)+'\n');coverage=dict(scope='Private boundary Derived campaign; direct visual review pending; exact repeat transfers recorded; raw and artboard-clipped edge differences remain distinct',receipt=bind(root/'receipt.json'),script=bind(__file__),observation=bind(out/'receipt.json'),sheets=sheets,transfers=transfers,counts=dict(frames=len(receipt['rows']),representatives=len(representatives),transfers=len(transfers),sheets=len(sheets),geometryPass=sum(not r['geometryFailures']for r in receipt['rows']),pixelPass=sum(not r['pixelFailures']for r in receipt['rows'])))
 (visual/'coverage.json').write_text(json.dumps(coverage,indent=2)+'\n');print(json.dumps(dict(observation=result['counts'],visual=coverage['counts'])))
if __name__=='__main__':main()
