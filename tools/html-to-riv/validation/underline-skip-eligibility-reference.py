#!/usr/bin/env python3
"""Recreate Chromium 153 skip-ink eligibility reference and /tmp Rust proposal.
Downloads only pinned upstream source; does not modify compiler/runtime source.
"""
import base64
import urllib.request
from pathlib import Path
chromium = "https://chromium.googlesource.com/chromium/src/+/153.0.8010.12/"
icu = "https://chromium.googlesource.com/chromium/deps/icu/+/8cc91d9b6ab9991802fd208ee03a69714fd0251c/"
sources = {
 "underline-chromium-LICENSE":chromium+"LICENSE",
 "underline-character-153.cc": chromium+"third_party/blink/renderer/platform/text/character.cc",
 "underline-character-data-153.h": chromium+"third_party/blink/renderer/platform/text/character_property_data.h",
 "underline-character-generator-153.cc": chromium+"third_party/blink/renderer/platform/text/character_property_data_generator.cc",
 **{"underline-"+name:icu+"source/data/unidata/"+name for name in ("ppucd.txt", "emoji-sequences.txt", "emoji-zwj-sequences.txt")},
}
for name, url in sources.items():
 Path("/tmp/"+name).write_bytes(base64.b64decode(urllib.request.urlopen(url+"?format=TEXT").read()))
import re
values=bytearray(0x110000)
block=0
def props(parts,base):
 for p in parts:
  if p=='EPres':base|=1
  if p=='ExtPict':base|=2
  if p=='-EPres':base&=~1
  if p=='-ExtPict':base&=~2
 return base
for line in open('/tmp/underline-ppucd.txt'):
 p=line.rstrip().split(';')
 if p[0] not in ['defaults','block','cp','unassigned']:continue
 r=p[1].split('..');lo=int(r[0],16);hi=int(r[-1],16)
 if p[0]=='defaults':base=0
 elif p[0]=='block':base=0;block=props(p[2:],0)
 elif p[0]=='cp':base=block
 else:base=0
 val=props(p[2:],base)
 values[lo:hi+1]=bytes([val])*(hi-lo+1)
import re,json,hashlib
from pathlib import Path
header=Path('/tmp/underline-character-data-153.h').read_text()
excluded=set([0x2f,0x5c,0x5f])
for kind in ['Array','Ranges']:
 body=header.split('kIsCjkIdeographOrSymbol'+kind+' =')[1].split('});')[0]
 body=re.sub(r'//[^\n]*','',body)
 numbers=[int(x,16) for x in re.findall(r'0x[\dA-Fa-f]+',body)]
 if kind=='Array':excluded.update(numbers)
 else:
  for a,b in zip(numbers[::2],numbers[1::2]):excluded.update(range(a,b+1))
properties=values
excluded.update(i for i,b in enumerate(properties) if b&1)
for name in ['emoji-sequences.txt','emoji-zwj-sequences.txt']:
 for line in Path('/tmp/underline-'+name).read_text().splitlines():
  line=line.split('#')[0].strip()
  if not line:continue
  cps,kind,*_=line.split(';')
  if kind.strip() not in ['RGI_Emoji_ZWJ_Sequence','RGI_Emoji_Modifier_Sequence']:continue
  excluded.update(int(c,16) for c in cps.split() if properties[int(c,16)]&2)
for a,b in [(0x1100,0x11ff),(0x3130,0x318f),(0xac00,0xd7af),(0xa960,0xa97f),(0xd7b0,0xd7ff),(0x10080,0x100ff)]:excluded.update(range(a,b+1))
intervals=[]
for n in sorted(excluded):
 if intervals and n==intervals[-1][1]+1:intervals[-1][1]=n
 else:intervals.append([n,n])
assert not any(0xd800<=n<=0xdfff for n in excluded)
checks=[]
for lo,hi in intervals:
 for cp in [lo-1,lo,hi,hi+1]:
  if 0<=cp<=0x10ffff and not 0xd800<=cp<=0xdfff:checks.append({'codepoint':f'{cp:04X}','eligible':cp not in excluded})
checks=list({x['codepoint']:x for x in checks}.values())
checks.sort(key=lambda x:int(x['codepoint'],16))
sources={name:hashlib.sha256(Path('/tmp/'+name).read_bytes()).hexdigest() for name in ['underline-character-153.cc','underline-character-data-153.h','underline-character-generator-153.cc','underline-ppucd.txt','underline-emoji-sequences.txt','underline-emoji-zwj-sequences.txt']}
ref={'chromium':'153.0.8010.12','icu_revision':'8cc91d9b6ab9991802fd208ee03a69714fd0251c','unicode':'17.0.0','excluded_scalar_count':len(excluded),'source_sha256':sources,'excluded_intervals':intervals,'boundary_cases':checks}
Path('tools/html-to-riv/validation/underline-skip-eligibility-reference.json').write_text(json.dumps(ref,indent=2)+'\n')
rust='''// Chromium 153.0.8010.12 Character::CanTextDecorationSkipInk.
// Generated from pinned Blink tables and ICU 78.2 / Unicode 17 data.
// See tools/html-to-riv/validation/underline-skip-eligibility-research.md.
pub fn can_text_decoration_skip_ink(ch: char) -> bool {
    let cp = ch as u32;
    EXCLUDED.binary_search_by(|&(start, end)| {
        if end < cp { std::cmp::Ordering::Less }
        else if start > cp { std::cmp::Ordering::Greater }
        else { std::cmp::Ordering::Equal }
    }).is_err()
}
const EXCLUDED: &[(u32,u32)] = &[\n'''
rust+=''.join(f'    (0x{lo:X}, 0x{hi:X}),\n' for lo,hi in intervals)
rust+='];\n\n#[cfg(test)] mod tests {\n use super::*;\n #[test] fn exact_boundaries() {\n'
for case in checks:rust+=f'  assert_eq!(can_text_decoration_skip_ink(\'\\u{{{case["codepoint"]}}}\'), {str(case["eligible"]).lower()});\n'
rust+=' }\n #[test] fn table_is_sorted_and_disjoint() { assert!(EXCLUDED.windows(2).all(|w|w[0].1+1<w[1].0)); }\n}\n'
license_text = Path('/tmp/underline-chromium-LICENSE').read_text()
rust = '// Copyright 2016 The Chromium Authors\n' + ''.join('// '+line+'\n' for line in license_text.splitlines()) + '\n' + rust
Path('/tmp/html-skip-eligibility.rs').write_text(rust)
print(len(intervals),'intervals',len(excluded),'excluded scalars',len(checks),'boundary cases')
