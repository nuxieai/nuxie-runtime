#!/usr/bin/env python3
"""Full unscaled source-bound image pairs; exact same-source clone/resize transfer."""
import hashlib,json,math,html,sys
from pathlib import Path
from PIL import Image,ImageDraw
BASE=Path(__file__).resolve().parents[1]
ROOT=BASE/'output/quantized-line-height-r1'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def bind(p):return {'path':str(p),'sha256':sha(p)}
def write_gallery(coverage):
 native=json.loads((ROOT/'native-receipt.json').read_text());ink=json.loads((ROOT/'ink-region-receipt.json').read_text())
 def href(p):return html.escape(__import__('os').path.relpath(p,ROOT/'visual'),quote=True)
 def state(ok):return 'pass' if ok else 'fail'
 cards=[]
 for sheet in coverage['sheets']:
  for member in sheet['members']:
   row=next(x for x in native['rows'] if x['name']==member['case'] and x['frame']==member['frame'])
   supplemental=next(x for x in ink['rows'] if x['name']==row['name'] and x['frame']==row['frame'])
   statuses=[state(not row['metricFailures']),state(not row['pixelFailures'] and row['ink']['passed']),state(not supplemental['failures'] and supplemental['inkPassed'])]
   badges=' '.join(f'<span class="badge {status}">{label}: {status.upper()}</span>' for label,status in zip(['Metrics','Original pixels','Ink region'],statuses))
   details=html.escape(json.dumps({'metricFailures':row['metricFailures'],'originalPixelFailures':row['pixelFailures'],'supplementalInkRegionFailures':supplemental['failures'],'baselineComparisonSkipped':row['baselineComparisonSkipped'],'browserBox':row['browserMetrics']['box'],'nativeOwner':row['nativeOwner'],'nativeBaselines':row['nativeBaselines'],'browserFragments':row['browserMetrics']['fragments'],'originalPixelMetrics':row['pixelMetrics'],'supplementalPixelMetrics':supplemental['metrics']},indent=2))
   links=' · '.join(f'<a href="{href(path)}">{label}</a>' for label,path in [('Chrome PNG',member['chrome']['path']),('Native PNG',member['native']['path']),('Diff PNG',row['prefix']+'.diff.png'),('Reviewed sheet',sheet['path'])])
   cards.append(f'<article data-name="{html.escape(row["name"])}" data-metrics="{statuses[0]}" data-pixels="{statuses[1]}" data-ink="{statuses[2]}"><h2>{html.escape(row["name"])} · frame {row["frame"]} · {row["width"]}×{row["height"]}</h2><p>{badges}</p><p>{links}</p><div class="pair"><figure><figcaption>Chrome153</figcaption><a href="{href(member["chrome"]["path"])}"><img loading="lazy" src="{href(member["chrome"]["path"])}" alt="Chrome reference"></a></figure><figure><figcaption>Immutable native</figcaption><a href="{href(member["native"]["path"])}"><img loading="lazy" src="{href(member["native"]["path"])}" alt="Native render"></a></figure></div><details><summary>Measured results</summary><pre>{details}</pre></details></article>')
 sheets=' '.join(f'<a href="{href(s["path"])}">{Path(s["path"]).stem}</a>' for s in coverage['sheets'])
 output='<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Quantized line-height · visual review</title><style>body{font:15px system-ui,sans-serif;margin:0;background:#f3f4f5;color:#17212b}main{max-width:1600px;margin:auto;padding:24px}h1{margin:0;font-size:28px}h2{font-size:17px}a{color:#125bc7}p{line-height:1.55}.controls{position:sticky;top:0;background:#f3f4f5;z-index:1;display:flex;gap:16px;flex-wrap:wrap;padding:12px 0;border-bottom:1px solid #ccd1d6}input,select{font:inherit;padding:6px}article{background:white;border:1px solid #d8dde2;border-radius:8px;margin:20px 0;padding:16px}.pair{display:grid;grid-template-columns:1fr 1fr;gap:12px}figure{margin:0;overflow:auto}figcaption{font-size:13px;color:#54616d;margin-bottom:6px}img{display:block;max-width:100%;height:auto;border:1px solid #d9dfe4;box-sizing:border-box}.badge{display:inline-block;border-radius:4px;padding:3px 7px;font-size:12px;margin:2px}.pass{background:#dcf3e6;color:#165930}.fail{background:#fbe2e2;color:#8a1e25}pre{overflow:auto;font-size:12px;max-height:600px;background:#f6f7f8;padding:12px}details{margin-top:12px}.sheets a{display:inline-block;margin:6px}.hidden{display:none!important}</style><main><h1>Quantized line-height visual review</h1><p>45 scenes · 360 original/clone frames · 135 distinct pairs reviewed on 27 full unscaled sheets · 225 exact same-source transfers. All 344 candidate metric checks pass; 16 old-control checks fail. Candidate paint remains partial: 286/344 original pixel passes and 16/344 supplemental ink-region passes. This is private candidate evidence.</p><p>Zero-height metric checks cover the owner rectangle only. Its DOM pixel region is empty; the separate ink-region gate measures observed painted bounds. All original thresholds and failures are retained. Previews fit the available panel; use full PNG links or reviewed sheets for original resolution.</p><p><a href="coverage.json">Coverage and hashes</a> · <a href="review-receipt.json">Review receipt</a> · <a href="../native-receipt.json">Native/Chrome observations</a> · <a href="../ink-region-receipt.json">Ink-region observations</a></p><details class="sheets"><summary>27 reviewed unscaled sheets</summary>'+sheets+'</details><div class="controls"><label>Case <input id="search" placeholder="Filter by name"></label><label>Result <select id="result"><option value="all">All distinct pairs</option><option value="metrics">Metric failures</option><option value="pixels">Original pixel failures</option><option value="ink">Ink-region failures</option></select></label><span id="count"></span></div>'+''.join(cards)+"</main><script>const search=document.querySelector('#search'),result=document.querySelector('#result'),cards=[...document.querySelectorAll('article')];function apply(){let n=0;for(const card of cards){const visible=card.dataset.name.includes(search.value.toLowerCase())&&(result.value==='all'||card.dataset[result.value]==='fail');card.classList.toggle('hidden',!visible);n+=visible}document.querySelector('#count').textContent=`${n} / ${cards.length} distinct pairs`;}search.addEventListener('input',apply);result.addEventListener('change',apply);apply();</script></html>"
 (ROOT/'visual/gallery.html').write_text(output)
