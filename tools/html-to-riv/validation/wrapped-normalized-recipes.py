"""Retain original decimal tokens while adding source metadata to authored recipes.

No geometry or Chrome observation enters this transformation. Numeric values
remain unchanged; source tokens are retained before any binary32 conversion.
"""
import hashlib,json,sys
from pathlib import Path
class AuthoredFloat(float):
 def __new__(cls,text):
  result=super().__new__(cls,text);result.source=text;return result
def annotate(value):
 if isinstance(value,dict):
  result={key:annotate(item)for key,item in value.items()}
  if 'value' in value and 'unit' in value:
   number=value['value']
   if isinstance(number,bool) or not isinstance(number,(int,AuthoredFloat)):raise ValueError('Expected authored numeric token')
   result['source']=number.source if isinstance(number,AuthoredFloat) else str(number)
  return result
 if isinstance(value,list):return [annotate(item)for item in value]
 return value
if __name__=='__main__':
 source,target=map(Path,sys.argv[1:])
 data=json.loads(source.read_text(),parse_float=AuthoredFloat)
 result=annotate(data)
 with target.open('x')as f:json.dump(result,f,indent=2);f.write('\n')
 sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
 target.with_suffix('.source.json').write_text(json.dumps(dict(input=str(source.resolve()),inputSha256=sha(source),outputSha256=sha(target),generatorSha256=sha(Path(__file__)),scope='Original numeric values with retained decimal token metadata; no browser measurements'),indent=2)+'\n')
