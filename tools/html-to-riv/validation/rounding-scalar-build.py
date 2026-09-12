"""Freeze private compiler construction with an explicit validation-only bridge.

No repository source, dependency, runtime or build file is modified. The copied
crate has a validation entrypoint; the ordinary .riv is emitted from Derived's
read-only records, without any post-construction record edits.
"""
from pathlib import Path
import difflib,hashlib,json,shutil,subprocess,sys,tomllib
M=Path(__file__).resolve().parents[1];R=M.parents[1]
out=Path(sys.argv[1]).resolve();out.mkdir(parents=True,exist_ok=False)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
bindings=[];patches=[]
def freeze(p,d):
 d.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,d)
 bindings.append(dict(source=str(p),snapshot=str(d),sha256=sha(p)))
def patch(p,text):
 old=p.read_text();p.write_text(text)
 patches.append(dict(path=str(p),beforeSha256=hashlib.sha256(old.encode()).hexdigest(),afterSha256=sha(p),diff=''.join(difflib.unified_diff(old.splitlines(True),text.splitlines(True),fromfile='original/'+p.name,tofile='validation/'+p.name))))
for p in sorted((M/'src').rglob('*')):
 if p.is_file():freeze(p,out/'inputs/src'/p.relative_to(M/'src'))
for name in ['Cargo.toml','Cargo.lock']:freeze(M/name,out/'inputs'/name)
freeze(Path(__file__).resolve(),out/'builder.py');freeze(M/'validation/rounding-scalar-bridge.rs',out/'bridge.rs')
freeze(R/'Cargo.toml',out/'inputs/workspace-Cargo.toml')
crate=out/'crate';shutil.copytree(out/'inputs/src',crate/'src');shutil.copy2(out/'inputs/Cargo.toml',crate/'Cargo.toml');shutil.copy2(out/'inputs/Cargo.lock',crate/'Cargo.lock')
for rel in ['crates/nuxie-schema','vendor/jpeg-decoder-0.3.2-rive-v9f']:
 for p in sorted((R/rel).rglob('*')):
  if p.is_file() and 'target' not in p.parts:freeze(p,out/'inputs/dependencies'/rel/p.relative_to(R/rel))
shutil.copytree(out/'inputs/dependencies',out/'dependencies')
schema=out/'dependencies/crates/nuxie-schema/Cargo.toml';metadata=tomllib.loads((R/'Cargo.toml').read_text())['workspace']['package'];s=schema.read_text()
for key in ['edition','license','repository']:s=s.replace(key+'.workspace = true',key+' = '+json.dumps(metadata[key]))
patch(schema,s+'\n[workspace]\n')
manifest=(crate/'Cargo.toml').read_text().replace('../../vendor/jpeg-decoder-0.3.2-rive-v9f',str(out/'dependencies/vendor/jpeg-decoder-0.3.2-rive-v9f')).replace('../../crates/nuxie-schema',str(out/'dependencies/crates/nuxie-schema'))
manifest+='\n[[bin]]\nname = "rounding-scalar-validation"\npath = "src/validation_main.rs"\n'
patch(crate/'Cargo.toml',manifest)
patch(crate/'src/compiler.rs',(crate/'src/compiler.rs').read_text()+'\n'+(out/'bridge.rs').read_text())
patch(crate/'src/lib.rs',(crate/'src/lib.rs').read_text()+'\n// Frozen validation-only export; absent from repository product API.\npub fn validation_rounding_recipe(input: &str, out: &std::path::Path) -> Result<(), String> { compiler::validation_rounding_recipe(input, out) }\n')
main='''fn main() {
 let a:Vec<_>=std::env::args().collect();
 if a.len()!=3 {eprintln!("usage: rounding-scalar-validation RECIPE_JSON OUTPUT_DIRECTORY");std::process::exit(2);}
 let input=std::fs::read_to_string(&a[1]).unwrap();
 if let Err(e)=nuxie_html_to_riv::validation_rounding_recipe(&input,std::path::Path::new(&a[2])){eprintln!("{e}");std::process::exit(1);}
}
'''
(crate/'src/validation_main.rs').write_text(main)
patches.append(dict(path=str(crate/'src/validation_main.rs'),newFile=True,sha256=sha(crate/'src/validation_main.rs'),content=main))
(out/'patches.json').write_text(json.dumps(patches,indent=2)+'\n')
(out/'source-bindings.json').write_text(json.dumps(bindings,indent=2)+'\n')
command=['cargo','build','--locked','--offline','--manifest-path',str(crate/'Cargo.toml'),'--bin','rounding-scalar-validation']
r=subprocess.run(command,cwd=crate,capture_output=True,text=True);(out/'build.log').write_text(r.stdout+r.stderr)
assert r.returncode==0,r.stderr
shutil.copy2(crate/'target/debug/rounding-scalar-validation',out/'candidate')
for b in bindings:
 assert sha(b['source'])==b['sha256'],'Source changed during build'
 assert sha(b['snapshot'])==b['sha256']
inputs=[dict(path=str(p),sha256=sha(p))for p in sorted(crate.rglob('*'))if p.is_file() and 'target' not in p.parts]
(out/'receipt.json').write_text(json.dumps(dict(scope='Private ordinary signed rounding constructor only; no public admission or rendering qualification',command=command,exitCode=r.returncode,candidateSha256=sha(out/'candidate'),buildLogSha256=sha(out/'build.log'),bridgeSha256=sha(out/'bridge.rs'),patchesSha256=sha(out/'patches.json'),sourceBindingsSha256=sha(out/'source-bindings.json'),effectiveCrateInputs=inputs),indent=2)+'\n')
print(json.dumps(dict(root=str(out),candidateSha256=sha(out/'candidate'))))
