"""Unit tests for check_layout_style_handlers.py."""

from __future__ import annotations

import contextlib
import io
import pathlib
import subprocess
import tempfile
import unittest

import check_layout_style_handlers as tool


NOOP_VIRTUALS = "\n".join(
    f"    virtual void {name}Changed() {{}}" for name in sorted(tool.INHERITED_NOOP_HANDLERS)
)

GENERATED_BASE = f"""class LayoutComponentStyleBase : public LayoutSizingStyle {{
public:
    virtual void aaaChanged() {{}}
    virtual void bbbChanged() {{}}
    virtual void stylePropChanged() {{}}
    virtual void layoutTypeValueChanged() {{}}
{NOOP_VIRTUALS}
}};
"""

SIZING_BASE = """class LayoutSizingStyleBase : public Component {
public:
    virtual void cccChanged() {}
};
"""

CONCRETE_HPP = """class LayoutComponentStyle : public LayoutComponentStyleBase {
public:
    void aaaChanged() override;
    void bbbChanged() override;
    void cccChanged() override;
    void stylePropChanged() override;
    void layoutTypeValueChanged() override;
};
"""

CPP = """void LayoutComponentStyle::aaaChanged() { markLayoutNodeDirty(); }
void LayoutComponentStyle::bbbChanged()
{
    markLayoutNodeDirty();
}
void LayoutComponentStyle::cccChanged() { markLayoutNodeDirty(); }
void LayoutComponentStyle::stylePropChanged() { markLayoutStyleDirty(); }
void LayoutComponentStyle::layoutTypeValueChanged()
{
#ifdef WITH_RIVE_LAYOUT
    if (parent()->is<LayoutComponent>())
    {
        parent()->as<LayoutComponent>()->layoutTypeChanged();
    }
#endif
}
"""

RUST_MODULE = """const NODE_DIRTY_PROPERTIES: &[&str] = &[
    "aaa",
    "bbb",
    "ccc",
    "layoutTypeValue",
];

const STYLE_DIRTY_PROPERTIES: &[&str] = &[
    "styleProp",
];

fn layout_type_changed() {}

fn dispatch() {
    let _ = "layoutTypeValue";
}
"""


