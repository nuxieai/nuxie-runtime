"""Exercise real native CLI group replay; synthetic pixels, not CSS qualification."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
from PIL import Image

binary, output = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
output.mkdir(parents=True, exist_ok=False)
def shape(color, bounds):
    l,t,r,b = bounds
    return (f'drawPath path={{id=1,fillRule=0,path={{verbs=[move,line,line,line,close],points=[({l},{t}),({r},{t}),({r},{b}),({l},{b})]}}}} '
            f'paint={{id=1,style=fill,color={color},thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=0}}')
source = '\n'.join(['rive-golden-stream-v1','frameSize width=16 height=16',
    'clearColor value=0xff000000','beginOpacity opacity=0.5',
    shape('0xffff0000',(0,0,12,12)), 'beginOpacity opacity=0.5',
    shape('0xff00ff00',(4,4,12,12)), 'endOpacity','endOpacity', 'frame'])+'\n'
stream=output/'nested.stream';stream.write_text(source)
rows=[]
for backend in ['rust-metal','rust-metal-atomic']:
    previous=None
    for repeat in range(2):
        png=output/f'{backend}-{repeat}.png'
        args=[str(binary),'--stream',str(stream),'--output',str(png),'--backend',backend,
              '--mode','clockwise-atomic','--clear','0xffffffff']
        result=subprocess.run(args,capture_output=True,text=True)
        (output/f'{backend}-{repeat}.log').write_text(result.stdout+result.stderr)
        assert result.returncode==0, result.stderr
        image=Image.open(png).convert('RGBA')
        for point, expected in [((2,2),(255,127,127,255)),((8,8),(191,191,127,255)),((14,14),(255,255,255,255))]:
            actual=image.getpixel(point)
            assert all(abs(a-b)<=1 for a,b in zip(actual,expected)), (backend,point,actual,expected)
        digest=hashlib.sha256(png.read_bytes()).hexdigest()
        if previous is not None: assert previous==digest
        previous=digest
        rows.append(dict(backend=backend,repeat=repeat,png=str(png),sha256=digest))
    translucent=output/f'{backend}-translucent-clear.png'
    result=subprocess.run([str(binary),'--stream',str(stream),'--output',str(translucent),
        '--backend',backend,'--mode','clockwise-atomic','--clear','0x800000ff'],capture_output=True,text=True)
    (output/f'{backend}-translucent-clear.log').write_text(result.stdout+result.stderr)
    assert result.returncode==0, result.stderr
    rgba=Image.open(translucent).convert('RGBA')
    assert abs(rgba.getpixel((14,14))[3]-128)<=1, 'clear alpha applied twice'
    assert abs(rgba.getpixel((2,2))[3]-192)<=1, 'group alpha over translucent clear'
    rejected=output/f'{backend}-truncated.png'
    result=subprocess.run([str(binary),'--stream',str(stream),'--output',str(rejected),
        '--backend',backend,'--mode','clockwise-atomic','--command-limit','2'],capture_output=True,text=True)
    (output/f'{backend}-truncated.log').write_text(result.stdout+result.stderr)
    assert result.returncode!=0 and not rejected.exists()
    assert 'unterminated group' in result.stderr
receipt=dict(status='cli-pixels-pass',images=rows,malformedGroupsRejected=2,
             binarySha256=hashlib.sha256(binary.read_bytes()).hexdigest(),
             scope='Synthetic nested alpha and clear override; two native execution modes, repeated PNG identity; no CSS/browser qualification.')
(output/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(dict(status=receipt['status'],images=len(rows),rejected=2)))
