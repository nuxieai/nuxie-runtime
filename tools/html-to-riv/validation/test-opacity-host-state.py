"""Native stream controls for inherited affine transforms and host modulation.
Synthetic analytic samples, not public CSS transform qualification.
Usage: SCRIPT RENDERER NEW_OUTPUT
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
from PIL import Image

binary, out = map(lambda s: Path(s).resolve(), sys.argv[1:])
out.mkdir(parents=True, exist_ok=False)
def shape(bounds):
    l,t,r,b=bounds
    return (f'drawPath path={{id=1,fillRule=0,path={{verbs=[move,line,line,line,close],points=[({l},{t}),({r},{t}),({r},{b}),({l},{b})]}}}} '
            'paint={id=1,style=fill,color=0xffff0000,thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=0}')
matrices = {'translation':[1,0,0,1,4,4], 'scale':[2,0,0,2,4,4],
            'rotation':[0,1,-1,0,24,4], 'shear':[1,0,.5,1,4,4]}
rows=[]
for name,m in matrices.items():
    for modulation in [1,.5]:
        stream=out/f'{name}-{modulation}.stream'
        # Two intersecting red shapes must receive group alpha only once.
        stream.write_text('\n'.join(['rive-golden-stream-v1','frameSize width=40 height=40',
            'clearColor value=0xffffffff','save',f'transform matrix={json.dumps(m, separators=(',', ':'))}',
            f'modulateOpacity opacity={modulation}','beginOpacity opacity=0.5',
            shape((0,0,10,10)),shape((4,4,12,12)),'endOpacity','restore','frame'])+'\n')
        for backend in ['rust-metal','rust-metal-atomic']:
            png=out/f'{name}-{modulation}-{backend}.png'
            result=subprocess.run([str(binary),'--stream',str(stream),'--output',str(png),
                '--backend',backend,'--mode','clockwise-atomic'],capture_output=True,text=True)
            (png.with_suffix('.log')).write_text(result.stdout+result.stderr)
            assert result.returncode==0,result.stderr
            im=Image.open(png).convert('RGBA');expected=(255,round(255*(1-.5*modulation)),round(255*(1-.5*modulation)),255)
            for x,y in [(2,2),(7,7)]:
                point=(int(m[0]*x+m[2]*y+m[4]),int(m[1]*x+m[3]*y+m[5]))
                actual=im.getpixel(point)
                assert all(abs(a-b)<=1 for a,b in zip(actual,expected)),(name,modulation,backend,point,actual,expected)
            assert im.getpixel((38,38))==(255,255,255,255)
            rows.append(dict(transform=name,modulation=modulation,backend=backend,png=str(png),sha256=hashlib.sha256(png.read_bytes()).hexdigest()))
receipt=dict(status='analytic-host-state-controls-pass',images=rows,binarySha256=hashlib.sha256(binary.read_bytes()).hexdigest(),scope='Affine ancestor transforms and host modulation with overlapping shapes. Synthetic runtime evidence, not public CSS transform qualification.')
(out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(dict(status=receipt['status'],images=len(rows))))
