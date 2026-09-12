"""Create deterministic authored image test assets, never rasterize HTML/CSS.
Usage: SCRIPT FRESH_OUTPUT_DIRECTORY. Keep the encoded files as source fixtures.
"""
from pathlib import Path
from PIL import Image,features,__version__
import hashlib,json,sys
root=Path(sys.argv[1]).resolve();root.mkdir(parents=True,exist_ok=False)
colors=[(236,51,62),(31,121,220),(22,173,111),(245,187,37),(111,66,193),(23,167,180),(238,115,33),(35,42,59)]
opaque=Image.new('RGB',(96,64));alpha=Image.new('RGBA',(96,64))
for y in range(64):
 for x in range(96):
  c=colors[(x//24+2*(y//16))%len(colors)]
  # Small asymmetric orientation markers and gradients exercise filtering.
  if x<8 and y<8:c=(255,255,255)
  if x>=88 and y>=56:c=(0,0,0)
  opaque.putpixel((x,y),c)
  alpha.putpixel((x,y),(*c,[0,85,170,255][(x//12)%4]))
outputs=[('opaque.png',opaque,dict(format='PNG',compress_level=9)),('alpha.png',alpha,dict(format='PNG',compress_level=9)),('baseline.jpg',opaque,dict(format='JPEG',quality=90,subsampling=0,progressive=False,optimize=False)),('progressive.jpg',opaque,dict(format='JPEG',quality=90,subsampling=0,progressive=True,optimize=False)),('lossless.webp',opaque,dict(format='WEBP',lossless=True,method=6)),('lossy.webp',opaque,dict(format='WEBP',quality=80,method=6)),('alpha.webp',alpha,dict(format='WEBP',lossless=True,method=6))]
rows=[]
for name,im,options in outputs:
 p=root/name;im.save(p,**options)
 rows.append(dict(name=name,width=96,height=64,sourceMode=im.mode,encodeOptions=options,bytes=p.stat().st_size,sha256=hashlib.sha256(p.read_bytes()).hexdigest()))
(root/'manifest.json').write_text(json.dumps(dict(scope='Programmatic authored image fixtures; no HTML/CSS screenshot or target-driven pixel adjustment.',pillow=__version__,jpegVersion=features.version('jpg'),webpVersion=features.version('webp'),generatorSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),files=rows),indent=2)+'\n')
print(json.dumps(rows,indent=2))
