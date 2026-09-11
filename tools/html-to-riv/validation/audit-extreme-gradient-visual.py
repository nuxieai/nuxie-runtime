"""Audit twelve directly reviewed pairs and exact repeated/clone image transfers.
Usage: SCRIPT NATIVE_DIRECTORY
Review notes are specific to the frozen extreme-native-r2 artifacts, not an automatic review.
"""
import hashlib,json,sys
from pathlib import Path
from PIL import Image
p=Path(sys.argv[1]).resolve();sha=lambda f:hashlib.sha256(f.read_bytes()).hexdigest()
assert sha(p/'receipt.json')=='34ce7143b7a2c358b4389dd21e7146379341ab7465ce262f5c313cbdb600c568', 'Not the directly inspected native receipt'
r=json.loads((p/'receipt.json').read_text());assert r['browser']=='153.0.8010.12'
assert r['backend']=='rust-metal-atomic' and r['mode']=='atomics'
assert sha(Path(r['renderer']))==r['rendererSha256']
rows=r['rows'];assert len(rows)==32
notes={'positive':'Both columns show solid red with matching box edges and white exterior.', 'negative':'Both columns show solid blue with matching box edges and white exterior.', 'symmetric-alpha':'Both columns show visually matching uniform lavender and white exterior; unchanged numeric pixel gate passes.', 'hard-interior':'Chrome is entirely blue. Native preserves a sharp midpoint red/blue split at all three widths. Intentional extreme precision divergence, not Chrome equivalence.'}
coverage=[]
for name,note in notes.items():
 group=[v for v in rows if v['name']==name];assert len(group)==8
 assert [v['frame'] for v in group]==list(range(8))
 sheet=p/(name+'.review.png');im=Image.open(sheet).convert('RGBA');assert im.size==(1536,960)
 for row in group:
  assert row['width']==[240,390,768,240][row['frame']%4]
  direct=next(v for v in group if v['frame']<3 and v['width']==row['width'])
  for kind in ['browser','native']:
   png=Path(row['prefix']+'.'+kind+'.png');assert sha(png)==row[kind+'Sha256']==direct[kind+'Sha256']
   if row['frame']<3:
    x=0 if kind=='browser' else 768;y=row['frame']*320
    assert im.crop((x,y,x+row['width'],y+320)).tobytes()==Image.open(png).convert('RGBA').tobytes()
  assert row['expectedDivergence']==(name=='hard-interior')
  if name!='hard-interior':assert not row['failures']
  coverage.append({'name':name,'frame':row['frame'],'review':'direct full original sheet' if row['frame']<3 else 'exact full PNG pair transfer','sourceFrame':direct['frame'],'browserSha256':row['browserSha256'],'nativeSha256':row['nativeSha256'],'sheetSha256':sha(sheet),'note':note})
receipt={'status':'reviewed-with-explicit-Chrome-divergence','scope':'12 direct visual pairs inspected at original detail;20 exact full PNG pair transfers;24 Chrome-equivalent pixel-gated frames and8 intentional hard-stop divergences. Geometry is independently captured in native receipt, unlike analytic test expectations.','nativeReceiptSha256':sha(p/'receipt.json'),'auditorSha256':sha(Path(__file__).resolve()),'coverage':coverage}
(p/'visual-coverage.json').write_text(json.dumps(receipt,indent=2)+'\n');print('12 direct +20 exact pair transfers verified;8 divergence frames retained')
