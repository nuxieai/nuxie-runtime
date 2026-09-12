"""Bounded private source-only folding experiment. No browser geometry input.
Usage: SCRIPT construct OUTPUT CONSTRUCTOR; SCRIPT capture OUTPUT
SCRIPT verify OUTPUT checks retained original/folded probes and evaluator bits.
"""
from pathlib import Path
import hashlib,json,shutil,subprocess,sys,struct,importlib.util,re
from PIL import Image
M=Path(__file__).resolve().parents[1];phase=sys.argv[1];out=Path(sys.argv[2]).resolve()
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
bind=lambda p:dict(path=str(Path(p).resolve()),sha256=sha(p))
def write(p,j):p.write_text(json.dumps(j,indent=2)+'\n')
def run(cmd,log):
 p=subprocess.run(cmd,capture_output=True);log.write_bytes(p.stdout+p.stderr)
 return dict(command=cmd,exitCode=p.returncode,log=bind(log))
def validate_capture(root):
 invocations=json.loads((root/'capture-invocations.json').read_text())
 assert len(invocations)==2 and all(r['exitCode']==0 for r in invocations),'Both capture drivers must succeed'
 for variant in ['original','folded']:
  receipt=json.loads((root/(variant+'-capture')/'receipt.json').read_text())
  assert receipt['status']=='passed-private-derived-baseline','Capture gate failed'
  assert receipt['rows'] and all(not r['geometryFailures'] and not r['pixelFailures'] and all(c['samePixels']for c in r['clearChecks']) for r in receipt['rows']),'Incomplete capture gates'
 return [bind(root/'capture-invocations.json'),*[bind(root/(v+'-capture')/'receipt.json')for v in ['original','folded']]]
def decode_wire(data,schema):
 # The ordinary two-bit header omits Bool fields. Pinned core registry types
 # are authoritative for those schema-known keys; never guess their byte size.
 kinds={int(k):v for k,v in re.findall(r'(\d+) => Some\(CoreRegistryFieldKind::(\w+)\)',schema)}
 field_ids={'Uint':0,'StringOrBytes':1,'Double':2,'Color':3,'Bool':4}
 assert data[:7]==b'RIVE\x07\x03\x00';pos=7
 def take(n):
  nonlocal pos
  assert 0<=n<=len(data)-pos,'Truncated wire';v=data[pos:pos+n];pos+=n;return v
 def uint():
  v=shift=0
  while True:
   byte=take(1)[0];v|=(byte&127)<<shift
   if byte<128:assert v<=0xffffffff;return v
   shift+=7;assert shift<35,'Invalid varuint'
 keys=[]
 while True:
  key=uint()
  if not key:break
  assert key not in keys;keys.append(key)
 toc={}
 for start in range(0,len(keys),4):
  packed=struct.unpack('<I',take(4))[0]
  for i,key in enumerate(keys[start:start+4]):toc[key]=(packed>>(2*i))&3
 result=[]
 while pos<len(data):
  kind=uint();props={}
  while True:
   key=uint()
   if not key:break
   assert key not in props,'Duplicate property'
   assert key in kinds and kinds[key]in field_ids,'Unqualified registry type'
   field=field_ids[kinds[key]]
   if field!=4:assert toc.get(key)==field,'Header/core field type disagreement'
   else:assert key not in toc,'Unexpected bool header entry'
   start=pos
   if field==0:value=uint()
   elif field==1:value=take(uint()).hex()
   elif field==2:value=struct.unpack('<f',take(4))[0]
   elif field==3:value=struct.unpack('<I',take(4))[0]
   else:value=take(1)[0];assert value in [0,1],'Invalid boolean payload'
   props[key]=dict(field=field,value=value,raw=data[start:pos].hex())
  result.append(dict(kind=kind,properties=props))
 return result
def validate_mapping(folder,evaluation):
 descriptor=json.loads((folder/'folding.json').read_text());path=folder/'paint-map.json'
 if descriptor.get('route')!='preserved-paint-graph':
  assert not path.exists(),'Unexpected map for compact paint route'
  return {}
 assert path.exists(),'Preserved graph requires paint-map'
 raw=json.loads(path.read_text());assert raw,'Preserved graph requires nonempty paint-map'
 mapping={int(k):v for k,v in raw.items()};assert len(mapping)==len(raw) and len(set(mapping.values()))==len(mapping),'Invalid map identity'
 schema=(M.parents[1]/'crates/nuxie-schema/src/generated/schema.rs').read_text()
 old=decode_wire((folder/'original.riv').read_bytes(),schema);new=decode_wire((folder/'scene.riv').read_bytes(),schema);start=evaluation['sizingEnd']
 assert len(old)==evaluation['originalRecords'] and len(new)==evaluation['foldedRecords'] and 0<start<=len(old),'Record count mismatch'
 schema=(M.parents[1]/'crates/nuxie-schema/src/generated/schema.rs').read_text();types={name:int(key)for name,key in re.findall(r'Definition \{\s+name: "([^"]+)",.*?type_key: Key \{\s+int: (\d+),',schema,re.S)}
 kept={types[name]for name in ['Shape','Rectangle','Fill','SolidColor','ClippingShape','DrawRules','DrawTarget']}
 expected={i-1 for i,r in enumerate(old)if i>=start and r['kind']in kept}
 assert set(mapping)==expected,'Incomplete kept paint record set'
 assert set(mapping.values())==set(range(start-1,len(new)-1)),'Mapped suffix coverage or prefix collision'
 assert len(new)==start+len(expected),'Unexpected suffix record count'
 for oldid,newid in mapping.items():assert old[oldid+1]['kind']==new[newid+1]['kind'],'Mapped record kind mismatch'
 # Prefix properties compare by encoded field payload representation; record
 # byte offsets naturally differ with the field-table, so ignore those offsets.
 def properties(r):return {k:(v['field'],struct.pack('<f',v['value']).hex()if v['field']==2 else v['value'])for k,v in r['properties'].items()}
 for a,b in zip(old[:start],new[:start]):assert a['kind']==b['kind'] and properties(a)==properties(b),'Changed prefix wire fields'
 return mapping
