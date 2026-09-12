"""Assemble current native evidence and unscaled, exact-pixel visual coverage.

The original 34-case run stays intact. Only five strengthened child cases replace
their parent-only counterparts in the current corpus; the originals remain bound
as separate historical controls. Image identity transfers visual inspection only,
never geometry, source semantics or runtime qualification.
"""
from pathlib import Path
import hashlib
import html
import json
from PIL import Image, ImageDraw

module=Path(__file__).resolve().parent.parent
root=module/'output/public-content-box-r2'
original_file=module/'output/public-content-box-r1/render/receipt.json'
boundary_file=root/'boundary-render/receipt.json'
read=lambda p:json.loads(Path(p).read_text())
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
bind=lambda p:dict(path=str(p),sha256=sha(p))
original=read(original_file);boundary=read(boundary_file)
cases=read(module/'validation/public-content-box-cases.json')
boundary_names={c['name']for c in cases if c['classification']=='numeric-boundary'}
assert len(cases)==34 and len(boundary_names)==5
rows=[];artifacts=[]
for case in cases:
    receipt=boundary if case['name'] in boundary_names else original
    selected=[r for r in receipt['rows']if r['name']==case['name']]
    assert len(selected)==8
    for row in selected:
        request=read(Path(row['prefix']).parent/'request.json')
        assert request==dict(html=case['html'],css=case['css'],width=390,height=160)
        assert sha(Path(row['prefix']).parent/'request.json')==row['requestSha256']
    rows+=selected
    artifacts.append(next(a for a in receipt['artifacts']if a['name']==case['name']))
for r in [original,boundary]:
    assert r['browser']=='153.0.8010.12' and r['effectiveMode']=='RasterOrdering'
    for key in ['probe','renderer']:
        assert r['toolHashes'][key]==original['toolHashes'][key]
    assert r['pixelGateSha256']==original['pixelGateSha256'] and r['resetSha256']==original['resetSha256']
assert len(rows)==272
def totals(selected):
    return dict(frames=len(selected),geometryPass=sum(not r['geometryFailures']for r in selected),pixelPass=sum(not r['pixelFailures']for r in selected),clearPass=sum(c['samePixels']for r in selected for c in r['clearChecks']))
historical_rows=[r for r in original['rows']if r['name']in boundary_names]
assert totals(rows)==dict(frames=272,geometryPass=264,pixelPass=266,clearPass=544)
assert totals(historical_rows)==dict(frames=40,geometryPass=40,pixelPass=40,clearPass=80)
assert totals(original['rows']+boundary['rows'])==dict(frames=312,geometryPass=304,pixelPass=306,clearPass=624)
aggregate=dict(scope='Current 34-case public content-box corpus, preserving strengthened geometry failures and fractional paint failures. Geometry and paint evidence are independent of the visual identity transfers.',sourceReceipts=[bind(original_file),bind(boundary_file)],cases=bind(module/'validation/public-content-box-cases.json'),rows=rows,artifacts=artifacts,
    **totals(rows),status='partial-with-known-failures',currentCases=34,reusedExactSourceOriginalCases=29,strengthenedChildCases=5,
    historicalParentOnly=dict(receipt=bind(original_file),names=sorted(boundary_names),cases=5,**totals(historical_rows)),
    allRetainedEvidence=dict(scope='Original 34 cases plus five strengthened additions; includes superseded parent-only controls.',cases=39,**totals(original['rows']+boundary['rows'])))
(root/'native-aggregate.json').write_text(json.dumps(aggregate,indent=2)+'\n')

history_file=module/'output/content-box-candidate-r2/render/receipt.json'
history=read(history_file)
history_review=module/'validation/content-box-review.md'
history_receipt=module/'validation/content-box-candidate-receipt.json'
prior=read(history_receipt)
assert prior['directVisualPairs']==36
assert any(b['path']==str(history_review) and b['sha256']==sha(history_review)for b in prior['sourceFiles'])
assert prior['nativeRun']['sha256']==sha(history_file)
def images(row):
    pair=[]
    for role in ['chrome','native']:
        source=Path(row['prefix']+'.'+role+'.png')
        assert sha(source)==row[role+'Sha256']
        image=Image.open(source).convert('RGBA')
        assert image.size==(row['width'],row['height'])
        pair.append((source,image))
    return pair
def identity(row):
    return tuple((im.size,hashlib.sha256(im.tobytes()).hexdigest())for _,im in images(row))
