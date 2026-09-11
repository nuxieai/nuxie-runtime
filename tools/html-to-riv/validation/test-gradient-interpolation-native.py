"""Analytic native controls for coexisting Rive/CSS interpolation modes.
Usage: SCRIPT RENDERER NEW_OUTPUT
Not a substitute for browser reference or visual qualification.
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
for stops in [[(0xffff0000, 0), (0x000000ff, 1)],
              [(0x40ff0000, 0), (0x8000ff00, .5), (0xc00000ff, 1)]]:
    for reverse in [False, True]:
        for opacity in [1, .5]:
            tag = f'{len(stops)}-{int(reverse)}-{opacity}'
            lines = ['rive-golden-stream-v1', 'frameSize width=100 height=80', 'clearColor value=0xffffffff']
            encoded = ','.join(f'{{color=0x{c:08x},stop={p}}}' for c,p in stops)
            modes = [('makeLinearGradient', False), ('makePremultipliedLinearGradient', True)]
            if reverse:
                modes.reverse()
            for shader, (command, premul) in enumerate(modes, 1):
                top = (shader-1)*40
                lines += [f'{command} id={shader} start=(0,0) end=(100,0) stops=[{encoded}]',
                    'save', f'modulateOpacity opacity={opacity}',
                    f'drawPath path={{id={shader},fillRule=0,path={{verbs=[move,line,line,line,close],points=[(0,{top}),(100,{top}),(100,{top+40}),(0,{top+40})]}}}} '
                    f'paint={{id={shader},style=fill,color=0xffffffff,thickness=1,join=0,cap=0,feather=0,blendMode=3,shader={shader}}}', 'restore']
            stream = out / (tag+'.stream')
            stream.write_text('\n'.join(lines+['frame'])+'\n')
            for backend in ['rust-metal', 'rust-metal-atomic']:
                png = out / f'{tag}-{backend}.png'
                result = subprocess.run([str(binary), '--stream', str(stream), '--output', str(png),
                    '--backend', backend, '--mode', 'clockwise-atomic'], capture_output=True, text=True)
                png.with_suffix('.log').write_text(result.stdout+result.stderr)
                assert result.returncode == 0, result.stderr
                image = Image.open(png).convert('RGBA')
                samples = []
                for row, (_, premul) in enumerate(modes):
                    for x in [9, 24, 49, 74, 89]:
                        t = (x+.5)/100
                        a,b = next((a,b) for a,b in zip(stops,stops[1:]) if a[1] <= t <= b[1])
                        u = (t-a[1])/(b[1]-a[1])
                        decode = lambda c: [(c >> shift & 255)/255 for shift in [16,8,0,24]]
                        ca,cb = decode(a[0]),decode(b[0])
                        # Native modulation rounds each authored stop alpha to u8.
                        ca[3] = round(ca[3]*255*opacity)/255
                        cb[3] = round(cb[3]*255*opacity)/255
                        alpha = ca[3]*(1-u)+cb[3]*u
                        rgb = [(ca[i]*ca[3]*(1-u)+cb[i]*cb[3]*u) if premul
                               else (ca[i]*(1-u)+cb[i]*u)*alpha for i in range(3)]
                        expected = tuple(round(255*(v+1-alpha)) for v in rgb)+(255,)
                        point = (x,row*40+20)
                        actual = image.getpixel(point)
                        samples.append(dict(point=point,premultiplied=premul,actual=actual,expected=expected))
                        assert max(abs(a-b) for a,b in zip(actual,expected)) <= 2, (tag,backend,point,premul,actual,expected)
                rows.append(dict(stops=len(stops),reverse=reverse,opacity=opacity,backend=backend,
                    png=str(png),sha256=hashlib.sha256(png.read_bytes()).hexdigest(),samples=samples))
receipt = dict(status='passed',scope='Synthetic straight/premultiplied coexistence, cache order, host modulation and both native backends',
    rendererSha256=hashlib.sha256(binary.read_bytes()).hexdigest(),images=rows)
(out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(dict(status='passed',images=len(rows),samples=sum(len(r['samples']) for r in rows))))
