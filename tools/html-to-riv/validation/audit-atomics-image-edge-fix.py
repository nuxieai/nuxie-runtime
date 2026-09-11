"""Audit corrected atomic composition image reviews, preserving the failing source.
Usage: SCRIPT PREVIOUS CURRENT STATIC LIFECYCLE OLD_TOOLCHAIN NEW_TOOLCHAIN
"""
from pathlib import Path
import hashlib,json,runpy,sys
from PIL import Image,ImageDraw

def read(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def audit(previous,current,static,lifecycle,old_toolchain,new_toolchain):
    scripts=Path(__file__).parent
    old_review=runpy.run_path(str(scripts/'audit-ordinary-atomics-visual-review.py'))['audit'](previous,static,lifecycle,old_toolchain)
    assert old_review['reviewed']==54 and old_review['status']=='complete-review-with-residual'
    old=read(previous/'receipt.json');new=read(current/'receipt.json');direct=read(current/'ordinary-atomics-visual-inspection.json')
    assert new['mode']==old['mode']=='atomics' and new['status']=='passed'
    assert new['referenceSha256']==old['referenceSha256']==sha(lifecycle/'replay.json')
    assert new['lifecycleSha256']==old['lifecycleSha256']
    assert new['rendererSha256']==sha(new_toolchain/'renderer-replay')==read(new_toolchain/'manifest.json')['files']['renderer-replay']['sha256']
    assert direct['backendReceiptSha256']==sha(current/'receipt.json') and not direct['residuals']
    before={r['name']:r for r in old['cases']};refs={r['name']:r for r in read(lifecycle/'replay.json')['cases']};inspected={r['name']:r for r in direct['direct']};assert len(inspected)==3
    transfers=[];seen=set()
    for row in new['cases']:
        name=row['name'];prior=before[name];assert name not in seen;seen.add(name)
        assert row['backend']=='rust-metal-atomic' and row['mode']=='atomics' and not row['failures']
        for field in ['width','frame','streamSha256','browserSha256']:assert row[field]==prior[field]
        assert sha(current/(name+'.atomic.png'))==row['nativeSha256']
        assert sha(Path(refs[name]['prefix']+'.browser.png'))==row['browserSha256']
        if name in inspected:
            assert 'opacity-overlap' in name
            for field in ['nativeSha256','browserSha256']:assert inspected[name][field]==row[field]
            assert row['nativeSha256']==refs[name]['nativeSha256']
            with Image.open(current/(name+'.atomic.png')) as im:
                point=(row['width']//2,319) if row['width']==240 else (0,250)
                assert im.convert('RGBA').getpixel(point)==(255,255,255,255)
        else:
            assert 'opacity-overlap' not in name
            assert row['nativeSha256']==prior['nativeSha256']==sha(previous/(name+'.atomic.png'))
            transfers.append(dict(name=name,width=row['width'],nativeSha256=row['nativeSha256'],browserSha256=row['browserSha256']))
    assert len(seen)==54 and len(transfers)==51
    expected=Image.new('RGB',(2328,1050),'#ddd');draw=ImageDraw.Draw(expected)
    for i,row in enumerate([r for r in new['cases'] if r['name'] in inspected]):
        for j,(label,p) in enumerate([('Chrome',Path(refs[row['name']]['prefix']+'.browser.png')),('Ordinary atomics',current/(row['name']+'.atomic.png')),('Difference',current/(row['name']+'.diff.png'))]):
            draw.text((j*776+4,i*350+4),f'{label} / {row["width"]}',fill='black');expected.paste(Image.open(p).convert('RGB'),(j*776+4,i*350+24))
    sheet=current/'review-sheets/linear-gradient-composition-opacity-overlap.png'
    assert Image.open(sheet).convert('RGB').tobytes()==expected.tobytes()
    assert all(r['sheetSha256']==sha(sheet) for r in inspected.values())
    paths=[previous/'ordinary-atomics-visual-coverage.json',previous/'receipt.json',current/'receipt.json',current/'ordinary-atomics-visual-inspection.json',lifecycle/'replay.json',new_toolchain/'manifest.json']
    result=dict(status='complete',scope='Ordinary atomics composition visual coverage; three viewport-edge failures corrected and inspected',total=54,reviewed=54,direct=3,exactTransfers=51,remaining=[],residualFailures=[],transfers=transfers,evidence=[dict(path=str(p),sha256=sha(p)) for p in paths])
    (current/'ordinary-atomics-visual-coverage.json').write_text(json.dumps(result,indent=2)+'\n');return result
if __name__=='__main__':
    result=audit(*[Path(p).resolve() for p in sys.argv[1:]])
    print(json.dumps({k:result[k] for k in ['status','reviewed','direct','exactTransfers']}))
