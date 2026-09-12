#!/usr/bin/env python3
"""Source/asset-bound full image review; no render or tolerance changes."""
import base64,copy,hashlib,html,json,math,re,struct,sys
from pathlib import Path
from PIL import Image,ImageDraw
BASE=Path(__file__).resolve().parents[1];ROOT=BASE/'output/ordinary-image-r2';OUT=ROOT/'visual'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def bind(p):return {'path':str(p),'sha256':sha(p)}
def finite(value):
 if isinstance(value,float):assert math.isfinite(value)
 elif isinstance(value,dict):
  for v in value.values():finite(v)
 elif isinstance(value,list):
  for v in value:finite(v)
def read_wire(data):
 # Exact bounded writer format, including ordinary bool clip property196.
 pos=7;assert data[:7]==b'RIVE\x07\x03\x00'
 def vu():
  nonlocal pos
  result=shift=0
  while True:
   b=data[pos];pos+=1;result|=(b&127)<<shift
   if not b&128:return result
   shift+=7;assert shift<35
 fields=[]
 while (key:=vu()):fields.append(key)
 kinds={}
 for offset in range(0,len(fields),4):
  word=struct.unpack_from('<I',data,pos)[0];pos+=4
  for i,key in enumerate(fields[offset:offset+4]):kinds[key]=(word>>(2*i))&3
 records=[]
 while pos<len(data):
  kind=vu();values={}
  while (key:=vu()):
   if key==196:values[key]=bool(data[pos]);pos+=1;continue
   t=kinds[key]
   if t==0:values[key]=vu()
   elif t==1:
    size=vu();values[key]=data[pos:pos+size];pos+=size
   else:values[key]=struct.unpack_from('<f' if t==2 else '<I',data,pos)[0];pos+=4
  records.append((kind,values))
 assert pos==len(data);return records

