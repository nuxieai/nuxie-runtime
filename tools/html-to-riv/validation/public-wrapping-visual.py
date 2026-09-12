#!/usr/bin/env python3
"""Create full-size public compiler visual review sheets with exact repeat transfers.

No renderer, pixel thresholds, geometry or qualification status is modified.
"""
from pathlib import Path
import collections,hashlib,importlib.util,json,sys
from PIL import Image,ImageDraw
M=Path(__file__).resolve().parents[1]
root=Path(sys.argv[1]).resolve();out=root/'public-visual';assert not out.exists();out.mkdir()
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
bind=lambda p:dict(path=str(Path(p).resolve()),sha256=sha(p))
r=json.loads((root/'receipt.json').read_text())
observations=[];controls=None;representatives=[];transfers=[];seen={}
for row in r['rows']:
 prefix=Path(row['prefix']);frame_manifest=prefix.parent/'probe/frames.json';stream=prefix.parent/'probe'/row['stream']
 assert sha(stream)==row['streamSha256']
 for kind in ['chrome','native']:
  assert sha(str(prefix)+f'.{kind}.png')==row[kind+'Sha256']
 key=(row['name'],row['width'],row['height'])
 if key in seen:
  donor=seen[key]
  assert all(row[k+'Sha256']==donor[k+'Sha256'] for k in ['chrome','native','riv'])
  assert row['boxes']==donor['boxes'] and row['geometry']==donor['geometry']
  assert row['pixelFailures']==donor['pixelFailures'] and row['geometryFailures']==donor['geometryFailures']
  transfers.append(dict(name=row['name'],frame=row['frame'],donorFrame=donor['frame'],chrome=bind(str(prefix)+'.chrome.png'),native=bind(str(prefix)+'.native.png')))
 else:seen[key]=row;representatives.append(row)
sheets=[]
# Group same-sized pairs; four complete images per row (Chrome/native/diff).
groups=collections.defaultdict(list)
for row in representatives:groups[(row['width'],row['height'])].append(row)
for (w,h),group in groups.items():
 for start in range(0,len(group),4):
  selected=group[start:start+4];canvas=Image.new('RGB',(3*(w+8)+8,len(selected)*(h+50)+26),'#ddd');draw=ImageDraw.Draw(canvas);draw.text((8,5),'Chrome / immutable native / pixel diff — full-size, no scaling',fill='black');members=[]
  for i,row in enumerate(selected):
   top=26+i*(h+50);draw.text((8,top),row['name'],fill='black');draw.text((8,top+14),f"frame {row['frame']} {w}x{h} | geometry {row['geometryFailures']} | pixels {row['pixelFailures']}",fill='black')
   files=[]
   for j,kind in enumerate(['chrome','native','diff']):
    p=Path(row['prefix']+'.'+kind+'.png');im=Image.open(p).convert('RGB');assert im.size==(w,h);canvas.paste(im,(8+j*(w+8),top+32));files.append(bind(p))
   members.append(dict(name=row['name'],frame=row['frame'],files=files,pixelFailures=row['pixelFailures']))
  p=out/f'sheet-{len(sheets):02}.png';canvas.save(p);sheets.append(dict(**bind(p),members=members))
coverage=dict(scope='Public compiler campaign; direct review pending. No native stream observer claim. Exact repeat transfers preserve per-frame gates.',receipt=bind(root/'receipt.json'),script=bind(Path(__file__)),sheets=sheets,transfers=transfers,counts=dict(frames=len(r['rows']),representatives=len(representatives),transfers=len(transfers),sheets=len(sheets),geometryPass=sum(not x['geometryFailures'] for x in r['rows']),pixelPass=sum(not x['pixelFailures'] for x in r['rows'])))
(out/'coverage.json').write_text(json.dumps(coverage,indent=2)+'\n');print(json.dumps(coverage['counts']))
