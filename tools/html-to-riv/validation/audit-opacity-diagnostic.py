"""Audit initial runtime-only opacity evidence; never qualify public CSS admission.
Usage: SCRIPT RECORDING REPLAY REVIEW COMPILER [ORACLE]
"""
from pathlib import Path
import collections
import hashlib
import json
import runpy
import subprocess
import struct
import sys
import tempfile

def read(p): return json.loads(p.read_text())
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def f32(value): return struct.unpack("f",struct.pack("f",value))[0]

def audit(recording, replay, review, compiler, oracle=None):
    source=read(recording/'lifecycle.json'); result=read(replay/'replay.json'); projection=read(review/'replay.json')
    public = source['kind']=='opacity-public'
    assert public or source['kind']=='opacity-diagnostic'
    assert result['browser']==source['browser']=='153.0.8010.12'
    assert result['lifecycleSha256']==sha(recording/'lifecycle.json')
    assert projection['sourceReplay']==str((replay/'replay.json').resolve())
    assert projection['sourceReplaySha256']==sha(replay/'replay.json')
    transferred_public = public and not (review/'visual-inspection.json').exists()
    verification=(runpy.run_path(str(Path(__file__).with_name('audit-opacity-public-visuals.py')))['audit'](review) if transferred_public else
        runpy.run_path(str(Path(__file__).with_name('record-visual-review.py')))['record'](review,audit=True))
    assert verification['remaining']==0
    reviewed={(r['browserSha256'],r['nativeSha256']) for r in projection['cases']}
    frames={r['name']:r for r in result['cases']}
    asset_dir=Path(__file__).parent.parent/'tests/assets'
    oracles = [oracle] if oracle else ([asset_dir/f'group-opacity-{name}-oracle.json' for name in ['initial','stacking','composition']]
        if public else [asset_dir/'group-opacity-initial-oracle.json'])
    known={r['name']:r for file in oracles for r in read(file)['cases']}
    assert len(frames)==len(result['cases'])==len(known)*8 and len(source['cases'])==len(known)
    seen=set(); artifacts=[]
    with tempfile.TemporaryDirectory() as directory:
        tmp=Path(directory)
        for fixture in source['cases']:
            name=fixture['name']; expected=known[name]
            assert fixture['html']==expected['html'] and fixture['css']==expected['css']
            alphas={}; stripped=''
            if public and 'expectedOpacity' in expected:
                alphas=expected['expectedOpacity']
            else:
                for rule in expected['css'].split('}'):
                    if not rule: continue
                    selector,body=rule.split('{'); stripped+=selector+'{'
                    for declaration in body.split(';'):
                        if not declaration: continue
                        if declaration.startswith('opacity:'):
                            assert selector.startswith('#')
                            alphas[selector[1:]]=float(declaration.split(':')[1])
                        else: stripped+=declaration+';'
                    stripped+='}'
            if public:
                assert fixture['compilerRequestCss']==expected['css'] and fixture['injectedOpacity'] is None
            else:
                assert fixture['compilerRequestCss']==stripped and fixture['injectedOpacity']==alphas
            request=dict(html=fixture['html'],css=expected['css'] if public else stripped,width=390,height=320)
            assets={}
            asset_dir=Path(__file__).parent.parent/'tests/assets'
            for flag in ['font','image']: assert fixture[flag]==expected.get(flag,False)
            if fixture['font']:
                assets['inter']=dict(kind='font',family='Inter',weight=400,bytes=list((asset_dir/'Inter-Regular.ttf').read_bytes()))
            if fixture['image']:
                assets['photo']=dict(kind='image',bytes=list((asset_dir/'quadrants.png').read_bytes()))
            if assets: request['assets']=assets
            (tmp/'input.json').write_text(json.dumps(request))
            subprocess.run([str(compiler),str(tmp/'input.json'),str(tmp/'scene.riv')],check=True,stdout=subprocess.DEVNULL)
            assert sha(tmp/'scene.riv')==sha(recording/f'{name}.riv')
            assert read(tmp/'scene.requirements.json')==fixture['runtimeRequirements']
            if public:
                ids={node['id']:node['object_id'] for node in read(tmp/'scene.map.json')}
                groups=fixture['runtimeRequirements'].get('layout_group_opacity',[])
                assert {entry['object_id']:f32(entry['opacity']) for entry in groups}=={ids[id]:f32(a) for id,a in alphas.items() if a<1}
                assert len(groups)==len({entry['object_id'] for entry in groups})
            artifacts.append(dict(name=name,rivSha256=sha(tmp/'scene.riv'),requestSha256=sha(tmp/'input.json')))
            assert len(fixture['views'])==8
            assert {(v['instance'],v['frame'],v['width'],v['height']) for v in fixture['views']}=={
                (i,i*4+f,w,320) for i in [0,1] for f,w in enumerate([240,390,768,240])}
            for view in fixture['views']:
                key=f"{name}-{view['instance']}-step{view['frame']}";assert key not in seen;seen.add(key)
                row=frames[key]
                assert row['qualification']==('public-compiler-lifecycle' if public else 'runtime-experiment-only') and not row['failures'] and not row['geometryFailures']
                for field in (['html','css','runtimeRequirements'] if public else ['html','css','compilerRequestCss','injectedOpacity','runtimeRequirements']): assert row[field]==fixture[field]
                for field in ['width','frame','instance']: assert row[field]==view[field]
                stream=Path(view['stream']);assert stream.is_absolute() and sha(stream)==row['streamSha256']
                chunks=stream.read_text().split('\nframe\n');commands=chunks[view['frame']].splitlines()
                actual=[];depth=0
                for command in commands:
                    if command.startswith('beginOpacity opacity='):
                        actual.append(f32(float(command.split('=')[1])));depth+=1
                    elif command=='endOpacity':
                        depth-=1;assert depth>=0
                assert depth==0 and collections.Counter(actual)==collections.Counter(f32(a) for a in alphas.values() if a<1)
                assert (row['browserSha256'],row['nativeSha256']) in reviewed
                for kind in ['browser','native']: assert sha(Path(row['prefix']+'.'+kind+'.png'))==row[kind+'Sha256']
    assert seen==set(frames)
    return dict(status='complete-public-opacity-artifact-audit' if public else 'complete-runtime-diagnostic-audit',scenes=len(known),frames=len(known)*8,views=len(known)*3,
        compilerSha256=sha(compiler),artifacts=artifacts,
        scope='Public authored CSS, reproduced Rive bytes/requirements, source-mapped alpha targets, clone/resize frames and reviewed image identity.' if public else 'Fresh reproduction of opacity-stripped compiler artifacts, explicit injected values and recorded group alpha/balance, lifecycle and reviewed image identity. Public opacity CSS remains unqualified.',
        evidence=[dict(path=str(p),sha256=sha(p)) for p in [recording/'lifecycle.json',replay/'replay.json',review/('visual-transfer.json' if transferred_public else 'visual-inspection.json'),review/'replay.json']])

if __name__=='__main__':
    recording,replay,review,compiler,*extra=map(lambda p:Path(p).resolve(),sys.argv[1:])
    assert len(extra)<=1
    result=audit(recording,replay,review,compiler,extra[0] if extra else None)
    path=replay/('public-opacity-audit.json' if result['status']=='complete-public-opacity-artifact-audit' else 'diagnostic-opacity-audit.json')
    if path.exists(): assert read(path)==result
    else: path.write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(dict(status=result['status'],frames=result['frames'],views=result['views'])))
