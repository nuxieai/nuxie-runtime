from pathlib import Path
import hashlib,json,shutil,subprocess,sys
base=Path(__file__).resolve().parent.parent
out=base/'output/solid-border-candidate-r1'/sys.argv[1] if len(sys.argv)>1 else base/'output/solid-border-candidate-r1'
out.mkdir(exist_ok=True)
seed=base/'output/flex-world-domains-r1'
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
src=out/'src';src.mkdir(exist_ok=False)
for original,name in [(base/'validation/solid-border-candidate-emitter.rs','main.rs'),(seed/'source/wire.rs','wire.rs'),(base/'validation/solid-border-candidate-cases.json','cases.json')]:shutil.copyfile(original,src/name)
cmd=['rustc','--edition=2024',str(src/'main.rs'),'-L','dependency='+str(base/'target/debug/deps'),'-C','debuginfo=0','-o',str(out/'candidate')]
inputs={str(p):sha(p) for p in src.iterdir()}
for name in ['nuxie_html_to_riv','nuxie_schema','serde_json']:
 p=next(seed.glob(f'lib{name}*.rlib'));dst=out/p.name;shutil.copy2(p,dst);inputs[str(dst)]=sha(dst);cmd+=['--extern',name+'='+str(dst)]
p=subprocess.run(cmd,capture_output=True,text=True);(out/'build.log').write_text(p.stdout+p.stderr);p.check_returncode()
(out/'build-receipt.json').write_text(json.dumps(dict(inputs=inputs,command=cmd,binarySha256=sha(out/'candidate'),seedBuildReceiptSha256=sha(seed/'build-receipt.json')),indent=2)+'\n')
