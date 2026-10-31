#!/usr/bin/env python3
"""Count-only-goes-down ratchets for the Luau VM work (docs/luau-safe-vm-plan.md, section 7).

    ratchets.py unsafe [--update] [--baseline FILE] [GLOB...]
    ratchets.py panic-lints [--update] [--baseline FILE] [CRATE_DIR...]

Run through tools/ratchets/unsafe-count.sh and tools/ratchets/panic-lints.sh.

Both compare per-crate counts with a committed baseline and exit 1 when any
count went up. A crate missing from the baseline is reported as untracked and
does not fail; `--update` adds it (and records lowered counts). Raising a
baseline needs a dated justification in docs/luau-safe-vm-plan.md.
Standard library only.
"""

from pathlib import Path
import argparse
import glob
import json
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
DEFAULT_GLOBS = ["vendor/luaur-*"]

UNSAFE_PATTERN = re.compile(r"unsafe (fn|\{|impl|extern)")

# The forced panic lints: the sites where a Rust crate can panic (or, for
# arithmetic, overflow-panic in debug) on input it did not validate.
PANIC_LINTS = [
    "clippy::unwrap_used",
    "clippy::expect_used",
    "clippy::indexing_slicing",
    "clippy::panic",
    "clippy::unreachable",
    "clippy::arithmetic_side_effects",
    "clippy::unwrap_in_result",
]


def read_baseline(path):
    """{key tuple: count}; keys are (crate,) or (crate, lint)."""
    entries = {}
    if path.exists():
        for line in path.read_text().splitlines():
            fields = line.split()
            if not fields or fields[0].startswith("#"):
                continue
            entries[tuple(fields[:-1])] = int(fields[-1])
    return entries


def write_baseline(path, header, entries):
    lines = [f"# {line}" if line else "#" for line in header]
    lines += [" ".join([*key, str(count)]) for key, count in sorted(entries.items())]
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines) + "\n")
    print(f"ratchet: wrote {shown(path)}")


def shown(path):
    return path.relative_to(ROOT) if path.is_relative_to(ROOT) else path


def crate_dirs(patterns, baseline):
    found = {key[0] for key in baseline}
    for pattern in [*DEFAULT_GLOBS, *patterns]:
        for match in glob.glob(pattern, root_dir=ROOT):
            if (ROOT / match / "Cargo.toml").is_file():
                found.add(match.rstrip("/"))
    return sorted(found)


def judge(rows, baseline, tracked, heading):
    """rows: [(key, count)]. Prints the table and returns (failed, lowered)."""
    failed = lowered = False
    width = max([len(" ".join(key)) for key, _ in rows] + [len(heading)])
    print(f"{heading:<{width}} {'now':>8} {'baseline':>9} {'delta':>7}  status")
    for key, count in rows:
        label = " ".join(key)
        if key[0] not in tracked:
            print(f"{label:<{width}} {count:>8} {'-':>9} {'-':>7}  untracked (add with --update)")
            continue
        before = baseline.get(key, 0)
        delta = count - before
        status = "ok"
        if delta > 0:
            status, failed = "UP (ratchet failure)", True
        elif delta < 0:
            status, lowered = "down (lock in with --update)", True
        print(f"{label:<{width}} {count:>8} {before:>9} {delta:>+7}  {status}")
    return failed, lowered


def finish(name, args, baseline, rows, header, failed, lowered, checked):
    if args.update:
        merged = {key: count for key, count in baseline.items() if key[0] not in checked}
        merged.update(dict(rows))
        merged = {key: count for key, count in merged.items() if (ROOT / key[0]).is_dir()}
        write_baseline(args.baseline, header, merged)
        return 0
    if failed:
        print(f"{name}: FAIL, a count went up against {shown(args.baseline)}", file=sys.stderr)
        return 1
    if lowered:
        print(f"{name}: counts went down; run with --update to lock them in")
    print(f"{name}: PASS")
    return 0


def unsafe_command(args):
    baseline = read_baseline(args.baseline)
    tracked = {key[0] for key in baseline}
    crates = crate_dirs(args.patterns, baseline)
    rows = []
    for crate in crates:
        directory = ROOT / crate
        if not directory.is_dir():
            print(f"{crate}: gone (drop it from the baseline with --update)")
            continue
        count = 0
        for source in sorted(directory.rglob("*.rs")):
            if "target" in source.relative_to(directory).parts:
                continue
            text = source.read_text(errors="replace")
            count += sum(1 for line in text.splitlines() if UNSAFE_PATTERN.search(line))
        rows.append(((crate,), count))
    failed, lowered = judge(rows, baseline, tracked, "crate")
    print(f"total {sum(count for _, count in rows)} unsafe sites over {len(rows)} crates")
    header = [
        "unsafe sites per crate: lines matching `unsafe (fn|\\{|impl|extern)` in *.rs files",
        "under the crate folder. Checked by tools/ratchets/unsafe-count.sh (make ratchet-unsafe).",
        "Counts may only go down; regenerate with tools/ratchets/unsafe-count.sh --update.",
    ]
    return finish("unsafe-count", args, baseline, rows, header, failed, lowered, {crate for (crate,), _ in rows})


