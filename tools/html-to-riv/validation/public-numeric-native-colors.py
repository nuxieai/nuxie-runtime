"""Compare exact ordinary color fields with actual interior pixels, beyond gates."""
import hashlib
import collections
import importlib.util
import json
import pathlib
import sys
from PIL import Image
root=pathlib.Path(__file__).resolve().parent.parent;out=pathlib.Path(sys.argv[1]).resolve()
read=lambda p:json.loads(pathlib.Path(p).read_text())
sha=lambda p:hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
decoder=root/'validation/content-box-rounding-wire.py'
spec=importlib.util.spec_from_file_location('numeric_wire',decoder);wire=importlib.util.module_from_spec(spec);spec.loader.exec_module(wire)
cases={c['name']:c for c in read(root/'validation/public-numeric-native-cases.json') if 'expectedOrdinaryArgb' in c}
receipt=read(out/'visible/render/receipt.json');rows=[]
def color(directory):
    data=wire.records((directory/'scene.riv').read_bytes());source=next(n for n in read(directory/'scene.map.json') if n['id']=='p')
    fills=[(i,r) for i,r in enumerate(data) if r['kind']==20 and r['properties'].get(5,{}).get('value')==source['object_id']];assert len(fills)==1
    index,_=fills[0];paints=[r for r in data if r['kind']==18 and r['properties'].get(5,{}).get('value')==index-1];assert len(paints)==1
    return paints[0]['properties'][37]['value']
for frame in receipt['rows']:
    if frame['name'] not in cases:continue
    case=cases[frame['name']];directory=pathlib.Path(frame['prefix']).parent
    argb=color(directory);assert argb==int(case['expectedOrdinaryArgb'],16)
    prior=color(out/'preflight'/frame['name'])
    pixels={};regions={}
    for role in ['chrome','native']:
        path=pathlib.Path(frame['prefix']+'.'+role+'.png');assert sha(path)==frame[role+'Sha256']
        image=Image.open(path).convert('RGBA');region=image.crop((10,10,150,40))
        colors=region.getcolors(region.width*region.height);assert colors is not None
        pixels[role]=[dict(count=count,rgba=list(rgba)) for count,rgba in sorted(colors,reverse=True)]
        regions[role]=list(region.getdata())
    differences=collections.Counter(tuple(a-b for a,b in zip(native,chrome)) for native,chrome in zip(regions['native'],regions['chrome']))
    rows.append(dict(name=frame['name'],frame=frame['frame'],width=frame['width'],height=frame['height'],ordinaryArgb=f'0x{argb:08x}',priorOrdinaryArgb=f'0x{prior:08x}',cssom=frame['computedStyles']['p']['backgroundColor'],interiorRegion=[10,10,150,40],rgbaHistograms=pixels,channelDeltaHistogram=[dict(count=count,delta=list(delta)) for delta,count in sorted(differences.items())],pixelFailures=frame['pixelFailures']))
result=dict(scope='Exact read-only ordinary colorValue and complete captured interior RGBA histograms/deltas, including nonuniform native interiors. A gate pass does not assert exact channel equality. Prior fields come from frozen-old-compiler preflight bytes, not rerendered old images.',renderReceiptSha256=sha(out/'visible/render/receipt.json'),driverSha256=sha(__file__),decoderSha256=sha(decoder),rows=rows)
(out/'color-interiors.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps([row for row in rows if row['frame']==0],indent=2))
