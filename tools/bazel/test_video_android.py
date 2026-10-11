import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import tempfile
import unittest
import zipfile


class AndroidVideoPublicationTest(unittest.TestCase):
    @unittest.skipUnless(sys.platform != "win32" and all(shutil.which(tool) for tool in ("bash", "install", "zip")),
                         "Android proof shell fixture requires POSIX permissions and build tools")
    def test_repeat_prepare_keeps_cache_immutable_and_replaces_private_library(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "runtime fixture with spaces"
            proof = root / "tools/video-qualification/android"
            proof.mkdir(parents=True)
            script = proof / "build.sh"
            shutil.copyfile(Path(__file__).resolve().parents[1] / "video-qualification/android/build.sh", script)
            fixtures = root / "fixtures/video"
            fixtures.mkdir(parents=True)
            for name in ("red-blue-720p.mp4", "red-blue-sync.mp4", "red-blue-audio.mp4", "parity-opaque.mp4"):
                (fixtures / name).write_bytes(("fixture " + name).encode())
            (proof / "AndroidManifest.xml").write_text("<manifest />")
            cache = root / "shared-action-cache"
            cache.mkdir()
            products = [cache / "first.so", cache / "changed.so"]
            for product, content in zip(products, (b"first immutable product", b"changed immutable product")):
                product.write_bytes(content)
                product.chmod(0o555)
            original_cache = [product.read_bytes() for product in products]
            selected = root / "selected-product"
            frontend = root / "tools/bazel/runtime.py"
            frontend.parent.mkdir()
            frontend.write_text("""from pathlib import Path
import shutil
root = Path(__file__).resolve().parents[2]
source = Path((root / 'selected-product').read_text())
destination = root / 'target/aarch64-linux-android/debug/libvideo_qualification.so'
destination.parent.mkdir(parents=True, exist_ok=True)
destination.unlink(missing_ok=True)
shutil.copy2(source, destination)
""")
            tools = root / "fake-tools"
            tools.mkdir()
            dispatcher = tools / "tool-fixture"
            dispatcher.write_text("#!" + sys.executable + "\n" + """from pathlib import Path
import shutil, sys, zipfile
name = Path(sys.argv[0]).name
args = sys.argv[1:]
if name == 'jar':
    Path(args[1]).write_bytes(b'fixture jar')
elif name == 'd8':
    (Path(args[args.index('--output') + 1]) / 'classes.dex').write_bytes(b'fixture dex')
elif name == 'llvm-strip':
    product = Path(args[-1])
    product.write_bytes(product.read_bytes() + b' postprocessed')
elif name == 'aapt2':
    with zipfile.ZipFile(args[args.index('-o') + 1], 'w') as archive:
        archive.writestr('AndroidManifest.xml', b'fixture manifest')
        for fixture in Path(args[args.index('-A') + 1]).iterdir():
            archive.write(fixture, 'assets/' + fixture.name)
elif name == 'zipalign':
    shutil.copyfile(args[-2], args[-1])
elif name == 'apksigner':
    shutil.copyfile(args[-1], args[args.index('--out') + 1])
elif name == 'keytool':
    Path(args[args.index('-keystore') + 1]).write_bytes(b'fixture debug key')
elif name != 'javac':
    raise AssertionError('unexpected fixture tool: ' + name)
""")
            dispatcher.chmod(0o755)
            for name in ("javac", "jar", "keytool"):
                (tools / name).symlink_to(dispatcher)
            (tools / "python3").symlink_to(sys.executable)
            sdk = root / "Android SDK"
            buildtools = sdk / "build-tools/36.1.0"
            buildtools.mkdir(parents=True)
            for name in ("d8", "aapt2", "zipalign", "apksigner"):
                (buildtools / name).symlink_to(dispatcher)
            ndk = sdk / "ndk/29.0.14206865"
            toolchain = ndk / "toolchains/llvm/prebuilt/darwin-x86_64"
            (toolchain / "bin").mkdir(parents=True)
            (toolchain / "bin/llvm-strip").symlink_to(dispatcher)
            libcxx = toolchain / "sysroot/usr/lib/aarch64-linux-android/libc++_shared.so"
            libcxx.parent.mkdir(parents=True)
            libcxx.write_bytes(b"fixture libc++")
            libcxx.chmod(0o755)
            private = root / "target/video-android-proof/lib/arm64-v8a/libvideo_qualification.so"
            private.parent.mkdir(parents=True)
            private.write_bytes(b"previous read-only proof library")
            private.chmod(0o555)
            environment = dict(os.environ, PATH=str(tools) + os.pathsep + os.defpath,
                               ANDROID_SDK_ROOT=str(sdk), ANDROID_NDK_HOME=str(ndk), VIDEO_PROOF_PROFILE="dev")
            for product in (products[0], products[0], products[1]):
                with self.subTest(product=product.name):
                    selected.write_text(str(product))
                    result = subprocess.run(["bash", str(script)], cwd=root, env=environment,
                                            stdin=subprocess.DEVNULL, text=True, capture_output=True)
                    self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                    self.assertEqual(private.read_bytes(), product.read_bytes() + b" postprocessed")
                    self.assertEqual(stat.S_IMODE(private.stat().st_mode), 0o755)
                    self.assertEqual([item.read_bytes() for item in products], original_cache)
                    self.assertEqual([stat.S_IMODE(item.stat().st_mode) for item in products], [0o555, 0o555])
                    self.assertFalse(private.is_symlink())
                    with zipfile.ZipFile(root / "target/video-android-proof/video-proof.apk") as archive:
                        self.assertEqual(archive.read("lib/arm64-v8a/libvideo_qualification.so"), private.read_bytes())
                        for fixture in fixtures.iterdir():
                            self.assertEqual(archive.read("assets/" + fixture.name), fixture.read_bytes())


if __name__ == "__main__":
    unittest.main()
