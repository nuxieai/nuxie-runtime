"""Transfer reviewed public border pixels to lifecycle frames with artifact checks.
Usage: python3 audit-border-lifecycle-review.py SOURCE_REPLAY [SOURCE_REPLAY ...] RECORDING TARGET_REPLAY
Recomputes all checks on every invocation. Never creates a direct inspection.
"""
import hashlib
import json
import runpy
import sys
from pathlib import Path
assert len(sys.argv)>=4, 'Expected source replay(s), recording and target replay'
*sources,recording,target=map(Path,sys.argv[1:])
def read(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
record=runpy.run_path(str(Path(__file__).with_name('record-visual-review.py')))['record']
life=read(recording/'lifecycle.json');assert life['kind']=='border-public'
current=read(target/'replay.json')
assert current['lifecycleSha256']==sha(recording/'lifecycle.json')
references={}; owners={}; source_receipts=[]
for source in sources:
 assert record(source,audit=True)['remaining']==0
 original=read(source/'replay.json')
 assert sha(Path(original['oracle']))==original['oracleSha256']
 assert read(Path(original['oracle']))['browser']==current['browser']
 for case in original['cases']:
  key=case['name'],case['width']
  assert key not in references, f'Ambiguous source reference: {key}'
  assert case['name'] not in owners or owners[case['name']]==source
  references[key]=case;owners[case['name']]=source
 source_receipts.append({'path':str(source),'sourceReplaySha256':sha(source/'replay.json'),'sourceReviewSha256':sha(source/'visual-inspection.json')})
frames={}
for fixture in life['cases']:
 name=fixture['name'];source=owners[name];request=read(source/(name+'.json'))
 assert request['html']==fixture['html'] and request['css']==fixture['css']
 assert (request['width'],request['height'])==(390,320)
 assert (source/(name+'.riv')).read_bytes()==(recording/(name+'.riv')).read_bytes()
 assert read(source/(name+'.requirements.json'))==fixture['runtimeRequirements']==read(recording/(name+'.requirements.json'))
 expected_assets={}
 if fixture.get('image'):
  image=Path(__file__).parent.parent/'tests/assets/quadrants.png'
  expected_assets['photo']={'kind':'image','bytes':list(image.read_bytes())}
 if fixture.get('font'):
  font=Path(__file__).parent.parent/'tests/assets/Inter-Regular.ttf'
  expected_assets['inter']={'kind':'font','family':'Inter','weight':400,'bytes':list(font.read_bytes())}
 assert request.get('assets',{})==expected_assets
 for view in fixture['views']:
  key=f'{name}-{view["instance"]}-step{view["frame"]}'
  assert key not in frames;frames[key]=(fixture,view)
transfers=[]
assert len(frames)==len(current['cases'])
for case in current['cases']:
 fixture,view=frames.pop(case['name']);name=fixture['name']
 assert case['qualification']=='public-compiler-lifecycle'
 assert case['html']==fixture['html'] and case['css']==fixture['css']
 assert case['runtimeRequirements']==fixture['runtimeRequirements']
 assert case['width']==view['width'] and case['frame']==view['frame'] and case['instance']==view['instance']
 assert case['streamSha256']==sha(Path(view['stream']))
 reference=references[(name,case['width'])]
 assert not case['geometryFailures'] and not case['failures']
 for kind in ['browser','native']:
  assert sha(Path(case['prefix']+'.'+kind+'.png'))==case[kind+'Sha256']==reference[kind+'Sha256']
 transfers.append({'name':case['name'],'width':case['width'],'sourceName':name,'browserSha256':case['browserSha256'],'nativeSha256':case['nativeSha256']})
assert not frames
receipt={'status':'complete','reviewed':len(transfers),'targetReplaySha256':sha(target/'replay.json'),'lifecycleSha256':sha(recording/'lifecycle.json'),'transfers':transfers}
if len(sources)==1:
 receipt.update({key:source_receipts[0][key] for key in ['sourceReplaySha256','sourceReviewSha256']})
else:receipt['sources']=source_receipts
(target/'public-visual-transfer.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({'reviewed':len(transfers),'remaining':0}))
