import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("inject_webgpu_imports.py")


class InjectWebGpuImportsTest(unittest.TestCase):
    def test_injects_custom_host_and_dynamic_wasm_bindgen_type_marker(self) -> None:
        source = """import * as import0 from \"env\"
/* @ts-self-types=\"./editor_product_host_webgpu.d.ts\" */
const imports = {
        \"env\": import0,
};
"""
        with tempfile.TemporaryDirectory() as directory:
            glue_path = Path(directory) / "editor_product_host_webgpu.js"
            glue_path.write_text(source)
            subprocess.run(
                [
                    sys.executable,
                    str(SCRIPT),
                    str(glue_path),
                    "./editor_product_host_webgpu_host.js",
                ],
                check=True,
            )
            injected = glue_path.read_text()

        self.assertNotIn('from "env"', injected)
        self.assertIn(
            'import { createWebGpuImports } from "./editor_product_host_webgpu_host.js";',
            injected,
        )
        self.assertIn('"env": createWebGpuImports(() => wasm)', injected)


if __name__ == "__main__":
    unittest.main()
