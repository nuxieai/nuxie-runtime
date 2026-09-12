"""Group already-proven distinct pairs by scene, retaining every pixel unscaled."""
import collections
import hashlib
import json
import pathlib
import sys
from PIL import Image,ImageDraw
out=pathlib.Path(sys.argv[1]).resolve()
read=lambda p:json.loads(pathlib.Path(p).read_text())
sha=lambda p:hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
coverage=read(out/'visual-coverage.json');render=read(out/'render/receipt.json')
assert coverage['renderReceiptSha256']==sha(out/'render/receipt.json')
direct={(r['name'],r['frame']) for r in coverage['coverage'] if r['method']=='direct'}
by_name=collections.defaultdict(list)
for row in render['rows']:
    if (row['name'],row['frame']) in direct:by_name[row['name']].append(row)
groups=[rows for rows in by_name.values() if len(rows)>1]
singles=[rows[0] for rows in by_name.values() if len(rows)==1]
groups += [singles[index:index+4] for index in range(0,len(singles),4)]
sheets=[];placements=[]
for index,group in enumerate(groups):
    sheet=Image.new('RGBA',(max(row['width']*2+30 for row in group),sum(row['height']+40 for row in group)),'#e8e8e8')
    draw=ImageDraw.Draw(sheet);y=0
    for row in group:
        failures=';'.join(row['geometryFailures']+row['pixelFailures']) or 'gates pass'
        draw.text((8,y+4),f"{row['name']} {row['width']}x{row['height']}; Chrome left / native right",fill='black')
        draw.text((8,y+15),failures,fill='black')
        for role,x in [('chrome',10),('native',row['width']+20)]:
            source=pathlib.Path(row['prefix']+'.'+role+'.png');assert sha(source)==row[role+'Sha256']
            original=Image.open(source).convert('RGBA');sheet.paste(original,(x,y+30))
            assert sheet.crop((x,y+30,x+original.width,y+30+original.height)).tobytes()==original.tobytes()
            placements.append(dict(name=row['name'],frame=row['frame'],role=role,sheet=index,sourceSha256=sha(source),completeRgbaExact=True))
        y+=row['height']+40
    dest=out/f'review-sheet-{index}.png';sheet.save(dest)
    sheets.append(dict(path=str(dest),sha256=sha(dest),pairs=[dict(name=row['name'],frame=row['frame'])for row in group]))
result=dict(scope='Unscaled complete pairs for subsequent direct inspection; no inspection claim made by this script.',driverSha256=sha(__file__),coverageSha256=sha(out/'visual-coverage.json'),directPairs=len(direct),sheets=sheets,placements=placements)
(out/'review-sheets.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(dict(directPairs=len(direct),sheets=len(sheets))))
