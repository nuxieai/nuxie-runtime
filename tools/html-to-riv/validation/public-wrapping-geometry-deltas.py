"""Report every nonzero authored-owner geometry delta without changing gates."""
from pathlib import Path
import hashlib,json,math,sys
root=Path(sys.argv[1]).resolve();out=root/'geometry-deltas.json';assert not out.exists()
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
receipt=root/'receipt.json';doc=json.loads(receipt.read_text());deltas=[];checks=0;maps={}
for row in doc['rows']:
 path=Path(row['prefix']).parent/'scene.map.json';assert sha(path)==row['sourceMapSha256']
 if str(path) not in maps:maps[str(path)]=json.loads(path.read_text())
 objects={x['objectId']:x for x in row['geometry']}
 assert {x['id']for x in maps[str(path)]}==set(row['boxes'])
 for owner in maps[str(path)]:
  obj=objects[owner['object_id']];expected=row['boxes'][owner['id']]
  actual=dict(x=obj['worldMatrix'][4],y=obj['worldMatrix'][5],width=obj['width'],height=obj['height'])
  for axis in actual:
   checks+=1;delta=actual[axis]-expected[axis];assert math.isfinite(delta)
   if delta!=0:deltas.append(dict(name=row['name'],frame=row['frame'],id=owner['id'],axis=axis,native=actual[axis],chrome=expected[axis],delta=delta))
result=dict(scope='All named-owner geometry differences from existing independently captured native/Chrome values. No tolerance changes or new native observation.',receipt=dict(path=str(receipt),sha256=sha(receipt)),script=dict(path=str(Path(__file__).resolve()),sha256=sha(__file__)),checks=checks,nonzero=len(deltas),maximumAbsoluteDelta=max([abs(x['delta'])for x in deltas],default=0),deltas=deltas)
out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:result[k]for k in ['checks','nonzero','maximumAbsoluteDelta']}))