if '--gallery-only' in sys.argv:
 write_gallery(json.loads((ROOT/'visual/coverage.json').read_text()));sys.exit(0)
r=json.loads((ROOT/'native-receipt.json').read_text());cases=json.loads((BASE/'validation/quantized-line-height-cases.json').read_text());ink=json.loads((ROOT/'ink-region-receipt.json').read_text());out=ROOT/'visual';out.mkdir(exist_ok=False)
assert len(cases)==len(r['counts'])==45 and len(r['rows'])==360
for binding in r['bindings']+ink['bindings']:assert sha(binding['path'])==binding['sha256'],binding['path']
pairs=[];transfers=[];source_bindings=[]
for c in cases:
 d=ROOT/c['name'];source=d/'render/reference-source.json';request=d/'request.json';reference=d/'render/reference.html'
 assert json.loads(source.read_text())==c and json.loads(request.read_text())==c['request']
 scene=d/'scene.riv';text_file=d/'text-observation/textmetrics.json';tm=json.loads(text_file.read_text())['frames'];rows=[x for x in r['rows'] if x['name']==c['name']]
 assert len(rows)==8 and [x['frame'] for x in rows]==list(range(8))
 proof=[bind(f) for f in [request,source,reference,scene,text_file,d/'plan.json',d/'generation.log']]
 source_bindings.append({'case':c['name'],'exactRequestAndReferenceFixture':True,'bindings':proof})
 for row in rows:
  assert row['rivSha256']==sha(scene) and row['textObservationSha256']==sha(text_file)
  assert row['nativeText']==next(t for t in tm[row['frame']]['texts'] if t['objectId']==6)
  probe=d/'render/probe';assert sha(probe/row['geometry'])==row['geometrySha256'];assert sha(probe/row['stream'])==row['streamSha256']
  geom=json.loads((probe/row['geometry']).read_text());assert row['nativeOwner']==next(x for x in geom if x['objectId']==4)
  for suffix,field in [('chrome.png','chromeSha256'),('native.png','nativeSha256')]:assert sha(row['prefix']+'.'+suffix)==row[field]
  if row['frame']<3:pairs.append(row);continue
  expected={3:0,4:0,5:1,6:2,7:0}[row['frame']];original=rows[expected]
  assert all(row[k]==original[k] for k in ['rivSha256','nativeText','nativeOwner','browserMetrics','nativeBaselines','metricFailures','pixelFailures','pixelMetrics','ink','width','height','baselineComparisonSkipped'])
  images=[]
  for suffix in ['chrome.png','native.png']:
   a=Path(original['prefix']+'.'+suffix);b=Path(row['prefix']+'.'+suffix);ai=Image.open(a).convert('RGBA');bi=Image.open(b).convert('RGBA');assert ai.size==bi.size and ai.tobytes()==bi.tobytes()
   images.append({'source':bind(a),'target':bind(b),'completeRgbaEqual':True})
  transfers.append({'case':row['name'],'frame':row['frame'],'sourceFrame':expected,'exactSameSource':proof,'sceneNativeBrowserMetricsAndGatesEqual':True,'images':images})
