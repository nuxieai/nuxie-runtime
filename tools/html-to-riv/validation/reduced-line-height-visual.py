#!/usr/bin/env python3
"""Unscaled full-image review sheets and exact same-source frame transfers."""
import hashlib,json
from pathlib import Path
from PIL import Image,ImageDraw
BASE=Path(__file__).resolve().parents[1]
ROOT=BASE/'output/reduced-line-height-r1'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
r=json.loads((ROOT/'native-receipt.json').read_text()); out=ROOT/'visual';out.mkdir(exist_ok=False)
pairs=[];transfers=[]
for c in r['counts']:
 rows=[x for x in r['rows'] if x['name']==c['name']]
 for row in rows:
  expected=row['frame'] if row['frame']<3 else {3:0,4:0,5:1,6:2,7:0}[row['frame']]
  if row['frame']<3:
   pairs.append(row);continue
  source=rows[expected]
  assert row['rivSha256']==source['rivSha256'] and row['nativeText']==source['nativeText'] and row['nativeOwner']==source['nativeOwner'] and row['browserMetrics']==source['browserMetrics']
  proof=[]
  for suffix in ['chrome.png','native.png','literal-control.png']:
   a=Path(source['prefix']+'.'+suffix);b=Path(row['prefix']+'.'+suffix);ai=Image.open(a).convert('RGBA');bi=Image.open(b).convert('RGBA');assert ai.size==bi.size and ai.tobytes()==bi.tobytes()
   proof.append({'source':str(a),'sourceSha256':sha(a),'target':str(b),'targetSha256':sha(b),'completeRgbaEqual':True})
  transfers.append({'case':row['name'],'frame':row['frame'],'sourceFrame':expected,'sameSourceRiv':True,'textOwnerAndBrowserMetricsEqual':True,'images':proof})
sheets=[]
for frame in range(3):
 selected=[r for r in pairs if r['frame']==frame]
 for chunk in range(2):
  rows=selected[chunk*4:(chunk+1)*4]; w=rows[0]['width'];h=rows[0]['height'];sheet=Image.new('RGB',(w*2+24,len(rows)*(h+38)+26),'#ddd');d=ImageDraw.Draw(sheet);d.text((8,6),'Chrome153 (left) / immutable native (right), complete images at original resolution',fill='black')
  members=[]
  for i,row in enumerate(rows):
   y=26+i*(h+38); d.text((8,y),f"{row['name']} frame {frame} | metrics {'PASS' if not row['metricFailures'] else 'FAIL'} / pixels {'PASS' if not row['pixelFailures'] and row['ink']['passed'] else 'FAIL'}",fill='black')
   for j,suffix in enumerate(['chrome.png','native.png']):
    p=Path(row['prefix']+'.'+suffix);im=Image.open(p).convert('RGB');assert im.size==(w,h);sheet.paste(im,(8+j*(w+8),y+20))
   members.append({'case':row['name'],'frame':frame,'chrome':row['prefix']+'.chrome.png','native':row['prefix']+'.native.png','chromeSha256':row['chromeSha256'],'nativeSha256':row['nativeSha256']})
  f=out/f'frame-{frame}-group-{chunk}.png';sheet.save(f);sheets.append({'path':str(f),'sha256':sha(f),'members':members})
coverage={'directPairs':len(pairs),'exactTransferredPairs':len(transfers),'totalPairs':len(r['rows']),'sheets':sheets,'transfers':transfers,'reviewCompleted':False}
(out/'coverage.json').write_text(json.dumps(coverage,indent=2)+'\n')
print(json.dumps({k:coverage[k] for k in ['directPairs','exactTransferredPairs','totalPairs']}))
