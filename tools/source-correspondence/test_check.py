import importlib.util
import pathlib
import tempfile
import textwrap
import unittest


TOOL = pathlib.Path(__file__).with_name("check.py")
SPEC = importlib.util.spec_from_file_location("source_correspondence_check", TOOL)
assert SPEC is not None and SPEC.loader is not None
CHECK = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECK)


class SourceCorrespondenceCheckTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)
        self.owner = self.root / "crates/runtime/src/foo.rs"
        self.owner.parent.mkdir(parents=True)
        self.owner.write_text("// owner\n")
        self.manifest = self.root / "manifest.toml"

    def write_manifest(self, body: str, *, direct: int = 1, shared: int = 0, explicit: int = 0, pending: int = 0, exceptions: str = "") -> None:
        self.manifest.write_text(
            textwrap.dedent(
                f"""\
                [source_correspondence_ratchet]
                applicable_rows = {direct + shared + explicit}
                direct_primary_owner_rows = {direct}
                adjudicated_shared_owner_rows = {shared}
                explicit_owner_exception_rows = {explicit}
                pending_rows = {pending}
                explicit_owner_exceptions = [{exceptions}]

                {body}
                """
            )
        )

    def run_check(self):
        return CHECK.check(self.root, self.manifest)

    def test_direct_primary_owner_passes(self) -> None:
        self.write_manifest(
            """\
            [[file]]
            upstream = "src/foo.cpp"
            status = "faithful"
            rust_module = "crates/runtime/src/foo.rs"
            note = ""
            """
        )
        errors, counts = self.run_check()
        self.assertEqual(errors, [])
        self.assertEqual(counts["direct_primary_owner_rows"], 1)

    def test_duplicate_primary_owner_fails_bijection(self) -> None:
        body = """\
        [[file]]
        upstream = "src/a/foo.cpp"
        status = "faithful"
        rust_module = "crates/runtime/src/foo.rs"
        note = ""

        [[file]]
        upstream = "src/b/foo.cpp"
        status = "faithful"
        rust_module = "crates/runtime/src/foo.rs"
        note = ""
        """
        self.write_manifest(body, direct=2)
        errors, _ = self.run_check()
        self.assertTrue(any("not bijective" in error for error in errors))

    def test_unadjudicated_packed_owner_fails(self) -> None:
        self.write_manifest(
            """\
            [[file]]
            upstream = "src/bar.cpp"
            status = "faithful"
            rust_module = "crates/runtime/src/foo.rs"
            note = ""
            """,
            direct=0,
        )
        errors, _ = self.run_check()
        self.assertTrue(any("no direct primary owner" in error for error in errors))

    def test_shared_owner_marker_is_counted(self) -> None:
        self.write_manifest(
            """\
            [[file]]
            upstream = "src/bar.cpp"
            status = "faithful"
            rust_module = "crates/runtime/src/foo.rs"
            note = "MR-3 exception: shared safe-Rust representation"
            """,
            direct=0,
            shared=1,
        )
        errors, _ = self.run_check()
        self.assertEqual(errors, [])

    def test_pending_row_must_not_claim_an_owner(self) -> None:
        self.write_manifest(
            """\
            [[file]]
            upstream = "src/bar.cpp"
            status = "pending"
            rust_module = "crates/runtime/src/foo.rs"
            note = ""
            """,
            direct=0,
            pending=1,
        )
        errors, _ = self.run_check()
        self.assertTrue(any("pending row declares" in error for error in errors))
