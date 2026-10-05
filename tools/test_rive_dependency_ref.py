"""C++ oracle adapters must consume the public runtime's exact yoga.ref."""
import pathlib
import shutil
import subprocess
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parent.parent


class YogaRefTests(unittest.TestCase):
    def check_resolvers(self, ref, present, expected_success):
        with tempfile.TemporaryDirectory() as directory:
            runtime = pathlib.Path(directory)
            deps = runtime / "dependencies"
            deps.mkdir()
            (deps / "premake5_yoga_v2.lua").write_text(
                "local ref = io.readfile(path.join(path.getdirectory(_SCRIPT), 'yoga.ref'))\n"
                "yoga = dependency.github('rive-app/yoga', (ref:gsub('%s', '')))\n"
            )
            if ref is not None:
                (deps / "yoga.ref").write_text(ref)
            exact = deps / "rive-app_yoga_rive_yoga_444312e4eb55"
            if present:
                exact.mkdir()
            # An older downloaded dependency must never rescue a missing pin.
            stale = deps / "rive-app_yoga_rive_changes_v2_0_1_4_grid"
            stale.mkdir()
            commands = [[
                "bash", "-c",
                'source "$1"; rive_dependency_dir "$2" rive-app/yoga dependencies/premake5_yoga_v2.lua',
                "test", str(ROOT / "tools/build-support/rive_dependency_dir.sh"), str(runtime),
            ]]
            lua = shutil.which("lua")
            self.assertIsNotNone(lua, "Lua is required to test the C++ oracle resolver")
            commands.append([
                lua, "-e",
                "local helper,root,exact,stale=arg[1],arg[2],arg[3],arg[4]; "
                "os.host=function() return 'fixture' end; "
                "os.isdir=function(p) return os.rename(p,p) ~= nil end; "
                "os.matchdirs=function() return {stale} end; "
                "print(dofile(helper).resolver('test',root).dir('rive-app/yoga','dependencies/premake5_yoga_v2.lua','/*/yoga-*'))",
                "-", str(ROOT / "tools/build-support/rive_dependencies.lua"), str(runtime), str(exact), str(stale),
            ])
            for command in commands:
                with self.subTest(resolver=command[0], ref=ref, present=present):
                    result = subprocess.run(command, input="", text=True, capture_output=True)
                    self.assertEqual(result.returncode == 0, expected_success, result.stderr)
                    if expected_success:
                        self.assertEqual(result.stdout.strip(), str(exact))

    def test_exact_public_ref_with_trailing_newline(self):
        self.check_resolvers("rive_yoga_444312e4eb55\n", True, True)

    def test_missing_ref_does_not_select_old_checkout(self):
        self.check_resolvers(None, True, False)

    def test_empty_ref_does_not_select_old_checkout(self):
        self.check_resolvers(" \n", True, False)

    def test_missing_pinned_checkout_does_not_select_old_checkout(self):
        self.check_resolvers("rive_yoga_444312e4eb55\n", False, False)


if __name__ == "__main__":
    unittest.main()
