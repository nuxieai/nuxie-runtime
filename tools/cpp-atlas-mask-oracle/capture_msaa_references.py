#!/usr/bin/env python3
"""Capture and validate the pinned C++ Dawn MSAA reference corpus."""

from __future__ import annotations

import argparse
import concurrent.futures
import ctypes
import errno
import hashlib
import math
import os
import pathlib
import re
import struct
import subprocess
import sys
import tempfile
import tomllib
import uuid
import zlib
from dataclasses import dataclass


RIVEABL_MAGIC = b"RIVEABL\0"
RIVEABL_HEADER_BYTES = 20
PNG_MAGIC = b"\x89PNG\r\n\x1a\n"
GM_CASE_KEYS = frozenset(
    {
        "id",
        "stream",
        "sha256",
        "source",
        "scene",
        "width",
        "height",
        "clear_color",
        "counts",
    }
)
RIV_CASE_KEYS = GM_CASE_KEYS | frozenset(
    {"profile", "artboard", "sample_seconds", "frame"}
)
CASE_ID_RE = re.compile(r"[A-Za-z0-9_-]+")
SHA256_RE = re.compile(r"[0-9a-f]{64}")
REVISION_RE = re.compile(r"[0-9a-f]{40}")
CLEAR_COLOR_RE = re.compile(r"0x[0-9a-fA-F]{8}")
PROVENANCE_KEY_RE = re.compile(r"[a-z][a-z0-9_]*")
OUTPUT_EXTENSIONS = (".rgba", ".png", ".provenance")
COORDINATOR_PROVENANCE_KEYS = (
    "artifact_sha256",
    "png_sha256",
    "frame_width",
    "frame_height",
    "sample_count",
)
UPSTREAM_PROVENANCE_KEYS = frozenset(
    {
        "backend",
        "renderer_implementation",
        "adapter_vendor",
        "adapter_architecture",
        "adapter_device",
        "adapter_description",
        "adapter_vendor_id",
        "adapter_device_id",
        "runtime_revision",
        "dawn_revision",
        "registry_sha256",
        "case_id",
        "stream_sha256",
    }
)
DEFAULT_CASE_TIMEOUT_SECONDS = 300.0


@dataclass(frozen=True)
class Case:
    case_id: str
    stream: pathlib.Path
    stream_sha256: str
    width: int
    height: int


@dataclass(frozen=True)
class CaseResult:
    case_id: str
    error: str | None


class CaseCommandError(Exception):
    pass


