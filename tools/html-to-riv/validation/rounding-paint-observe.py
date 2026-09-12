#!/usr/bin/env python3
"""Observe live rounded paint clips and prepare direct visual evidence.

Does not change scene bytes, renderer, CSS reference or metric gates.
"""
from pathlib import Path
import collections,hashlib,importlib.util,json,re,sys
from PIL import Image,ImageDraw
M=Path(__file__).resolve().parents[1];root=Path(sys.argv[1]).resolve();out=root/'visual';assert not out.exists();out.mkdir()
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
bind=lambda p:dict(path=str(Path(p).resolve()),sha256=sha(p))
r=json.loads((root/'receipt.json').read_text())
spec=importlib.util.spec_from_file_location('path_observer',M/'validation/wrapped-clip-stream-check.py');obs=importlib.util.module_from_spec(spec);spec.loader.exec_module(obs)
path_rows=[];pairs=[];transfers=[];seen={}
for row in r['rows']:
 folder=Path(row['prefix']).parent;manifest=json.loads((folder/'probe/frames.json').read_text());frame=manifest['frames'][row['frame']];stream=folder/'probe'/frame['stream']
 assert sha(stream)==row['streamSha256'];lines=obs.extract(stream,row['frame']);stack=[];translation=[0.,0.];draws=[];clips=[]
 for line in lines:
  if line=='save':stack.append(translation[:])
  elif line=='restore':translation=stack.pop()
  elif line.startswith('transform '):
   vals=[obs.f32(x)for x in line.split('[')[1].split(']')[0].split(',')];assert vals[:4]==[1,0,0,1]
   translation=[obs.f32(a+b)for a,b in zip(translation,vals[4:])]
  elif line.startswith('clipPath '):
   assert translation==[0,0];clips.append(obs.aabb(line))
  elif line.startswith('drawPath '):
   path,paint=line.split(' paint=',1);b=obs.aabb(path.replace('drawPath','clipPath',1));color=re.search(r'color=(0x[0-9a-f]+)',paint)[1]
   assert 'style=fill' in paint and 'feather=0' in paint and 'shader=0' in paint
   world=[obs.f32(b[i]+translation[i%2])for i in range(4)];draws.append(dict(color=color,bounds=world))
 assert not stack and translation==[0,0] and len(clips)==5 and clips[0]==[0,0,row['width'],row['height']]
 assert len(draws)==2 and draws[0]==dict(color='0xffffffff',bounds=[0,0,row['width'],row['height']])
 assert draws[1]==dict(color='0x80ff6030',bounds=[0,0,32768,32768])
 # Derive Chrome's snapped paint bounds from its independently observed layout
 # box; never feed these measurements to native construction.
 import math
 box=row['boxes']['v'];expected=[]
 for axis,extent in [('x','width'),('y','height')]:
  start=math.floor(box[axis]+.5);size=math.floor(box[axis]+box[extent]+.5)-start
  if size==0 and box[extent]>1/16:size=1
  expected.append((start,start+size))
 expected_masks=[[expected[0][0],0,expected[0][0]+32768,32768],[expected[0][1]-32768,0,expected[0][1],32768],[0,expected[1][0],32768,expected[1][0]+32768],[0,expected[1][1]-32768,32768,expected[1][1]]]
 assert clips[1:]==expected_masks,(row['name'],row['frame'],clips[1:],expected_masks)
 intersection=[max(c[0]for c in clips),max(c[1]for c in clips),min(c[2]for c in clips),min(c[3]for c in clips)]
 path_rows.append(dict(name=row['name'],frame=row['frame'],stream=bind(stream),draws=draws,clips=clips,intersection=intersection,chromeSnappedAxes=expected))
 key=(row['name'],row['width'],row['height']);prefix=row['prefix']
 for kind in ['chrome','native']:assert sha(prefix+'.'+kind+'.png')==row[kind+'Sha256']
 if key in seen:
  donor=seen[key]
  assert all(row[k+'Sha256']==donor[k+'Sha256']for k in ['chrome','native','riv'])
  assert row['geometry']==donor['geometry'] and row['boxes']==donor['boxes']
  transfers.append(dict(name=row['name'],frame=row['frame'],donorFrame=donor['frame'],chrome=bind(prefix+'.chrome.png'),native=bind(prefix+'.native.png')))
 else:seen[key]=row;pairs.append(row)
(root/'path-receipt.json').write_text(json.dumps(dict(scope='Current live masks agree with independent Chrome snapped box edges, including thin correction; ordinary artboard clipping and full vector paint',receipt=bind(root/'receipt.json'),observer=bind(Path(__file__)),pathParser=bind(M/'validation/wrapped-clip-stream-check.py'),frames=path_rows),indent=2)+'\n')
sheets=[];groups=collections.defaultdict(list)
for row in pairs:groups[(row['width'],row['height'])].append(row)
for(w,h),group in groups.items():
 for start in range(0,len(group),4):
  selected=group[start:start+4];canvas=Image.new('RGB',(3*(w+8)+8,len(selected)*(h+48)+24),'#ddd');d=ImageDraw.Draw(canvas);d.text((8,5),'Chrome / native / diff — full size',fill='black');members=[]
  for i,row in enumerate(selected):
   top=24+i*(h+48);d.text((8,top),row['name'],fill='black');d.text((8,top+14),f"frame {row['frame']} | {row['pixelFailures']}",fill='black');files=[]
   for j,kind in enumerate(['chrome','native','diff']):
    p=Path(row['prefix']+'.'+kind+'.png');im=Image.open(p).convert('RGB');assert im.size==(w,h);canvas.paste(im,(8+j*(w+8),top+30));files.append(bind(p))
   members.append(dict(name=row['name'],frame=row['frame'],files=files,pixelFailures=row['pixelFailures']))
  p=out/f'sheet-{len(sheets):02}.png';canvas.save(p);sheets.append(dict(**bind(p),members=members))
coverage=dict(scope='Private live rounded paint; visual inspection pending; no public admission',receipt=bind(root/'receipt.json'),paths=bind(root/'path-receipt.json'),sheets=sheets,transfers=transfers,counts=dict(frames=len(r['rows']),representatives=len(pairs),transfers=len(transfers),sheets=len(sheets),geometryPass=sum(not x['geometryFailures']for x in r['rows']),pixelPass=sum(not x['pixelFailures']for x in r['rows'])))
(out/'coverage.json').write_text(json.dumps(coverage,indent=2)+'\n');print(json.dumps(coverage['counts']))
