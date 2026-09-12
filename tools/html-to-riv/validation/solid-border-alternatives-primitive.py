"""Bind the diagnostic samebounds primitive comparison and full pixel identity."""
from pathlib import Path
from PIL import Image,ImageDraw
import json,hashlib,subprocess
b=Path(__file__).resolve().parent.parent;o=b/'output/solid-border-alternatives-r1';run=o/'r3/render'
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
r=json.loads((run/'receipt.json').read_text());direct=[];coverage=[];reproductions=[]
sheet=Image.new('RGBA',(248,224),(245,245,245,255));draw=ImageDraw.Draw(sheet)
for index,name in enumerate(['layout-fill-height-25','layout-fill-height-25-5']):
 row=next(v for v in r['rows']if v['name']==name and v['frame']==0);y=index*112;draw.text((4,y+4),name+' | Chrome / native',fill='black')
 for i,kind in enumerate(['chrome','native']):sheet.paste(Image.open(row['prefix']+'.'+kind+'.png').convert('RGBA'),(4+i*124,y+24))
 direct.append(dict(name=name,prefix=row['prefix'],cropY=y+24,chromeSha256=row['chromeSha256'],nativeSha256=row['nativeSha256']))
path=o/'primitive.png';sheet.save(path)
for row in r['rows']:
 d=next(v for v in direct if v['chromeSha256']==row['chromeSha256']and v['nativeSha256']==row['nativeSha256'])
 for i,kind in enumerate(['chrome','native']):
  image=Image.open(row['prefix']+'.'+kind+'.png').convert('RGBA');assert sha(row['prefix']+'.'+kind+'.png')==row[kind+'Sha256'];assert image.tobytes()==Image.open(d['prefix']+'.'+kind+'.png').convert('RGBA').tobytes();assert image.tobytes()==sheet.crop((4+i*124,d['cropY'],104+i*124,d['cropY']+80)).tobytes()
 coverage.append(dict(name=row['name'],frame=row['frame'],requestSha256=row['requestSha256'],rivSha256=row['rivSha256'],sourceMapSha256=row['sourceMapSha256'],geometrySha256=row['geometrySha256'],chromeSha256=row['chromeSha256'],nativeSha256=row['nativeSha256'],directReview=d['name'],method='direct full-size sheet'if row['name']==d['name']and row['frame']==0 else'exact full RGBA transfer'))
folder=o/'primitive-reproduction';folder.mkdir(exist_ok=False)
for a in r['artifacts']:
 dest=folder/(a['name']+'.riv');p=subprocess.run([str(o/'r3/candidate'),str(run/a['name']/'request.json'),str(dest)],capture_output=True,text=True);(folder/(a['name']+'.log')).write_text(p.stdout+p.stderr);p.check_returncode();assert sha(dest)==a['rivSha256'];assert sha(dest.with_suffix('.map.json'))==a['mapSha256'];reproductions.append(dict(name=a['name'],rivSha256=sha(dest),mapSha256=sha(dest.with_suffix('.map.json'))))
controls=[]
for a in r['artifacts']:
 rows=[v for v in r['rows']if v['name']==a['name']];first=rows[0];controls.append(dict(name=a['name'],geometryPass=sum(not v['geometryFailures']for v in rows),pixelPass=sum(not v['pixelFailures']for v in rows),clearPass=sum(c['samePixels']for v in rows for c in v['clearChecks']),row25NativeRgba=Image.open(first['prefix']+'.native.png').getpixel((20,25)),row25ChromeRgba=Image.open(first['prefix']+'.chrome.png').getpixel((20,25)),chromeSha256=first['chromeSha256'],nativeSha256=first['nativeSha256']))
result=dict(scope='diagnostic paint primitive identity; unchanged public compiler admission',sourceSha256=sha(__file__),runReceipt=dict(path=str((run/'receipt.json').relative_to(b)),sha256=sha(run/'receipt.json')),buildReceipt=dict(path=str((o/'r3/build-receipt.json').relative_to(b)),sha256=sha(o/'r3/build-receipt.json')),sheet=dict(path=str(path.relative_to(b)),sha256=sha(path)),controls=controls,directPairs=2,frames=coverage,reproductions=reproductions)
(o/'primitive-receipt.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(dict(directPairs=2,fullPixelCoverage=len(coverage),exactReproductions=len(reproductions))))
