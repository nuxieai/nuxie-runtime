#!/usr/bin/env python3
"""Compare ordinary rounded/integral files at fractional native viewport sizes.
No browser, renderer replay, compilation, timing or runtime changes.
"""
from pathlib import Path
import copy,hashlib,importlib.util,json,math,re,shutil,struct,subprocess,sys
M=Path(__file__).resolve().parents[1]
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
bind=lambda p:dict(path=str(Path(p).resolve()),sha256=sha(p))
f32=lambda x:struct.unpack('<f',struct.pack('<f',float(x)))[0]
bits=lambda x:struct.unpack('<I',struct.pack('<f',float(x)))[0]
value=lambda b:struct.unpack('<f',struct.pack('<I',b))[0]
def require(v,m):
 if not v:raise ValueError(m)
spec=importlib.util.spec_from_file_location('clips',M/'validation/wrapped-clip-stream-check.py');clips=importlib.util.module_from_spec(spec);spec.loader.exec_module(clips)
N=clips.NUMBER

def intersection(a,b):return [max(a[0],b[0]),max(a[1],b[1]),min(a[2],b[2]),min(a[3],b[3])]
def canonical(a):return None if a[0]>=a[2] or a[1]>=a[3] else a

def observe_stream(lines,size,visible,slots,recipe,rounded):
 art=[0.,0.,*size];state={'t':[0.,0.],'masks':[]};stack=[];draws=[];mask_count=0
 reps=[]
 for i in range(len(visible)):
  for j in range(i,len(visible)):
   v=visible[j];x,y=v['worldMatrix'][4:];w,h=v['width'],v['height'];r=x+w;b=y+h
   quad=[[x,0,x+32768,32768],[r-32768,0,r,32768],[0,y,32768,y+32768],[0,b-32768,32768,b]]if rounded else[]
   active=[0,0,32768,32768];inactive=[0,65536,32768,98304]
   masks=quad.copy()
   if i:masks.append(active if slots[i-1]['worldMatrix'][5]!=slots[i]['worldMatrix'][5] else inactive)
   if i!=j:masks.append(active if slots[i]['worldMatrix'][5]==slots[j]['worldMatrix'][5] else inactive)
   reps.append(dict(owner=j,masks=masks,color=recipe['slots'][j]['color'],world=[x,y,r,b],local=[0,0,32768,32768]if rounded else[0,0,w,h],translation=[0,0]if rounded else[x,y]))
 allowed={tuple(art)}|{tuple(m)for r in reps for m in r['masks']}
 for index,line in enumerate(lines):
  token=line.split(' ',1)[0]
  if line=='save':stack.append(copy.deepcopy(state))
  elif line=='restore':require(stack,'restore underflow');state=stack.pop()
  elif token=='transform':
   m=re.fullmatch(r'transform matrix=\[('+N+r'(?:,'+N+r'){5})\]',line);require(m,'bad transform');a=[f32(x)for x in m[1].split(',')];require(a[:4]==[1,0,0,1],'nonidentity linear transform');state['t']=[f32(x+y)for x,y in zip(state['t'],a[4:])]
  elif token=='clipPath':
   require(state['t']==[0,0],'clip matrix not identity');b=clips.aabb(line);require(tuple(b)in allowed,'unknown mask');
   if mask_count==0:require(b==art and not state['masks'],'first artboard clip')
   else:require(state['masks'] and state['masks'][0]==art,'mask outside artboard')
   state['masks'].append(b);mask_count+=1
  elif token=='drawPath':
   m=re.fullmatch(r'drawPath (path=\{.*\}) paint=\{id=\d+,style=fill,color=(0x[0-9a-fA-F]{8}),thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=0\}',line);require(m,'unsupported paint');b=clips.aabb('clipPath '+m[1]);color=int(m[2],16)
   if not draws:require(b==art and color==0xffffffff and state['t']==[0,0] and state['masks']==[art],'background mismatch');paint=art;owner=None
   else:
    require(len(draws)<=len(reps),'extra draw');r=reps[len(draws)-1];owner=r['owner'];require(b==r['local'] and state['t']==r['translation'],'draw extent/position mismatch');require(state['masks']==[art]+r['masks'],'wrong replica masks/order');require(color==r['color'],'wrong paint order/color');paint=[f32(b[0]+state['t'][0]),f32(b[1]+state['t'][1]),f32(b[2]+state['t'][0]),f32(b[3]+state['t'][1])]
   for m in state['masks']:paint=intersection(paint,m)
   draws.append(dict(owner=owner,color=color,paint=canonical(paint)))
  elif token in ['makeRenderPaint','makeEmptyRenderPath']:require(re.fullmatch(token+r' \{.*\}',line),'bad resource')
  elif token=='frameSize':require(line==f'frameSize width={math.ceil(size[0])} height={math.ceil(size[1])}','wrong recording canvas')
  elif token=='clearColor':require(line=='clearColor value=0xffffffff','wrong clear')
  elif token=='sample':require(re.fullmatch(r'sample seconds='+N,line),'bad sample')
  else:raise ValueError('unknown command '+token)
 require(not stack and state=={'t':[0.,0.],'masks':[]},'unbalanced state');require(len(draws)==len(reps)+1,'missing draws');require(mask_count==1+sum(len(r['masks'])for r in reps),'missing masks')
 return dict(draws=draws,maskCount=mask_count)