if phase=='construct':
 out.mkdir(parents=True,exist_ok=False);candidate=Path(sys.argv[3]).resolve();shutil.copy2(candidate,out/'candidate');shutil.copy2(__file__,out/'experiment.py');shutil.copy2(M/'validation/public-fixed-folding-cases.json',out/'cases.json')
 cases=json.loads((out/'cases.json').read_text());rows=[]
 for c in cases:
  f=out/c['name'];f.mkdir();write(f/'recipe.json',c['recipe']);commands=[]
  for trial in ['first','repeat']:
   r=run([str(out/'candidate'),str(f/'recipe.json'),str(f/trial)],f/(trial+'.log'));commands.append(r);assert r['exitCode']==0,r
  artifacts={}
  names=['base.riv','original.riv','scene.riv','scene.map.json','evaluation.json','folding.json']
  assert (f/'first/paint-map.json').exists()==(f/'repeat/paint-map.json').exists(),'Map presence differs'
  if (f/'first/paint-map.json').exists():names.append('paint-map.json')
  for n in names:
   assert (f/'first'/n).read_bytes()==(f/'repeat'/n).read_bytes(),n;artifacts[n]=bind(f/'first'/n)
  rows.append(dict(name=c['name'],recipe=bind(f/'recipe.json'),commands=commands,artifacts=artifacts))
 write(out/'construction.json',dict(scope='Source-authored private folding only; no observations used as constructor input',candidate=bind(out/'candidate'),script=bind(out/'experiment.py'),cases=bind(out/'cases.json'),rows=rows))
 # Adapters select exact authored requests and hash-bound existing ordinary files.
 # The bridge already constructs twice above; the adapter adds no geometry input.
 for variant,artifact in [('original','original.riv'),('folded','scene.riv')]:
  adapter='''#!/usr/bin/env python3
from pathlib import Path
import hashlib,json,shutil,sys
O=Path(__file__).resolve().parent
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
request=json.loads(Path(sys.argv[1]).read_text());cases=json.loads((O/'cases.json').read_text())
match=[c for c in cases if request==dict(html=c['html'],css=c['css'],width=c['compileViewport'][0],height=c['compileViewport'][1])];assert len(match)==1
c=match[0];receipt=json.loads((O/'construction.json').read_text());assert sha(O/'cases.json')==receipt['cases']['sha256']
r=next(r for r in receipt['rows']if r['name']==c['name']);a=r['artifacts'][ARTIFACT];m=r['artifacts']['scene.map.json']
for b in [a,m,r['recipe']]:assert sha(b['path'])==b['sha256']
t=Path(sys.argv[2]);shutil.copy2(a['path'],t);shutil.copy2(m['path'],t.with_suffix('.map.json'))
'''.replace('ARTIFACT',repr(artifact))
  p=out/(variant+'-adapter.py');p.write_text(adapter);p.chmod(0o755)
 print('Constructed twice; capture requires separate coordinated invocation.')
elif phase=='capture':
 assert out.is_relative_to(M/'output/playwright'),'browser output must be under output/playwright'
 probe=M/'output/wrapped-snapped-gate-r1/node-probe';renderer=M/'output/immutable-baseline-toolchain-r2/renderer-replay'
 assert sha(probe)=='2831265baa244ae03a26389fd82daaaa79eb88b77f542f6e8bc617343ca15b53'
 assert sha(renderer)=='276a49d810c05de7cfb0f0a4109853d12216da64e5d9d8e131f7400357f8e51f'
 results=[]
 for variant in ['original','folded']:
  command=['node',str(M/'validation/check-wrapped-rounded-baseline.mjs'),str(out/'cases.json'),str(out/(variant+'-adapter.py')),str(probe),str(renderer),str(out/(variant+'-capture'))]
  results.append(run(command,out/(variant+'-capture.log')))
 write(out/'capture-invocations.json',results);print(json.dumps(results,indent=2));raise SystemExit(1 if any(r['exitCode']!=0 for r in results)else 0)
