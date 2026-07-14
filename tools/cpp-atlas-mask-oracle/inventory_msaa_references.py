#!/usr/bin/env python3
"""Inventory gated MSAA corpus rows for strict Dawn reference capture."""

import argparse
import collections
import hashlib
import importlib.util
import json
import pathlib
import re
import sys
import tomllib


ROOT = pathlib.Path(__file__).resolve().parents[2]
STRICT_ROOT = pathlib.Path("fixtures/renderer/reference/dawn-webgpu-metal")
SOURCE_RE = re.compile(r'^source file="([^"]*)" artboard="([^"]*)" scene="([^"]*)"$')
FRAME_SIZE_RE = re.compile(r"^frameSize width=(\d+) height=(\d+)$")
CLEAR_COLOR_RE = re.compile(r"^clearColor value=(0x[0-9a-f]{8})$")


def load_generator():
    path = pathlib.Path(__file__).with_name("generate_path_stream_replay.py")
    spec = importlib.util.spec_from_file_location("strict_path_replay", path)
    if spec is None or spec.loader is None:
        raise RuntimeError("could not load strict path replay generator")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def command_counts(lines: list[str]) -> dict[str, int]:
    return dict(collections.Counter(line.partition(" ")[0] for line in lines))


def inspect_upstream_gm(entry: dict, generator) -> tuple[dict | None, str | None]:
    stream = ROOT / entry["stream"]
    lines = stream.read_text(encoding="utf-8").splitlines()
    source_indices = [index for index, line in enumerate(lines) if line.startswith("source file=")]
    if len(source_indices) != 1:
        return None, "strict-replay-gm-header"
    source_index = source_indices[0]
    if source_index + 2 >= len(lines):
        return None, "strict-replay-gm-header"
    source = SOURCE_RE.fullmatch(lines[source_index])
    frame_size = FRAME_SIZE_RE.fullmatch(lines[source_index + 1])
    clear_color = CLEAR_COLOR_RE.fullmatch(lines[source_index + 2])
    if source is None or frame_size is None or clear_color is None:
        return None, "strict-replay-gm-header"
    source_file, artboard, scene = source.groups()
    if artboard:
        return None, "strict-replay-gm-header"
    width, height = (int(value) for value in frame_size.groups())
    commands = lines[1:source_index] + lines[source_index + 3 : -1]
    case = {
        "id": entry["id"],
        "stream": entry["stream"],
        "sha256": hashlib.sha256(stream.read_bytes()).hexdigest(),
        "source": source_file,
        "scene": scene,
        "width": width,
        "height": height,
        "clear_color": clear_color.group(1),
        "counts": command_counts(commands),
    }
    try:
        generator.generate_include(
            stream=stream,
            profile="gm",
            expected_sha256=case["sha256"],
            expected_source=case["source"],
            expected_source_suffix=None,
            expected_artboard="",
            expected_scene=case["scene"],
            expected_width=case["width"],
            expected_height=case["height"],
            expected_clear_color=case["clear_color"],
            expected_sample_seconds=None,
            expected_counts=case["counts"],
            function_name="inventoryOnly",
            blend_mode_override=None,
        )
    except ValueError as error:
        message = str(error)
        if "makeLinearGradient" in message or "makeRadialGradient" in message:
            return None, "strict-replay-gradient-paint"
        if "makeRenderBuffer" in message:
            return None, "strict-replay-render-buffer"
        return None, f"strict-replay-compiler: {message}"
    return case, None


def has_strict_reference(entry: dict) -> bool:
    reference = pathlib.Path(entry["reference"])
    if STRICT_ROOT not in reference.parents:
        return False
    png = ROOT / reference
    return png.is_file() and png.with_suffix(".provenance").is_file()


def build_inventory(corpus_path: pathlib.Path, manifest_path: pathlib.Path) -> dict:
    with corpus_path.open("rb") as source:
        corpus = tomllib.load(source)["entry"]
    with manifest_path.open("rb") as source:
        manifest_cases = tomllib.load(source)["case"]
    manifest_ids = {case["id"] for case in manifest_cases}
    generator = load_generator()
    gated_msaa = [
        entry for entry in corpus if entry["mode"] == "msaa" and entry["status"] == "gated"
    ]
    rows = []
    for entry in gated_msaa:
        if has_strict_reference(entry):
            continue
        row = {
            "id": entry["id"],
            "kind": entry["kind"],
            "stream": entry["stream"],
            "reference": entry["reference"],
        }
        if entry["kind"] == "upstream-gm-stream":
            case, rejection = inspect_upstream_gm(entry, generator)
            if case is not None:
                row["result"] = "accepted"
                row["case"] = case
                if entry["id"] not in manifest_ids:
                    raise ValueError(
                        f"accepted strict case is missing from the capture manifest: {entry['id']}"
                    )
            else:
                row["result"] = "unsupported"
                row["reason"] = rejection
        elif entry["kind"] == "riv-stream":
            row["result"] = "unsupported"
            row["reason"] = "strict-replay-riv-frame-selection"
        else:
            row["result"] = "unsupported"
            row["reason"] = "strict-replay-gm-header"
        rows.append(row)
    results = collections.Counter(row["result"] for row in rows)
    reasons = collections.Counter(
        row["reason"] for row in rows if row["result"] == "unsupported"
    )
    return {
        "version": 1,
        "summary": {
            "gated_msaa_rows": len(gated_msaa),
            "strict_provenance_rows": len(gated_msaa) - len(rows),
            "missing_strict_provenance_rows": len(rows),
            "accepted": results["accepted"],
            "unsupported": results["unsupported"],
            "unsupported_by_reason": dict(sorted(reasons.items())),
            "capture_manifest_cases": len(manifest_cases),
        },
        "entry": rows,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--corpus", type=pathlib.Path, default=ROOT / "corpus-r.toml")
    parser.add_argument(
        "--manifest",
        type=pathlib.Path,
        default=pathlib.Path(__file__).with_name("msaa-reference-corpus.toml"),
    )
    parser.add_argument("--output", type=pathlib.Path)
    parser.add_argument("--check", type=pathlib.Path)
    args = parser.parse_args()
    if bool(args.output) == bool(args.check):
        parser.error("specify exactly one of --output or --check")
    rendered = json.dumps(
        build_inventory(args.corpus, args.manifest), indent=2, sort_keys=True
    ) + "\n"
    if args.output:
        args.output.write_text(rendered, encoding="utf-8")
    else:
        if args.check.read_text(encoding="utf-8") != rendered:
            print(f"inventory drifted: {args.check}", file=sys.stderr)
            return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
