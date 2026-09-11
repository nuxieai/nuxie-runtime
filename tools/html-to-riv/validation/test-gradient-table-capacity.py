"""Native exact stop-table capacity and adjacent-row controls.
Usage: SCRIPT RENDERER NEW_OUTPUT
Synthetic analytic evidence, not public CSS qualification or a benchmark.
"""
import bisect
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import sys
from PIL import Image

binary,out=(Path(s).resolve() for s in sys.argv[1:])
out.mkdir(parents=True,exist_ok=False)
f32=lambda x:struct.unpack('f',struct.pack('f',x))[0]
fixtures=[]
# Full supported table, compiler maximum plus padding, small CSS tables, and
# ordinary ramps coexist. Different colors prevent content dedup masking rows.
for count in [2,258,510,3,510,258]:
    index=len(fixtures)
    stops=[f32(i/(count-1)) for i in range(count)]
    colors=[(255,round(255*(1-p)),index*31,round(255*p)) for p in stops]
    fixtures.append(dict(count=count,stops=stops,colors=colors,css=True))
fixtures.append(dict(count=3,stops=[0,.5,1],colors=[(255,255,255,0),(255,0,255,0),(255,0,255,255)],css=False))
lines=['rive-golden-stream-v1','frameSize width=1024 height=140','clearColor value=0xffffffff']
for index,fixture in enumerate(fixtures):
    command='makePremultipliedLinearGradient' if fixture['css'] else 'makeLinearGradient'
    colors=[sum(c << shift for c,shift in zip(color,[24,16,8,0])) for color in fixture['colors']]
    stops=','.join(f'{{color=0x{c:08x},stop={p:.9g}}}' for c,p in zip(colors,fixture['stops']))
    shader=index+1;top=index*20
    lines += [f'{command} id={shader} start=(0,0) end=(1024,0) stops=[{stops}]',
        f'drawPath path={{id={shader},fillRule=0,path={{verbs=[move,line,line,line,close],points=[(0,{top}),(1024,{top}),(1024,{top+20}),(0,{top+20})]}}}} '
        f'paint={{id={shader},style=fill,color=0xffffffff,thickness=1,join=0,cap=0,feather=0,blendMode=3,shader={shader}}}']
stream=out/'capacity.stream';stream.write_text('\n'.join(lines+['frame'])+'\n')
rows=[]
for backend in ['rust-metal','rust-metal-atomic']:
    png=out/(backend+'.png')
    result=subprocess.run([str(binary),'--stream',str(stream),'--output',str(png),'--backend',backend,'--mode','clockwise-atomic'],capture_output=True,text=True)
    (out/(backend+'.log')).write_text(result.stdout+result.stderr)
    assert result.returncode==0,result.stderr
    im=Image.open(png).convert('RGBA')
    for index,fixture in enumerate(fixtures):
        worst=0
        for x in range(1024):
            t=(x+.5)/1024
            before=max(0,bisect.bisect_right(fixture['stops'],t)-1);after=min(before+1,fixture['count']-1)
            lo,hi=fixture['stops'][before],fixture['stops'][after]
            u=(t-lo)/(hi-lo) if hi>lo else 0
            a,b=fixture['colors'][before],fixture['colors'][after]
            expected=tuple(round(a[c]*(1-u)+b[c]*u) for c in [1,2,3])+(255,)
            actual=im.getpixel((x,index*20+10));error=max(abs(a-b) for a,b in zip(actual,expected));worst=max(worst,error)
            assert error<=2,(backend,index,fixture['count'],x,actual,expected)
        rows.append(dict(backend=backend,index=index,stops=fixture['count'],css=fixture['css'],samples=1024,maxChannelError=worst))
# One more than the native table capacity must fail before GPU rendering.
invalid=out/'over-capacity.stream'
encoded=','.join(f'{{color=0xffff0000,stop={i/510:.9g}}}' for i in range(511))
invalid.write_text('rive-golden-stream-v1\nframeSize width=20 height=20\nmakePremultipliedLinearGradient id=1 start=(0,0) end=(20,0) stops=['+encoded+']\nframe\n')
result=subprocess.run([str(binary),'--stream',str(invalid),'--output',str(out/'invalid.png'),'--backend','rust-metal','--mode','clockwise-atomic'],capture_output=True,text=True)
(out/'invalid.log').write_text(result.stdout+result.stderr)
assert result.returncode!=0 and 'premultiplied linear gradients' in result.stderr,result.stderr
receipt=dict(status='passed',rendererSha256=hashlib.sha256(binary.read_bytes()).hexdigest(),streamSha256=hashlib.sha256(stream.read_bytes()).hexdigest(),rows=rows,overCapacityRejected=True)
(out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(dict(status='passed',samples=sum(r['samples'] for r in rows),overCapacityRejected=True)))
