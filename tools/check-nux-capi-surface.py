#!/usr/bin/env python3
"""Fail closed when product/SDK semantics enter the shipped C interface."""

from __future__ import annotations

import argparse
import pathlib
import re
import sys


CONTRACT_FILES = (
    "include/nux_capi.generated.h",
    "include/nux_capi.h",
    "include/nux_capi_apple.h",
    "include/module.modulemap",
    "exports-v3-portable.txt",
    "exports-v3-apple-metal-extension.txt",
    "exports-v3-android-vulkan-extension.txt",
)
IDENTIFIER = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
COMMENTS = re.compile(r"/\*.*?\*/|//[^\r\n]*", re.DOTALL)
RETIRED_SEMANTICS = re.compile(
    r"(?i)(?:"
    r"experience|"
    r"flow_?session|"
    r"journey|"
    r"nuxie_?host_?command|"
    r"nuxie_?(?:host_?module|script_?host)|"
    r"package_?(?:auth|authentication)|"
    r"product(?!ion)|"
    r"session|"
    r"package|"
    r"authentication|"
    r"response_?set|"
    r"sdk_?session|"
    r"screen_?(?:context|session|operation|output|player|schema|value)"
    r")"
)


def strip_comments(source: str) -> str:
    """Blank comments while retaining offsets and line numbers for diagnostics."""

    return COMMENTS.sub(
        lambda match: "".join(
            "\n" if character == "\n" else " " for character in match.group(0)
        ),
        source,
    )


def check_contract(root: pathlib.Path) -> list[str]:
    errors: list[str] = []
    for relative in CONTRACT_FILES:
        path = root / relative
        if not path.is_file():
            errors.append(f"{relative}: required shipped-interface input is missing")
            continue
        source = strip_comments(path.read_text())
        for match in IDENTIFIER.finditer(source):
            identifier = match.group(0)
            if RETIRED_SEMANTICS.search(identifier) is None:
                continue
            line = source.count("\n", 0, match.start()) + 1
            errors.append(
                f"{relative}:{line}: retired product/SDK semantic identifier {identifier!r}"
            )
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--contract-root",
        type=pathlib.Path,
        default=pathlib.Path(__file__).resolve().parents[1] / "crates/nux-capi",
    )
    arguments = parser.parse_args()
    errors = check_contract(arguments.contract_root.resolve())
    if errors:
        for error in errors:
            print(f"nux-capi surface guard failed: {error}", file=sys.stderr)
        return 1
    print("nux-capi shipped surface contains only runtime/platform semantics")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
