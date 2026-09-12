#!/usr/bin/env python3
"""Offline bounded-corpus normalization check; no compiler execution or visual claim."""
from pathlib import Path
from fractions import Fraction
from decimal import Decimal
import copy, hashlib, json, math, struct
M=Path(__file__).resolve().parents[1]
C=M/'output/wrapped-normalized-constructor-r1'
bindings={}
def bind(p,h=None):
 p=Path(p);a=hashlib.sha256(p.read_bytes()).hexdigest()
 if h is not None:assert a==h,p
 bindings[str(p.resolve())]=a;return a
def read(p):bind(p);return json.loads(Path(p).read_text())
def bits(v):return struct.unpack('<I',struct.pack('<f',v))[0]
def value(b):return struct.unpack('<f',struct.pack('<I',b))[0]
def f32(v):return value(bits(v))
def require(ok,label):
 if not ok:raise ValueError(label)
def source_bits(text):
 # Exact rational distance to neighboring binary32 values, tie to even.
 # The seed only locates nearby candidates; distance/tie choice never uses f64.
 rational=Fraction(Decimal(text));seed=bits(float(text));require(seed<0x7f800000,'positive finite corpus source')
 candidates=[b for b in range(max(0,seed-2),min(0x7f800000,seed+3))]
 chosen=min(candidates,key=lambda b:(abs(Fraction(value(b))-rational),b&1))
 # This corpus does not distinguish the separately reviewed double-rounding path.
 require(chosen==bits(float(text)),'source needs separate f64/direct-f32 semantics audit')
 return chosen
KEYS={'owner','width','height','minWidth','minHeight','maxWidth','maxHeight'}
def check_length(q,p,label):
 if q is None:
  require(p=={'kind':'auto'},label+' auto');return 'auto'
 computed=source_bits(q['source']);require(computed==bits(q['value']),label+' source token mismatch')
 require(isinstance(p.get('sourceProvenance'),str) and bool(p['sourceProvenance']),label+' provenance absent')
 if q['unit']=='%':
  require(set(p)=={'kind','native','nativeBits','sourceProvenance'},label+' percent fields')
  require(p['kind']=='percent' and p['nativeBits']==computed and bits(p['native'])==computed,label+' percent changed');return 'percent'
 require(q['unit']=='px',label+' unit')
 require(set(p)=={'kind','computedBits','rawUnits','emitted','emittedBits','sourceProvenance'},label+' fixed fields')
 source=value(computed);require(math.isfinite(source) and source>=0,label+' bounded fixed input')
 raw=int(Fraction(source)*64);require(0<=raw<=2147483647,label+' bounded fixed raw')
 emitted=f32(float(Fraction(raw,64)))
 require(p['kind']=='fixed' and p['computedBits']==computed,label+' retained computed bits')
 require(p['rawUnits']==raw and p['emittedBits']==bits(emitted) and bits(p['emitted'])==bits(emitted),label+' raw/emitted mismatch')
 return 'fixed'
def check_style(owner,dims,p,label):
 require(set(p)==KEYS and p['owner']==owner,label+' role owner/fields')
 counts={'fixed':0,'percent':0,'auto':0}
 for axis,q in zip(['Width','Height'],dims):
  for key,number in [(axis.lower(),q),('min'+axis,q.get('min') or {'value':0,'unit':'px','source':'0'}),('max'+axis,q.get('max'))]:
   counts[check_length(number,p[key],label+'.'+key)]+=1
 return counts
def check(q,trace,proof):
 n=proof['normalization'];require(set(n)=={'parent','roles'},'normalization fields')
 require(len(trace['slots'])==len(trace['visible'])==len(q['slots']),'trace lengths')
 owners=[trace['parent'],*trace['slots'],*trace['visible']];require(len(set(owners))==len(owners),'trace owner uniqueness')
 expected={trace['parent']:[q['parent']['width'],q['parent']['height']]}
 for i,slot in enumerate(q['slots']):
  dims=[slot['main'],slot['cross']];visible=[slot['visibleMain'],slot['visibleCross']]
  if not q['row']:dims.reverse();visible.reverse()
  expected[trace['slots'][i]]=dims;expected[trace['visible'][i]]=visible
 require(len(n['roles'])==2*len(q['slots']),'normalization role count')
 actual={p['owner']:p for p in [n['parent'],*n['roles']]};require(len(actual)==len(expected) and set(actual)==set(expected),'role owner mismatch')
 require(n['parent']['owner']==trace['parent'],'parent owner mismatch')
 counts={'fixed':0,'percent':0,'auto':0}
 for owner,dims in expected.items():
  c=check_style(owner,dims,actual[owner],str(owner))
  for k,v in c.items():counts[k]+=v
 return counts

