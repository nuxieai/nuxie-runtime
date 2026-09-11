"""Negative controls for predecessor reuse, without running a renderer."""
import json,runpy,tempfile,unittest
from pathlib import Path
from PIL import Image
m=runpy.run_path(str(Path(__file__).with_name('replay-full-gallery-from-verified-migration.py')))
sha=m['sha'];verify=m['verify_predecessor']

def write(path,value):path.write_text(json.dumps(value))
def fixture(root):
 prior=root/'prior';prior.mkdir();source=root/'source';source.mkdir();prefix=str(source/'case')
 for suffix in m['SUFFIXES']:
  p=Path(prefix+'.'+suffix)
  if suffix.endswith('.png'):Image.new('RGBA',(2,2),(255,0,0,255)).save(p)
  else:p.write_text('fixture '+suffix)
 case=dict(name='case',width=2,height=2,artifactPrefix=prefix,status='passed',errors=[],metrics={'backend':'rust-metal','browserVersion':'153.0.8010.12'})
 write(source/'review.json',dict(status='passed',errors=[],checks=[dict(status='passed',errors=[],project='native')],expected=1,cases=[case]))
 write(source/'combined-visual-comparison.json',dict(status='complete',remaining=[],reviewSha256=sha(source/'review.json'),reviewedPairs=1,scenePairs=1))
 binding=dict(index=0,name='case',width=2,height=2,files={s:dict(path=prefix+'.'+s,sha256=sha(prefix+'.'+s)) for s in m['SUFFIXES']})
 write(prior/'source-bindings.json',[binding]);output=prior/'00000-new-renderer.native.png';output.write_bytes(Path(prefix+'.native.png').read_bytes())
 row=dict(index=0,name='case',width=2,streamSha256=sha(prefix+'.stream'),expectedNativeSha256=sha(output),**{'source-controlSha256':sha(output),'new-rendererSha256':sha(output)},reviewTransferred=True,output=str(output))
 (prior/'rows.jsonl').write_text(json.dumps(row)+'\n')
 receipt=dict(status='complete',expected=1,completed=1,reviewedPairs=1,remaining=[],backend='rust-metal',mode='clockwise-atomic',rowsSha256=sha(prior/'rows.jsonl'),sourceBindingsSha256=sha(prior/'source-bindings.json'),sourceGallery=str(source),sourceReviewSha256=sha(source/'review.json'),sourceVisualProofSha256=sha(source/'combined-visual-comparison.json'))
 for label in ['old','new']:
  directory=root/label;directory.mkdir();(directory/'renderer-replay').write_text(label)
  write(directory/'manifest.json',{'files':{'renderer-replay':{'sha256':sha(directory/'renderer-replay')}}})
  receipt.update({label+'Manifest':str(directory/'manifest.json'),label+'ManifestSha256':sha(directory/'manifest.json'),label+'RendererSha256':sha(directory/'renderer-replay')})
 write(prior/'receipt.json',receipt);return prior

class Controls(unittest.TestCase):
 def test_valid_fixture(self):
  with tempfile.TemporaryDirectory() as temp:verify(fixture(Path(temp)),expected=1)
 def test_production_requires_8052(self):
  with tempfile.TemporaryDirectory() as temp:
   with self.assertRaises(AssertionError):verify(fixture(Path(temp)))
 def test_mutations_are_rejected(self):
  for mutation in ['running','incomplete','remaining','journal','bindings','stream','request','browser','native','old-binary','new-manifest','output','false-control','identity','duplicate']:
   with self.subTest(mutation=mutation),tempfile.TemporaryDirectory() as temp:
    root=Path(temp);prior=fixture(root);receipt=json.loads((prior/'receipt.json').read_text())
    if mutation in ['running','incomplete','remaining']:
     receipt.update({'running':{'status':'running'},'incomplete':{'completed':0},'remaining':{'remaining':['case']}}[mutation]);write(prior/'receipt.json',receipt)
    elif mutation in ['journal','bindings']:(prior/('rows.jsonl' if mutation=='journal' else 'source-bindings.json')).write_text('[]')
    elif mutation in ['stream','request','browser','native']:
     suffix={'stream':'stream','request':'json','browser':'browser.png','native':'native.png'}[mutation];(root/'source'/('case.'+suffix)).write_text('tampered')
    elif mutation=='old-binary':(root/'old'/'renderer-replay').write_text('tampered')
    elif mutation=='new-manifest':(root/'new'/'manifest.json').write_text('{}')
    elif mutation=='output':(prior/'00000-new-renderer.native.png').write_text('tampered')
    else:
     row=json.loads((prior/'rows.jsonl').read_text());row.update({'false-control':{'source-controlSha256':'0'*64},'identity':{'name':'other'},'duplicate':{}}[mutation])
     (prior/'rows.jsonl').write_text((json.dumps(row)+'\n')*(2 if mutation=='duplicate' else 1));receipt['rowsSha256']=sha(prior/'rows.jsonl');write(prior/'receipt.json',receipt)
    with self.assertRaises((AssertionError,KeyError,ValueError)):verify(prior,expected=1)

if __name__=='__main__':unittest.main()
