"""Bind exact changed files to native streams and full-resolution visual coverage."""
from pathlib import Path
import hashlib
import importlib.util
import json
import subprocess
from PIL import Image,ImageDraw

module=Path(__file__).resolve().parent.parent
root=module/'output/numeric-token-changed-outputs-r1'
read=lambda p:json.loads(Path(p).read_text())
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
bind=lambda p:dict(path=str(p),sha256=sha(p))
native_file=root/'native/receipt.json';native=read(native_file)
audit_file=root/'field-audit.json';audit=read(audit_file)
assert native['status']=='passed-public-baseline' and len(native['rows'])==24
assert native['toolHashes']['compiler']==audit['compiler']['sha256']
assert all(not r['geometryFailures']and not r['pixelFailures']and all(c['samePixels']for c in r['clearChecks'])for r in native['rows'])
assert all(r['nativeIdentical']and r['chromeIdentical']for r in native['repeated'])
spec=importlib.util.spec_from_file_location('wire',module/'validation/content-box-rounding-wire.py')
wire=importlib.util.module_from_spec(spec);spec.loader.exec_module(wire)
transfers=[]
for case in audit['rows']:
    name='numeric-regression-'+case['name'];scene=root/'native'/name
    driver_request=read(scene/'request.json')
    assert driver_request['html']==case['request']['html'] and driver_request['css']==case['request']['css']
    assert [driver_request['width'],driver_request['height']]==[390,160]
    assert [case['request']['width'],case['request']['height']]==[400,100]
    # Existing driver remains unchanged. Its two artboard initialization fields
    # differ; the historical-request file is probed independently at every size.
    a=wire.records(Path(case['newRiv']['path']).read_bytes());b=wire.records((scene/'scene.riv').read_bytes())
    differences=[]
    for i,(x,y)in enumerate(zip(a,b)):
        assert x['kind']==y['kind']and x['properties'].keys()==y['properties'].keys()
        for key,value in x['properties'].items():
            if value!=y['properties'][key]:differences.append((i,key,value['value'],y['properties'][key]['value']))
    assert differences==[(1,7,400.0,390.0),(1,8,100.0,160.0)]
    assert Path(case['newMap']['path']).read_bytes()==(scene/'scene.map.json').read_bytes()
    out=root/('exact-file-'+case['name']);assert not out.exists()
    command=[native['tools']['probe'],case['newRiv']['path'],str(out),'400x100','200x80','100x40','400x100']
    result=subprocess.run(command,capture_output=True)
    (root/('exact-file-'+case['name']+'.log')).write_bytes(result.stdout+result.stderr)
    assert result.returncode==0 and sha(out/'scene.riv')==case['newRiv']['sha256']
    frame_pairs=[]
    for frame in range(8):
        for suffix in ['geometry.json','stream']:
            original=out/f'frame-{frame}.{suffix}';rendered=scene/'probe'/f'frame-{frame}.{suffix}'
            assert original.read_bytes()==rendered.read_bytes()
            frame_pairs.append(dict(exactFileObservation=bind(original),renderedObservation=bind(rendered)))
    transfers.append(dict(name=name,command=command,exactRiv=case['newRiv'],driverRiv=bind(scene/'scene.riv'),
                          initialArtboardOnlyChanges=differences,allEightGeometryAndStreamsExact=True,frames=frame_pairs))
(root/'exact-file-transfer.json').write_text(json.dumps(dict(scope='The historical-request files have identical measured geometry and recorded drawing commands at all eight frames. Native pixel evidence transfers by exact replay input; source and maps are independently bound.',native=bind(native_file),audit=bind(audit_file),rows=transfers),indent=2)+'\n')

def images(row):
    result=[]
    for role in ['chrome','native']:
        file=Path(row['prefix']+'.'+role+'.png');assert sha(file)==row[role+'Sha256']
        im=Image.open(file).convert('RGBA');assert im.size==(row['width'],row['height'])
        result.append((file,im))
    return result
def identity(row):return tuple((im.size,hashlib.sha256(im.tobytes()).hexdigest())for _,im in images(row))
def ref(row):return {k:row[k]for k in ['name','frame','prefix','width','height']}
known={};direct=[];copies=[]
for row in native['rows']:
    key=identity(row)
    if key in known:
        source=known[key]
        for(_,a),(_,b)in zip(images(row),images(source)):assert a.tobytes()==b.tobytes()and a.size==b.size
        copies.append(dict(source=ref(source),target=ref(row),bothImagesCompleteRgbaIdentical=True))
    else:known[key]=row;direct.append(row)
assert len(direct)==3 and len(copies)==21
sheet=Image.new('RGBA',(840,360),'#d0d0d0');draw=ImageDraw.Draw(sheet);placements=[]
for i,row in enumerate(direct):
    y=i*120;draw.text((4,y+2),f"{row['width']}x{row['height']}: Chrome left, native right; original transparent source",fill='black')
    for role,(source,im),x in zip(['chrome','native'],images(row),[4,432]):
        sheet.paste(im,(x,y+18));assert sheet.crop((x,y+18,x+im.width,y+18+im.height)).tobytes()==im.tobytes()
        placements.append(dict(pair=ref(row),role=role,source=bind(source),xy=[x,y+18],size=list(im.size)))
file=root/'visual-pairs.png';sheet.save(file)
(root/'visual-evidence.json').write_text(json.dumps(dict(scope='New full-pair image review only. Historical controls had geometry evidence and no Chrome/PNG review. Transparent boxes make these images insufficient evidence of width correctness.',native=bind(native_file),sheet=bind(file),directPairs=[ref(r)for r in direct],placements=placements,transfers=copies),indent=2)+'\n')
print(json.dumps(dict(exactHistoricalFiles=3,exactGeometryAndStreamFrames=24,geometryPass=24,pixelPass=24,clearPass=48,directVisualPairs=3,imagePairTransfers=21)))