def gallery(coverage,native):
 def link(p):return html.escape(__import__('os').path.relpath(p,OUT),quote=True)
 def state(ok):return 'pass' if ok else 'fail'
 cards=[]
 for sheet in coverage['sheets']:
  for member in sheet['members']:
   row=next(x for x in native['rows'] if x['name']==member['case'] and x['frame']==member['frame']);states=[state(not row['metricFailures']),state(not row['pixelFailures']),state(row['imagePresence']['passed'])]
   badges=' '.join(f'<span class="badge {s}">{label}: {s.upper()}</span>' for label,s in zip(['Geometry','Pixels','Presence'],states))
   links=' · '.join(f'<a href="{link(row["prefix"]+suffix)}">{label}</a>' for label,suffix in [('Chrome PNG','.chrome.png'),('Native PNG','.native.png'),('Diff PNG','.diff.png')])+f' · <a href="{link(sheet["path"])}">Reviewed sheet</a>'
   details=html.escape(json.dumps({k:row[k] for k in ['browserMetrics','nativeOwner','nativeImage','metricFailures','pixelMetrics','pixelFailures','imagePresence','rivSha256','assetSha256','sourceHtmlSha256']},indent=2))
   figures=''.join(f'<figure><figcaption>{label}</figcaption><a href="{link(row["prefix"]+suffix)}"><img loading="lazy" src="{link(row["prefix"]+suffix)}" alt="{label}"></a></figure>' for label,suffix in [('Chrome153','.chrome.png'),('Immutable native','.native.png')])
   cards.append(f'<article data-name="{row["name"]}" data-geometry="{states[0]}" data-pixels="{states[1]}" data-presence="{states[2]}"><h2>{row["name"]} · frame{row["frame"]} · {row["width"]}×{row["height"]}</h2><p>{badges}</p><p>{links}</p><div class="pair">{figures}</div><details><summary>Measured results and identities</summary><pre>{details}</pre></details></article>')
 sheets=' '.join(f'<a href="{link(s["path"])}">{Path(s["path"]).stem}</a>' for s in coverage['sheets'])
 counts=coverage['counts'];summary=f"{counts['sourceCases']} cases · {counts['totalPairs']} original/clone frames · {counts['directPairs']} distinct pairs · {counts['exactTransferredPairs']} exact same-source transfers. Geometry {counts['geometryPass']}/{counts['totalPairs']}; pixels {counts['pixelOnlyPass']}/{counts['totalPairs']}; image presence {counts['presencePass']}/{counts['totalPairs']}."
 page='''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Ordinary image · visual review</title><style>body{font:15px system-ui,sans-serif;margin:0;background:#f3f4f5;color:#17212b}main{max-width:1600px;margin:auto;padding:24px}h1{margin:0;font-size:28px}h2{font-size:17px}a{color:#125bc7}p{line-height:1.55}.controls{position:sticky;top:0;background:#f3f4f5;z-index:1;display:flex;gap:16px;flex-wrap:wrap;padding:12px 0;border-bottom:1px solid #ccd1d6}input,select{font:inherit;padding:6px}article{background:white;border:1px solid #d8dde2;border-radius:8px;margin:20px 0;padding:16px}.pair{display:grid;grid-template-columns:1fr 1fr;gap:12px}figure{margin:0;overflow:auto}figcaption{font-size:13px;color:#54616d;margin-bottom:6px}img{display:block;max-width:100%;height:auto;border:1px solid #d9dfe4;box-sizing:border-box}.badge{display:inline-block;border-radius:4px;padding:3px 7px;font-size:12px;margin:2px}.pass{background:#dcf3e6;color:#165930}.fail{background:#fbe2e2;color:#8a1e25}pre{overflow:auto;font-size:12px;max-height:600px;background:#f6f7f8;padding:12px}details{margin-top:12px}.sheets a{display:inline-block;margin:6px}.hidden{display:none!important}</style><main><h1>Ordinary embedded image visual review</h1><p>'''+summary+'''</p><p>Private immutable-runtime evidence. Three direct controls retain intrinsic-axis geometry failures; ratio compositions repair those tested cases. Fractional sampling/alignment paint failures remain. Presence measures non-background image content separately from geometry and pixel error. Previews fit the panel; full PNG links and reviewed sheets preserve original resolution.</p><p><a href="coverage.json">Coverage and hashes</a> · <a href="review-receipt.json">Review receipt</a> · <a href="../native-receipt.json">Native/Chrome receipt</a></p><details class="sheets"><summary>Reviewed full unscaled sheets</summary>'''+sheets+'''</details><div class="controls"><label>Case <input id="search" placeholder="Filter case name"></label><label>Result <select id="result"><option value="all">All distinct pairs</option><option value="geometry">Geometry failures</option><option value="pixels">Pixel failures</option><option value="presence">Presence failures</option></select></label><span id="count"></span></div>'''+''.join(cards)+'''</main><script>const search=document.querySelector('#search'),result=document.querySelector('#result'),cards=[...document.querySelectorAll('article')];function apply(){let n=0;for(const card of cards){const show=card.dataset.name.includes(search.value.toLowerCase())&&(result.value==='all'||card.dataset[result.value]==='fail');card.classList.toggle('hidden',!show);n+=show}document.querySelector('#count').textContent=`${n} / ${cards.length} distinct pairs`;}search.addEventListener('input',apply);result.addEventListener('change',apply);apply();</script></html>'''
 (OUT/'gallery.html').write_text(page)

native=json.loads((ROOT/'native-receipt.json').read_text());assert len(native['rows'])==248 and not native['errors'];finite(native)
if '--gallery-only' in sys.argv:gallery(json.loads((OUT/'coverage.json').read_text()),native);sys.exit(0)
OUT.mkdir(exist_ok=False)
cases=json.loads((BASE/'validation/ordinary-image-cases.json').read_text());assert cases==json.loads((ROOT/'cases.json').read_text()) and len(cases)==31
generation=json.loads((ROOT/'generation-receipt.json').read_text());build=json.loads((ROOT/'build-receipt.json').read_text())
for b in native['bindings']+build['bindings']:
 assert sha(b.get('snapshot',b['path']))==b['sha256']
 # Use frozen snapshots if parent public work advances current production files.
 if 'snapshot' not in b:assert sha(b['path'])==b['sha256']
