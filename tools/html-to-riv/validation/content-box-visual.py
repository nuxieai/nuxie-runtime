from pathlib import Path
import hashlib
import json
from PIL import Image, ImageDraw

module = Path(__file__).resolve().parent.parent
root = module / 'output/content-box-candidate-r2'
receipt = json.loads((root / 'render/receipt.json').read_text())
selected = [r for r in receipt['rows'] if r['frame'] < 3]
assert len(selected) == 36
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()
placements = []
for start in range(0, len(selected), 6):
    sheet = Image.new('RGBA', (1584, 1360), 'white')
    draw = ImageDraw.Draw(sheet)
    for offset, row in enumerate(selected[start:start+6]):
        y = offset * 224
        draw.text((8, y + 2), f"{row['name']} {row['width']}x{row['height']} | Chrome left, immutable native right", fill='black')
        for role, x in [('chrome', 8), ('native', 800)]:
            source = Path(row['prefix'] + '.' + role + '.png')
            image = Image.open(source).convert('RGBA')
            assert image.size == (row['width'], row['height'])
            assert sha(source) == row[role+'Sha256']
            sheet.paste(image, (x, y+20))
            assert sheet.crop((x,y+20,x+image.width,y+20+image.height)).tobytes() == image.tobytes()
            placements.append(dict(name=row['name'], frame=row['frame'], role=role, source=str(source), sha256=sha(source), sheet=start//6, xy=[x,y+20], size=list(image.size)))
    sheet.save(root / f'visual-{start//6}.png')
transfers=[]
for row in receipt['rows']:
    if row['frame'] < 3: continue
    first=next(r for r in selected if r['name']==row['name'] and r['width']==row['width'] and r['height']==row['height'])
    for role in ['chrome','native']:
        source=Path(row['prefix']+'.'+role+'.png'); target=Path(first['prefix']+'.'+role+'.png')
        a=Image.open(source).convert('RGBA');b=Image.open(target).convert('RGBA')
        assert a.size==b.size and a.tobytes()==b.tobytes()
        transfers.append(dict(name=row['name'], frame=row['frame'], fromFrame=first['frame'], role=role, sha256=sha(source), fromSha256=sha(target)))
(root/'visual-evidence.json').write_text(json.dumps(dict(directPairs=36,placements=placements,exactRepeatedTransfers=transfers,sheets=[dict(path=str(root/f'visual-{i}.png'),sha256=sha(root/f'visual-{i}.png'))for i in range(6)]),indent=2)+'\n')
