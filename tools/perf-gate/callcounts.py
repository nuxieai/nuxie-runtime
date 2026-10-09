#!/usr/bin/env python3
"""Per-function call counts for one perf fixture's frame loop, C++ vs Rust.

Runs the coverage-instrumented runners from build-callcount-runners.sh through
perf-compare (the same samples, artboard and state machine the timing gate
uses), keeps only the frame loop's counters, and writes one TSV per runner:
`calls<TAB>function`, sorted by calls. Counts are taken before inlining, so
they are source-level calls; comparing a C++ method with its Rust port shows
whether Rust performs more operations or the same operations at a higher cost.

Example:
  RIVE_RUNTIME_DIR=<pin> tools/perf-gate/callcounts.py zombie_skins --frames 600
"""

import argparse
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
CPP_RUNNER = REPO / "tools/golden-runner/build/macosx/bin/release/rive_golden_runner_scripted_callcount"
RUST_RUNNER = REPO / "target/callcount/rust/release/rust-golden-runner"
PERF_COMPARE = REPO / "target/release/perf-compare"


def rust_profdata() -> str:
    # The reader must understand the writer's raw profile version: use the
    # LLVM that rustc itself was built with.
    llvm = re.search(r"LLVM version: (\d+)", subprocess.check_output(["rustc", "-vV"], text=True))
    for candidate in ("/opt/homebrew/opt/llvm/bin/llvm-profdata", shutil.which("llvm-profdata")):
        if candidate and os.path.exists(candidate):
            version = subprocess.run([candidate, "--version"], capture_output=True, text=True).stdout
            if llvm and f"version {llvm.group(1)}." in version:
                return candidate
    sys.exit(f"no llvm-profdata matching rustc's LLVM {llvm.group(1) if llvm else '?'}")


def tool(side: str, name: str) -> list[str]:
    if side == "cpp":
        return ["xcrun", name]  # Apple clang built the C++ runner.
    return [str(Path(rust_profdata()).with_name(name))]


def wrapper(path: Path, runner: Path, profile_pattern: Path) -> Path:
    path.write_text(
        "#!/bin/sh\n"
        f'LLVM_PROFILE_FILE="{profile_pattern}" RIVE_GOLDEN_COVERAGE_FRAME_ONLY=1 '
        f'exec "{runner}" "$@"\n'
    )
    path.chmod(0o755)
    return path


def function_counts(side: str, profiles: list[Path], out: Path) -> list[tuple[int, str]]:
    merged = out / f"{side}.profdata"
    subprocess.run(tool(side, "llvm-profdata") + ["merge", "-o", merged, *profiles], check=True)
    shown = subprocess.run(
        tool(side, "llvm-profdata") + ["show", "--all-functions", merged],
        check=True, capture_output=True, text=True,
    ).stdout
    counts, name = {}, None
    for line in shown.splitlines():
        header = re.match(r"^  (\S.*):$", line)
        if header:
            name = header.group(1)
            continue
        value = re.match(r"^\s+Function count: (\d+)", line)
        if value and name is not None:
            # Static functions are recorded as "file;symbol".
            key = name.rsplit(";", 1)[-1]
            counts[key] = counts.get(key, 0) + int(value.group(1))
            name = None
    names = list(counts)
    demangled = subprocess.run(
        tool(side, "llvm-cxxfilt"), input="\n".join(names), capture_output=True, text=True, check=True
    ).stdout.splitlines()
    # Drop Rust's trailing ::h<hash> so the same function lines up across builds.
    rows = [(counts[n], re.sub(r"::h[0-9a-f]{16}$", "", d)) for n, d in zip(names, demangled)]
    rows = [row for row in rows if row[0] > 0]
    rows.sort(key=lambda row: -row[0])
    return rows


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("fixture", help="perf corpus id, e.g. zombie_skins")
    parser.add_argument("--frames", type=int, default=600)
    parser.add_argument("--out", type=Path, help="default: target/callcount/<fixture>")
    args = parser.parse_args()

    rive_runtime = os.environ.get("RIVE_RUNTIME_DIR", "/Users/levi/dev/oss/rive-runtime")
    out = (args.out or REPO / "target/callcount" / args.fixture).resolve()
    shutil.rmtree(out, ignore_errors=True)
    out.mkdir(parents=True)
    for runner in (CPP_RUNNER, RUST_RUNNER, PERF_COMPARE):
        if not runner.exists():
            sys.exit(f"missing {runner}; run tools/perf-gate/build-callcount-runners.sh "
                     "and cargo build --release -p perf-compare")

    cpp = wrapper(out / "cpp-runner.sh", CPP_RUNNER, out / "cpp-%p.profraw")
    rust = wrapper(out / "rust-runner.sh", RUST_RUNNER, out / "rust-%p.profraw")
    subprocess.run(
        [PERF_COMPARE, "--cpp-runner", cpp, "--rust-runner", rust,
         "--rive-runtime-dir", rive_runtime, "--corpus", REPO / "corpus.toml",
         "--corpus-ids", args.fixture, "--iterations", "1", "--warmups", "0",
         "--aggregate", "median", "--runner-order", "cpp-first", "--runner-benchmark",
         "--benchmark-frames", str(args.frames), "--benchmark-hz", "60",
         "--rust-execute-scripts"],
        check=True, cwd=REPO, stdout=subprocess.DEVNULL,
    )

    for side in ("cpp", "rust"):
        profiles = sorted(out.glob(f"{side}-*.profraw"))
        if len(profiles) != 1:
            sys.exit(f"expected one {side} frame-loop profile, found {len(profiles)}")
        rows = function_counts(side, profiles, out)
        (out / f"{side}.tsv").write_text("".join(f"{c}\t{n}\n" for c, n in rows))
        total = sum(c for c, _ in rows)
        print(f"{side}: {total / args.frames:,.0f} calls/frame across {len(rows)} functions -> {out / (side + '.tsv')}")


if __name__ == "__main__":
    main()