def package_name(crate):
    manifest = (ROOT / crate / "Cargo.toml").read_text()
    section = re.search(r"^\[package\]\s*$(.*?)(?=^\[|\Z)", manifest, re.S | re.M)
    name = section and re.search(r'^name\s*=\s*"([^"]+)"', section.group(1), re.M)
    if not name:
        raise SystemExit(f"panic-lints: no [package] name in {crate}/Cargo.toml")
    return name.group(1)


def bazel_command():
    sys.path.insert(0, str(ROOT / "tools" / "bazel"))
    import runtime  # noqa: E402  (repo-local module, same launcher as tools/bazel/runtime.py)

    return runtime.bazel_command()


def panic_lints_command(args):
    baseline = read_baseline(args.baseline)
    tracked = {key[0] for key in baseline}
    crates = [crate.rstrip("/") for crate in args.patterns] or crate_dirs([], baseline)
    crates = [crate for crate in crates if (ROOT / crate / "Cargo.toml").is_file()]
    if not crates:
        print("panic-lints: no crates to lint", file=sys.stderr)
        return 2
    labels = {crate: f"//{crate}:{package_name(crate)}" for crate in crates}
    # The Bazel clippy aspect lints any rust_library, including the vendored
    # luaur crates that cargo clippy skips (they are workspace-excluded path
    # patches), with the pinned toolchain's clippy, so counts do not drift
    # with the host's cargo. Lib targets only, default feature graph.
    flags = [
        "--aspects=@rules_rust//rust:defs.bzl%rust_clippy_aspect",
        "--output_groups=clippy_output",
        "--@rules_rust//rust/settings:clippy_output_diagnostics=true",
        "--show_result=1000",
        *[f"--@rules_rust//rust/settings:clippy_flag=--force-warn={lint}" for lint in PANIC_LINTS],
    ]
    command = [*bazel_command(), "build", *flags, *labels.values()]
    print("panic-lints: " + " ".join(command), file=sys.stderr)
    result = subprocess.run(command, cwd=ROOT, stderr=subprocess.PIPE, text=True)
    if result.returncode != 0:
        sys.stderr.write(result.stderr[-20000:])
        print("panic-lints: Bazel clippy build failed", file=sys.stderr)
        return 2
    outputs = {}
    for line in result.stderr.splitlines():
        line = line.strip()
        if line.endswith(".clippy.diagnostics"):
            outputs[Path(line).parent.as_posix()] = ROOT / line
    rows = []
    others = {}
    for crate in crates:
        diagnostics = outputs.get(f"bazel-bin/{crate}")
        if diagnostics is None or not diagnostics.is_file():
            print(f"panic-lints: no clippy diagnostics for {labels[crate]}", file=sys.stderr)
            return 2
        counts = dict.fromkeys(PANIC_LINTS, 0)
        other = 0
        for line in diagnostics.read_text(errors="replace").splitlines():
            try:
                message = json.loads(line)
            except json.JSONDecodeError:
                continue
            if message.get("$message_type") != "diagnostic":
                continue
            code = (message.get("code") or {}).get("code") or ""
            if code in counts:
                counts[code] += 1
            elif code.startswith("clippy::"):
                other += 1
        rows += [((crate, lint), count) for lint, count in counts.items()]
        others[crate] = other
    failed, lowered = judge(rows, baseline, tracked, "crate lint")
    for crate in crates:
        total = sum(count for (owner, _), count in rows if owner == crate)
        print(f"{crate}: {total} forced panic-lint sites ({others[crate]} other clippy diagnostics, not ratcheted)")
    header = [
        "Forced clippy panic-lint sites per crate and lint (lib target, default feature graph,",
        "pinned toolchain clippy via the Bazel clippy aspect, each lint --force-warn).",
        "Checked by tools/ratchets/panic-lints.sh (make ratchet-panic-lints).",
        "Counts may only go down; regenerate with tools/ratchets/panic-lints.sh --update.",
    ]
    return finish("panic-lints", args, baseline, rows, header, failed, lowered, set(crates))


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("ratchet", choices=["unsafe", "panic-lints"])
    parser.add_argument("--update", action="store_true", help="rewrite the baseline entries for the crates checked")
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("patterns", nargs="*", help="unsafe: extra crate globs; panic-lints: crate folders (default: baseline crates and vendor/luaur-*)")
    args = parser.parse_args()
    if args.ratchet == "unsafe":
        args.baseline = (args.baseline or ROOT / "tests/ratchets/unsafe-baseline.txt").resolve()
        return unsafe_command(args)
    args.baseline = (args.baseline or ROOT / "tests/ratchets/panic-lints-baseline.txt").resolve()
    return panic_lints_command(args)


if __name__ == "__main__":
    raise SystemExit(main())
