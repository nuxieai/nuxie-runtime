"""Compare runtime gradient geometry with solid-color Chrome ramp samples.
This tests endpoints, not the native gradient renderer or public CSS admission.
Usage: SCRIPT CHROME_ORACLE_DIRECTORY NEW_RECEIPT
"""
from pathlib import Path
import hashlib
import json
import subprocess
import sys
import tempfile
from PIL import Image

oracle, receipt = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
assert not receipt.exists()
source = Path(__file__).resolve().parents[3] / 'crates/nuxie-runtime/src/mechanical_port/source/css_linear_gradient.rs'
cases = json.loads((oracle/'oracle.json').read_text())
assert cases['browser']=='153.0.8010.12'
angles = {'default':180, 'right':90, 'top':0, 'left':270, 'angle45':45,
          'corner':None, 'angle-unit':90, 'negative-angle':-90}
with tempfile.TemporaryDirectory() as temp:
    temp=Path(temp)
    main=temp/'main.rs'
    main.write_text('#[path='+json.dumps(str(source))+'] mod gradient;\nuse gradient::CssGradientDirection as D;\nfn main(){\n'
        + 'for width in [200.,350.,728.] {for direction in [D::Degrees(180.),D::Degrees(90.),D::Degrees(0.),D::Degrees(270.),D::Degrees(45.),D::Corner{right:true,bottom:true},D::Degrees(90.),D::Degrees(-90.)]{let l=direction.resolve(width,140.).unwrap();println!("{} {} {} {} {}",width,l.start[0],l.start[1],l.end[0],l.end[1]);}}}\n')
    subprocess.run(['rustc','--edition=2024','-Awarnings',str(main),'-o',str(temp/'geometry')],check=True)
    lines=iter(subprocess.check_output([str(temp/'geometry')],text=True).splitlines())
    rows=[]
    for viewport in [240,390,768]:
        for name in angles:
            width,sx,sy,ex,ey=map(float,next(lines).split())
            assert width==viewport-40
            png=oracle/f'linear-gradient-{name}-{viewport}.png';im=Image.open(png).convert('RGB')
            dx,dy=ex-sx,ey-sy
            for fx,fy in [(0.1,0.1),(.25,.75),(.5,.5),(.75,.25),(.9,.9)]:
                x,y=int(width*fx),int(140*fy)
                t=max(0,min(1,((x+.5-sx)*dx+(y+.5-sy)*dy)/(dx*dx+dy*dy)))
                expected=[255*(1-t),0,255*t];actual=im.getpixel((x+20,y+20))
                error=max(abs(a-b) for a,b in zip(actual,expected))
                assert error<=2,(name,viewport,x,y,actual,expected,error)
                rows.append(dict(name=name,width=viewport,point=[x+20,y+20],actual=actual,expected=expected,maxChannelError=error))
result=dict(status='chrome-gradient-geometry-samples-pass',samples=len(rows),runtimeSourceSha256=hashlib.sha256(source.read_bytes()).hexdigest(),oracleSha256=hashlib.sha256((oracle/'oracle.json').read_bytes()).hexdigest(),rows=rows,scope='Runtime endpoint geometry against Chrome red/blue ramp samples with two channel units for quantization; not native pixels or public compiler qualification.')
receipt.write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(dict(status=result['status'],samples=len(rows))))
