#!/usr/bin/env python3
"""Record or verify the independently committed Apple LP64 C ABI layout."""

from __future__ import annotations

import argparse
import json
import pathlib
import re
import subprocess
import sys
import tempfile

REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(REPO_ROOT))

from tools.apple_runtime_contract import ContractError, validate_layout_oracle


def public_structs(header: str) -> dict[str, list[str]]:
    structs: dict[str, list[str]] = {}
    for name, body in re.findall(
        r"typedef struct (Nux\w+)\s*\{(.*?)\}\s*\1\s*;", header, re.DOTALL
    ):
        body = re.sub(r"/\*.*?\*/", "", body, flags=re.DOTALL)
        fields: list[str] = []
        for declaration in body.split(";"):
            declaration = declaration.strip()
            if not declaration:
                continue
            match = re.search(r"\(\*([A-Za-z_]\w*)\)", declaration)
            if match is None:
                match = re.search(
                    r"([A-Za-z_]\w*)\s*(?:\[[^]]*\])?\s*$", declaration
                )
            if match is None:
                raise ContractError(
                    f"cannot identify a field in public struct {name}: {declaration}"
                )
            fields.append(match.group(1))
        structs[name] = fields
    if not structs:
        raise ContractError("generated header declares no public value structs")
    return structs


def compile_probe(source: str, *, run: bool) -> str:
    include = REPO_ROOT / "crates/nux-capi/include"
    with tempfile.TemporaryDirectory(prefix="nux-capi-layout-") as directory:
        root = pathlib.Path(directory)
        source_path = root / "layout.c"
        output_path = root / "layout"
        source_path.write_text(source, encoding="utf-8")
        command = [
            "xcrun",
            "clang",
            "-DNUX_CAPI_APPLE_METAL",
            "-std=c11",
            "-Wall",
            "-Wextra",
            "-Werror",
            f"-I{include}",
            str(source_path),
        ]
        if run:
            command.extend(["-o", str(output_path)])
        else:
            command.append("-fsyntax-only")
        subprocess.run(command, check=True)
        if not run:
            return ""
        return subprocess.run(
            [str(output_path)], check=True, capture_output=True, text=True
        ).stdout


def record_layout(structs: dict[str, list[str]]) -> dict[str, object]:
    lines = [
        "#include <stddef.h>",
        "#include <stdio.h>",
        '#include "nux_capi_apple.h"',
        "int main(void) {",
    ]
    for name, fields in sorted(structs.items()):
        lines.append(
            f'printf("TYPE\\t{name}\\t%zu\\t%zu\\n", sizeof({name}), _Alignof({name}));'
        )
        for field in fields:
            lines.append(
                f'printf("FIELD\\t{name}\\t{field}\\t%zu\\n", offsetof({name}, {field}));'
            )
    lines.extend(["return 0;", "}"])
    output = compile_probe("\n".join(lines) + "\n", run=True)
    records: dict[str, dict[str, object]] = {}
    for line in output.splitlines():
        parts = line.split("\t")
        if parts[0] == "TYPE" and len(parts) == 4:
            records[parts[1]] = {
                "name": parts[1],
                "size": int(parts[2]),
                "alignment": int(parts[3]),
                "fields": [],
            }
        elif parts[0] == "FIELD" and len(parts) == 4:
            records[parts[1]]["fields"].append(
                {"name": parts[2], "offset": int(parts[3])}
            )
        else:
            raise ContractError(f"layout probe emitted malformed output: {line}")
    return {
        "schemaVersion": 1,
        "dataModel": "apple-lp64",
        "types": [records[name] for name in sorted(records)],
    }


def verify_layout(
    structs: dict[str, list[str]], oracle: dict[str, object]
) -> None:
    validate_layout_oracle(oracle)
    records = {record["name"]: record for record in oracle["types"]}
    if set(records) != set(structs):
        missing = sorted(set(structs) - set(records))
        extra = sorted(set(records) - set(structs))
        raise ContractError(
            f"layout oracle type set differs: missing={missing}, extra={extra}"
        )
    lines = [
        "#include <stddef.h>",
        '#include "nux_capi_apple.h"',
    ]
    for name, fields in sorted(structs.items()):
        record = records[name]
        recorded_fields = [field["name"] for field in record["fields"]]
        if recorded_fields != fields:
            raise ContractError(
                f"layout oracle fields differ for {name}: "
                f"expected={fields}, recorded={recorded_fields}"
            )
        lines.append(
            f'_Static_assert(sizeof({name}) == {record["size"]}, "{name} size");'
        )
        lines.append(
            f'_Static_assert(_Alignof({name}) == {record["alignment"]}, "{name} alignment");'
        )
        for field in record["fields"]:
            lines.append(
                f'_Static_assert(offsetof({name}, {field["name"]}) == '
                f'{field["offset"]}, "{name}.{field["name"]} offset");'
            )
    compile_probe("\n".join(lines) + "\n", run=False)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--record", action="store_true")
    parser.add_argument(
        "--header",
        type=pathlib.Path,
        default=REPO_ROOT / "crates/nux-capi/include/nux_capi.generated.h",
    )
    parser.add_argument(
        "--oracle",
        type=pathlib.Path,
        default=REPO_ROOT / "crates/nux-capi/abi-layout-v3.json",
    )
    arguments = parser.parse_args()
    structs = public_structs(arguments.header.read_text(encoding="utf-8"))
    if arguments.record:
        print(json.dumps(record_layout(structs), indent=2))
        return 0
    oracle = json.loads(arguments.oracle.read_text(encoding="utf-8"))
    verify_layout(structs, oracle)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (ContractError, OSError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"nux-capi-layout: {error}") from error
