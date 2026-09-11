"""Audit direct ordinary-atomics sheets and exact raster image review transfers.
Usage: SCRIPT BACKEND_OUTPUT STATIC_REPLAY LIFECYCLE_REPLAY TOOLCHAIN
Inspection notes must already exist; this script never performs visual inspection.
"""
import hashlib,json,runpy,sys
from pathlib import Path
from PIL import Image,ImageDraw

def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def read(p):return json.loads(Path(p).read_text())
def audit(out,static,lifecycle,toolchain):
    proof=runpy.run_path(str(Path(__file__).with_name('audit-gradient-composition.py')))['audit'](static)
    assert proof==read(static/'composition-artifact-audit.json')
    receipt=read(out/'receipt.json');review=read(out/'ordinary-atomics-visual-inspection.json')
    assert receipt['mode']=='atomics' and receipt['status']=='passed'
    assert review['backendReceiptSha256']==sha(out/'receipt.json')
    assert receipt['rendererSha256']==sha(toolchain/'renderer-replay')==read(toolchain/'manifest.json')['files']['renderer-replay']['sha256']
    assert receipt['referenceSha256']==sha(lifecycle/'replay.json')
    refs={c['name']:c for c in read(lifecycle/'replay.json')['cases']}
    original={(c['name'],c['width']):c for c in read(static/'replay.json')['cases']}
    rows={c['name']:c for c in receipt['cases']};assert len(rows)==54
    direct={c['name']:c for c in review['direct']};assert len(direct)==39
    transferred=[]
    for name,row in rows.items():
        prior=refs[name];stem=name.split('-0-step')[0];old=original[(stem,row['width'])]
        assert row['mode']=='atomics' and row['backend']=='rust-metal-atomic' and not row['failures']
        assert row['streamSha256']==prior['streamSha256']
        assert all(prior[k]==old[k] for k in ('html','css','browserSha256','nativeSha256'))
        assert row['browserSha256']==sha(prior['prefix']+'.browser.png')
        assert row['nativeSha256']==sha(out/(name+'.atomic.png'))
        if name in direct:
            assert direct[name]['nativeSha256']==row['nativeSha256']
            assert direct[name]['browserSha256']==row['browserSha256']
            assert direct[name]['notes']
        if row['nativeSha256']==old['nativeSha256'] and row['browserSha256']==old['browserSha256']:
            assert sha(old['prefix']+'.native.png')==row['nativeSha256']
            assert sha(old['prefix']+'.browser.png')==row['browserSha256']
            transferred.append(dict(name=name,width=row['width'],sourceName=stem,nativeSha256=row['nativeSha256'],browserSha256=row['browserSha256']))
    for stem in {name.split('-0-step')[0] for name in direct}:
        image=Image.new('RGB',(2328,1050),'#ddd');draw=ImageDraw.Draw(image)
        for i,row in enumerate([r for r in receipt['cases'] if r['name'].split('-0-step')[0]==stem]):
            name=row['name']
            for j,(label,p) in enumerate([('Chrome',Path(refs[name]['prefix']+'.browser.png')),('Ordinary atomics',out/(name+'.atomic.png')),('Difference',out/(name+'.diff.png'))]):
                draw.text((j*776+4,i*350+4),f'{label} / {row["width"]}',fill='black');image.paste(Image.open(p).convert('RGB'),(j*776+4,i*350+24))
        sheet=out/'review-sheets'/(stem+'.png')
        assert Image.open(sheet).convert('RGB').tobytes()==image.tobytes()
        assert all(d['sheetSha256']==sha(sheet) for d in direct.values() if d['name'].split('-0-step')[0]==stem)
    covered=set(direct)|{t['name'] for t in transferred};assert covered==set(rows)
    result=dict(status='complete-review-with-residual',scope='All54 ordinary atomics image pairs reviewed; viewport-edge opacity residual remains open',reviewed=54,direct=39,exactRasterTransfers=18,overlap=3,remaining=[],transferred=transferred,residuals=review['residuals'],evidence=[dict(path=str(p),sha256=sha(p)) for p in [out/'receipt.json',out/'ordinary-atomics-visual-inspection.json',static/'composition-artifact-audit.json',static/'composition-visual-inspection.json',static/'replay.json',lifecycle/'replay.json',toolchain/'manifest.json']])
    assert len(transferred)==18
    (out/'ordinary-atomics-visual-coverage.json').write_text(json.dumps(result,indent=2)+'\n')
    return result
if __name__=='__main__':
    result=audit(*[Path(p).resolve() for p in sys.argv[1:]])
    print(json.dumps({k:result[k] for k in ('status','reviewed','direct','exactRasterTransfers','overlap')}))
