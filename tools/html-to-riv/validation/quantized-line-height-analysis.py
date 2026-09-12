"""Recompute metric residuals and exact historical control/reference joins.
Observations are analysis inputs only; this script never emits a scene.
"""
from pathlib import Path
from PIL import Image
import hashlib,json,sys
module=Path(__file__).resolve().parents[1]
root=Path(sys.argv[1]).resolve()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def rgba_equal(a,b):
 a=Image.open(a).convert('RGBA');b=Image.open(b).convert('RGBA')
 return a.size==b.size and a.tobytes()==b.tobytes()
native_path=root/'native-receipt.json';ink_path=root/'ink-region-receipt.json'
r=json.loads(native_path.read_text());ink=json.loads(ink_path.read_text())
cases=json.loads((root/'cases.json').read_text());case_by_name={c['name']:c for c in cases}
rows=[]
for row in r['rows']:
 c=case_by_name[row['name']];box=row['browserMetrics']['box'];owner=row['nativeOwner']
 errors={k:owner[k]-box[k] for k in ['width','height']}
 errors.update(x=owner['worldMatrix'][4]-box['x'],y=owner['worldMatrix'][5]-box['y'])
 baselines=[] if row['baselineComparisonSkipped'] else [n-v['baseline'] for n,v in zip(row['nativeBaselines'],row['browserMetrics']['fragments'])]
 rows.append(dict(name=row['name'],frame=row['frame'],candidate=c['request']['mode']=='chromium-leading',boxResidual=errors,baselineResidual=baselines,metricFailures=row['metricFailures'],baselineComparisonSkipped=row['baselineComparisonSkipped']))
candidate=[r for r in rows if r['candidate']]
browser_path=module/'output/line-height-quantization-browser-r1/receipt.json'
browser=json.loads(browser_path.read_text());browser_joins=[]
for prior in browser['rows']:
 new=next(row for row in r['rows'] if row['name']==prior['name'] and row['frame']==1)
 old_png=browser_path.parent/(prior['name']+'.chrome.png');new_png=Path(new['prefix']+'.chrome.png')
 baseline_equal=[f['baseline'] for f in prior['observed']['fragments']]==[f['baseline'] for f in new['browserMetrics']['fragments']]
 box_equal=prior['observed']['box']==new['browserMetrics']['box']
 image_equal=rgba_equal(old_png,new_png)
 assert baseline_equal and box_equal and image_equal,prior['name']
 browser_joins.append(dict(name=prior['name'],prior=str(old_png),current=str(new_png),priorSha256=sha(old_png),currentSha256=sha(new_png),completeRgbaEqual=image_equal,boxAndBaselineEqual=True,priorMarkerUsable=prior['markerUsable']))
control_joins=[]
for old_name,new_name in [('direct-16-multiline','old-direct-16-control'),('wrapper-16-multiline','old-symmetric-16-control')]:
 old_dir=module/'output/unitless-line-height-r1'/old_name;new_dir=root/new_name
 assert sha(old_dir/'scene.riv')==sha(new_dir/'scene.riv')
 frames=[]
 for frame in range(8):
  comparisons={}
  for suffix in ['chrome.png','native.png']:
   old=old_dir/'render'/f'frame-{frame}.{suffix}';new=new_dir/'render'/f'frame-{frame}.{suffix}'
   equal=rgba_equal(old,new);assert equal,(old_name,frame,suffix)
   comparisons[suffix]=dict(prior=str(old),current=str(new),priorSha256=sha(old),currentSha256=sha(new),completeRgbaEqual=True)
  frames.append(dict(frame=frame,images=comparisons))
 control_joins.append(dict(prior=old_name,current=new_name,exactRivSha256=sha(new_dir/'scene.riv'),frames=frames))
summary=dict(cases=len(cases),candidateCases=sum(c['request']['mode']=='chromium-leading' for c in cases),frames=len(rows),candidateFrames=len(candidate),candidateMetricPass=sum(not c['metricFailures'] for c in candidate),candidateBaselineSkipped=sum(c['baselineComparisonSkipped'] is not None for c in candidate),candidateMaxBoxResidual=max(abs(v) for c in candidate for v in c['boxResidual'].values()),candidateMaxBaselineResidual=max(abs(v) for c in candidate for v in c['baselineResidual']),originalPixelPass=sum(c['pixelPass'] for c in r['counts']),supplementalInkPass=sum(c['paintRegionPass'] for c in ink['counts']),independentBrowserJoins=len(browser_joins),exactHistoricalControlFrames=sum(len(c['frames']) for c in control_joins))
receipt=dict(scope='Private source-derived line-height arithmetic; precise residuals and independent browser/historical-control joins. No public text or general paint qualification.',summary=summary,rows=rows,browserJoins=browser_joins,controlJoins=control_joins,bindings=[dict(path=str(p),sha256=sha(p)) for p in [Path(__file__).resolve(),native_path,ink_path,browser_path,root/'cases.json']])
(root/'analysis-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(summary,indent=2))