def main():
 out=Path(sys.argv[1]).resolve() if len(sys.argv)>1 else(M/'output/wrapped-integral-fractional-r1').resolve();require(not out.exists(),'fresh output required');out.mkdir()
 probe=M/'output/wrapped-snapped-gate-r1/node-probe';require(sha(probe)=='2831265baa244ae03a26389fd82daaaa79eb88b77f542f6e8bc617343ca15b53','wrong probe');shutil.copy2(probe,out/'node-probe');shutil.copy2(__file__,out/'runner.py')
 old=M/'output/wrapped-resource-r1';new=M/'output/wrapped-integral-resource-r1';receipts=[json.loads((p/'construction-receipt.json').read_text())for p in[old,new]]
 down=lambda x:value(bits(x)-1);up=lambda x:value(bits(x)+1)
 sizes=[(down(60),up(60)),(60,59.5),(up(60),down(60)),(59.5,60.5),(60.5,59.5),(down(160),up(160)),(up(160),down(160)),(down(60),up(60))]
 results=[];commands=[];case_results=[]
 for count in [1,3,8,34]:
  name=f'owners-{count}';folder=out/name;folder.mkdir();cases=[p/'cases'/name for p in[old,new]]
  require((cases[0]/'base.riv').read_bytes()==(cases[1]/'base.riv').read_bytes(),'base scene differs')
  trace=json.loads((cases[1]/'trace.json').read_text());recipe=json.loads((cases[1]/'recipe.json').read_text());require(recipe['row'] and not recipe['reverseMain'] and recipe['wrap']==1 and recipe['lineFraction']==recipe['mainFraction']==0,'unsupported recipe')
  manifests=[];inputs=[]
  for index,(label,case)in enumerate(zip(['rounded','integral'],cases)):
   expected=next(r for r in receipts[index]['cases']if r['owners']==count)
   require(sha(case/'scene.riv')==expected['artifacts']['scene.riv']['sha256'],'unbound scene')
   dest=folder/label;cmd=[str(out/'node-probe'),str(case/'scene.riv'),str(dest),*[f'{w:.17g}x{h:.17g}'for w,h in sizes]];r=subprocess.run(cmd,capture_output=True,text=True);log=folder/(label+'.log');log.write_text(r.stdout+r.stderr);commands.append(dict(command=cmd,exitCode=r.returncode,log=bind(log)));require(r.returncode==0,r.stderr)
   manifests.append(json.loads((dest/'frames.json').read_text()));inputs.append(dict(scene=bind(case/'scene.riv'),base=bind(case/'base.riv'),recipe=bind(case/'recipe.json'),proof=bind(case/'proof.json'),trace=bind(case/'trace.json')))
  observed=[]
  for i,(a,b)in enumerate(zip(manifests[0]['frames'],manifests[1]['frames'])):
   size=[f32(a['width']),f32(a['height'])];require(size==[f32(v)for v in sizes[a['step']]],'fractional viewport changed');require(a==b,'frame metadata differs')
   pair=[];paths=[]
   for label,f in zip(['rounded','integral'],[a,b]):
    d=folder/label;raw=json.loads((d/f['geometry']).read_text());objects={o['objectId']:{**o,'width':f32(o['width']),'height':f32(o['height']),'worldMatrix':[f32(x)for x in o['worldMatrix']]}for o in raw}
    relevant=[objects[n]for n in [0,trace['parent']]+trace['slots']+trace['visible']];visible=[objects[n]for n in trace['visible']];slots=[objects[n]for n in trace['slots']]
    for v in visible:
     require(v['worldMatrix'][:4]==[1,0,0,1],'visible not axis aligned')
     x,y=v['worldMatrix'][4:];edges=[x,y,f32(x+v['width']),f32(y+v['height'])];require(all(math.isfinite(e) and e==math.trunc(e)for e in edges),'visible fractional edge');require(v['width']==v['height']==20,'unexpected owner size')
    stream=observe_stream(clips.extract(d/f['stream'],f['frame']),size,visible,slots,recipe,label=='rounded');pair.append(dict(geometry=relevant,stream=stream));paths.append(dict(geometry=bind(d/f['geometry']),stream=bind(d/f['stream'])))
   require(pair[0]['geometry']==pair[1]['geometry'],'optimized geometry differs');require(pair[0]['stream']['draws']==pair[1]['stream']['draws'],'effective paint differs')
   row=dict(name=name,frame=i,instance=a['instance'],step=a['step'],viewport=size,viewportBits=[bits(v)for v in size],geometry=pair[1]['geometry'],draws=pair[1]['stream']['draws'],roundedMaskCount=pair[0]['stream']['maskCount'],integralMaskCount=pair[1]['stream']['maskCount'],artifacts=paths);results.append(row);observed.append(row)
  steps=len(sizes);require(len(observed)==2*steps,'missing clone frames');require(all(observed[i]['geometry']==observed[i+steps]['geometry'] and observed[i]['draws']==observed[i+steps]['draws']for i in range(steps)),'clone mismatch');require(observed[0]['geometry']==observed[7]['geometry'] and observed[0]['draws']==observed[7]['draws'],'repeat mismatch')
  case_results.append(dict(name=name,owners=count,inputs=inputs,framePairs=len(observed),cloneExact=True,repeatExact=True,visibleEdgeChecks=len(observed)*count*4))
 receipt=dict(scope='Native fractional viewport ordinary original/clone comparison, no fractional-browser or pixel-render claim',script=bind(out/'runner.py'),probe=bind(out/'node-probe'),probeSource=bind(M/'output/wrapped-snapped-gate-r1/node-probe.rs'),probeBuildCommand=bind(M/'output/wrapped-snapped-gate-r1/probe-command.json'),resourceReceipts=[bind(p/'construction-receipt.json')for p in[old,new]],commands=commands,cases=case_results,frames=results,counts=dict(cases=len(case_results),framePairs=len(results),nativeFrames=2*len(results),visibleEdgeChecks=sum(c['visibleEdgeChecks']for c in case_results),geometryDifferences=0,effectivePaintDifferences=0))
 (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt['counts']))
if __name__=='__main__':main()
