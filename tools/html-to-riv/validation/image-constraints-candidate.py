"""Probe ordinary outer min/max fields; never opens public admission."""
import argparse
import importlib.util
import json
from pathlib import Path
import re
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
parser.add_argument('--composition',choices=['direct','ratio-and-inner'],default='direct')
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
cases_path = MODULE / 'validation/image-constraints-cases.json'
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
    'schema':bind(schema), 'variant':args.composition, 'fields':fields, 'browserGeometryConsumed':False})
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
    if args.composition=='ratio-and-inner':
        def set_field(props,record,key,value,kind,reason):
            changes.append({'record':record,'field':key,'before':props.get(key),'after':value,'reason':reason})
            kinds[key]=kind;props[key]=value
        if 'intrinsic-' in c['name']:
            set_field(owner,artboard+object_id,8,0.,2,'Derive intrinsic height from ordinary ratio instead of retaining a second fixed intrinsic axis')
            set_field(style,style_id,608,3,0,'Automatic ratio-derived height')
            set_field(style,style_id,656,2,0,'Hug ratio-derived height')
            set_field(style,style_id,524,1.5,2,'Encoded fixture asset ratio 96/64')
        elif 'padded-' in c['name']:
            inner=[(idx,props) for idx,(kind,props) in enumerate(records) if kind==409 and props.get(5)==object_id]
            assert len(inner)==1
            inner_style_id=artboard+inner[0][1][494];inner_style=records[inner_style_id][1]
            value=c['bounds']['maxWidth']-(0 if c['contentBox'] else c['pointInsets'])
            set_field(inner_style,inner_style_id,fields['maxWidth'],float(max(value,0)),2,'Clamp inner ratio/content width during measurement; convert original authored point bound to content box')
            set_field(inner_style,inner_style_id,fields['maxWidthUnitsValue'],1,0,'Point content bound derived from source, not browser geometry')
        elif c['name']=='image-constraints-fixed-width-auto-height-max-height':
            set_field(style,style_id,fields['maxWidth'],120.,2,'Protect authored fixed width from native opposite-axis maximum ratio transfer')
            set_field(style,style_id,fields['maxWidthUnitsValue'],1,0,'Exact authored width as synthetic maximum')
        else:raise AssertionError('Composition has no source recipe for '+c['name'])
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
