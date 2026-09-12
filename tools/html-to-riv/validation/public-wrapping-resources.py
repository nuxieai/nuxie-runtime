"""Public CLI owner-count boundary observations, with deterministic repeats.
Run with a fresh output directory; timings include process startup and IO.
"""
from pathlib import Path
import hashlib,json,subprocess,sys,time,shutil
M=Path(__file__).resolve().parents[1];B=M/'output/public-wrapping-product-build-r1/frozen';O=Path(sys.argv[1]).resolve();O.mkdir(parents=True,exist_ok=False)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
bind=lambda p:dict(path=str(Path(p).resolve()),sha256=sha(p))
shutil.copy2(__file__,O/'runner.py');shutil.copy2(B/'html-to-riv',O/'html-to-riv')
rows=[]
for profile,counts in [('integer',[1,3,8,34,84,1562,1563]),('fractional',[1,3,8,34,35])]:
 for count in counts:
  name=f'{profile}-{count}';d=O/name;d.mkdir();size='20' if profile=='integer' else '20.125'
  request=dict(html='<div id="p">'+''.join(f'<div id="v{i}"></div>' for i in range(count))+'</div>',css=f'#p{{width:320px;height:240px;flex-direction:row;flex-wrap:wrap;align-content:flex-start}}#p>div{{width:{size}px;height:{size}px;background:#2167b1}}',width=320,height=240)
  (d/'request.json').write_text(json.dumps(request,indent=2)+'\n');runs=[]
  for repeat in ['first','repeat']:
   dest=d/repeat;dest.mkdir();cmd=[str(O/'html-to-riv'),str(d/'request.json'),str(dest/'scene.riv')];started=time.perf_counter_ns()
   try:r=subprocess.run(cmd,capture_output=True,timeout=60);code=r.returncode;stdout=r.stdout;stderr=r.stderr;termination='exit'
   except subprocess.TimeoutExpired as e:code=None;stdout=e.stdout or b'';stderr=e.stderr or b'';termination='timeout'
   (dest/'stdout.log').write_bytes(stdout);(dest/'stderr.log').write_bytes(stderr)
   artifacts={n:bind(dest/n) for n in ['scene.riv','scene.map.json'] if (dest/n).exists()}
   runs.append(dict(command=cmd,exitCode=code,termination=termination,elapsedNs=time.perf_counter_ns()-started,stdout=bind(dest/'stdout.log'),stderr=bind(dest/'stderr.log'),artifacts=artifacts))
  assert runs[0]['exitCode']==runs[1]['exitCode'],name
  accepted=runs[0]['exitCode']==0
  if accepted:
   assert all(len(r['artifacts'])==2 for r in runs)
   for artifact in runs[0]['artifacts']:assert runs[0]['artifacts'][artifact]['sha256']==runs[1]['artifacts'][artifact]['sha256']
   ids=json.loads((d/'first/scene.map.json').read_text());assert len(ids)==count+1
   assert [n['id']for n in ids]==['p']+[f'v{i}'for i in range(count)]
  else:assert all(not r['artifacts'] for r in runs)
  row=dict(name=name,profile=profile,owners=count,request=bind(d/'request.json'),accepted=accepted,runs=runs,bytes=(d/'first/scene.riv').stat().st_size if accepted else None,diagnostic=(d/'first/stderr.log').read_text() if not accepted else None)
  rows.append(row);(O/'partial.json').write_text(json.dumps(rows,indent=2)+'\n');print(json.dumps({k:row[k]for k in ['name','accepted','bytes','diagnostic']}),flush=True)
receipt=dict(scope='Observed public CLI boundary requests, each executed twice. Not native performance, exhaustive capacity or browser qualification.',compiler=bind(O/'html-to-riv'),origin=bind(B/'html-to-riv'),runner=bind(O/'runner.py'),cases=rows)
(O/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
