"""Audit exactly the three changed prior outputs and retain unchanged source text."""
from pathlib import Path
from fractions import Fraction
import importlib.util
import hashlib
import json
import struct

module=Path(__file__).resolve().parent.parent
root=module/'output/numeric-token-changed-outputs-r1'
root.mkdir(exist_ok=False)
read=lambda p:json.loads(Path(p).read_text())
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
bind=lambda p:dict(path=str(p),sha256=sha(p))
spec=importlib.util.spec_from_file_location('wire',module/'validation/content-box-rounding-wire.py')
wire=importlib.util.module_from_spec(spec);spec.loader.exec_module(wire)
manifest_file=module/'output/numeric-token-prior-regression-r1/manifest.json'
manifest=read(manifest_file);changed=[r for r in manifest['results']if not r['exact']]
assert manifest['total']==694 and manifest['passed']==691 and len(changed)==3
compiler=module/'output/public-numeric-token-r3/frozen/html-to-riv'
assert sha(compiler)==manifest['compilerSha256']
fixtures=[];audits=[]
for item in changed:
    request=Path(item['request']);source=read(request);name=request.parent.name
    assert name in ['direct','fallback','variable'] and item['exitCode']==0
    assert sha(request)==item['requestSha256']
    old=Path(item['priorRiv']);new=Path(item['result'])/'scene.riv'
    old_map=Path(item['priorMap']);new_map=Path(item['result'])/'scene.map.json'
    assert sha(old)==item['rivSha256'] and sha(new)==item['actualRivSha256']
    assert old_map.read_bytes()==new_map.read_bytes()
    assert sha(old_map)==item['mapSha256']==item['actualMapSha256']
    before=wire.records(old.read_bytes());after=wire.records(new.read_bytes());assert len(before)==len(after)
    node=read(old_map)[0];assert node['id']=='a';changes=[]
    for index,(a,b)in enumerate(zip(before,after)):
        assert a['kind']==b['kind'] and a['properties'].keys()==b['properties'].keys()
        for key,value in a['properties'].items():
            if value!=b['properties'][key]:changes.append(dict(fileRecordIndex=index,typeKey=a['kind'],propertyKey=key,before=value,after=b['properties'][key]))
    assert len(changes)==1
    change=changes[0];assert (change['fileRecordIndex'],change['typeKey'],change['propertyKey'])==(node['object_id']+1,409,7)
    old_width=change['before']['value'];new_width=change['after']['value']
    assert old_width==100.71399688720703 and new_width==100.71428680419922
    assert change['before']['field']==change['after']['field']==2
    byte_changes=[i for i,(a,b)in enumerate(zip(old.read_bytes(),new.read_bytes()))if a!=b]
    assert len(old.read_bytes())==len(new.read_bytes()) and all(change['before']['offset']<=i<change['before']['offset']+4 for i in byte_changes)
    ideal=Fraction('100.71428680419922')
    audits.append(dict(name=name,source=bind(request),request=source,oldRiv=bind(old),newRiv=bind(new),oldMap=bind(old_map),newMap=bind(new_map),mapsExact=True,
                       changes=changes,changedByteOffsets=byte_changes,authoredDecimal=str(ideal),oldAbsoluteError=str(abs(Fraction(old_width)-ideal)),newAbsoluteError=str(abs(Fraction(new_width)-ideal))))
    fixtures.append(dict(name='numeric-regression-'+name,html=source['html'],css=source['css'],features=['source-identical changed numeric width'],viewports=[[400,100],[200,80],[100,40],[400,100]]))
fixture_file=module/'validation/public-numeric-regression-cases.json'
fixture_file.write_text(json.dumps(fixtures,indent=2)+'\n')
(root/'field-audit.json').write_text(json.dumps(dict(scope='Three changed historical400x100 requests; only LayoutComponent width field7 changes, maps and every other byte remain exact.',manifest=bind(manifest_file),compiler=bind(compiler),fixtures=bind(fixture_file),rows=audits),indent=2)+'\n')
print(json.dumps(dict(changedCases=len(audits),changedFields=3,mapsExact=3,oldWidth=old_width,newWidth=new_width)))