elif phase=='verify':
 def read(p):return json.loads(p.read_text(),parse_int=lambda s:-0.0 if s=='-0'else int(s))
 def bits(x):return struct.unpack('<I',struct.pack('<f',x))[0]
 capture_bindings=validate_capture(out)
 rows=[];failures=[];count=0
 for c in read(out/'cases.json'):
  name=c['name'];evaluation=read(out/name/'first/evaluation.json');values=evaluation['worlds'];assert len({v['objectId']for v in values})==len(values)
  om=out/'original-capture'/name/'probe';fm=out/'folded-capture'/name/'probe';oldframes=read(om/'frames.json')['frames'];newframes=read(fm/'frames.json')['frames'];assert len(oldframes)==len(newframes)==8
  source_ids={n['object_id']for n in read(out/name/'first/scene.map.json')}
  paint_mapping=validate_mapping(out/name/'first',evaluation)
  assert len(set(paint_mapping.values()))==len(paint_mapping), 'Duplicate mapped paint IDs'
  for i in range(8):
   a=oldframes[i];b=newframes[i];assert [a[k]for k in ['frame','instance','step','width','height']]==[b[k]for k in ['frame','instance','step','width','height']]
   old=read(om/a['geometry']);new=read(fm/b['geometry']);assert len({o['objectId']for o in old})==len(old);assert len({o['objectId']for o in new})==len(new)
   old={o['objectId']:o for o in old};new={o['objectId']:o for o in new};prefix={id for id in old if id<evaluation['sizingEnd']-1};assert prefix=={id for id in new if id<evaluation['sizingEnd']-1};assert source_ids<=prefix
   delta=[]
   image_bindings={}
   for kind in ['native','chrome']:
    pa=out/'original-capture'/name/f'frame-{i}.{kind}.png';pb=out/'folded-capture'/name/f'frame-{i}.{kind}.png'
    image_bindings[kind]=dict(original=bind(pa),folded=bind(pb),identical=pa.read_bytes()==pb.read_bytes())
    with Image.open(pa) as ia, Image.open(pb) as ib:
     rgba_equal=ia.size==ib.size and ia.convert('RGBA').tobytes()==ib.convert('RGBA').tobytes()
    image_bindings[kind]['decodedIdentical']=rgba_equal
    if not rgba_equal:delta.append(dict(kind=kind+'-image-rgba',original=sha(pa),folded=sha(pb)))
   for id in sorted(prefix):
    ao=old[id];bo=new[id];assert len(ao['worldMatrix'])==len(bo['worldMatrix'])==6
    av=[ao['width'],ao['height'],*ao['worldMatrix']];bv=[bo['width'],bo['height'],*bo['worldMatrix']]
    for j in range(8):
     count+=1
     if bits(av[j])!=bits(bv[j]):delta.append(dict(kind='prefix',objectId=id,field=j,original=bits(av[j]),folded=bits(bv[j])))
   for old_id,new_id in paint_mapping.items():
    if old_id not in old:continue # Fill/color/clip/rule records have no transform.
    assert new_id in new, ('Missing mapped transform',old_id,new_id)
    for field in ['width','height','worldMatrix']:
     av=old[old_id][field];bv=new[new_id][field]
     if field!='worldMatrix':av=[av];bv=[bv]
     assert len(av)==len(bv)
     for axis,(x,y) in enumerate(zip(av,bv)):
      count+=1
      if bits(x)!=bits(y):delta.append(dict(kind='mapped-paint',objectId=old_id,mappedId=new_id,field=field,axis=axis,original=bits(x),folded=bits(y)))
   if paint_mapping:
    assert set(new)==prefix|{paint_mapping[id]for id in paint_mapping if id in old}, 'Unexpected folded transform owner'
   assert {v['objectId']for v in values}==set(old), 'Evaluator/native owner set mismatch'
   for v in values:
    assert len(v['worldBits'])==6;id=v['objectId'];assert id in old
    for axis in range(6):
     count+=1;actual=bits(old[id]['worldMatrix'][axis])
     if v['worldBits'][axis]!=actual:delta.append(dict(kind='evaluator',objectId=id,axis=axis,expected=v['worldBits'][axis],actual=actual))
   row=dict(name=name,frame=i,prefixOwners=len(prefix),evaluatedOwners=len(values),images=image_bindings,differences=delta);rows.append(row)
   if delta:failures.append(row)
 write(out/'verification-observations.json',rows);write(out/'verification.json',dict(scope='Exact unchanged prefix geometry and original-native/evaluator full world matrices; separate from pixel qualification',frames=len(rows),scalarChecks=count,failedFrames=len(failures),failures=failures,observations=bind(out/'verification-observations.json'),script=bind(__file__),captureBindings=capture_bindings,wireDecoder='local pinned-core-registry decoder in verifier script',schema=bind(M.parents[1]/'crates/nuxie-schema/src/generated/schema.rs')))
 print(json.dumps(dict(frames=len(rows),scalarChecks=count,failedFrames=len(failures))))
 raise SystemExit(1 if failures else 0)
else:raise SystemExit('unknown phase')
