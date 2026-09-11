"""Negative controls for full-gallery transfer admission; never qualification evidence."""
import copy
import json
from pathlib import Path
import runpy
import tempfile
import unittest

api=runpy.run_path(str(Path(__file__).with_name('audit-gradient-full-review.py')))
root=Path(__file__).resolve().parents[3]

class Controls(unittest.TestCase):
    def setUp(self):
        self.keys={('unit-scene',240),('unit-scene',390),('unit-scene',768)}
        self.report=dict(status='passed',errors=[],expected=3,
            checks=[dict(status='passed',errors=[],project='native') for _ in range(3)],
            cases=[dict(name=name,width=width,status='passed',errors=[],metrics={'backend':'rust-metal'}) for name,width in sorted(self.keys)])

    def test_valid_small_control_is_admitted(self):
        api['validate_gallery'](self.report,self.keys)

    def test_incomplete_and_failed_report_rejected(self):
        for status in ['running','failed','interrupted']:
            bad=copy.deepcopy(self.report);bad['status']=status
            with self.assertRaises(AssertionError):api['validate_gallery'](bad,self.keys)
        bad=copy.deepcopy(self.report);bad['errors']=['renderer failed']
        with self.assertRaises(AssertionError):api['validate_gallery'](bad,self.keys)

    def test_missing_failed_or_wrong_project_check_rejected(self):
        for mutation in [lambda r:r['checks'].pop(),lambda r:r['checks'][0].update(status='failed'),lambda r:r['checks'][0].update(project='geometry')]:
            bad=copy.deepcopy(self.report);mutation(bad)
            with self.assertRaises(AssertionError):api['validate_gallery'](bad,self.keys)

    def test_duplicate_missing_or_substituted_scene_rejected(self):
        for mutation in [lambda r:r['cases'].pop(),lambda r:r['cases'].append(copy.deepcopy(r['cases'][0])),lambda r:r['cases'][0].update(name='unexpected'),lambda r:r['cases'][0].update(width=999)]:
            bad=copy.deepcopy(self.report);mutation(bad)
            with self.assertRaises(AssertionError):api['validate_gallery'](bad,self.keys)

    def test_hidden_case_failure_and_wrong_backend_rejected(self):
        for mutation in [lambda r:r['cases'][0].update(status='failed'),lambda r:r['cases'][0].update(errors=['pixel mismatch']),lambda r:r['cases'][0]['metrics'].update(backend='canvas')]:
            bad=copy.deepcopy(self.report);mutation(bad)
            with self.assertRaises(AssertionError):api['validate_gallery'](bad,self.keys)

    def test_actual_transport_mutations_rejected(self):
        source=root/'output/playwright/html-to-riv/linear-gradient-composition-native-r3/linear-gradient-composition-fractional-border'
        with tempfile.TemporaryDirectory() as temp:
            target=Path(temp)/'scene'
            for suffix in ['.riv','.map.json','.requirements.json']:
                Path(str(target)+suffix).write_bytes(Path(str(source)+suffix).read_bytes())
            api['verify_transport'](source,target)
            for suffix in ['.riv','.map.json','.requirements.json']:
                path=Path(str(target)+suffix);original=path.read_bytes()
                if suffix=='.riv':path.write_bytes(original+b'corrupt')
                else:
                    value=json.loads(original)
                    if suffix=='.map.json':value[0]['object_id']+=1
                    else:value['layout_linear_gradients'][0]['stops'][0]['color']^=1
                    path.write_text(json.dumps(value))
                with self.assertRaises(AssertionError):api['verify_transport'](source,target)
                path.write_bytes(original)

    def test_actual_png_mutation_and_stale_binding_rejected(self):
        report=api['read'](root/'output/playwright/html-to-riv/linear-gradient-composition-native-r3/replay.json')
        original=report['cases'][0]
        with tempfile.TemporaryDirectory() as temp:
            row=copy.deepcopy(original);row['prefix']=str(Path(temp)/'frame')
            for kind in ['browser','native']:
                Path(row['prefix']+'.'+kind+'.png').write_bytes(Path(original['prefix']+'.'+kind+'.png').read_bytes())
            api['verify_images'](row)
            for kind in ['browser','native']:
                path=Path(row['prefix']+'.'+kind+'.png');data=path.read_bytes();path.write_bytes(data+b'changed')
                with self.assertRaises(AssertionError):api['verify_images'](row)
                path.write_bytes(data)
            binding=dict(path=row['prefix']+'.browser.png',sha256='0'*64)
            with self.assertRaises(AssertionError):api['verify_bindings']([binding])

    def test_current_receipt_paths_and_schema_are_present(self):
        directory=root/'output/playwright/html-to-riv'
        initial=api['read'](directory/'linear-gradient-public-lifecycle-tiled-r1/visual-public-gradient-transfer.json')
        api['verify_bindings'](initial['evidence'])
        paths=[Path(e['path']) for e in initial['evidence']]
        self.assertEqual(sum(p.name=='lifecycle.json' for p in paths),1)
        self.assertEqual(sum(p.name=='manifest.json' for p in paths),1)
        proof=api['read'](directory/'linear-gradient-composition-native-r3/composition-artifact-audit.json')
        self.assertEqual(proof['status'],'passed');self.assertEqual(proof['frames'],54)
        self.assertEqual(proof['replaySha256'],api['sha'](directory/'linear-gradient-composition-native-r3/replay.json'))

if __name__=='__main__':unittest.main()
