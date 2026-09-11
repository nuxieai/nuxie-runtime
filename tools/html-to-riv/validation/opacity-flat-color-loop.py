"""Native repro: extra opacity passes must preserve solid interior color to one LSB.
Usage: SCRIPT RENDERER NEW_OUTPUT
"""
from pathlib import Path
import hashlib,json,subprocess,sys
from PIL import Image
binary,out=map(lambda x:Path(x).resolve(),sys.argv[1:]);out.mkdir(parents=True,exist_ok=False)
rows=[]
for depth in [0,1,2,3]:
 for alpha in [1,0.5]:
  name=f'depth{depth}-alpha{alpha}'
  draw='drawPath path={id=1,fillRule=0,path={verbs=[move,line,line,line,close],points=[(32,32),(132,32),(132,112),(32,112)]}} paint={id=1,style=fill,color=0xffe66428,thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=0}'
  commands=['rive-golden-stream-v1','frameSize width=390 height=320','clearColor value=0xffffffff']
  commands += [f'beginOpacity opacity={alpha if i==0 else 1}' for i in range(depth)]
  commands += [draw]+['endOpacity']*depth+['frame']
  stream=out/(name+'.stream');stream.write_text('\n'.join(commands)+'\n')
  png=out/(name+'.png');p=subprocess.run([str(binary),'--stream',str(stream),'--output',str(png),'--backend','rust-metal','--mode','clockwise-atomic'],capture_output=True,text=True)
  (out/(name+'.log')).write_text(p.stdout+p.stderr);assert p.returncode==0,p.stderr
  im=Image.open(png).convert('RGBA');pixels=[im.getpixel((x,y)) for x in range(37,77) for y in range(37,67)]
  low=[min(p[c] for p in pixels) for c in range(4)];high=[max(p[c] for p in pixels) for c in range(4)]
  spread=max(b-a for a,b in zip(low,high));rows.append(dict(name=name,minimum=low,maximum=high,spread=spread,failed=spread>1,sha256=hashlib.sha256(png.read_bytes()).hexdigest()))
result=dict(status='failed' if any(r['failed'] for r in rows) else 'passed',rows=rows,binarySha256=hashlib.sha256(binary.read_bytes()).hexdigest())
(out/'receipt.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result));sys.exit(result['status']=='failed')
