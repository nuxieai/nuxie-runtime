"""Independent product publication and execution-root lookup contracts."""

import importlib.util
import os
from pathlib import Path
import stat
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
spec = importlib.util.spec_from_file_location("browser_tools", Path(__file__).with_name("browser-tools.py"))
browser_tools = importlib.util.module_from_spec(spec)
spec.loader.exec_module(browser_tools)


class BrowserToolsTest(unittest.TestCase):
    def test_publishing_two_worktrees_preserves_shared_input_and_private_products(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "cached-tool"
            source.write_bytes(b"immutable tool bytes")
            source.chmod(0o555)
            first = root / "worktree one/bin/wasm-bindgen"
            second = root / "worktree two/bin/wasm-bindgen"
            browser_tools.publish(source, first)
            browser_tools.publish(source, second)
            self.assertEqual(source.read_bytes(), first.read_bytes())
            self.assertFalse(first.samefile(second))
            first.write_bytes(b"first checkout replacement")
            self.assertEqual(second.read_bytes(), b"immutable tool bytes")
            self.assertEqual(source.read_bytes(), b"immutable tool bytes")
            self.assertEqual(stat.S_IMODE(source.stat().st_mode), 0o555)

    def test_warm_publication_preserves_mtime(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "source"
            destination = root / "bin/tool"
            source.write_bytes(b"tool")
            browser_tools.publish(source, destination)
            before = destination.stat().st_mtime_ns
            browser_tools.publish(source, destination)
            self.assertEqual(before, destination.stat().st_mtime_ns)

    def test_symlinked_output_preserves_external_file(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "source"
            external = root / "external"
            destination = root / "tool"
            source.write_bytes(b"new tool")
            external.write_bytes(b"external bytes")
            destination.symlink_to(external)
            with self.assertRaisesRegex(ValueError, "symlinked"):
                browser_tools.publish(source, destination)
            self.assertEqual(external.read_bytes(), b"external bytes")

    def test_external_product_is_resolved_from_execution_root(self):
        with tempfile.TemporaryDirectory(prefix="tool lookup ' ") as directory:
            root = Path(directory)
            checkout = root / "checkout"
            checkout.mkdir()
            execution_root = root / "bazel execroot"
            product = execution_root / "external/pinned-tool/wasm-bindgen"
            product.parent.mkdir(parents=True)
            product.write_text("#!/bin/sh\necho 'wasm-bindgen 0.2.126'\n")
            product.chmod(0o555)
            frontend = root / "bazel.py"
            frontend.write_text('''import sys
from pathlib import Path
if sys.argv[1] == "cquery":
    print("external/pinned-tool/wasm-bindgen")
elif sys.argv[1] == "info":
    print(Path(__file__).parent / "bazel execroot")
''')
            output = checkout / "tools output"
            with patch.object(browser_tools, "ROOT", checkout), \
                 patch.object(browser_tools, "bazel_command", return_value=[sys.executable, str(frontend)]), \
                 patch.object(sys, "argv", ["browser-tools.py", "--output", str(output)]):
                browser_tools.main()
            self.assertEqual((output / "bin/wasm-bindgen").read_bytes(), product.read_bytes())
            self.assertEqual(stat.S_IMODE(product.stat().st_mode), 0o555)


if __name__ == "__main__":
    unittest.main()