class CheckLayoutStyleHandlersTest(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self._tmp.cleanup)
        root = pathlib.Path(self._tmp.name)

        self.upstream = root / "rive-runtime"
        for relative, content in (
            (tool.UPSTREAM_GENERATED_HPPS[0], GENERATED_BASE),
            (tool.UPSTREAM_GENERATED_HPPS[1], SIZING_BASE),
            (tool.UPSTREAM_CONCRETE_HPP, CONCRETE_HPP),
            (tool.UPSTREAM_CPP, CPP),
        ):
            self.write_upstream(relative, content)
        for command in (
            ["git", "init", "--quiet"],
            ["git", "-c", "user.email=t@t", "-c", "user.name=t", "commit", "--quiet", "--allow-empty", "-m", "pin"],
        ):
            subprocess.run(command, cwd=self.upstream, check=True, capture_output=True)
        self.ref = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=self.upstream,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()

        self.repo = root / "repo"
        self.write_repo(tool.RUST_STYLE_MODULE, RUST_MODULE)
        self.manifest = self.repo / "file-correspondence-manifest.toml"
        self.manifest.write_text(f'upstream_ref = "{self.ref}"\n')

    def write_upstream(self, relative: str, content: str) -> None:
        path = self.upstream / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)

    def write_repo(self, relative: str, content: str) -> None:
        path = self.repo / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)

    def run_check(self) -> tuple[int, str, str]:
        stdout, stderr = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            code = tool.main(
                [
                    "--repo-root",
                    str(self.repo),
                    "--rive-runtime-dir",
                    str(self.upstream),
                    "--file-manifest",
                    str(self.manifest),
                ]
            )
        return code, stdout.getvalue(), stderr.getvalue()

    def test_green_path_passes(self) -> None:
        code, stdout, stderr = self.run_check()
        self.assertEqual(code, 0, stderr)
        self.assertIn("layout-style-handlers: overrides=5", stdout)

    def test_pin_mismatch_fails(self) -> None:
        self.manifest.write_text(f'upstream_ref = "{"0" * 40}"\n')
        code, _, stderr = self.run_check()
        self.assertEqual(code, 1)
        self.assertIn("pins " + "0" * 40, stderr)

    def test_malformed_pin_fails(self) -> None:
        self.manifest.write_text('upstream_ref = "not-a-sha"\n')
        code, _, stderr = self.run_check()
        self.assertEqual(code, 1)
        self.assertIn("not a 40-hex commit", stderr)

    def test_node_handler_missing_from_table_fails(self) -> None:
        self.write_repo(
            tool.RUST_STYLE_MODULE, RUST_MODULE.replace('    "bbb",\n', "")
        )
        code, _, stderr = self.run_check()
        self.assertEqual(code, 1)
        self.assertIn("bbbChanged routes markLayoutNodeDirty", stderr)

    def test_style_handler_missing_from_table_fails(self) -> None:
        self.write_repo(
            tool.RUST_STYLE_MODULE, RUST_MODULE.replace('    "styleProp",\n', "")
        )
        code, _, stderr = self.run_check()
        self.assertEqual(code, 1)
        self.assertIn("stylePropChanged routes markLayoutStyleDirty", stderr)

    def test_stale_node_entry_fails(self) -> None:
        self.write_repo(
            tool.RUST_STYLE_MODULE,
            RUST_MODULE.replace('    "aaa",', '    "aaa",\n    "gone",'),
        )
        code, _, stderr = self.run_check()
        self.assertEqual(code, 1)
        self.assertIn("NODE_DIRTY_PROPERTIES entry gone", stderr)

    def test_duplicate_node_entry_fails(self) -> None:
        self.write_repo(
            tool.RUST_STYLE_MODULE,
            RUST_MODULE.replace('    "aaa",', '    "aaa",\n    "aaa",'),
        )
        code, _, stderr = self.run_check()
        self.assertEqual(code, 1)
        self.assertIn("duplicate entries: aaa", stderr)

    def test_untriaged_new_virtual_fails(self) -> None:
        self.write_upstream(
            tool.UPSTREAM_GENERATED_HPPS[1],
            SIZING_BASE.replace(
                "    virtual void cccChanged() {}",
                "    virtual void cccChanged() {}\n    virtual void newPropChanged() {}",
            ),
        )
        code, _, stderr = self.run_check()
        self.assertEqual(code, 1)
        self.assertIn("newPropChanged with no LayoutComponentStyle override", stderr)

    def test_unrecognized_route_fails(self) -> None:
        self.write_upstream(
            tool.UPSTREAM_CONCRETE_HPP,
            CONCRETE_HPP.replace(
                "    void aaaChanged() override;",
                "    void aaaChanged() override;\n    void animationStyleTypeChanged() override;",
            ),
        )
        self.write_upstream(
            tool.UPSTREAM_CPP,
            CPP + "void LayoutComponentStyle::animationStyleTypeChanged() { special(); }\n",
        )
        code, _, stderr = self.run_check()
        self.assertEqual(code, 1)
        self.assertIn(
            "INHERITED_NOOP_HANDLERS lists animationStyleType but "
            "LayoutComponentStyle now overrides it",
            stderr,
        )

    def test_new_bespoke_handler_without_route_fails(self) -> None:
        self.write_upstream(
            tool.UPSTREAM_GENERATED_HPPS[0],
            GENERATED_BASE.replace(
                "    virtual void aaaChanged() {}",
                "    virtual void aaaChanged() {}\n    virtual void oddPropChanged() {}",
            ),
        )
        self.write_upstream(
            tool.UPSTREAM_CONCRETE_HPP,
            CONCRETE_HPP.replace(
                "    void aaaChanged() override;",
                "    void aaaChanged() override;\n    void oddPropChanged() override;",
            ),
        )
        self.write_upstream(
            tool.UPSTREAM_CPP,
            CPP + "void LayoutComponentStyle::oddPropChanged() { special(); }\n",
        )
        code, _, stderr = self.run_check()
        self.assertEqual(code, 1)
        self.assertIn("oddPropChanged has no recognized route", stderr)

    def test_missing_bespoke_marker_fails(self) -> None:
        self.write_repo(
            tool.RUST_STYLE_MODULE,
            RUST_MODULE.replace("fn layout_type_changed() {}\n", ""),
        )
        code, _, stderr = self.run_check()
        self.assertEqual(code, 1)
        self.assertIn(
            "bespoke handler layoutTypeValueChanged requires marker "
            "'fn layout_type_changed'",
            stderr,
        )

    def test_override_without_definition_fails(self) -> None:
        self.write_upstream(
            tool.UPSTREAM_CPP,
            CPP.replace(
                "void LayoutComponentStyle::cccChanged() { markLayoutNodeDirty(); }\n",
                "",
            ),
        )
        code, _, stderr = self.run_check()
        self.assertEqual(code, 1)
        self.assertIn("does not define the cccChanged override", stderr)

    def test_missing_rust_table_fails(self) -> None:
        self.write_repo(
            tool.RUST_STYLE_MODULE,
            RUST_MODULE.replace("NODE_DIRTY_PROPERTIES", "RENAMED_PROPERTIES"),
        )
        code, _, stderr = self.run_check()
        self.assertEqual(code, 1)
        self.assertIn("no longer defines NODE_DIRTY_PROPERTIES", stderr)


if __name__ == "__main__":
    unittest.main()