sheets=[]
for frame in range(3):
 selected=[x for x in pairs if x['frame']==frame]
 for chunk in range(math.ceil(len(selected)/5)):
  rows=selected[chunk*5:(chunk+1)*5];w=rows[0]['width'];h=rows[0]['height'];sheet=Image.new('RGBA',(w*2+24,len(rows)*(h+42)+26),'#ddd');draw=ImageDraw.Draw(sheet);draw.text((8,6),'Chrome153 left / immutable native right - full unscaled RGBA',fill='black');members=[]
  for i,row in enumerate(rows):
   y=26+i*(h+42);supp=next(x for x in ink['rows'] if x['name']==row['name'] and x['frame']==frame);mp='PASS' if not row['metricFailures'] else 'FAIL';pp='PASS' if not row['pixelFailures'] and row['ink']['passed'] else 'FAIL';ip='PASS' if not supp['failures'] and supp['inkPassed'] else 'FAIL';draw.text((8,y),f"{row['name']} | frame{frame} metrics {mp} / pixels {pp} / ink-region {ip}",fill='black')
   for j,suffix in enumerate(['chrome.png','native.png']):
    f=Path(row['prefix']+'.'+suffix);im=Image.open(f).convert('RGBA');assert im.size==(w,h);sheet.paste(im,(8+j*(w+8),y+20))
   members.append({'case':row['name'],'frame':frame,'sourceBindings':next(x for x in source_bindings if x['case']==row['name']),'chrome':bind(row['prefix']+'.chrome.png'),'native':bind(row['prefix']+'.native.png'),'metricFailures':row['metricFailures'],'pixelFailures':row['pixelFailures'],'supplementalInkRegionFailures':supp['failures']})
  f=out/f'frame-{frame}-group-{chunk}.png';sheet.save(f);sheets.append({**bind(f),'members':members})
coverage={'scope':'Source-bound full unscaled Chrome/native visual review. Original metric and pixel failures preserved; supplemental ink-region failures shown independently. No public qualification.', 'bindings':[bind(f) for f in [ROOT/'native-receipt.json',ROOT/'ink-region-receipt.json',ROOT/'build-receipt.json',BASE/'validation/quantized-line-height-cases.json',Path(__file__)]],'directPairs':len(pairs),'exactTransferredPairs':len(transfers),'totalPairs':len(r['rows']),'sourceBindings':source_bindings,'sheets':sheets,'transfers':transfers,'reviewCompleted':False}
(out/'coverage.json').write_text(json.dumps(coverage,indent=2)+'\n');print(json.dumps({k:coverage[k] for k in ['directPairs','exactTransferredPairs','totalPairs']}))

write_gallery(coverage)
