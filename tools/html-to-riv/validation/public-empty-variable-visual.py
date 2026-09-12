"""Prepare complete unscaled pairs and verify exact white-canvas review transfers."""
import hashlib
import json
import pathlib
import sys
from PIL import Image, ImageDraw

out = pathlib.Path(sys.argv[1]).resolve()
receipt_path = out/'render/receipt.json'
receipt = json.loads(receipt_path.read_text())
def sha(path): return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()
def image(row, role):
    path = pathlib.Path(row['prefix']+'.'+role+'.png')
    assert sha(path) == row[role+'Sha256']
    return Image.open(path).convert('RGBA')
def same_white_canvas(a,b):
    size = (max(a.width,b.width), max(a.height,b.height))
    canvases=[]
    for source in (a,b):
        canvas=Image.new('RGBA',size,'white');canvas.paste(source,(0,0));canvases.append(canvas)
    return canvases[0].tobytes()==canvases[1].tobytes()

direct=[];coverage=[]
for row in receipt['rows']:
    for reference in direct:
        if row['name'] != reference['name']: continue
        assert all(row[key]==reference[key] for key in ['requestSha256','rivSha256','sourceMapSha256'])
        if all(same_white_canvas(image(row,role),image(reference,role)) for role in ['chrome','native']):
            coverage.append(dict(name=row['name'],frame=row['frame'],method='complete-rgba-white-canvas',referenceFrame=reference['frame']))
            break
    else:
        direct.append(row)
        coverage.append(dict(name=row['name'],frame=row['frame'],method='direct'))

large=[[row] for row in direct if row['width']>240]
small=[row for row in direct if row['width']<=240]
groups=large+[small[index:index+4] for index in range(0,len(small),4)]
sheets=[];placements=[]
for index,group in enumerate(groups):
    sheet=Image.new('RGBA',(max(row['width']*2+30 for row in group),sum(row['height']+40 for row in group)),'#e8e8e8')
    draw=ImageDraw.Draw(sheet);y=0
    for row in group:
        draw.text((8,y+6),f"{row['name']} {row['width']}x{row['height']}; Chrome left / native right",fill='black')
        for role,x in [('chrome',10),('native',row['width']+20)]:
            original=image(row,role);sheet.paste(original,(x,y+30))
            assert sheet.crop((x,y+30,x+original.width,y+30+original.height)).tobytes()==original.tobytes()
            placements.append(dict(name=row['name'],frame=row['frame'],role=role,sheet=index,position=[x,y+30],completeRgbaExact=True))
        y+=row['height']+40
    path=out/f'visual-{index}.png';sheet.save(path)
    sheets.append(dict(path=str(path),sha256=sha(path),pairs=[dict(name=row['name'],frame=row['frame']) for row in group]))

result=dict(scope='Full-frame source images prepared for direct visual review; review completion is recorded separately. Transfers require complete decoded RGBA equality after explicit white-canvas extension, including all cropped-away regions.',renderReceiptSha256=sha(receipt_path),driverSha256=sha(__file__),directPairs=len(direct),transferredPairs=len(coverage)-len(direct),coverage=coverage,sheets=sheets,placements=placements)
(out/'visual-coverage.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(dict(directPairs=len(direct),transferredPairs=len(coverage)-len(direct),sheets=len(sheets))))
