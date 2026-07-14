#!/usr/bin/env python3
"""Inventory gated MSAA corpus rows for strict Dawn reference capture."""

import argparse
import collections
import hashlib
import importlib.util
import json
import pathlib
import re
import struct
import sys
import tomllib


sys.dont_write_bytecode = True
ROOT = pathlib.Path(__file__).resolve().parents[2]
STRICT_ROOT = pathlib.Path("fixtures/renderer/reference/dawn-webgpu-metal")
SOURCE_RE = re.compile(r'^source file="([^"]*)" artboard="([^"]*)" scene="([^"]*)"$')
FRAME_SIZE_RE = re.compile(r"^frameSize width=(\d+) height=(\d+)$")
CLEAR_COLOR_RE = re.compile(r"^clearColor value=(0x[0-9a-f]{8})$")
EXPECTED_RUNTIME_REVISION = "7c778d13c5d903b3b74eec1dd6bb68a811dea5f2"
EXPECTED_DAWN_REVISION = "211333b2e3e429c3508f25c81c547f602adf448c"
EXPECTED_ADAPTER = {
    "adapter_vendor": "apple",
    "adapter_architecture": "metal-3",
    "adapter_device": "Apple M5 Max",
    "adapter_description": "Metal driver on macOS Version 26.4.1 (Build 25E253)",
    "adapter_vendor_id": "4203",
    "adapter_device_id": "0",
}


def load_module(name: str, filename: str):
    path = pathlib.Path(__file__).with_name(filename)
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"could not load {filename}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def load_generator():
    return load_module("strict_path_replay", "generate_path_stream_replay.py")


def load_capture_validator():
    return load_module("strict_capture_validator", "capture_msaa_references.py")


def load_registry_generator():
    return load_module("strict_registry_generator", "generate_msaa_reference_registry.py")


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


def inspect_riv(entry: dict, generator) -> tuple[dict | None, str | None]:
    stream = ROOT / entry["stream"]
    lines = stream.read_text(encoding="utf-8").splitlines()
    frame_index = entry.get("frame", 0)
    try:
        selection = generator.select_riv_frame(lines, frame_index)
        source_marker = "tests/unit_tests/assets/"
        if source_marker not in selection.source_file:
            raise ValueError("RIV source path has no stable asset suffix")
        source_suffix = source_marker + selection.source_file.partition(source_marker)[2]
        case = {
            "id": entry["id"],
            "stream": entry["stream"],
            "sha256": hashlib.sha256(stream.read_bytes()).hexdigest(),
            "source": source_suffix,
            "scene": selection.scene,
            "width": selection.width,
            "height": selection.height,
            "clear_color": "0x00000000",
            "counts": command_counts(selection.command_lines),
            "profile": "riv",
            "artboard": selection.artboard,
            "sample_seconds": selection.sample_seconds,
            "frame": frame_index,
        }
        generator.generate_include(
            stream=stream,
            profile="riv",
            expected_sha256=case["sha256"],
            expected_source=None,
            expected_source_suffix=case["source"],
            expected_artboard=case["artboard"],
            expected_scene=case["scene"],
            expected_width=case["width"],
            expected_height=case["height"],
            expected_clear_color=None,
            expected_sample_seconds=case["sample_seconds"],
            expected_counts=case["counts"],
            function_name="inventoryOnly",
            blend_mode_override=None,
            frame_index=frame_index,
        )
    except ValueError as error:
        message = str(error)
        if "makeLinearGradient" in message or "makeRadialGradient" in message:
            return None, "strict-replay-gradient-paint"
        if any(
            command in message
            for command in ("makeRenderBuffer", "bufferData", "drawImageMesh")
        ):
            return None, "strict-replay-render-buffer"
        return None, f"strict-replay-compiler: {message}"
    return case, None


def registry_sha256(manifest_path: pathlib.Path, registry_generator) -> str:
    _, digest = registry_generator.generate_registry(
        manifest_path,
        ROOT,
        EXPECTED_RUNTIME_REVISION,
        EXPECTED_DAWN_REVISION,
    )
    return digest


def canonical_riveabl_sha256(width: int, height: int, pixels: bytes, capture) -> str:
    artifact = (
        capture.RIVEABL_MAGIC
        + struct.pack("<III", 1, width, height)
        + pixels
    )
    return hashlib.sha256(artifact).hexdigest()


def validate_strict_reference(
    png: pathlib.Path,
    case: dict,
    expected_registry_sha256: str,
    capture,
) -> None:
    provenance = png.with_suffix(".provenance")
    _, fields = capture.parse_provenance(provenance)
    expected_keys = set(capture.UPSTREAM_PROVENANCE_KEYS) | set(
        capture.COORDINATOR_PROVENANCE_KEYS
    )
    if set(fields) != expected_keys:
        missing = sorted(expected_keys - set(fields))
        extra = sorted(set(fields) - expected_keys)
        raise ValueError(
            f"strict provenance schema mismatch: missing={missing} extra={extra}"
        )
    expected_fields = {
        "backend": "metal",
        "renderer_implementation": "cpp-dawn-webgpu",
        **EXPECTED_ADAPTER,
        "runtime_revision": EXPECTED_RUNTIME_REVISION,
        "dawn_revision": EXPECTED_DAWN_REVISION,
        "registry_sha256": expected_registry_sha256,
        "case_id": case["id"],
        "stream_sha256": case["sha256"],
        "frame_width": str(case["width"]),
        "frame_height": str(case["height"]),
        "sample_count": "4",
    }
    for key, expected in expected_fields.items():
        if fields[key] != expected:
            raise ValueError(
                f"strict provenance {key} mismatch: expected {expected}, got {fields[key]}"
            )
    for key in ("artifact_sha256", "png_sha256"):
        if capture.SHA256_RE.fullmatch(fields[key]) is None:
            raise ValueError(f"strict provenance {key} is not a canonical SHA-256")
    actual_png_sha256 = capture.sha256_file(png)
    if fields["png_sha256"] != actual_png_sha256:
        raise ValueError(
            "strict provenance png_sha256 mismatch: "
            f"expected {actual_png_sha256}, got {fields['png_sha256']}"
        )
    pixels = capture.decode_png_rgba8(png, case["width"], case["height"])
    actual_artifact_sha256 = canonical_riveabl_sha256(
        case["width"], case["height"], pixels, capture
    )
    if fields["artifact_sha256"] != actual_artifact_sha256:
        raise ValueError(
            "strict provenance artifact_sha256 mismatch: "
            f"expected {actual_artifact_sha256}, got {fields['artifact_sha256']}"
        )


