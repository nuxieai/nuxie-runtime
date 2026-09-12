"""Probe propagated point bounds and an inner ratio owner after preferred-axis clamping; no public admission."""
import argparse
import importlib.util
import json
from pathlib import Path
import re
import struct
import math
import shutil
import subprocess
import sys

MODULE = Path(__file__).resolve().parents[1]
codec_path = MODULE / 'validation/image-percent-padding-candidate.py'
spec = importlib.util.spec_from_file_location('ordinary_codec', codec_path)
codec = importlib.util.module_from_spec(spec)
spec.loader.exec_module(codec)
read, write, bind, sha = codec.read, codec.write, codec.bind, codec.sha
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('root',type=Path)
parser.add_argument('--cases',type=Path,default=MODULE/'validation/image-preferred-constraints-followup-cases.json')
parser.add_argument('--only',help='Comma-separated complete case names')
args=parser.parse_args()
root = args.root.resolve()
root.mkdir(parents=True, exist_ok=False)
build = MODULE / 'output/public-image-percent-padding-build-r2'
frozen = build / 'frozen'
for b in read(frozen / 'source-bindings.json')['files']:
    assert sha(b.get('snapshot', b['path'])) == b['sha256']
(root / 'frozen').symlink_to(frozen, target_is_directory=True)
schema = MODULE.parents[1] / 'crates/nuxie-schema/src/generated/schema.rs'
text = schema.read_text()
fields = {}
for name in ['minWidth','minHeight','maxWidth','maxHeight']:
    for field in [name, name+'UnitsValue']:
        matches = re.findall(r'name: "'+field+r'",\s*key: Key \{\s*int: (\d+)', text)
        assert len(matches) == 1
        fields[field] = int(matches[0])
cases_path = args.cases.resolve()
cases = read(cases_path)
if args.only:
    names=set(args.only.split(','));cases=[c for c in cases if c['name'] in names]
    assert {c['name'] for c in cases}==names
write(root / 'cases.json', cases)
shutil.copy2(cases_path,root/'all-source-cases.json')
shutil.copy2(__file__, root / 'candidate-generator.py')
shutil.copy2(codec_path, root / 'ordinary-codec.py')
authoring = [{**bind(cases_path), 'snapshot':str(root/'all-source-cases.json')},
             {**bind(codec_path), 'snapshot':str(root/'ordinary-codec.py')}]
for asset in sorted({a for c in cases for a in c['assetFiles'].values()}):
    snapshot = root / 'authoring-inputs' / asset
    snapshot.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(MODULE / asset, snapshot)
    authoring.append({**bind(MODULE / asset),'snapshot':str(snapshot)})
write(root / 'authoring-bindings.json', authoring)
write(root / 'build-receipt.json', {'scope':'Private ordinary-file composition; no public admission or native qualification',
    'baseline':codec.BASELINE, 'seedBuild':str(build), 'seedCompiler':bind(frozen/'html-to-riv'),
    'generator':bind(root/'candidate-generator.py'), 'codec':bind(root/'ordinary-codec.py'),
    'schema':bind(schema), 'variant':'source-propagated point bounds or outer clamp with inner ratio owner', 'fields':fields, 'browserGeometryConsumed':False})
