# Requires fonttools==4.61.1. Input is the pinned original font; see assets README.
from fontTools import subset
from fontTools.ttLib import TTFont
from pathlib import Path
import sys, hashlib
source = Path(sys.argv[1])
assert hashlib.sha256(source.read_bytes()).hexdigest() == "68a3fc98800b2a27b371f2fb79991daf3633bd89309d4ffaa6946fd587f375b5"
font=TTFont(source)
options=subset.Options(); options.name_IDs=['*']; options.name_legacy=True; options.name_languages=['*'];options.recalc_timestamp=False
sub=subset.Subsetter(options=options)
sub.populate(unicodes=list(range(0x530))+list(range(0x2000,0x3100))+list(range(0x31f0,0x3200))+list(range(0xfb00,0xfb07))+list(range(0xff00,0xfff0))+list(range(0x1b000,0x1b170)))
sub.subset(font)
for record in font['name'].names:
 if record.nameID in [1,2,3,4,6,16,17]:
  value={1:'Nuxie Japanese Fixture',2:'Regular',3:'NuxieJapaneseFixture-Regular-Subset-1',4:'Nuxie Japanese Fixture Regular',6:'NuxieJapaneseFixture-Regular',16:'Nuxie Japanese Fixture',17:'Regular'}[record.nameID]
  record.string=value.encode(record.getEncoding())
if 'CFF ' in font:
 cff=font['CFF '].cff;cff.fontNames=['NuxieJapaneseFixture-Regular'];cff.topDictIndex[0].FullName='Nuxie Japanese Fixture Regular';cff.topDictIndex[0].FamilyName='Nuxie Japanese Fixture'
font.save(Path(__file__).resolve().parents[1] / 'tests/assets/NuxieJapaneseFixture-Regular.otf')
