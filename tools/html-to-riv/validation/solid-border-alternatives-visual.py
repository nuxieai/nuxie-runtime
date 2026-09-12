"""Build unchanged-resolution sheets and exact visual-transfer bindings."""
from pathlib import Path
from PIL import Image,ImageDraw
import json,hashlib
b=Path(__file__).resolve().parent.parent;o=b/'output/solid-border-alternatives-r1';v=o/'visual';v.mkdir(exist_ok=False)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
known={};direct=[];coverage=[]
prior_receipt=b/'validation/solid-border-candidate-receipt.json'
prior_coverage=json.loads((b/'output/solid-border-candidate-r1/visual-coverage.json').read_text())
for run in ['r2/render','r3/render']:
 for row in json.loads((b/'output/solid-border-candidate-r1'/run/'receipt.json').read_text())['rows']:
  ref=next(q for q in prior_coverage if q['frame']==f"{run}/{row['name']}/{row['frame']}")
  known[(row['chromeSha256'],row['nativeSha256'])]=dict(prefix=row['prefix'],sheet=ref['sheet'],sheetSha256=ref['sheetSha256'],review='prior verified review',reviewReceiptSha256=sha(prior_receipt))
for run in ['render','r2/render']:
 receipt=json.loads((o/run/'receipt.json').read_text())
 for artifact in receipt['artifacts']:
  rows=[r for r in receipt['rows']if r['name']==artifact['name']];fresh=[]
  for row in rows:
   key=(row['chromeSha256'],row['nativeSha256'])
   if key not in known:
    binding=dict(prefix=row['prefix'],sheet=str((v/(row['name']+'.png')).relative_to(b)),sheetSha256=None,review='new direct full-size sheet')
    known[key]=binding;fresh.append(row);direct.append(binding)
  if fresh:
   canvas=Image.new('RGBA',(1568,sum(r['height']+32 for r in fresh)),(245,245,245,255));draw=ImageDraw.Draw(canvas);y=0
   for row in fresh:
    draw.text((8,y+4),f"{row['name']} {row['width']} x {row['height']} | Chrome left, ordinary native right",fill='black')
    for i,kind in enumerate(['chrome','native']):canvas.paste(Image.open(row['prefix']+'.'+kind+'.png').convert('RGBA'),(8+i*784,y+24))
    known[(row['chromeSha256'],row['nativeSha256'])]['cropY']=y+24;y+=row['height']+32
   path=v/(fresh[0]['name']+'.png');canvas.save(path)
   for row in fresh:known[(row['chromeSha256'],row['nativeSha256'])]['sheetSha256']=sha(path)
  for row in rows:
   ref=known[(row['chromeSha256'],row['nativeSha256'])]
   for kind in ['chrome','native']:
    original=Path(row['prefix']+'.'+kind+'.png');assert sha(original)==row[kind+'Sha256']
    assert Image.open(original).convert('RGBA').tobytes()==Image.open(ref['prefix']+'.'+kind+'.png').convert('RGBA').tobytes()
   if ref['review']=='new direct full-size sheet':
    sheet=Image.open(b/ref['sheet']).convert('RGBA');y=ref['cropY']
    for i,kind in enumerate(['chrome','native']):assert sheet.crop((8+i*784,y,8+i*784+row['width'],y+row['height'])).tobytes()==Image.open(row['prefix']+'.'+kind+'.png').convert('RGBA').tobytes()
   coverage.append(dict(run=run,name=row['name'],frame=row['frame'],requestSha256=row['requestSha256'],rivSha256=row['rivSha256'],sourceMapSha256=row['sourceMapSha256'],geometrySha256=row['geometrySha256'],streamSha256=row['streamSha256'],chromeSha256=row['chromeSha256'],nativeSha256=row['nativeSha256'],review=ref))
(o/'visual-coverage.json').write_text(json.dumps(dict(directPairs=direct,frames=coverage),indent=2)+'\n')
print(json.dumps(dict(newDirectPairs=len(direct),frames=len(coverage),sheets=len(list(v.glob('*.png'))))))
