"""Tiny file/CLI fixtures test proof rejection without Cargo, WASM or GPU."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec=importlib.util.spec_from_file_location('freeze',Path(__file__).with_name('freeze-public-compiler.py'))
api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)

class FreezeTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.root=Path(self.temp.name).resolve();self.module=self.root/'module';self.module.mkdir()
        for name in ('src','js','tests','validation'):(self.module/name).mkdir()
        for name in ('Cargo.toml','Cargo.lock','package.json','package-lock.json','src/reset.css'):(self.module/name).write_text('fixture')
        self.native=self.root/'native';self.native.write_text('#!/usr/bin/env python3\nimport pathlib,sys\np=pathlib.Path(sys.argv[2]);p.write_bytes(b"riv");p.with_suffix(".map.json").write_bytes(b"[]")\n');self.native.chmod(0o755)
        self.wasm=self.root/'wasm';self.wasm.write_bytes(b'wasm')
        self.run=self.root/'run';self.run.mkdir();folder=self.run/'case';folder.mkdir();probe=folder/'probe';probe.mkdir()
        (self.run/'source-cases.json').write_text(json.dumps([dict(name='case',html='<div/>',css='')]))
        (self.run/'browser-reset.css').write_text('fixture');(folder/'request.json').write_text(json.dumps(dict(html='<div/>',css='',width=10,height=10)))
        (folder/'scene.riv').write_bytes(b'riv');(probe/'scene.riv').write_bytes(b'riv');(folder/'scene.map.json').write_bytes(b'[]')
        artifact=dict(name='case',requestSha256=api.sha(folder/'request.json'),rivSha256=api.sha(folder/'scene.riv'),mapSha256=api.sha(folder/'scene.map.json'))
        rows=[];frames=[]
        for i in range(8):
            identity=dict(frame=i,instance=i//4,step=i%4,width=10,height=10)
            frames.append(dict(**identity,stream=f'frame-{i}.stream',geometry=f'frame-{i}.geometry.json'))
            row=dict(name='case',**identity,geometryFailures=[],pixelFailures=[],prefix=str(folder/f'frame-{i}'),requestSha256=artifact['requestSha256'],rivSha256=artifact['rivSha256'],sourceMapSha256=artifact['mapSha256'])
            for p,k in [(probe/f'frame-{i}.stream','streamSha256'),(probe/f'frame-{i}.geometry.json','geometrySha256'),(folder/f'frame-{i}.chrome.png','chromeSha256'),(folder/f'frame-{i}.native.png','nativeSha256')]:p.write_bytes(b'synthetic evidence');row[k]=api.sha(p)
            rows.append(row)
        (probe/'frames.json').write_text(json.dumps(dict(frames=frames)));artifact['probeManifestSha256']=api.sha(probe/'frames.json')
        self.receipt=dict(status='passed-public-baseline',artifacts=[artifact],rows=rows,fixturesSha256=api.sha(self.run/'source-cases.json'),driverSha256='notavailable',pixelGateSha256='notavailable',tools=dict(probe=str(self.native),renderer=str(self.native)),toolHashes=dict(probe=api.sha(self.native),renderer=api.sha(self.native)))
        self.path=self.run/'receipt.json';self.save()
    def tearDown(self):self.temp.cleanup()
    def save(self):self.path.write_text(json.dumps(self.receipt))
    def freeze(self):return api.freeze(self.native,self.wasm,self.root/'out',[self.path],self.module)
    def test_success(self):
        result=self.freeze();self.assertEqual(result['status'],'verified-byte-identical');self.assertEqual(len(result['artifacts']),1);self.assertEqual(result['artifacts'][0]['frames'],8)
        with self.assertRaisesRegex(ValueError,'fresh'):self.freeze()
    def test_missing_and_failed_receipts(self):
        for change in [lambda r:r.update(status='running'),lambda r:r['rows'].pop(),lambda r:r['rows'][0]['pixelFailures'].append('mismatch')]:
            with self.subTest(change=change):
                original=copy.deepcopy(self.receipt);change(self.receipt);self.save()
                with self.assertRaises(ValueError):self.freeze()
                import shutil
                shutil.rmtree(self.root/'out');self.receipt=original;self.save()
    def test_original_tampering(self):
        (self.run/'case/scene.riv').write_bytes(b'tampered')
        with self.assertRaisesRegex(ValueError,'changed evidence'):self.freeze()
    def test_output_byte_mismatch(self):
        self.native.write_text(self.native.read_text().replace('b"riv"','b"different"'))
        self.receipt['toolHashes']={k:api.sha(self.native) for k in ('probe','renderer')};self.save()
        with self.assertRaisesRegex(ValueError,'complete scene.riv bytes differ'):self.freeze()
    def test_source_modified_during_compile(self):
        text=self.native.read_text()+f'pathlib.Path({str(self.module/"src/reset.css")!r}).write_text("changed")\n'
        self.native.write_text(text);self.receipt['toolHashes']={k:api.sha(self.native) for k in ('probe','renderer')};self.save()
        with self.assertRaisesRegex(ValueError,'source inventory/content changed'):self.freeze()

    def test_clear_checks_cannot_be_failed_or_tampered(self):
        import shutil
        image=self.run/'case/frame-0.clear-cyan.png';image.write_bytes(b'clear evidence')
        check=dict(name='cyan',path=str(image),sha256=api.sha(image),samePixels=False)
        self.receipt['rows'][0]['clearChecks']=[check];self.save()
        with self.assertRaisesRegex(ValueError,'canvas-clear'):self.freeze()
        shutil.rmtree(self.root/'out');check['samePixels']=True;self.save()
        image.write_bytes(b'tampered clear evidence')
        with self.assertRaisesRegex(ValueError,'changed evidence'):self.freeze()

if __name__=='__main__':unittest.main()