pairs=[];transfers=[];sources=[];composition=[]
for c in cases:
 d=ROOT/c['name'];req=c['request'];reference=d/'render/reference.html';saved_source=d/'render/reference-source.json';asset=d/c['asset'];scene=d/'scene.riv';observe=d/'image-observation/image-metrics.json';obs=json.loads(observe.read_text())['frames'];g=next(x for x in generation if x['name']==c['name'])
 assert json.loads((d/'request.json').read_text())==req and json.loads(saved_source.read_text())==c
 mime={'.png':'image/png','.jpg':'image/jpeg','.webp':'image/webp'}[asset.suffix];exact='<!doctype html><meta charset="utf-8"><style>html,body{margin:0;background:white}body{padding:20px;box-sizing:border-box}'+c['css']+'</style><img id="image" src="data:'+mime+';base64,'+base64.b64encode(asset.read_bytes()).decode()+'">';assert reference.read_text()==exact
 assert sha(asset)==g['assetSha256'] and sha(scene)==g['rivSha256'] and (d/'image-observation/scene.riv').read_bytes()==scene.read_bytes() and (d/'render/probe/scene.riv').read_bytes()==scene.read_bytes()
 asset_snapshot=next(b for b in build['bindings'] if b['path'].endswith('/fixtures/images/ordinary-r1/'+c['asset']));assert asset.read_bytes()==Path(asset_snapshot['snapshot']).read_bytes()
 with Image.open(asset) as im:assert im.size==(req['assetWidth'],req['assetHeight'])
 css=dict(part.split(':',1) for part in c['css'].split('{')[1].rstrip('}').split(';'));size=lambda v:'auto' if v['unit']=='auto' else str(v['value'])+{'px':'px','percent':'%'}[v['unit']]
 assert css['width']==size(req['width']) and css['height']==size(req['height']) and css['display']=='block' and css['background']=='#e9f0f6'
 assert css['object-fit']=={7:'fill',1:'contain',2:'cover',5:'none',6:'scale-down'}[req['fit']]
 assert [float(v.rstrip('%')) for v in css['object-position'].split()]==[(req[k]+1)*50 for k in ['alignmentX','alignmentY']]
 assert css['image-rendering']==('pixelated' if req['nearest'] else 'auto') and req['clip'] is True
 records=read_wire(scene.read_bytes());contents=[v[212] for kind,v in records if kind==106];assert contents==[asset.read_bytes()]
 images=[v for kind,v in records if kind==100];assert len(images)==1;image=images[0];assert image[206]==0 and image[974]==req['fit'] and image[975]==req['alignmentX'] and image[976]==req['alignmentY'] and image[380]==image[381]==0
 assert image.get(1076)==(2 if req['nearest'] else None)
 ratios=[v[524] for kind,v in records if 524 in v];assert ratios==([req['assetWidth']/req['assetHeight']] if req['aspectRatio'] else [])
 binding=[bind(f) for f in [d/'request.json',saved_source,reference,scene,asset,observe,d/'plan.json',d/'generation.log']]
 sources.append({'case':c['name'],'bindings':binding,'exactRequestCssAssetAndReference':True,'embeddedAssetBytesEqual':True,'sourceAssetSnapshot':asset_snapshot})
 composition.append({'case':c['name'],'recordCount':len(records),'imageFit':image[974],'imageAlignment':[image[975],image[976]],'samplerOverride':image.get(1076),'ordinaryAspectRatios':ratios,'assetByteLength':len(contents[0])})
 rows=[r for r in native['rows'] if r['name']==c['name']];assert [r['frame'] for r in rows]==list(range(8))
 for row in rows:
  k=row['frame'];o=obs[k];assert o['viewport']==[row['width'],row['height']] and o['instance']==('original' if row['instance']==0 else 'clone') and o['step']==row['step'] and o['images']==[row['nativeImage']]
  assert row['nativeImage']['fit']==req['fit'] and row['nativeImage']['originAlignment']==[0,0,req['alignmentX'],req['alignmentY']]
  assert row['browserMetrics']['naturalWidth']==req['assetWidth'] and row['browserMetrics']['naturalHeight']==req['assetHeight'] and row['browserMetrics']['complete']
  for field,file in [('rivSha256',scene),('assetSha256',asset),('sourceHtmlSha256',reference),('imageObservationSha256',observe),('geometrySha256',d/'render/probe'/row['geometry']),('streamSha256',d/'render/probe'/row['stream'])]:assert sha(file)==row[field]
  geom=json.loads((d/'render/probe'/row['geometry']).read_text());assert row['nativeOwner']==next(x for x in geom if x['objectId']==4)
  for suffix,field in [('chrome.png','chromeSha256'),('native.png','nativeSha256'),('diff.png','diffSha256')]:assert sha(row['prefix']+'.'+suffix)==row[field]
  # The baseline recording must carry the exact encoded source to actual replay.
  stream=(d/'render/probe'/row['stream']).read_text();decoded=re.findall(r'^decodeImage .* data=([0-9a-f]+)$',stream,re.M);assert decoded and all(bytes.fromhex(v)==asset.read_bytes() for v in decoded)
  if k<3:pairs.append(row);continue
  origin=rows[{3:0,4:0,5:1,6:2,7:0}[k]]
  for key in ['rivSha256','assetSha256','sourceHtmlSha256','browserMetrics','nativeOwner','metricFailures','pixelMetrics','pixelFailures','imagePresence','width','height']:assert row[key]==origin[key]
  a,b=copy.deepcopy(origin['nativeImage']),copy.deepcopy(row['nativeImage']);pid_a=a['participant'].pop('identity');pid_b=b['participant'].pop('identity');assert a==b
  assert (pid_a==pid_b)==(row['instance']==origin['instance'])
  proof=[]
  for suffix in ['chrome.png','native.png','diff.png']:
   src=Path(origin['prefix']+'.'+suffix);target=Path(row['prefix']+'.'+suffix);ai=Image.open(src).convert('RGBA');bi=Image.open(target).convert('RGBA');assert ai.size==bi.size and ai.tobytes()==bi.tobytes();proof.append({'source':bind(src),'target':bind(target),'completeRgbaEqual':True})
  transfers.append({'case':c['name'],'frame':k,'sourceFrame':origin['frame'],'sourceBindings':binding,'sameSourceAssetViewportAndMetrics':True,'participantIdentity':{'original':pid_a,'target':pid_b,'sameInstance':row['instance']==origin['instance'],'otherImageMetricsIdentical':True},'images':proof})