def has_strict_reference(
    entry: dict,
    manifest_cases: dict[str, dict],
    expected_registry_sha256: str,
    capture,
) -> bool:
    reference = pathlib.Path(entry["reference"])
    if STRICT_ROOT not in reference.parents:
        return False
    png = ROOT / reference
    case = manifest_cases.get(entry["id"])
    if (
        case is None
        or case["stream"] != entry["stream"]
        or png.name != f"{entry['id']}.png"
    ):
        return False
    try:
        validate_strict_reference(png, case, expected_registry_sha256, capture)
    except (OSError, UnicodeDecodeError, ValueError):
        return False
    return True


def require_capture_case(discovered: dict, manifest_cases: dict[str, dict]) -> None:
    expected = manifest_cases.get(discovered["id"])
    if expected is None:
        return
    if discovered != expected:
        differing_fields = sorted(
            key
            for key in set(discovered) | set(expected)
            if discovered.get(key) != expected.get(key)
        )
        raise ValueError(
            f"accepted strict case does not match capture manifest: {discovered['id']}; "
            f"fields={differing_fields}"
        )


def build_inventory(corpus_path: pathlib.Path, manifest_path: pathlib.Path) -> dict:
    with corpus_path.open("rb") as source:
        corpus = tomllib.load(source)["entry"]
    with manifest_path.open("rb") as source:
        manifest_cases = tomllib.load(source)["case"]
    manifest_cases_by_id = {case["id"]: case for case in manifest_cases}
    generator = load_generator()
    capture = load_capture_validator()
    registry_generator = load_registry_generator()
    expected_registry_sha256 = registry_sha256(manifest_path, registry_generator)
    gated_msaa = [
        entry for entry in corpus if entry["mode"] == "msaa" and entry["status"] == "gated"
    ]
    rows = []
    for entry in gated_msaa:
        if has_strict_reference(
            entry,
            manifest_cases_by_id,
            expected_registry_sha256,
            capture,
        ):
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
                require_capture_case(case, manifest_cases_by_id)
            else:
                row["result"] = "unsupported"
                row["reason"] = rejection
        elif entry["kind"] == "riv-stream":
            case, rejection = inspect_riv(entry, generator)
            if case is not None:
                row["result"] = "accepted"
                row["case"] = case
                require_capture_case(case, manifest_cases_by_id)
            else:
                row["result"] = "unsupported"
                row["reason"] = rejection
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


def render_manifest(cases: list[dict]) -> str:
    lines = ["version = 1", ""]
    base_fields = (
        "id",
        "stream",
        "sha256",
        "source",
        "scene",
        "width",
        "height",
        "clear_color",
    )
    riv_fields = ("profile", "artboard", "sample_seconds", "frame")
    for case in cases:
        lines.append("[[case]]")
        for field in base_fields:
            value = case[field]
            rendered = json.dumps(value) if isinstance(value, str) else str(value)
            lines.append(f"{field} = {rendered}")
        if case.get("profile", "gm") == "riv":
            for field in riv_fields:
                value = case[field]
                rendered = json.dumps(value) if isinstance(value, str) else str(value)
                lines.append(f"{field} = {rendered}")
        counts = ", ".join(
            f"{command} = {count}"
            for command, count in sorted(case["counts"].items())
        )
        lines.extend((f"counts = {{ {counts} }}", ""))
    return "\n".join(lines)


def sync_manifest(manifest_path: pathlib.Path, inventory: dict) -> None:
    with manifest_path.open("rb") as source:
        cases = tomllib.load(source)["case"]
    known_ids = {case["id"] for case in cases}
    for row in inventory["entry"]:
        if row["result"] == "accepted" and row["id"] not in known_ids:
            cases.append(row["case"])
            known_ids.add(row["id"])
    manifest_path.write_text(render_manifest(cases), encoding="utf-8")


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
    parser.add_argument("--sync-manifest", action="store_true")
    args = parser.parse_args()
    if sum((bool(args.output), bool(args.check), args.sync_manifest)) != 1:
        parser.error("specify exactly one of --output, --check, or --sync-manifest")
    inventory = build_inventory(args.corpus, args.manifest)
    if args.sync_manifest:
        sync_manifest(args.manifest, inventory)
        return 0
    rendered = json.dumps(inventory, indent=2, sort_keys=True) + "\n"
    if args.output:
        args.output.write_text(rendered, encoding="utf-8")
    else:
        if args.check.read_text(encoding="utf-8") != rendered:
            print(f"inventory drifted: {args.check}", file=sys.stderr)
            return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
