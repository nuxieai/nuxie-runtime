#!/usr/bin/env python3
"""Bound public-image candidate gallery/review; never renders or changes gates."""
import argparse, collections, hashlib, html, json, math, os, re, struct
from pathlib import Path
from PIL import Image, ImageDraw
BASE = Path(__file__).resolve().parents[1]
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def bind(p): return {'path':str(Path(p).resolve()), 'sha256':sha(p)}
def read(p): return json.loads(Path(p).read_text())
def save(p, value): Path(p).write_text(json.dumps(value, indent=2)+'\n')
def checked(binding):
    p=Path(binding.get('snapshot',binding['path'])); assert sha(p)==binding['sha256'],p
    return binding

def finite(value):
    if isinstance(value,float): assert math.isfinite(value)
    elif isinstance(value,dict):
        for v in value.values(): finite(v)
    elif isinstance(value,list):
        for v in value: finite(v)

def records(data):
    pos=7; assert data[:7]==b'RIVE\x07\x03\x00'
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
    result=[]
    while pos<len(data):
        kind=vu();values={}
        while (key:=vu()):
            if key==196:values[key]=bool(data[pos]);pos+=1;continue
            t=kinds[key]
            if t==0:values[key]=vu()
            elif t==1:
                size=vu();values[key]=data[pos:pos+size];pos+=size
            else:values[key]=struct.unpack_from('<f' if t==2 else '<I',data,pos)[0];pos+=4
        result.append((kind,values))
    assert pos==len(data);return result

def rgba_equal_on_white(a,b):
    with Image.open(a) as raw: ai=raw.convert('RGBA')
    with Image.open(b) as raw: bi=raw.convert('RGBA')
    size=(max(ai.width,bi.width),max(ai.height,bi.height))
    aa=Image.new('RGBA',size,'white');bb=Image.new('RGBA',size,'white')
    aa.paste(ai,(0,0));bb.paste(bi,(0,0))
    return aa.tobytes()==bb.tobytes(),list(size),list(ai.size),list(bi.size)

def visual_metadata(row):
    """Separate only recorded background-chain border boxes from strict data.

    These rectangles are per-frame observation provenance, not transferred
    measurements. Content rectangles, paint colors, chain identities and every
    other browser/presence field remain part of the strict comparison.
    """
    data=json.loads(json.dumps({key:row[key] for key in ['browserMetrics','imagePresence']}))
    boxes={}
    def separate(chain,path):
        if not isinstance(chain,list):return
        for index,entry in enumerate(chain):
            if not isinstance(entry,dict):continue
            box=entry.get('borderBox')
            if not isinstance(box,dict) or set(box)!={'x','y','width','height'}:continue
            if not all(type(value) in (int,float) and math.isfinite(value) for value in box.values()):continue
            key=(*path,index,'borderBox');boxes[key]=box
            entry['borderBox']={'perFrameBackgroundBoxMetadata':True}
    for id,value in data['browserMetrics'].get('images',{}).items():
        separate(value.get('backgroundChain'),('browserMetrics','images',id,'backgroundChain'))
    for id,value in data['imagePresence'].items():
        separate(value.get('background',{}).get('chain'),('imagePresence',id,'background','chain'))
    return data,boxes

def matching_visual_metadata(origin,row):
    if any(row[key]!=origin[key] for key in ['name','rivSha256','mapSha256','requestSha256','htmlSha256','nativeBoxes']):return False,[]
    left,left_boxes=visual_metadata(origin);right,right_boxes=visual_metadata(row)
    if left!=right or left_boxes.keys()!=right_boxes.keys():return False,[]
    differences=[{'path':list(path),'sourceValue':value,'targetValue':right_boxes[path],'transferred':False}
                 for path,value in left_boxes.items() if value!=right_boxes[path]]
    return True,differences

