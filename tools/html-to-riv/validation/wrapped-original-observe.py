#!/usr/bin/env python3
"""Offline original/integral/rounded Derived paint observation; never renders or compiles.

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

spec=importlib.util.spec_from_file_location('integral_observer',M/'validation/wrapped-integral-observe.py');integral_observer=importlib.util.module_from_spec(spec);spec.loader.exec_module(integral_observer)

def nonoverlap(rectangles):
 for i,a in enumerate(rectangles):
  require(all(math.isfinite(v)and v==math.trunc(v)for v in a)and a[2]>a[0]and a[3]>a[1],'original paint requires positive integral bounds')
  for b in rectangles[:i]:require(max(a[0],b[0])>=min(a[2],b[2])or max(a[1],b[1])>=min(a[3],b[3]),'original paint interiors overlap')

def observe(row,case):
 proof=json.loads((case/'proof.json').read_text());owners=proof.get('originalPaintOwners')
 result=integral_observer.observe(row,case)
 if owners is None:return result
 trace=json.loads((case/'trace.json').read_text());recipe=json.loads((case.parent/'recipe.json').read_text())
 require(len(set(owners))==len(owners)and set(owners)==set(trace['visible']),'original paint owner coverage')
 require(set(owners)==set(proof['integralPaintOwners'])and not proof['paintBoxes'],'original paint must cover exactly integral owners')
 require(set(owners).issubset({r['owner']for r in proof['normalization']['roles']}),'original source normalization ownership')
 require(proof['recordCosts'][2]==0,'original paint has nonempty suffix')
 objects={o['objectId']:o for o in row['geometry']};paints=[]
 for i,owner in enumerate(trace['visible']):
  o=objects[owner];x,y=map(f32,o['worldMatrix'][4:]);w,h=f32(o['width']),f32(o['height']);bounds=[x,y,f32(x+w),f32(y+h)]
  require(w>0 and h>0 and w==math.trunc(w)and h==math.trunc(h),'original paint positive integral extent')
  paints.append(dict(owner=owner,original=True,integral=True,bounds=bounds,color=recipe['slots'][i]['color'],masks=[]))
 nonoverlap([r['bounds']for r in paints]);result['replicas']=paints;result['originalOwners']=owners;return result

def stream_observe(lines,w,h,replicas):
 if not replicas or not replicas[0].get('original'):return integral_observer.stream_observe(lines,w,h,replicas)
 art=[0.,0.,f32(w),f32(h)];state={'translation':[0.,0.],'clips':[]};stack=[];observed=[];draws=[];remaining={r['owner']:r for r in replicas if r['color']>>24!=0};visible_count=len(remaining);transparent=[r['owner']for r in replicas if r['color']>>24==0]
 nonoverlap([r['bounds']for r in replicas])
 for index,line in enumerate(lines):
  token=line.split(' ',1)[0]
  if line=='save':stack.append(copy.deepcopy(state))
  elif line=='restore':require(stack,'restore underflow');state=stack.pop()
  elif token=='transform':
   m=re.fullmatch(r'transform matrix=\[('+N+r'(?:,'+N+r'){5})\]',line);require(m,'invalid transform');matrix=[f32(v)for v in m[1].split(',')];require(matrix[:4]==[1,0,0,1],'unsupported linear transform')
   state['translation']=[f32(a+b)for a,b in zip(state['translation'],matrix[4:])];require(all(math.isfinite(v)for v in state['translation']),'nonfinite translation')
  elif token=='clipPath':
   b=clips.aabb(line);require(not observed and not state['clips']and state['translation']==[0,0]and b==art,'original paint requires only the artboard clip');state['clips'].append(b);observed.append(dict(command=index,bounds=b,intersection=b))
  elif token=='drawPath':
   m=re.fullmatch(r'drawPath (path=\{.*\}) paint=\{id=\d+,style=fill,color=(0x[0-9a-fA-F]{8}),thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=0\}',line);require(m,'unrecognized original paint serialization');b=clips.aabb('clipPath '+m[1]);color=int(m[2],16)
   require(state['clips']==[art],'original draw escaped artboard clip')
   if not draws:require(color==0xffffffff and b==art and state['translation']==[0,0],'incorrect file background');owner=None;world=art
   else:
    matches=[]
    for owner_id,r in remaining.items():
     box=r['bounds'];local=[0,0,f32(box[2]-box[0]),f32(box[3]-box[1])]
     if color==r['color']and ((state['translation']==[0,0]and b==box)or(state['translation']==box[:2]and b==local)):matches.append(owner_id)
    require(len(matches)==1,'missing/duplicate/unexpected original owner paint');owner=matches[0];world=remaining.pop(owner)['bounds']
   intersection=[max(0.,world[0]),max(0.,world[1]),min(w,world[2]),min(h,world[3])]
   draws.append(dict(command=index,owner=owner,color=color,bounds=b,translation=list(state['translation']),masks=[art],intersection=intersection,empty=intersection[0]>=intersection[2]or intersection[1]>=intersection[3]))
  elif token in ['makeRenderPaint','makeEmptyRenderPath']:require(re.fullmatch(token+r' \{.*\}',line),'bad resource')
  elif token=='sample':require(re.fullmatch(r'sample seconds='+N,line),'bad sample')
  elif token=='frameSize':require(line==f'frameSize width={w} height={h}','wrong frame size')
  elif token=='clearColor':require(line=='clearColor value=0xffffffff','wrong clear')
  else:raise ValueError('unsupported command '+token)
 require(not remaining and len(draws)==1+visible_count and len(observed)==1,'missing original paint/clip')
 require(not stack and state=={'translation':[0.,0.],'clips':[]},'incomplete state restoration')
 return dict(clips=observed,draws=draws,omittedTransparentOwners=transparent)

def controls(lines,w,h,replicas):
 if not replicas or not replicas[0].get('original'):yield from integral_observer.controls(lines,w,h,replicas);return
 first=next(i for i,l in enumerate(lines)if l.startswith('clipPath'));paint=[i for i,l in enumerate(lines)if l.startswith('drawPath')][1]
 tests={'extra-clip':lines[:paint]+[lines[first]]+lines[paint:],'missing-original':lines[:paint]+lines[paint+1:],'duplicate-original':lines[:paint]+[lines[paint]]+lines[paint:],'unknown':lines+['unknownClipPolicy value=1'],'stack':lines+['save'],'matrix':lines[:paint]+['transform matrix=[1,0,0,1,0.5,0]']+lines[paint:],'color':lines[:paint]+[re.sub(r'color=0x[0-9a-fA-F]{8}','color=0x01020304',lines[paint])]+lines[paint+1:]}
 if any(r['color']>>24==0 for r in replicas):
  tests['unexpected-transparent-draw']=lines[:paint]+[re.sub(r'color=0x[0-9a-fA-F]{8}','color=0x00000000',lines[paint])]+lines[paint:]
 for name,modified in tests.items():
  try:stream_observe(modified,w,h,replicas)
  except ValueError as e:yield dict(name=name,rejected=str(e))
  else:raise ValueError('negative control accepted '+name)
 if any(r['color']>>24==0 for r in replicas):
  changed=copy.deepcopy(replicas);next(r for r in changed if r['color']>>24==0)['color']=0x01000000
  try:stream_observe(lines,w,h,changed)
  except ValueError as e:yield dict(name='nonzero-alpha-owner-cannot-be-omitted',rejected=str(e))
  else:raise ValueError('nonzero alpha omission accepted')
 if replicas:
  changed=copy.deepcopy(replicas);changed.append(copy.deepcopy(changed[0]))
  try:stream_observe(lines,w,h,changed)
  except ValueError as e:yield dict(name='overlap',rejected=str(e))
  else:raise ValueError('negative overlap accepted')

def main():
 root=Path(sys.argv[1]).resolve();reobserve=len(sys.argv)>2 and sys.argv[2]=='--reobserve';receipt=json.loads((root/'receipt.json').read_text());out=root/('original-observation-r2'if reobserve else'original-observation');require(not out.exists(),'fresh output required');out.mkdir();visual=root/'visual';require(visual.exists()if reobserve else not visual.exists(),'visual output state mismatch');
 if not reobserve:visual.mkdir()
 observations=[];representatives=[];transfers=[];seen={};negative={}
 for row in receipt['rows']:
  prefix=Path(row['prefix']);case=prefix.parent/'derived-construction';o=observe(row,case);stream=prefix.parent/'probe'/row['stream'];require(sha(stream)==row['streamSha256'],'stream hash mismatch')
  lines=clips.extract(stream,row['frame'])
  try:s=stream_observe(lines,row['width'],row['height'],o['replicas']);error=None
  except ValueError as e:s=None;error=str(e)
  kind=('original-transparent'if any(r['color']>>24==0 for r in o['replicas'])else'original')if o.get('originalOwners')is not None else'mixed'if any(x['integral']for x in o['replicas'])and any(not x['integral']for x in o['replicas'])else'integral'if all(x['integral']for x in o['replicas'])else'rounded'
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
   p=visual/f'sheet-{len(sheets):02}.png'
   if reobserve:
    import io
    image_bytes=io.BytesIO();canvas.save(image_bytes,format='PNG');require(p.read_bytes()==image_bytes.getvalue(),'reobservation changed review image')
   else:canvas.save(p)
   sheets.append(dict(**bind(p),members=members))
 result=dict(scope='Actual private Derived scalar and strict command observation, not direct GPU-state or public qualification',source=bind(root/'receipt.json'),script=bind(__file__),parser=bind(M/'validation/wrapped-clip-stream-check.py'),fallbackObserver=bind(M/'validation/wrapped-integral-observe.py'),transparentCullSources=[bind(M.parents[1]/p)for p in ['crates/nuxie-runtime/src/mechanical_port/source/layout_component.rs','crates/nuxie-runtime/src/mechanical_port/source/shapes/paint/solid_color.rs','crates/nuxie-runtime/src/mechanical_port/source/shapes/paint/shape_paint.rs']],negativeControls=negative,frames=observations,counts=dict(frames=len(observations),originalFrames=sum(o['scalar'].get('originalOwners')is not None for o in observations),originalOwnerChecks=sum(len(o['scalar'].get('originalOwners',[]))for o in observations),integralChecks=sum(len(o['scalar']['integralChecks'])for o in observations),integralFailures=sum(not c['pass_']for o in observations for c in o['scalar']['integralChecks']),scalarChecks=sum(len(o['scalar']['checks'])for o in observations),scalarFailures=sum(len(o['scalar']['failures'])for o in observations),zeroSignDifferences=sum(o['scalar']['zeroSignDifferences']for o in observations),streamFailures=sum(o['streamError']is not None for o in observations),chromeEdgeComparisons=sum(len(o['scalar']['edges'])for o in observations),chromeEdgeDifferences=sum(not e['same']for o in observations for e in o['scalar']['edges']),chromeClippedEdgeDifferences=sum(not e['clippedSame']for o in observations for e in o['scalar']['edges']),saturationIntersectionFailures=sum(not e['saturationPreservesNativeIntersection']for o in observations for e in o['scalar']['edges'])))
 (out/'receipt.json').write_text(json.dumps(result,indent=2)+'\n');coverage=dict(scope='Private boundary Derived campaign; direct visual review pending; exact repeat transfers recorded; raw and artboard-clipped edge differences remain distinct',receipt=bind(root/'receipt.json'),script=bind(__file__),observation=bind(out/'receipt.json'),sheets=sheets,transfers=transfers,counts=dict(frames=len(receipt['rows']),representatives=len(representatives),transfers=len(transfers),sheets=len(sheets),geometryPass=sum(not r['geometryFailures']for r in receipt['rows']),pixelPass=sum(not r['pixelFailures']for r in receipt['rows'])))
 (visual/('coverage-r2.json'if reobserve else'coverage.json')).write_text(json.dumps(coverage,indent=2)+'\n');print(json.dumps(dict(observation=result['counts'],visual=coverage['counts'])))
if __name__=='__main__':main()
