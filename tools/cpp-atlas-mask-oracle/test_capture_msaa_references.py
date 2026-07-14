#!/usr/bin/env python3
"""Unit tests for the standalone MSAA reference capture coordinator."""

from __future__ import annotations

import hashlib
import importlib.util
import os
import pathlib
import stat
import subprocess
import sys
import tempfile
import textwrap
import unittest


SCRIPT = pathlib.Path(__file__).with_name("capture_msaa_references.py")
RUNTIME_REVISION = "a" * 40
DAWN_REVISION = "b" * 40
REGISTRY_SHA256 = "c" * 64
SPEC = importlib.util.spec_from_file_location("capture_msaa_references", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
capture = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = capture
SPEC.loader.exec_module(capture)


FAKE_CAPTURE = r'''#!/usr/bin/env python3
import os
import pathlib
import struct
import sys
import time
import tomllib

_, _, _, artifact, mode, case_id, provenance = sys.argv
assert mode == "msaa-reference"
state = pathlib.Path(os.environ["FAKE_STATE"])
active = state / "active"
maximum = state / "maximum"
lock = state / "lock"
active.parent.mkdir(parents=True, exist_ok=True)
while True:
    try:
        lock.mkdir()
        break
    except FileExistsError:
        time.sleep(0.001)
try:
    current = int(active.read_text()) if active.exists() else 0
    active.write_text(str(current + 1))
    maximum.write_text(str(max(int(maximum.read_text()) if maximum.exists() else 0, current + 1)))
finally:
    lock.rmdir()
try:
    time.sleep(float(os.environ.get("FAKE_DELAY", "0")))
    if case_id == os.environ.get("FAKE_TIMEOUT_CASE"):
        time.sleep(60)
    if case_id == os.environ.get("FAKE_FAIL_CASE"):
        print("intentional capture failure", file=sys.stderr)
        raise SystemExit(17)
    width, height = (2, 3) if case_id == "slow" else (1, 1)
    pixels = bytes((index * 29 + 7) % 256 for index in range(width * height * 4))
    if case_id == os.environ.get("FAKE_RED_ARTIFACT_CASE"):
        pixels = bytes((255, 0, 0, 255)) * (width * height)
    pathlib.Path(artifact).write_bytes(
        b"RIVEABL\0" + struct.pack("<III", 1, width, height) + pixels
    )
    with pathlib.Path(os.environ["FAKE_MANIFEST"]).open("rb") as source:
        manifest = tomllib.load(source)
    stream_sha256 = next(row["sha256"] for row in manifest["case"] if row["id"] == case_id)
    fields = [
        ("backend", "metal"),
        ("renderer_implementation", "cpp-dawn-webgpu"),
        ("adapter_vendor", "test-vendor"),
        ("adapter_architecture", "test-architecture"),
        ("adapter_device", "test-device"),
        ("adapter_description", "test-description"),
        ("adapter_vendor_id", "1"),
        ("adapter_device_id", "2"),
        ("runtime_revision", os.environ["FAKE_RUNTIME_REVISION"]),
        ("dawn_revision", os.environ["FAKE_DAWN_REVISION"]),
        ("registry_sha256", os.environ["FAKE_REGISTRY_SHA256"]),
        ("case_id", case_id),
        ("stream_sha256", stream_sha256),
    ]
    omitted_key = os.environ.get("FAKE_OMIT_PROVENANCE_KEY")
    if omitted_key:
        fields = [field for field in fields if field[0] != omitted_key]
    duplicate_key = os.environ.get("FAKE_DUPLICATE_PROVENANCE_KEY")
    if duplicate_key:
        fields.append((duplicate_key, dict(fields)[duplicate_key]))
    coordinator_key = os.environ.get("FAKE_UPSTREAM_COORDINATOR_KEY")
    if coordinator_key:
        fields.append((coordinator_key, "premature"))
    pathlib.Path(provenance).write_text("".join(f"{key}={value}\n" for key, value in fields))
finally:
    while True:
        try:
            lock.mkdir()
            break
        except FileExistsError:
            time.sleep(0.001)
    try:
        active.write_text(str(int(active.read_text()) - 1))
    finally:
        lock.rmdir()
'''

FAKE_CONVERTER = r'''#!/usr/bin/env python3
import os
import pathlib
import struct
import sys
import time
import zlib

artifact = pathlib.Path(sys.argv[sys.argv.index("--artifact") + 1])
output = pathlib.Path(sys.argv[sys.argv.index("--output") + 1])
if artifact.stem == os.environ.get("FAKE_CONVERTER_TIMEOUT_CASE"):
    time.sleep(60)
if artifact.stem == os.environ.get("FAKE_BAD_PNG_CASE"):
    output.write_bytes(b"not a png")
    raise SystemExit(0)
data = artifact.read_bytes()
width, height = struct.unpack_from("<II", data, 12)
pixels = data[20:]
if artifact.stem == os.environ.get("FAKE_BLACK_PNG_CASE"):
    pixels = bytes(width * height * 4)
def chunk(kind, data):
    return (
        struct.pack(">I", len(data))
        + kind
        + data
        + struct.pack(">I", zlib.crc32(kind + data) & 0xffffffff)
    )
def paeth(left, up, upper_left):
    estimate = left + up - upper_left
    distances = (abs(estimate - left), abs(estimate - up), abs(estimate - upper_left))
    return (left, up, upper_left)[distances.index(min(distances))]
filter_type = int(os.environ.get("FAKE_PNG_FILTER", "0"))
stride = width * 4
raw = bytearray()
previous = bytes(stride)
for y in range(height):
    row = pixels[y * stride:(y + 1) * stride]
    filtered = bytearray(stride)
    for index, value in enumerate(row):
        left = row[index - 4] if index >= 4 else 0
        up = previous[index]
        upper_left = previous[index - 4] if index >= 4 else 0
        predictors = (0, left, up, (left + up) // 2, paeth(left, up, upper_left))
        filtered[index] = (value - predictors[filter_type]) & 0xff
    raw.extend((filter_type,))
    raw.extend(filtered)
    previous = row
output.write_bytes(
    b"\x89PNG\r\n\x1a\n"
    + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
    + chunk(b"IDAT", zlib.compress(raw))
    + chunk(b"IEND", b"")
)
'''


class CaptureMsaaReferencesTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = pathlib.Path(self.temp.name)
        self.repo = self.root / "repo"
        self.repo.mkdir()
        self.state = self.root / "state"
        self.capture_binary = self.write_executable("fake-capture", FAKE_CAPTURE)
        self.converter = self.write_executable("fake-converter", FAKE_CONVERTER)

    def tearDown(self) -> None:
        self.temp.cleanup()

    def write_executable(self, name: str, content: str) -> pathlib.Path:
        path = self.root / name
        path.write_text(textwrap.dedent(content))
        path.chmod(path.stat().st_mode | stat.S_IXUSR)
        return path

    def write_manifest(self, cases: list[tuple[str, bytes]]) -> pathlib.Path:
        rows = ["version = 1", ""]
        for case_id, stream_data in cases:
            stream = self.repo / "streams" / f"{case_id}.rive-stream"
            stream.parent.mkdir(exist_ok=True)
            stream.write_bytes(stream_data)
            sha = hashlib.sha256(stream_data).hexdigest()
            rows.extend(
                [
                    "[[case]]",
                    f'id = "{case_id}"',
                    f'stream = "streams/{case_id}.rive-stream"',
                    f'sha256 = "{sha}"',
                    'source = "test:case"',
                    f'scene = "{case_id}"',
                    "width = 2" if case_id == "slow" else "width = 1",
                    "height = 3" if case_id == "slow" else "height = 1",
                    'clear_color = "0xff000000"',
                    "counts = { drawPath = 1 }",
                    "",
                ]
            )
        manifest = self.root / "corpus.toml"
        manifest.write_text("\n".join(rows))
        return manifest

    def run_cli(
        self,
        manifest: pathlib.Path,
        output_dir: pathlib.Path,
        jobs: int,
        timeout: float | None = None,
        extra_args: list[str] | None = None,
        **environment: str,
    ) -> subprocess.CompletedProcess[str]:
        env = os.environ | {
            "FAKE_STATE": str(self.state),
            "FAKE_MANIFEST": str(manifest),
            "FAKE_RUNTIME_REVISION": RUNTIME_REVISION,
            "FAKE_DAWN_REVISION": DAWN_REVISION,
            "FAKE_REGISTRY_SHA256": REGISTRY_SHA256,
        } | environment
        arguments = [
                sys.executable,
                str(SCRIPT),
                "--binary",
                str(self.capture_binary),
                "--converter",
                str(self.converter),
                "--manifest",
                str(manifest),
                "--output-dir",
                str(output_dir),
                "--repo-root",
                str(self.repo),
                "--jobs",
                str(jobs),
                "--runtime-revision",
                RUNTIME_REVISION,
                "--dawn-revision",
                DAWN_REVISION,
                "--registry-sha256",
                REGISTRY_SHA256,
            ]
        if timeout is not None:
            arguments.extend(("--case-timeout-seconds", str(timeout)))
        arguments.extend(extra_args or [])
        return subprocess.run(
            arguments,
            check=False,
            text=True,
            capture_output=True,
            env=env,
        )

    def test_runs_bounded_parallel_capture_and_installs_provenance(self) -> None:
        manifest = self.write_manifest([("slow", b"slow"), ("fast", b"fast"), ("third", b"third")])
        output = self.root / "out"
        result = self.run_cli(
            manifest, output, jobs=2, FAKE_DELAY="0.08", FAKE_PNG_FILTER="4"
        )

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(result.stdout.splitlines()[:3], ["OK slow", "OK fast", "OK third"])
        self.assertEqual(int((self.state / "maximum").read_text()), 2)
        provenance = (output / "slow.provenance").read_text()
        self.assertIn("backend=metal\n", provenance)
        self.assertIn("renderer_implementation=cpp-dawn-webgpu\n", provenance)
        self.assertIn("case_id=slow\n", provenance)
        self.assertIn("stream_sha256=" + hashlib.sha256(b"slow").hexdigest(), provenance)
        self.assertEqual(provenance.count("runtime_revision="), 1)
        self.assertEqual(provenance.count("dawn_revision="), 1)
        self.assertEqual(provenance.count("registry_sha256="), 1)
        self.assertIn("runtime_revision=" + RUNTIME_REVISION + "\n", provenance)
        self.assertIn("dawn_revision=" + DAWN_REVISION + "\n", provenance)
        self.assertIn("registry_sha256=" + REGISTRY_SHA256 + "\n", provenance)
        self.assertIn("frame_width=2\nframe_height=3\nsample_count=4\n", provenance)
        self.assertTrue((output / "slow.rgba").is_file())
        self.assertTrue((output / "slow.png").is_file())
        _, fields = capture.parse_provenance(output / "slow.provenance")
        self.assertEqual(
            fields["artifact_sha256"],
            hashlib.sha256((output / "slow.rgba").read_bytes()).hexdigest(),
        )
        self.assertEqual(
            fields["png_sha256"],
            hashlib.sha256((output / "slow.png").read_bytes()).hexdigest(),
        )

    def test_accepts_riv_profile_capture_metadata(self) -> None:
        manifest = self.write_manifest([("riv-case", b"riv")])
        manifest.write_text(
            manifest.read_text().replace(
                "counts = { drawPath = 1 }",
                'profile = "riv"\nartboard = "Main"\n'
                'sample_seconds = "0.5"\nframe = 2\n'
                "counts = { drawPath = 1 }",
            )
        )
        cases = capture.load_cases(manifest, self.repo.resolve())
        self.assertEqual([case.case_id for case in cases], ["riv-case"])

    def test_failure_keeps_isolated_failed_directory_and_does_not_install(self) -> None:
        manifest = self.write_manifest([("first", b"first"), ("bad", b"bad")])
        output = self.root / "out"
        result = self.run_cli(manifest, output, jobs=2, FAKE_FAIL_CASE="bad")

        self.assertEqual(result.returncode, 1)
        self.assertEqual(
            result.stdout.splitlines(),
            ["OK first", "FAIL bad: intentional capture failure"],
        )
        self.assertFalse(output.exists())
        failed_dirs = list(self.root.glob(".out.failed-*"))
        self.assertEqual(len(failed_dirs), 1)
        self.assertTrue((failed_dirs[0] / "first.rgba").is_file())

    def test_rejects_png_whose_pixels_do_not_match_riveabl(self) -> None:
        manifest = self.write_manifest([("only", b"only")])
        output = self.root / "out"
        result = self.run_cli(
            manifest,
            output,
            jobs=1,
            FAKE_RED_ARTIFACT_CASE="only",
            FAKE_BLACK_PNG_CASE="only",
        )

        self.assertEqual(result.returncode, 1)
        self.assertIn("PNG pixels do not match RIVEABL payload", result.stdout)
        self.assertFalse(output.exists())

    def test_rejects_duplicate_and_mismatched_upstream_provenance(self) -> None:
        manifest = self.write_manifest([("only", b"only")])
        duplicate = self.run_cli(
            manifest,
            self.root / "duplicate-provenance",
            jobs=1,
            FAKE_DUPLICATE_PROVENANCE_KEY="runtime_revision",
        )
        self.assertEqual(duplicate.returncode, 1)
        self.assertIn("duplicate provenance key: runtime_revision", duplicate.stdout)

        mismatch = self.run_cli(
            manifest,
            self.root / "mismatched-provenance",
            jobs=1,
            FAKE_RUNTIME_REVISION="d" * 40,
        )
        self.assertEqual(mismatch.returncode, 1)
        self.assertIn("provenance runtime_revision mismatch", mismatch.stdout)

        missing_producer = self.run_cli(
            manifest,
            self.root / "missing-producer-provenance",
            jobs=1,
            FAKE_OMIT_PROVENANCE_KEY="backend",
        )
        self.assertEqual(missing_producer.returncode, 1)
        self.assertIn("upstream provenance schema mismatch", missing_producer.stdout)

        premature = self.run_cli(
            manifest,
            self.root / "premature-coordinator-field",
            jobs=1,
            FAKE_UPSTREAM_COORDINATOR_KEY="artifact_sha256",
        )
        self.assertEqual(premature.returncode, 1)
        self.assertIn("upstream provenance schema mismatch", premature.stdout)
        self.assertIn("artifact_sha256", premature.stdout)

        missing = self.run_cli(
            manifest,
            self.root / "missing-provenance",
            jobs=1,
            FAKE_OMIT_PROVENANCE_KEY="stream_sha256",
        )
        self.assertEqual(missing.returncode, 1)
        self.assertIn("upstream provenance schema mismatch", missing.stdout)
        self.assertIn("stream_sha256", missing.stdout)

    def test_rejects_casefold_colliding_output_basenames(self) -> None:
        manifest = self.write_manifest([("Case", b"same"), ("case", b"same")])
        result = self.run_cli(manifest, self.root / "out", jobs=1)

        self.assertEqual(result.returncode, 2)
        self.assertIn("output basename collision", result.stderr)

    def test_capture_timeout_is_deterministic_and_preserves_failure_dir(self) -> None:
        manifest = self.write_manifest([("only", b"only")])
        output = self.root / "out"
        result = self.run_cli(
            manifest,
            output,
            jobs=1,
            timeout=0.05,
            FAKE_TIMEOUT_CASE="only",
        )

        self.assertEqual(result.returncode, 1)
        self.assertEqual(
            result.stdout.splitlines(),
            ["FAIL only: capture timed out after 0.05 seconds"],
        )
        self.assertFalse(output.exists())
        self.assertEqual(len(list(self.root.glob(".out.failed-*"))), 1)

    def test_converter_timeout_is_deterministic(self) -> None:
        manifest = self.write_manifest([("only", b"only")])
        result = self.run_cli(
            manifest,
            self.root / "out",
            jobs=1,
            timeout=0.5,
            FAKE_CONVERTER_TIMEOUT_CASE="only",
        )

        self.assertEqual(result.returncode, 1)
        self.assertEqual(
            result.stdout.splitlines(),
            ["FAIL only: converter timed out after 0.5 seconds"],
        )

    def test_decodes_sub_up_and_average_png_filters(self) -> None:
        manifest = self.write_manifest([("slow", b"slow")])
        for filter_type in (1, 2, 3):
            with self.subTest(filter_type=filter_type):
                output = self.root / f"filter-{filter_type}"
                result = self.run_cli(
                    manifest,
                    output,
                    jobs=1,
                    FAKE_PNG_FILTER=str(filter_type),
                )
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_directory_install_does_not_clobber_destination_created_during_race(self) -> None:
        stage = self.root / "stage"
        stage.mkdir()
        (stage / "artifact").write_text("complete")
        destination = self.root / "out"

        def create_destination_then_rename(source: pathlib.Path, target: pathlib.Path) -> None:
            target.mkdir()
            capture.rename_directory_no_replace(source, target)

        with self.assertRaises(FileExistsError):
            capture.install_directory_no_clobber(
                stage,
                destination,
                rename_no_replace=create_destination_then_rename,
            )
        self.assertTrue(stage.is_dir())
        self.assertTrue(destination.is_dir())
        self.assertFalse((destination / "artifact").exists())

    def test_rejects_zero_jobs_duplicate_ids_and_existing_output(self) -> None:
        manifest = self.write_manifest([("only", b"only")])
        zero_jobs = self.run_cli(manifest, self.root / "zero", jobs=0)
        self.assertEqual(zero_jobs.returncode, 2)
        self.assertIn("--jobs must be greater than zero", zero_jobs.stderr)

        invalid_timeout = self.run_cli(manifest, self.root / "timeout", jobs=1, timeout=0)
        self.assertEqual(invalid_timeout.returncode, 2)
        self.assertIn(
            "--case-timeout-seconds must be a finite number greater than zero",
            invalid_timeout.stderr,
        )

        invalid_revision = self.run_cli(
            manifest,
            self.root / "invalid-revision",
            jobs=1,
            extra_args=["--runtime-revision", "A" * 40],
        )
        self.assertEqual(invalid_revision.returncode, 2)
        self.assertIn("--runtime-revision must be lowercase hexadecimal", invalid_revision.stderr)

        duplicate = self.root / "duplicate.toml"
        first_case = manifest.read_text().split("[[case]]", 1)[1]
        duplicate.write_text(manifest.read_text() + "\n[[case]]" + first_case)
        duplicate_result = self.run_cli(duplicate, self.root / "duplicate-out", jobs=1)
        self.assertEqual(duplicate_result.returncode, 2)
        self.assertIn("duplicate MSAA reference case id", duplicate_result.stderr)

        existing = self.root / "existing"
        existing.mkdir()
        collision = self.run_cli(manifest, existing, jobs=1)
        self.assertEqual(collision.returncode, 2)
        self.assertIn("would collide", collision.stderr)

        boolean_version = self.root / "boolean-version.toml"
        boolean_version.write_text(manifest.read_text().replace("version = 1", "version = true"))
        boolean_result = self.run_cli(boolean_version, self.root / "boolean-version-out", jobs=1)
        self.assertEqual(boolean_result.returncode, 2)
        self.assertIn("must be version 1", boolean_result.stderr)

    def test_rejects_converter_output_that_is_not_valid_png(self) -> None:
        manifest = self.write_manifest([("only", b"only")])
        output = self.root / "out"
        result = self.run_cli(manifest, output, jobs=1, FAKE_BAD_PNG_CASE="only")

        self.assertEqual(result.returncode, 1)
        self.assertIn("converter did not produce a PNG", result.stdout)
        self.assertFalse(output.exists())


if __name__ == "__main__":
    unittest.main()