rows=[]
for c in cases:
    folder=root/c['name'];folder.mkdir()
    request=dict(c['input']);request['assets']={k:{'kind':'image','bytes':list((MODULE/v).read_bytes())} for k,v in c['assetFiles'].items()}
    write(folder/'request.json',request)
    rejected=subprocess.run([str(frozen/'html-to-riv'),str(folder/'request.json'),str(folder/'public.riv')],capture_output=True)
    (folder/'public.log').write_bytes(rejected.stdout+rejected.stderr)
    assert rejected.returncode != 0
    diagnostics=json.loads(rejected.stderr)
    assert diagnostics[0]['code']=='unsupported-target-semantics'
    seed=dict(request);seed['css']+='\n#image{min-width:0;min-height:0;max-width:none;max-height:none}'
    write(folder/'seed-request.json',seed)
    accepted=subprocess.run([str(frozen/'html-to-riv'),str(folder/'seed-request.json'),str(folder/'seed.riv')],capture_output=True)
    (folder/'seed.log').write_bytes(accepted.stdout+accepted.stderr)
    assert accepted.returncode==0,accepted.stderr
    encoded=(folder/'seed.riv').read_bytes();kinds,records=codec.decode(encoded)
    assert codec.encode(kinds,records)==encoded
    artboard=next(i for i,(kind,_) in enumerate(records) if kind==1)
    object_id=next(n['object_id'] for n in read(folder/'seed.map.json') if n['id']=='image')
    owner=records[artboard+object_id][1];style_id=artboard+owner[494];style=records[style_id][1]
    changes=[]
    for name,value in c['bounds'].items():
        percent=isinstance(value,str) and value.endswith('%')
        coefficient=float(value[:-1]) if percent else float(value)
        assert not (percent and c['contentBox']), 'Mixed content-box percentage bounds need another candidate'
        if c['contentBox']:coefficient+=c['pointInsets']
        for field,value,kind in [(fields[name],coefficient,2),(fields[name+'UnitsValue'],2 if percent else 1,0)]:
            changes.append({'record':style_id,'field':field,'before':style.get(field),'after':value,'reason':'Authored bound on ordinary outer owner; content-box points include authored point insets'})
            kinds[field]=kind;style[field]=value
    # Preserve both explicit reset minima. A missing min vector component is
    # not equivalent to CSS reset min:0 in the immutable native ratio path.
    for name in ['minWidth','minHeight']:
        if name not in c['bounds']:
            for key,value,kind in [(fields[name],0.,2),(fields[name+'UnitsValue'],1,0)]:
                changes.append({'record':style_id,'field':key,'before':style.get(key),'after':value,'reason':'Explicit original CSS reset zero minimum'})
                kinds[key]=kind;style[key]=value
    for name,value in c['guard'].items():
        assert isinstance(value,str) and value.endswith('%')
        for key,value,kind in [(fields[name],float(value[:-1]),2),(fields[name+'UnitsValue'],2,0)]:
            changes.append({'record':style_id,'field':key,'before':style.get(key),'after':value,'reason':'Synthetic maximum equal to authored responsive preferred axis suppresses opposite-axis max transfer'})
            kinds[key]=kind;style[key]=value
    preferred=c['preferredAxis']; automatic=1-preferred
    names=['Width','Height']; f32=lambda v:struct.unpack('<f',struct.pack('<f',v))[0]
    ratio=f32(96/64)
    if c['composition']=='coefficient-clamp':
        # Every coefficient here resolves against the same original containing
        # axis. Clamp coefficients at source, retaining a responsive percentage.
        number=lambda name,default:float(c['bounds'][name][:-1]) if name in c['bounds'] else default
        low=number('min'+names[preferred],0)
        high=number('max'+names[preferred],math.inf)
        coefficient=max(low,min(float(c['preferredCoefficient']),high))
        key=7 if preferred==0 else 8
        changes.append({'record':artboard+object_id,'field':key,'before':owner[key],'after':coefficient,'reason':'Same-original-basis percentage coefficients clamped directly from authored source values'})
        owner[key]=coefficient
        for name in ['minWidth','minHeight','maxWidth','maxHeight']:
            for field in [fields[name],fields[name+'UnitsValue']]:
                style.pop(field,None)
        changes.append({'reason':'Bound arbitration is incorporated into the responsive preferred coefficient; ordinary ratio owner uses unconstrained defaults','selectedCoefficient':coefficient,'sourceCoefficient':c['preferredCoefficient'],'sourceBounds':c['bounds']})
    elif c['composition']=='propagated-point-bounds':
        low=float(c['bounds'].get('min'+names[preferred],0))
        high=c['bounds'].get('max'+names[preferred])
        if high is not None:high=max(float(high),low)
        transfer=lambda v:f32(v/ratio) if automatic==1 else f32(v*ratio)
        for prefix,value in [('min',transfer(low)),('max',None if high is None else transfer(high))]:
            if value is None:continue
            name=prefix+names[automatic]
            for key,val,kind in [(fields[name],value,2),(fields[name+'UnitsValue'],1,0)]:
                changes.append({'record':style_id,'field':key,'before':style.get(key),'after':val,'reason':'Source point bound propagated through intrinsic ratio after minimum-over-maximum normalization'})
                kinds[key]=kind;style[key]=val
    else:
        assert c['composition']=='outer-clamp-inner-ratio'
        # Retain authored preferred percent/min on the original outer owner.
        # The new inner owner resolves 100% of that clamped preferred dimension;
        # its automatic ratio axis supplies the outer intrinsic content size.
        changes.append({'record':style_id,'field':524,'before':style.pop(524,None),'after':None,'reason':'Outer owner clamps preferred dimension before the separate inner ratio measurement'})
        style[598]=1 if preferred==0 else 3
        style[632]=6 if preferred==0 else 2
        inner_id=len(records)-artboard
        inner_style={524:ratio,598:1,632:6,607:2 if preferred==0 else 3,608:2 if preferred==1 else 3,655:0 if preferred==0 else 2,656:0 if preferred==1 else 2}
        inner={5:object_id,7:100. if preferred==0 else 0.,8:100. if preferred==1 else 0.,494:inner_id+1,196:True}
        records.append((409,inner));records.append((420,inner_style))
        image_id=next(i for i,(kind,props) in enumerate(records) if kind==100 and props.get(5)==object_id)
        changes.append({'record':image_id,'field':5,'before':object_id,'after':inner_id,'reason':'Ordinary Image belongs to the source-driven inner ratio owner'})
        records[image_id][1][5]=inner_id
        changes.append({'newOwner':inner_id,'parent':object_id,'component':inner,'style':inner_style,'reason':'100% preferred-axis inner ratio owner; no browser dimensions'})
    data=codec.encode(kinds,records)
    assert codec.encode(*codec.decode(data))==data
    assert [p[212] for kind,p in records if kind==106]==[p[212] for kind,p in codec.decode(encoded)[1] if kind==106]
    (folder/'scene.riv').write_bytes(data);shutil.copy2(folder/'seed.map.json',folder/'scene.map.json')
    write(folder/'candidate-changes.json',changes)
    row={'name':c['name'],'compiled':True,'status':0,'scope':'Private source-recipe file composition; actual public request remains rejected',
        'publicDiagnostics':diagnostics,'browserGeometryConsumed':False,'requestSha256':sha(folder/'request.json'),
        'rivSha256':sha(folder/'scene.riv'),'mapSha256':sha(folder/'scene.map.json'),
        'changes':bind(folder/'candidate-changes.json'),'seedRequest':bind(folder/'seed-request.json'),'seedScene':bind(folder/'seed.riv')}
    rows.append(row);write(folder/'compile-result.json',row)
write(root/'compile-receipt.json',rows)
print(json.dumps({'privateFiles':len(rows),'publicAdmission':False}))