def sheets(rows,out,prefix,columns=('chrome','native')):
    result=[]
    groups=collections.defaultdict(list)
    for row in rows:groups[(row['width'],row['height'])].append(row)
    for (w,h),group in groups.items():
        rows_per_sheet=max(1,min(4,(1500-28)//(h+45)))
        for start in range(0,len(group),rows_per_sheet):
            selected=group[start:start+rows_per_sheet];canvas=Image.new('RGBA',(len(columns)*(w+8)+8,len(selected)*(h+45)+28),'#ddd');draw=ImageDraw.Draw(canvas)
            draw.text((8,6),' / '.join(columns)+' — complete source PNG pixels, no scaling',fill='black');members=[]
            for i,row in enumerate(selected):
                y=28+i*(h+45)
                label=f"{row['name']} f{row['frame']} | geometry {'FAIL' if row['metricFailures'] else 'PASS'} pixels {'FAIL' if row['pixelFailures'] else 'PASS'}"
                draw.text((8,y),label,fill='black')
                images={}
                for side,column in enumerate(columns):
                    p=Path(row.get(column+'Path',row['prefix']+'.'+column+'.png'))
                    with Image.open(p) as raw: im=raw.convert('RGBA')
                    assert im.size==(w,h),(p,im.size,(w,h));canvas.paste(im,(8+side*(w+8),y+21));images[column]=bind(p)
                members.append({'case':row['name'],'frame':row['frame'],'images':images,'pixelFailures':row['pixelFailures'],'metricFailures':row['metricFailures']})
            file=out/f'{prefix}-{len(result):02}.png';canvas.save(file)
            result.append({**bind(file),'members':members})
    return result

def gallery(coverage,out,reviewed=False,supplementary=()):
    def rel(p):return html.escape(os.path.relpath(p,out),quote=True)
    cards=[]
    for sheet in coverage['sheets']:
        for m in sheet['members']:
            row=next(r for r in coverage['representatives'] if r['name']==m['case'] and r['frame']==m['frame'])
            failure=bool(row['pixelFailures']);images=''.join(f'<figure><figcaption>{label}</figcaption><a href="{rel(p["path"])}"><img loading="lazy" src="{rel(p["path"])}"></a></figure>' for label,p in m['images'].items())
            targets=[t for t in coverage['transfers'] if t['case']==row['name'] and t['sourceFrame']==row['frame']]
            detail={'measured':row,'visualTransfers':targets}
            cards.append(f'<article data-name="{row["name"]}" data-fail="{str(failure).lower()}" data-geometry="{str(bool(row['metricFailures'])).lower()}" data-presence="{str(not all(v['passed'] for v in row['imagePresence'].values())).lower()}"><h2>{row["name"]} · frame {row["frame"]} · {row["width"]}×{row["height"]}</h2><p class="{"fail" if failure else "pass"}">Geometry: {"FAIL" if row["metricFailures"] else "PASS"} · Pixels: {"FAIL" if failure else "PASS"} · Presence: {"PASS" if all(v["passed"] for v in row["imagePresence"].values()) else "FAIL"}</p><p><a href="{rel(row["prefix"]+".diff.png")}">Diff</a> · <a href="{rel(sheet["path"])}">Unscaled sheet</a> · {len(targets)} exact visual transfers</p><div class="pair">{images}</div><details><summary>Measurements, failures, exact transfer proofs</summary><pre>{html.escape(json.dumps(detail,indent=2))}</pre></details></article>')
    failures=''.join(f'<tr><td>{r["name"]}</td><td>{r["frame"]}</td><td>{html.escape(", ".join(r["pixelFailures"]))}</td><td><a href="{rel(r["prefix"]+".chrome.png")}">Chrome</a> / <a href="{rel(r["prefix"]+".native.png")}">native</a> / <a href="{rel(r["prefix"]+".diff.png")}">diff</a></td></tr>' for r in coverage['pixelFailureRows'])
    comparisons=''.join(f'<p><a href="{rel(sheet["path"])}">Chrome / previous native / current native</a></p>' for sheet in coverage['beforeAfterSheets'])
    supplementary_links=''.join(f'<p><a href="{rel(b["path"])}">{html.escape(Path(b["path"]).name)}</a>: {html.escape(b["observation"])}</p>' for b in supplementary)
    counts=coverage['counts'];text=f"{counts['sourceCases']} cases; {counts['frames']} frames; {counts['representatives']} directly reviewed pairs; {counts['visualTransfers']} exact visual transfers. Geometry {counts['geometryPass']}/{counts['frames']}, pixels {counts['pixelPass']}/{counts['frames']}, presence {counts['presencePass']}/{counts['frames']}." if reviewed else f"Prepared coverage: {counts}. Direct inspection pending."
    page='''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Public image corpus · visual review</title><style>body{font:15px system-ui;margin:0;background:#f3f4f5;color:#17212b}main{max-width:1600px;margin:auto;padding:24px}h1{font-size:28px}h2{font-size:17px}a{color:#125bc7}p{line-height:1.55}article{background:white;border:1px solid #ccd1d6;border-radius:8px;margin:20px 0;padding:16px}.pair{display:grid;grid-template-columns:1fr 1fr;gap:12px}figure{margin:0;overflow:auto}figcaption{margin-bottom:6px}img{display:block;max-width:100%;height:auto}pre{overflow:auto;font-size:12px;max-height:600px;background:#f6f7f8;padding:12px}.fail{color:#a02020}.pass{color:#165930}td,th{padding:6px;border-bottom:1px solid #ddd;text-align:left}.controls{position:sticky;top:0;background:#f3f4f5;padding:12px 0}input,select{font:inherit;padding:6px}.hidden{display:none}</style><main><h1>Public image compiler corpus</h1><p>'''+html.escape(text)+'''</p><p>This is visual evidence for frozen emitted scenes, not a claim of complete public API qualification. Fractional failures are retained. Exact white-canvas extensions transfer complete visible RGBA evidence only; each frame retains its own geometry, pixel and presence gates and viewport. Recorded background-chain borderBox metadata may differ; those exact per-frame values are recorded in transfer proofs and are not transferred. Gallery previews fit the panel; linked PNGs and sheets are unscaled.</p><p><a href="coverage.json">Coverage</a> · <a href="review-receipt.json">Review receipt</a> · <a href="../combined-receipt.json">Combined native receipt</a></p><details><summary>All retained pixel failures</summary><table><tr><th>Case</th><th>Frame</th><th>Failure</th><th>Evidence</th></tr>'''+failures+'''</table></details><details><summary>Optional exact-source before and after</summary>'''+comparisons+'''</details><details><summary>Supplementary inspected evidence</summary>'''+supplementary_links+'''</details><div class="controls"><input id="search" placeholder="Filter case"><select id="result"><option value="all">All representatives</option><option value="failed">Pixel failures</option><option value="geometry">Geometry failures</option><option value="presence">Presence failures</option></select><span id="count"></span></div>'''+''.join(cards)+'''</main><script>const cards=[...document.querySelectorAll('article')],search=document.querySelector('#search'),result=document.querySelector('#result');function filter(){let n=0;for(const c of cards){const show=c.dataset.name.includes(search.value.toLowerCase())&&(result.value==='all'||(result.value==='failed'?c.dataset.fail:c.dataset[result.value])==='true');c.classList.toggle('hidden',!show);n+=show}document.querySelector('#count').textContent=` ${n}/${cards.length} representatives`}search.oninput=result.onchange=filter;filter();</script></html>'''
    (out/'gallery.html').write_text(page)

parser=argparse.ArgumentParser();parser.add_argument('--root',type=Path,default=BASE/'output/public-image-layout-r2');parser.add_argument('--complete',type=Path);parser.add_argument('--label',default='visual',help='Fresh review directory name inside the native root');parser.add_argument('--before-native',type=Path,help='Optional prior native receipt for exact-source before/after comparisons');args=parser.parse_args();assert re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9_-]*',args.label),'Review label must be one directory name';root=args.root.resolve();out=root/args.label
if args.complete:
    coverage=read(out/'coverage.json');notes=read(args.complete)
    completion_helper=out/'completion-helper.py';assert not completion_helper.exists();completion_helper.write_bytes(Path(__file__).read_bytes())
    for b in coverage['bindings']+coverage['artifacts']:checked(b)
    expected=coverage['sheets']+coverage['beforeAfterSheets'];assert len(notes['inspectedSheets'])==len(expected)
    assert {x['path'] for x in notes['inspectedSheets']}=={x['path'] for x in expected}
    for b in notes['inspectedSheets']:checked(b);assert b['observation'].strip()
    for b in expected:checked(b)
    for b in notes.get('supplementaryInspection',[]):checked(b);assert b['observation'].strip()
    assert notes['reviewCompleted'] is True
    receipt={'scope':coverage['scope'],'visualReviewCompleted':True,'counts':coverage['counts'],'bindings':[bind(out/'coverage.json'),bind(args.complete),bind(completion_helper)],'inspectedSheets':notes['inspectedSheets'],'observations':notes['observations'],'limitations':notes['limitations'],'supplementaryInspection':notes.get('supplementaryInspection',[])}
    save(out/'review-receipt.json',receipt);gallery(coverage,out,True,notes.get('supplementaryInspection',[]));print(json.dumps(receipt['counts'],indent=2));raise SystemExit

native=read(root/'combined-receipt.json');finite(native);assert not native['errors'] and native['rows']
build=read(root/'build-receipt.json');compiled={r['name']:r for r in read(root/'compile-receipt.json')};cases={c['name']:c for c in read(root/'cases.json')}
source_bindings_path=root/'frozen/source-bindings.json'
source_bindings=read(source_bindings_path)['files']
authoring_path=root/'authoring-bindings.json'
authoring_bindings=read(authoring_path) if authoring_path.exists() else []
for b in native['bindings']+build.get('bindings',source_bindings)+authoring_bindings:checked(b)
extra_bindings=[bind(source_bindings_path)]
if authoring_bindings:extra_bindings.append(bind(authoring_path))
if 'bindingsSha256' in build:assert sha(source_bindings_path)==build['bindingsSha256']
if 'summarySha256' in build:
    summary=Path(build['build'])/'summary.json';assert sha(summary)==build['summarySha256'];extra_bindings.append(bind(summary))
assert sha(root/'frozen/html-to-riv')==build['compilerSha256']
assert all(r['matchesExpectation'] for r in compiled.values())
assert len(native['rows'])==8*sum(r['compiled'] for r in compiled.values())
asset_sources={Path(b['path']).resolve():Path(b.get('snapshot',b['path'])) for b in source_bindings+authoring_bindings}
assert {r['name'] for r in native['rows']}=={name for name,c in compiled.items() if c['compiled']}
assert not out.exists(),'Preserve existing output; choose a new output root for a new review';out.mkdir()
helper_snapshot=out/'preparation-helper.py';helper_snapshot.write_bytes(Path(__file__).read_bytes())
rows=native['rows'];sources=[];file_bindings={};representatives=[];transfers=[]
def record_file(p):
    p=Path(p);file_bindings[str(p)]=bind(p);return file_bindings[str(p)]
for name in dict.fromkeys(r['name'] for r in rows):
    case=cases[name];current=root/name;req=read(current/'request.json');scene=(current/'scene.riv').read_bytes();mapping=read(current/'scene.map.json');group=[r for r in rows if r['name']==name]
    assert [r['frame'] for r in group]==list(range(8));assert all(req[k]==case['input'][k] for k in case['input'])
    expected_assets={key:bytes(req['assets'][key]['bytes']) for key in sorted(req['assets'])};unique=list(dict.fromkeys(expected_assets.values()))
    asset_bindings={}
    for key,relpath in case['assetFiles'].items():
        asset_source=asset_sources[(BASE/relpath).resolve()]
        assert expected_assets[key]==asset_source.read_bytes()
        asset_bindings[key]=record_file(asset_source)
    assert sha(current/'request.json')==compiled[name]['requestSha256']
    assert sha(current/'scene.riv')==compiled[name]['rivSha256']
    assert sha(current/'scene.map.json')==compiled[name]['mapSha256']
    wire=records(scene);assert [v[212] for kind,v in wire if kind==106]==unique
    source={'case':name,'request':record_file(current/'request.json'),'scene':record_file(current/'scene.riv'),'map':record_file(current/'scene.map.json'),'assetSha256':{key:hashlib.sha256(value).hexdigest() for key,value in expected_assets.items()},'assetSources':asset_bindings,'uniqueEmbeddedAssets':len(unique),'exactEmbeddedBytes':True,'rows':[]}
    own=[]
    for row in group:
        prefix=Path(row['prefix']);render=prefix.parent;original=render.parent;probe=Path(row['probeDirectory']);e=row['evidence'];checked({'path':e['receipt'],'sha256':e['receiptSha256']});checked({'path':e['result'],'sha256':e['resultSha256']})
        prior=read(e['result']);assert all(row[k]==v for k,v in prior.items()),(name,row['frame'],'row transfer differs')
        assert read(original/'request.json')==req and (original/'scene.riv').read_bytes()==scene and read(original/'scene.map.json')==mapping
        reset=original.parent/'frozen/inputs/src/reset.css';base=f'http://html-to-riv.invalid/{render.name}/{name}/'
        reference=f'<!doctype html><meta charset="utf-8"><base href="{base}"><style>{reset.read_text()}\n{req["css"]}</style>{req["html"]}'
        assert (render/'reference.html').read_text()==reference
        for field,path in [('rivSha256',original/'scene.riv'),('requestSha256',original/'request.json'),('mapSha256',original/'scene.map.json'),('htmlSha256',render/'reference.html'),('streamSha256',probe/row['stream']),('geometrySha256',probe/row['geometry']),('chromeSha256',str(prefix)+'.chrome.png'),('nativeSha256',str(prefix)+'.native.png')]:assert sha(path)==row[field],(name,row['frame'],field);record_file(path)
        record_file(str(prefix)+'.diff.png');assert (probe/'scene.riv').read_bytes()==scene
        frames=read(probe/'frames.json')['frames'];frame=frames[row['frame']];assert all(row[k]==v for k,v in frame.items());assert row['instance']==row['frame']//4 and row['step']==row['frame']%4
        assert [row['width'],row['height']]==[[240,240],[390,320],[768,560],[240,240]][row['step']]
        geometry=read(probe/row['geometry'])
        for id,box in row['nativeBoxes'].items():assert box==next(v for v in geometry if v['objectId']==next(v['object_id'] for v in mapping if v.get('id')==id))
        assert set(row['browserMetrics']['rectangles'])==set(case['observeIds'])
        for im in row['browserMetrics']['images'].values():assert im['complete'] and im['naturalWidth']>0 and im['naturalHeight']>0 and im['src'] in expected_assets
        decoded=[bytes.fromhex(v) for v in re.findall(r'^decodeImage .* data=([0-9a-f]+)$',(probe/row['stream']).read_text(),re.M)]
        assert set(decoded)==set(unique),(name,'stream assets')
        for suffix in ['chrome','native']:
            with Image.open(str(prefix)+'.'+suffix+'.png') as image:assert image.size==(row['width'],row['height'])
        source['rows'].append({'frame':row['frame'],'evidence':e,'probeFrames':record_file(probe/'frames.json'),'browserAssets':record_file(render/'browser-assets.json'),'browserRequests':record_file(render/'browser-requests.json')})
        matched=None
        for origin in own:
            compatible,metadata_differences=matching_visual_metadata(origin,row)
            if not compatible:continue
            proof=[]
            for suffix in ['chrome','native']:
                a=origin['prefix']+'.'+suffix+'.png';b=row['prefix']+'.'+suffix+'.png';equal,size,aa,bb=rgba_equal_on_white(a,b)
                if not equal:break
                proof.append({'source':record_file(a),'target':record_file(b),'sourceSize':aa,'targetSize':bb,'comparisonCanvas':size,'completeRgbaEqualAfterOpaqueWhiteExtension':True})
            if len(proof)==2:matched=(origin,proof,metadata_differences);break
        if matched:
            origin,proof,metadata_differences=matched;transfers.append({'case':name,'frame':row['frame'],'instance':row['instance'],'sourceFrame':origin['frame'],'sourceInstance':origin['instance'],'sameRequestSceneMapReferenceAndMeasurements':not metadata_differences,'sameRequestSceneMapReferenceNativeBoxesAndOtherMetadata':True,'backgroundBoxMetadataDifferences':metadata_differences,'backgroundBoxMetadataTransferred':False,'targetFrameEvidence':row['evidence'],'metricFailures':row['metricFailures'],'pixelFailures':row['pixelFailures'],'pixelMetrics':row['pixelMetrics'],'imagePresence':row['imagePresence'],'images':proof})
        else:own.append(row);representatives.append(row)
    sources.append(source)
before_after=[];before_rows=[]
if args.before_native:
    old=read(args.before_native);finite(old);extra_bindings.append(bind(args.before_native))
    for b in old.get('bindings',[]):checked(b)
    old_rows={(r['name'],r['frame']):r for r in old['rows']}
    for name in dict.fromkeys(r['name'] for r in rows):
        current=[r for r in rows if r['name']==name]
        previous=[old_rows.get((name,r['frame'])) for r in current]
        if any(r is None for r in previous):continue
        if all(a['rivSha256']==b['rivSha256'] for a,b in zip(previous,current)):continue
        for a,b in zip(previous,current):
            assert all(a[k]==b[k] for k in ['requestSha256','browserMetrics','width','height','instance','step'])
            for side in ['chrome','native']:
                path=a['prefix']+'.'+side+'.png';assert sha(path)==a[side+'Sha256'];record_file(path)
            assert rgba_equal_on_white(a['prefix']+'.chrome.png',b['prefix']+'.chrome.png')[0]
            before_after.append({'case':name,'frame':b['frame'],'previous':a,'current':b,'previousGeometryPass':not a['metricFailures'],'currentGeometryPass':not b['metricFailures']})
        a,b=previous[0],current[0];witness=dict(b);witness['oldNativePath']=a['prefix']+'.native.png';witness['newNativePath']=b['prefix']+'.native.png'
        before_rows.append(witness)
counts={'sourceCases':len(sources),'frames':len(rows),'representatives':len(representatives),'visualTransfers':len(transfers),'backgroundBoxDifferenceTransfers':sum(bool(t['backgroundBoxMetadataDifferences']) for t in transfers),'geometryPass':sum(not r['metricFailures'] for r in rows),'pixelPass':sum(not r['pixelFailures'] for r in rows),'presencePass':sum(all(v['passed'] for v in r['imagePresence'].values()) for r in rows),'freshFrames':native['summary']['freshFrames'],'unchangedSceneTransfers':native['summary']['transferredFrames'],'beforeAfterFrames':len(before_after)}
coverage={'scope':'Visual evidence for frozen public-image candidate scenes; full API qualification is separate. Complete opaque-white RGBA extension proves only visible pixel transfer, not offscreen content or shared numeric gates. Only background-chain borderBox metadata can differ: exact source/target values remain recorded and are not transferred; all other browser, native-box and presence metadata must match.','counts':counts,'bindings':[bind(p) for p in [root/'combined-receipt.json',root/'build-receipt.json',root/'compile-receipt.json',root/'cases.json',helper_snapshot]]+extra_bindings,'sources':sources,'artifacts':list(file_bindings.values()),'representatives':representatives,'transfers':transfers,'sheets':sheets(representatives,out,'review-sheet'),'beforeAfterSheets':sheets(before_rows,out,'before-after',('chrome','oldNative','newNative')),'beforeAfter':before_after,'pixelFailureRows':[r for r in rows if r['pixelFailures']],'reviewCompleted':False}
save(out/'coverage.json',coverage);gallery(coverage,out);print(json.dumps(counts,indent=2));print('sheets',len(coverage['sheets']),'before/after sheets',len(coverage['beforeAfterSheets']))