sheets=[]
for frame in range(3):
 selected=[r for r in pairs if r['frame']==frame]
 for group in range(math.ceil(len(selected)/4)):
  rows=selected[group*4:(group+1)*4];w=rows[0]['width'];h=rows[0]['height'];sheet=Image.new('RGBA',(2*w+24,len(rows)*(h+42)+26),'#ddd');draw=ImageDraw.Draw(sheet);draw.text((8,6),'Chrome153 left / immutable native right - complete unscaled RGBA',fill='black');members=[]
  for i,row in enumerate(rows):
   y=26+i*(h+42);gp='PASS' if not row['metricFailures'] else 'FAIL';pp='PASS' if not row['pixelFailures'] else 'FAIL';ip='PASS' if row['imagePresence']['passed'] else 'FAIL';draw.text((8,y),f"{row['name']} frame{frame} | geometry {gp} / pixels {pp} / presence {ip}",fill='black')
   for side,suffix in enumerate(['chrome.png','native.png']):
    image=Image.open(row['prefix']+'.'+suffix).convert('RGBA');assert image.size==(w,h);sheet.paste(image,(8+side*(w+8),y+20))
   members.append({'case':row['name'],'frame':frame,'chrome':bind(row['prefix']+'.chrome.png'),'native':bind(row['prefix']+'.native.png'),'diff':bind(row['prefix']+'.diff.png'),'metricFailures':row['metricFailures'],'pixelFailures':row['pixelFailures'],'presencePassed':row['imagePresence']['passed']})
  file=OUT/f'frame-{frame}-group-{group}.png';sheet.save(file);sheets.append({**bind(file),'members':members})
counts={'sourceCases':len(cases),'directPairs':len(pairs),'exactTransferredPairs':len(transfers),'totalPairs':len(native['rows']),'geometryPass':sum(not r['metricFailures'] for r in native['rows']),'pixelOnlyPass':sum(not r['pixelFailures'] for r in native['rows']),'presencePass':sum(r['imagePresence']['passed'] for r in native['rows']),'combinedPixelPresencePass':sum(not r['pixelFailures'] and r['imagePresence']['passed'] for r in native['rows'])}
coverage={'scope':'Private ordinary-image source/composition audit and full visual review. No public asset qualification.', 'counts':counts,'bindings':[bind(p) for p in [ROOT/'native-receipt.json',ROOT/'build-receipt.json',ROOT/'generation-receipt.json',BASE/'validation/ordinary-image-cases.json',Path(__file__)]],'sources':sources,'decodedComposition':composition,'sheets':sheets,'transfers':transfers,'reviewCompleted':False}
(OUT/'coverage.json').write_text(json.dumps(coverage,indent=2)+'\n');gallery(coverage,native);print(json.dumps(counts,indent=2))