def sha256_file(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def require_string(case: dict, key: str) -> str:
    value = case[key]
    if not isinstance(value, str) or not value:
        raise ValueError(f"MSAA reference case has invalid {key}: {case.get('id')!r}")
    return value


def require_positive_int(case: dict, key: str) -> int:
    value = case[key]
    if isinstance(value, bool) or not isinstance(value, int) or value <= 0:
        raise ValueError(f"MSAA reference case has invalid {key}: {case.get('id')!r}")
    return value


def load_cases(manifest: pathlib.Path, repo_root: pathlib.Path) -> list[Case]:
    with manifest.open("rb") as source:
        document = tomllib.load(source)
    if (
        set(document) != {"version", "case"}
        or type(document["version"]) is not int
        or document["version"] != 1
    ):
        raise ValueError("MSAA reference manifest must be version 1 with only case rows")
    rows = document["case"]
    if not isinstance(rows, list) or not rows:
        raise ValueError("MSAA reference manifest must contain at least one case")

    cases: list[Case] = []
    seen_ids: set[str] = set()
    output_basenames: dict[str, str] = {}
    for row in rows:
        profile = row.get("profile", "gm") if isinstance(row, dict) else None
        expected_keys = RIV_CASE_KEYS if profile == "riv" else GM_CASE_KEYS
        if (
            not isinstance(row, dict)
            or profile not in ("gm", "riv")
            or set(row) != expected_keys
        ):
            case_id = row.get("id") if isinstance(row, dict) else None
            raise ValueError(f"MSAA reference case has unexpected fields: {case_id!r}")
        case_id = require_string(row, "id")
        if CASE_ID_RE.fullmatch(case_id) is None:
            raise ValueError(f"unsafe MSAA reference case id: {case_id!r}")
        if case_id in seen_ids:
            raise ValueError(f"duplicate MSAA reference case id: {case_id}")
        seen_ids.add(case_id)
        for extension in OUTPUT_EXTENSIONS:
            basename = f"{case_id}{extension}"
            folded = basename.casefold()
            previous = output_basenames.get(folded)
            if previous is not None:
                raise ValueError(f"output basename collision: {previous} and {basename}")
            output_basenames[folded] = basename

        stream_text = require_string(row, "stream")
        stream_relative = pathlib.PurePosixPath(stream_text)
        if stream_relative.is_absolute() or ".." in stream_relative.parts:
            raise ValueError(f"unsafe MSAA reference stream path: {stream_text}")
        stream = repo_root.joinpath(*stream_relative.parts).resolve()
        try:
            stream.relative_to(repo_root)
        except ValueError:
            raise ValueError(f"unsafe MSAA reference stream path: {stream_text}") from None
        if not stream.is_file():
            raise ValueError(f"MSAA reference stream does not exist: {stream}")

        stream_sha256 = require_string(row, "sha256")
        if SHA256_RE.fullmatch(stream_sha256) is None:
            raise ValueError(f"invalid MSAA reference stream SHA-256: {case_id}")
        actual_stream_sha256 = sha256_file(stream)
        if actual_stream_sha256 != stream_sha256:
            raise ValueError(
                f"MSAA reference stream SHA-256 mismatch for {case_id}: "
                f"expected {stream_sha256}, got {actual_stream_sha256}"
            )

        require_string(row, "source")
        require_string(row, "scene")
        width = require_positive_int(row, "width")
        height = require_positive_int(row, "height")
        clear_color = require_string(row, "clear_color")
        if CLEAR_COLOR_RE.fullmatch(clear_color) is None:
            raise ValueError(f"invalid MSAA reference clear color: {case_id}")
        counts = row["counts"]
        if not isinstance(counts, dict) or not counts:
            raise ValueError(f"MSAA reference case {case_id} has no command counts")
        for command, count in counts.items():
            if not isinstance(command, str) or not command:
                raise ValueError(f"MSAA reference case {case_id} has invalid command count name")
            if isinstance(count, bool) or not isinstance(count, int) or count < 0:
                raise ValueError(f"MSAA reference case {case_id} has invalid command count")
        if profile == "riv":
            if not isinstance(row["artboard"], str):
                raise ValueError(f"MSAA reference case {case_id} has invalid artboard")
            require_string(row, "sample_seconds")
            frame = row["frame"]
            if isinstance(frame, bool) or not isinstance(frame, int) or frame < 0:
                raise ValueError(f"MSAA reference case {case_id} has invalid frame")
        cases.append(Case(case_id, stream, stream_sha256, width, height))
    return cases


def validate_riveabl(path: pathlib.Path, case: Case) -> tuple[int, int, bytes]:
    data = path.read_bytes()
    if len(data) < RIVEABL_HEADER_BYTES or data[:8] != RIVEABL_MAGIC:
        raise ValueError("invalid or truncated RIVEABL header")
    version, width, height = struct.unpack_from("<III", data, 8)
    if version != 1:
        raise ValueError(f"unsupported RIVEABL version: {version}")
    if not width or not height:
        raise ValueError("RIVEABL dimensions must be nonzero")
    if width != case.width or height != case.height:
        raise ValueError(
            f"RIVEABL dimensions for {case.case_id} are {width}x{height}, "
            f"expected {case.width}x{case.height}"
        )
    if len(data) != RIVEABL_HEADER_BYTES + width * height * 4:
        raise ValueError("RIVEABL length mismatch")
    return width, height, data[RIVEABL_HEADER_BYTES:]


def paeth_predictor(left: int, up: int, upper_left: int) -> int:
    estimate = left + up - upper_left
    left_distance = abs(estimate - left)
    up_distance = abs(estimate - up)
    upper_left_distance = abs(estimate - upper_left)
    if left_distance <= up_distance and left_distance <= upper_left_distance:
        return left
    if up_distance <= upper_left_distance:
        return up
    return upper_left


def decode_png_rgba8(path: pathlib.Path, expected_width: int, expected_height: int) -> bytes:
    data = path.read_bytes()
    if len(data) < 8 or data[:8] != PNG_MAGIC:
        raise ValueError("converter did not produce a PNG")
    offset = 8
    saw_ihdr = False
    saw_iend = False
    idat = bytearray()
    while offset < len(data):
        if len(data) - offset < 12:
            raise ValueError("truncated PNG chunk")
        length = struct.unpack_from(">I", data, offset)[0]
        chunk_type = data[offset + 4 : offset + 8]
        data_start = offset + 8
        data_end = data_start + length
        if data_end + 4 > len(data):
            raise ValueError("truncated PNG chunk data")
        chunk_data = data[data_start:data_end]
        expected_crc = struct.unpack_from(">I", data, data_end)[0]
        actual_crc = zlib.crc32(chunk_type + chunk_data) & 0xFFFFFFFF
        if actual_crc != expected_crc:
            raise ValueError("PNG chunk CRC mismatch")
        if not saw_ihdr and chunk_type != b"IHDR":
            raise ValueError("PNG must begin with an IHDR chunk")
        if chunk_type == b"IHDR":
            if saw_ihdr:
                raise ValueError("PNG contains duplicate IHDR chunks")
            if length != 13:
                raise ValueError("PNG must begin with an IHDR chunk")
            width, height, bit_depth, color_type, compression, filter_, interlace = (
                struct.unpack(">IIBBBBB", chunk_data)
            )
            if (width, height) != (expected_width, expected_height):
                raise ValueError("PNG dimensions do not match RIVEABL dimensions")
            if (bit_depth, color_type, compression, filter_, interlace) != (8, 6, 0, 0, 0):
                raise ValueError("PNG is not an RGBA8 non-interlaced image")
            saw_ihdr = True
        elif chunk_type == b"IDAT":
            if not saw_ihdr:
                raise ValueError("PNG IDAT appears before IHDR")
            idat.extend(chunk_data)
        if chunk_type == b"IEND":
            if length != 0 or data_end + 4 != len(data):
                raise ValueError("invalid PNG IEND chunk")
            saw_iend = True
            break
        offset = data_end + 4
    if not saw_ihdr or not saw_iend:
        raise ValueError("PNG is missing required chunks")
    if not idat:
        raise ValueError("PNG is missing image data")
    try:
        decompressor = zlib.decompressobj()
        scanlines = decompressor.decompress(idat) + decompressor.flush()
    except zlib.error as error:
        raise ValueError(f"invalid PNG image data: {error}") from None
    if not decompressor.eof or decompressor.unused_data or decompressor.unconsumed_tail:
        raise ValueError("PNG image data contains an incomplete or trailing zlib stream")
    stride = expected_width * 4
    if len(scanlines) != expected_height * (1 + stride):
        raise ValueError("PNG image data length does not match its dimensions")
    pixels = bytearray(expected_height * stride)
    previous = bytearray(stride)
    offset = 0
    for row_index in range(expected_height):
        filter_type = scanlines[offset]
        offset += 1
        if filter_type > 4:
            raise ValueError(f"unsupported PNG filter type: {filter_type}")
        filtered = scanlines[offset : offset + stride]
        offset += stride
        reconstructed = bytearray(stride)
        for index, value in enumerate(filtered):
            left = reconstructed[index - 4] if index >= 4 else 0
            up = previous[index]
            upper_left = previous[index - 4] if index >= 4 else 0
            if filter_type == 0:
                predictor = 0
            elif filter_type == 1:
                predictor = left
            elif filter_type == 2:
                predictor = up
            elif filter_type == 3:
                predictor = (left + up) // 2
            else:
                predictor = paeth_predictor(left, up, upper_left)
            reconstructed[index] = (value + predictor) & 0xFF
        row_start = row_index * stride
        pixels[row_start : row_start + stride] = reconstructed
        previous = reconstructed
    return bytes(pixels)


def parse_provenance(path: pathlib.Path) -> tuple[str, dict[str, str]]:
    text = path.read_text(encoding="utf-8")
    if not text:
        raise ValueError("capture produced empty provenance")
    if "\r" in text:
        raise ValueError("provenance must use newline-terminated key=value lines")
    fields: dict[str, str] = {}
    for line_number, line in enumerate(text.splitlines(), 1):
        if not line or line.count("=") != 1:
            raise ValueError(f"invalid provenance line {line_number}: expected key=value")
        key, value = line.split("=", 1)
        if PROVENANCE_KEY_RE.fullmatch(key) is None or not value:
            raise ValueError(f"invalid provenance line {line_number}: expected key=value")
        if key in fields:
            raise ValueError(f"duplicate provenance key: {key}")
        fields[key] = value
    return text, fields


def validate_and_append_provenance(
    path: pathlib.Path,
    case: Case,
    artifact: pathlib.Path,
    png: pathlib.Path,
    width: int,
    height: int,
    runtime_revision: str,
    dawn_revision: str,
    registry_sha256: str,
) -> None:
    existing, upstream = parse_provenance(path)
    expected_upstream = {
        "backend": "metal",
        "renderer_implementation": "cpp-dawn-webgpu",
        "runtime_revision": runtime_revision,
        "dawn_revision": dawn_revision,
        "registry_sha256": registry_sha256,
        "case_id": case.case_id,
        "stream_sha256": case.stream_sha256,
    }
    if set(upstream) != UPSTREAM_PROVENANCE_KEYS:
        missing = sorted(UPSTREAM_PROVENANCE_KEYS - set(upstream))
        extra = sorted(set(upstream) - UPSTREAM_PROVENANCE_KEYS)
        raise ValueError(
            f"upstream provenance schema mismatch: missing={missing} extra={extra}"
        )
    for key, expected in expected_upstream.items():
        actual = upstream.get(key)
        if actual is None:
            raise ValueError(f"provenance is missing required key: {key}")
        if actual != expected:
            raise ValueError(
                f"provenance {key} mismatch: expected {expected}, got {actual}"
            )
    for key in ("adapter_vendor_id", "adapter_device_id"):
        if not upstream[key].isdigit():
            raise ValueError(f"provenance {key} must be an unsigned decimal integer")
    for key in COORDINATOR_PROVENANCE_KEYS:
        if key in upstream:
            raise ValueError(f"upstream provenance already contains coordinator key: {key}")
    appended = {
        "artifact_sha256": sha256_file(artifact),
        "png_sha256": sha256_file(png),
        "frame_width": str(width),
        "frame_height": str(height),
        "sample_count": "4",
    }
    separator = "" if existing.endswith("\n") else "\n"
    suffix = "".join(f"{key}={appended[key]}\n" for key in COORDINATOR_PROVENANCE_KEYS)
    path.write_text(existing + separator + suffix, encoding="utf-8")
    _, final_fields = parse_provenance(path)
    if any(final_fields.get(key) != value for key, value in appended.items()):
        raise ValueError("could not validate augmented provenance")


def run_case(
    binary: pathlib.Path,
    converter: pathlib.Path,
    stage_dir: pathlib.Path,
    case: Case,
    runtime_revision: str,
    dawn_revision: str,
    registry_sha256: str,
    timeout_seconds: float,
) -> CaseResult:
    artifact = stage_dir / f"{case.case_id}.rgba"
    png = stage_dir / f"{case.case_id}.png"
    provenance = stage_dir / f"{case.case_id}.provenance"
    try:
        run_checked_command(
            [
                os.fspath(binary),
                os.devnull,
                os.devnull,
                os.fspath(artifact),
                "msaa-reference",
                case.case_id,
                os.fspath(provenance),
            ],
            "capture",
            timeout_seconds,
        )
        width, height, artifact_pixels = validate_riveabl(artifact, case)
        if not provenance.is_file():
            raise ValueError("capture did not produce provenance")
        run_checked_command(
            [os.fspath(converter), "--artifact", os.fspath(artifact), "--output", os.fspath(png)],
            "converter",
            timeout_seconds,
        )
        png_pixels = decode_png_rgba8(png, width, height)
        if png_pixels != artifact_pixels:
            raise ValueError("PNG pixels do not match RIVEABL payload")
        validate_and_append_provenance(
            provenance,
            case,
            artifact,
            png,
            width,
            height,
            runtime_revision,
            dawn_revision,
            registry_sha256,
        )
    except (OSError, CaseCommandError, ValueError, UnicodeDecodeError) as error:
        return CaseResult(case.case_id, str(error))
    return CaseResult(case.case_id, None)


def run_checked_command(command: list[str], label: str, timeout_seconds: float) -> None:
    try:
        subprocess.run(
            command,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            timeout=timeout_seconds,
        )
    except subprocess.TimeoutExpired:
        raise CaseCommandError(
            f"{label} timed out after {timeout_seconds:g} seconds"
        ) from None
    except subprocess.CalledProcessError as error:
        detail = error.stderr.strip() or error.stdout.strip() or f"exit status {error.returncode}"
        raise CaseCommandError(detail) from None


def validate_hex_argument(
    parser: argparse.ArgumentParser, name: str, value: str, pattern: re.Pattern[str]
) -> str:
    if pattern.fullmatch(value) is None:
        parser.error(f"{name} must be lowercase hexadecimal with the required length")
    return value


def make_stage_dir(output_dir: pathlib.Path) -> pathlib.Path:
    parent = output_dir.parent
    parent.mkdir(parents=True, exist_ok=True)
    return pathlib.Path(
        tempfile.mkdtemp(prefix=f".{output_dir.name}.capture-", dir=parent)
    )


def rename_directory_no_replace(source: pathlib.Path, destination: pathlib.Path) -> None:
    source_bytes = os.fsencode(source)
    destination_bytes = os.fsencode(destination)
    libc = ctypes.CDLL(None, use_errno=True)
    if sys.platform == "darwin":
        rename = libc.renamex_np
        rename.argtypes = (ctypes.c_char_p, ctypes.c_char_p, ctypes.c_uint)
        rename.restype = ctypes.c_int
        result = rename(source_bytes, destination_bytes, 0x00000004)
    elif sys.platform.startswith("linux"):
        try:
            rename = libc.renameat2
        except AttributeError:
            raise OSError(
                errno.ENOTSUP, "renameat2 is required for no-clobber installation"
            ) from None
        rename.argtypes = (
            ctypes.c_int,
            ctypes.c_char_p,
            ctypes.c_int,
            ctypes.c_char_p,
            ctypes.c_uint,
        )
        rename.restype = ctypes.c_int
        result = rename(-100, source_bytes, -100, destination_bytes, 0x00000001)
    elif os.name == "nt":
        os.rename(source, destination)
        return
    else:
        raise OSError(
            errno.ENOTSUP,
            f"no race-safe directory install is available on {sys.platform}",
        )
    if result != 0:
        error_number = ctypes.get_errno()
        raise OSError(
            error_number,
            os.strerror(error_number),
            os.fspath(source),
            os.fspath(destination),
        )


def install_directory_no_clobber(
    stage_dir: pathlib.Path,
    output_dir: pathlib.Path,
    rename_no_replace=rename_directory_no_replace,
) -> None:
    rename_no_replace(stage_dir, output_dir)


def preserve_failed_stage(stage_dir: pathlib.Path, output_dir: pathlib.Path) -> pathlib.Path:
    while True:
        failed_dir = stage_dir.with_name(f".{output_dir.name}.failed-{uuid.uuid4().hex}")
        try:
            rename_directory_no_replace(stage_dir, failed_dir)
            return failed_dir
        except FileExistsError:
            continue


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=pathlib.Path, required=True)
    parser.add_argument("--converter", type=pathlib.Path, required=True)
    parser.add_argument("--manifest", type=pathlib.Path, required=True)
    parser.add_argument("--output-dir", type=pathlib.Path, required=True)
    parser.add_argument("--repo-root", type=pathlib.Path, required=True)
    parser.add_argument("--jobs", type=int, required=True)
    parser.add_argument(
        "--case-timeout-seconds", type=float, default=DEFAULT_CASE_TIMEOUT_SECONDS
    )
    parser.add_argument("--runtime-revision", required=True)
    parser.add_argument("--dawn-revision", required=True)
    parser.add_argument("--registry-sha256", required=True)
    args = parser.parse_args(argv)

    if args.jobs <= 0:
        parser.error("--jobs must be greater than zero")
    if not math.isfinite(args.case_timeout_seconds) or args.case_timeout_seconds <= 0:
        parser.error("--case-timeout-seconds must be a finite number greater than zero")
    runtime_revision = validate_hex_argument(
        parser, "--runtime-revision", args.runtime_revision, REVISION_RE
    )
    dawn_revision = validate_hex_argument(
        parser, "--dawn-revision", args.dawn_revision, REVISION_RE
    )
    registry_sha256 = validate_hex_argument(
        parser, "--registry-sha256", args.registry_sha256, SHA256_RE
    )
    binary = args.binary.resolve()
    converter = args.converter.resolve()
    manifest = args.manifest.resolve()
    repo_root = args.repo_root.resolve()
    output_dir = args.output_dir.resolve()
    if not binary.is_file() or not os.access(binary, os.X_OK):
        parser.error(f"--binary is not an executable file: {binary}")
    if not converter.is_file() or not os.access(converter, os.X_OK):
        parser.error(f"--converter is not an executable file: {converter}")
    if output_dir.exists() or output_dir.is_symlink():
        parser.error(
            f"--output-dir already exists and would collide with this capture: {output_dir}"
        )

    try:
        cases = load_cases(manifest, repo_root)
    except (OSError, tomllib.TOMLDecodeError, ValueError) as error:
        parser.error(str(error))
    stage_dir = make_stage_dir(output_dir)
    results: dict[str, CaseResult] = {}
    try:
        with concurrent.futures.ThreadPoolExecutor(max_workers=args.jobs) as executor:
            futures = [
                executor.submit(
                    run_case,
                    binary,
                    converter,
                    stage_dir,
                    case,
                    runtime_revision,
                    dawn_revision,
                    registry_sha256,
                    args.case_timeout_seconds,
                )
                for case in cases
            ]
            for future in concurrent.futures.as_completed(futures):
                result = future.result()
                results[result.case_id] = result
        failed = False
        for case in cases:
            result = results[case.case_id]
            if result.error is None:
                print(f"OK {case.case_id}")
            else:
                failed = True
                print(f"FAIL {case.case_id}: {result.error}")
        if failed:
            failed_dir = preserve_failed_stage(stage_dir, output_dir)
            print(f"FAILED OUTPUTS: {failed_dir}", file=sys.stderr)
            return 1
        try:
            install_directory_no_clobber(stage_dir, output_dir)
        except FileExistsError:
            failed_dir = preserve_failed_stage(stage_dir, output_dir)
            print(
                f"INSTALL FAILED: output directory appeared during capture: {output_dir}",
                file=sys.stderr,
            )
            print(f"FAILED OUTPUTS: {failed_dir}", file=sys.stderr)
            return 1
        print(f"OUTPUTS: {output_dir}")
        return 0
    except BaseException:
        if stage_dir.exists():
            failed_dir = preserve_failed_stage(stage_dir, output_dir)
            print(f"FAILED OUTPUTS: {failed_dir}", file=sys.stderr)
        raise


if __name__ == "__main__":
    raise SystemExit(main())