def main():
 construction=read(C/'construction-receipt.json');require(len(construction['cases'])==80,'80-case corpus')
 for key,name in [('constructorSha256','candidate'),('recipesSha256','recipes.json'),('scriptSha256','construct.py')]:bind(C/name,construction[key])
 recipes=read(C/'recipes.json');build=read(C/'receipt.json');require(build['exitCode']==0,'constructor build')
 for key,name in [('candidateSha256','candidate'),('bridgeSha256','bridge.rs'),('patchesSha256','patches.json'),('sourceBindingsSha256','source-bindings.json'),('buildLogSha256','build.log')]:bind(C/name,build[key])
 for b in build['effectiveCrateInputs']:bind(b['path'],b['sha256'])
 frozen=read(C/'source-bindings.json')
 for b in frozen:bind(b['snapshot'],b['sha256'])
 rows=[];negative=[];totals={'fixed':0,'percent':0,'auto':0}
 for c in construction['cases']:
  require(c['exitCode']==0 and c['deterministic'],'construction outcome');folder=C/'cases'/c['name']
  bind(folder/'recipe.json',c['recipeSha256']);bind(folder/'construct.log',c['logSha256'])
  for name,h in c['artifacts'].items():
   for sub in ['', 'first','repeat']:bind(folder/sub/name,h)
  q=read(folder/'recipe.json');require(q==recipes[c['name']],'recipe registry mismatch');trace=read(folder/'trace.json');proof=read(folder/'proof.json');counts=check(q,trace,proof)
  rows.append(dict(name=c['name'],roles=1+2*len(q['slots']),dimensions=sum(counts.values()),counts=counts,recipeSha256=c['recipeSha256'],proofSha256=c['artifacts']['proof.json'],traceSha256=c['artifacts']['trace.json']))
  for k,v in counts.items():totals[k]+=v
  for mutation in ['rawUnits','emittedBits','roleOwner','sourceToken']:
   changed=copy.deepcopy(proof);source=copy.deepcopy(q)
   if mutation in ['rawUnits','emittedBits']:
    found=next(p for style in [changed['normalization']['parent'],*changed['normalization']['roles']]for k,p in style.items()if k!='owner'and p['kind']=='fixed');found[mutation]+=1
   elif mutation=='roleOwner':changed['normalization']['roles'][0]['owner']=trace['parent']
   else:source['slots'][0]['main']['source']=str(float(source['slots'][0]['main']['value'])+1.)
   try:check(source,trace,changed)
   except ValueError as e:negative.append(dict(name=c['name'],mutation=mutation,rejected=str(e)))
   else:raise AssertionError('negative control accepted '+mutation)
 bind(__file__)
 result=dict(status='verified-frozen-normalization-proofs',scope='80 frozen recipe proofs only: exact bounded decimal/f32 correspondence, truncation to1/64 fixed units, percent preservation, six dimensions per mapped role; no wire-field redecoder, public admission or native/visual qualification. Current corpus has no direct-f32/f64-then-f32 difference; general decimal semantics are separately audited.',cases=len(rows),dimensions=sum(totals.values()),counts=totals,negativeControls=negative,rows=rows,bindings=[dict(path=p,sha256=h)for p,h in sorted(bindings.items())])
 out=M/'output/wrapped-normalized-proof-verification-r1.json';out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(dict(status=result['status'],cases=len(rows),dimensions=result['dimensions'],counts=totals,negativeControls=len(negative),bindings=len(bindings))))
if __name__=='__main__':main()
