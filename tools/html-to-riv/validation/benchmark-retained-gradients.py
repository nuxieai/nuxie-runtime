"""Run retained gradient controls after all competing GPU validation is terminal.
Usage: SCRIPT FROZEN_BENCHMARK NEW_OUTPUT [--rounds 3] [--samples 100]
Do not invoke while another native rendering benchmark or visual suite is active.
"""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import platform
import random
import subprocess

p=argparse.ArgumentParser(description=__doc__)
p.add_argument('binary',type=Path);p.add_argument('output',type=Path)
p.add_argument('--rounds',type=int,default=3);p.add_argument('--samples',type=int,default=100)
p.add_argument('--warmups',type=int,default=20)
p.add_argument('--include-clockwise',action='store_true',help='add explicitly non-CSS-equivalent forced-winding controls')
a=p.parse_args()
if a.rounds<1 or a.samples<3 or a.warmups<0:p.error('invalid sampling configuration')
binary=a.binary.resolve();out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
receipt={'status':'running','startedAt':datetime.datetime.now(datetime.timezone.utc).isoformat(),
 'binary':str(binary),'binarySha256':hashlib.sha256(binary.read_bytes()).hexdigest(),
 'harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
 'environment':{'platform':platform.platform()},'seed':260911,'cases':[],
 'metric':'retained-resource completed-frame wall latency including GPU wait/readback; not GPU-only',
 'backendSemantics':{'rust-metal':'default semantic-preserving Metal selection','rust-metal-atomic':'ordinary Atomics preserving path fill rules','rust-metal-clockwise':'optional forced winding; not CSS equivalence evidence'},
 'limitations':['workstation load and thermals uncontrolled','no absolute performance threshold']}
for label,command in [('power',['pmset','-g','batt']),('hardware',['sysctl','-n','machdep.cpu.brand_string'])]:
 r=subprocess.run(command,capture_output=True,text=True);receipt['environment'][label]=r.stdout
backends=['rust-metal','rust-metal-atomic'] + (['rust-metal-clockwise'] if a.include_clockwise else [])
configs=[(b,k,w,h,c,s) for b in backends for k in ['ordinary','css','tiled']
 for w,h in [(240,160),(1024,768)] for c in [2,256] for s in ['distinct','shared']]
rng=random.Random(260911)
for round_index in range(a.rounds):
 order=configs[:];rng.shuffle(order)
 for backend,kind,width,height,stops,sharing in order:
  name=f'{round_index}-{backend}-{kind}-{width}-{height}-{stops}-{sharing}'
  folder=out/name
  command=[str(binary),str(folder),backend,kind,str(width),str(height),str(stops),str(a.warmups),str(a.samples),sharing]
  r=subprocess.run(command,capture_output=True,text=True)
  (out/(name+'.log')).write_text(r.stdout+r.stderr)
  entry={'name':name,'round':round_index,'command':command,'exitCode':r.returncode}
  if (folder/'receipt.json').exists():
   entry['result']=json.loads((folder/'receipt.json').read_text())
   entry['receiptSha256']=hashlib.sha256((folder/'receipt.json').read_bytes()).hexdigest()
   entry['pngHashes']={n:hashlib.sha256((folder/n).read_bytes()).hexdigest() for n in ['first.png','last.png']}
  receipt['cases'].append(entry)
  if r.returncode:receipt['status']='failed'
  (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
  if r.returncode:raise SystemExit(f'{name} failed; preserved log')
 print(f'round {round_index+1}/{a.rounds} complete',flush=True)
receipt['status']='measured';receipt['finishedAt']=datetime.datetime.now(datetime.timezone.utc).isoformat()
(out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