def ref(row):
    return dict(name=row['name'],frame=row['frame'],prefix=row['prefix'],width=row['width'],height=row['height'])
known={}
for row in history['rows']:
    if row['frame']<3: known.setdefault(identity(row),('historical',row))
direct=[];transfers=[]
for row in rows:
    key=identity(row)
    if key in known:
        kind,source=known[key]
        for (_,a),(_,b) in zip(images(row),images(source)):
            assert a.size==b.size and a.tobytes()==b.tobytes()
        transfers.append(dict(target=ref(row),source=ref(source),sourceKind=kind,completePairRgbaIdentical=True))
    else:
        known[key]=('current-direct',row);direct.append(row)
historical_transfers=[]
for row in historical_rows:
    kind,source=known[identity(row)]
    for (_,a),(_,b) in zip(images(row),images(source)):
        assert a.size==b.size and a.tobytes()==b.tobytes()
    historical_transfers.append(dict(target=ref(row),source=ref(source),sourceKind=kind,completePairRgbaIdentical=True))
visual=root/'visual';visual.mkdir(exist_ok=True)
placements=[];sheets=[]
for start in range(0,len(direct),6):
    group=direct[start:start+6]
    sheet=Image.new('RGBA',(1584,len(group)*224),'white');draw=ImageDraw.Draw(sheet)
    for index,row in enumerate(group):
        y=index*224
        label=f"{row['name']} {row['width']}x{row['height']} | Chrome left; native right"
        draw.text((8,y+2),label,fill='black')
        for role,(source,im),x in zip(['chrome','native'],images(row),[8,800]):
            sheet.paste(im,(x,y+20))
            assert sheet.crop((x,y+20,x+im.width,y+20+im.height)).tobytes()==im.tobytes()
            placements.append(dict(pair=ref(row),role=role,source=bind(source),xy=[x,y+20],size=list(im.size),sheet=start//6))
    output=visual/f'sheet-{start//6}.png'
    if output.exists():
        prior_image=Image.open(output).convert('RGBA')
        assert prior_image.size==sheet.size and prior_image.tobytes()==sheet.tobytes(),f'Previously reviewed sheet changed: {output}'
    else: sheet.save(output)
    sheets.append(bind(output))
evidence=dict(scope='Complete current-frame and retained parent-only RGBA coverage; transferred inspection does not transfer source semantics or geometry qualification.',nativeAggregate=bind(root/'native-aggregate.json'),historicalReview=bind(history_review),historicalReceipt=bind(history_receipt),historicalFrames=bind(history_file),directPairs=[ref(r)for r in direct],transfers=transfers,historicalParentOnlyTransfers=historical_transfers,placements=placements,sheets=sheets)
(root/'visual-evidence.json').write_text(json.dumps(evidence,indent=2)+'\n')
cells=[]
for row in rows:
    if row['frame']>2:continue
    pair=[]
    for role in ['chrome','native']:
        relative=Path(row['prefix']+'.'+role+'.png').relative_to(module)
        pair.append(f'<figure><figcaption>{role}</figcaption><img src="../../{relative}" alt="{role}"></figure>')
    errors='; '.join(row['geometryFailures']+row['pixelFailures']) or 'Geometry and pixel gates pass'
    cells.append(f'<section><h2>{html.escape(row["name"])} · {row["width"]}×{row["height"]}</h2><p>{html.escape(errors)}</p><div>{"".join(pair)}</div></section>')
(root/'gallery.html').write_text('<!doctype html><meta charset="utf-8"><title>Content-box / immutable Rive</title><style>body{font:15px system-ui;margin:24px}section{margin:32px 0}section div{display:flex;gap:16px}figure{margin:0}img{max-width:46vw;border:1px solid #aaa}</style><h1>Content-box compiler: Chrome and immutable Rive</h1><p>Known fractional paint and large-size descendant geometry failures remain. Original/clone repeats are recorded in the evidence receipt.</p>'+''.join(cells))
print(json.dumps(dict(frames=len(rows),geometryPass=aggregate['geometryPass'],pixelPass=aggregate['pixelPass'],clearPass=aggregate['clearPass'],directPairs=len(direct),historicalTransfers=sum(t['sourceKind']=='historical'for t in transfers),currentTransfers=sum(t['sourceKind']=='current-direct'for t in transfers),sheets=len(sheets))))
