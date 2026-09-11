"""Analytic tiled-gradient host transform, modulation and shared-stop controls.
Usage: SCRIPT FROZEN_RENDERER NEW_OUTPUT
This synthetic native check does not qualify browser or public compiler fidelity.
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
from PIL import Image

binary, out = (Path(s).resolve() for s in sys.argv[1:])
out.mkdir(parents=True, exist_ok=False)
rows = []
failures = []
tiles = [[0.,0.,40.,30.], [-11.,3.,27.,19.]]
for matrix in [[1.,0.,0.,1.,20.,20.], [1.2,.2,-.15,.9,20.,20.]]:
    a,b,c,d,e,f = matrix
    det = a*d-b*c
    for opacity in [1., .5]:
        for reverse in [False, True]:
            tag = f'{int(a!=1)}-{opacity}-{int(reverse)}'
            lines = ['rive-golden-stream-v1', 'frameSize width=160 height=140',
                     'clearColor value=0xffffffff', 'save',
                     'transform matrix='+json.dumps(matrix,separators=(',',':')),
                     f'modulateOpacity opacity={opacity}']
            for index in ([1,0] if reverse else [0,1]):
                top = index*40
                tile = ','.join(str(v) for v in tiles[index])
                shader = index+1
                lines += [f'makeTiledPremultipliedLinearGradient id={shader} start=(0,0) end=(40,30) tile=({tile}) stops=[{{color=0xffff0000,stop=0}},{{color=0xff0000ff,stop=1}}]',
                    f'drawPath path={{id={shader},fillRule=0,path={{verbs=[move,line,line,line,close],points=[(0,{top}),(90,{top}),(90,{top+30}),(0,{top+30})]}}}} '
                    f'paint={{id={shader},style=fill,color=0xffffffff,thickness=1,join=0,cap=0,feather=0,blendMode=3,shader={shader}}}']
            stream = out/(tag+'.stream')
            stream.write_text('\n'.join(lines+['restore','frame'])+'\n')
            for backend in ['rust-metal','rust-metal-atomic']:
                png = out/f'{tag}-{backend}.png'
                result = subprocess.run([str(binary),'--stream',str(stream),'--output',str(png),
                    '--backend',backend,'--mode','clockwise-atomic'],capture_output=True,text=True)
                png.with_suffix('.log').write_text(result.stdout+result.stderr)
                assert result.returncode==0,result.stderr
                image = Image.open(png).convert('RGBA')
                samples = []
                for y in range(10,125,7):
                    for x in range(10,150,7):
                        dx,dy=x+.5-e,y+.5-f
                        lx,ly=(d*dx-c*dy)/det,(-b*dx+a*dy)/det
                        index=0 if 2<ly<28 else 1 if 42<ly<68 else None
                        if index is None or not 2<lx<88: continue
                        tx,ty,w,h=tiles[index]
                        ux,uy=(lx-tx)%w,(ly-ty)%h
                        # Exclude sample footprints crossing repeat discontinuities.
                        if min(ux,w-ux,uy,h-uy)<1.5: continue
                        t=max(0.,min(1.,((tx+ux)*40+(ty+uy)*30)/2500))
                        alpha=round(255*opacity)/255
                        expected=tuple(round(255*(v*alpha+1-alpha)) for v in [1-t,0,t])+(255,)
                        actual=image.getpixel((x,y))
                        error=max(abs(a-b) for a,b in zip(actual,expected))
                        sample=dict(point=[x,y],tile=index,local=[lx,ly],actual=actual,expected=expected,error=error)
                        samples.append(sample)
                        if error>2: failures.append(dict(case=tag,backend=backend,**sample))
                assert len(samples)>30
                rows.append(dict(case=tag,backend=backend,matrix=matrix,opacity=opacity,reverse=reverse,
                    streamSha256=hashlib.sha256(stream.read_bytes()).hexdigest(),
                    pngSha256=hashlib.sha256(png.read_bytes()).hexdigest(),samples=samples))
receipt=dict(status='failed' if failures else 'passed',scope=__doc__,
    rendererSha256=hashlib.sha256(binary.read_bytes()).hexdigest(),images=rows,failures=failures)
(out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(dict(status=receipt['status'],images=len(rows),samples=sum(len(r['samples']) for r in rows),failures=len(failures))))
if failures: raise SystemExit(1)
