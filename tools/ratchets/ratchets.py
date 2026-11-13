#!/usr/bin/env python3
"""Count-only-goes-down ratchets for the Luau VM work (docs/luau-safe-vm-plan.md, section 7).

    ratchets.py unsafe [--update] [--baseline FILE] [GLOB...]
    ratchets.py panic-lints [--update] [--baseline FILE] [CRATE_DIR[:TARGET][@PLATFORM]...]

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
import hashlib
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


def crate_folder(key):
    """The crate folder of a baseline key; panic-lint variant rows are `CRATE_DIR:TARGET@PLATFORM`."""
    return key.partition("@")[0].partition(":")[0]


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
        merged = {key: count for key, count in merged.items() if (ROOT / crate_folder(key[0])).is_dir()}
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


# Platforms a variant row may name after `@`, with the flags of the artifact
# built for it (tools/luau-audio-web/pool.py build for the browser audio kernel).
PLATFORMS = {
    "wasm": [
        "--platforms=@rules_rust//rust/platform:wasm",
        "--@rules_rust//rust/settings:extra_rustc_flag=-Cpanic=abort",
    ],
}
DIAGNOSTICS_SUFFIX = ".clippy.diagnostics"


def lint_target(spec):
    """`CRATE_DIR[:TARGET][@PLATFORM]` -> (crate dir, Bazel label, platform or None).

    A bare crate folder lints its default label `//CRATE_DIR:<package name>`.
    TARGET names another rust_library in the crate's package, for example a
    feature variant such as `nuxie-scripting__publisher`; PLATFORM is a key of
    PLATFORMS and builds that target the way the artifact using it is built.
    """
    rest, _, platform = spec.rstrip("/").partition("@")
    crate, _, target = rest.partition(":")
    crate = crate.rstrip("/")
    if platform and platform not in PLATFORMS:
        raise ValueError(f"unknown platform {platform!r} in {spec!r} (known: {', '.join(sorted(PLATFORMS))})")
    if not (ROOT / crate / "Cargo.toml").is_file():
        raise ValueError(f"no Cargo.toml in {crate!r} ({spec!r})")
    return crate, f"//{crate}:{target or package_name(crate)}", platform or None


def diagnostics_key(path):
    """`bazel-bin/<pkg>/<target>.clippy.diagnostics` (or under bazel-out/<config>/bin) -> `<pkg>/<target>`."""
    parts = Path(path).parts
    if parts[:1] == ("bazel-bin",):
        parts = parts[1:]
    elif parts[:1] == ("bazel-out",) and len(parts) > 3 and parts[2] == "bin":
        parts = parts[3:]
    return Path(*parts).as_posix()[: -len(DIAGNOSTICS_SUFFIX)]


def label_key(label):
    package, _, name = label.removeprefix("//").partition(":")
    return f"{package}/{name}"


def clippy_diagnostics(stderr):
    """{`<pkg>/<target>`: diagnostics file} for every clippy output the build reported."""
    outputs = {}
    for line in stderr.splitlines():
        line = line.strip()
        if line.endswith(DIAGNOSTICS_SUFFIX):
            # Resolved now: bazel-bin moves with the configuration of the next build.
            outputs[diagnostics_key(line)] = (ROOT / line).resolve()
    matches = re.findall(r'^Pool build result: 0; evidence: (.+)$', stderr, re.MULTILINE)
    if not matches:
        return outputs
    evidence = Path(matches[-1]).resolve()
    if not evidence.is_relative_to((ROOT / '.nuxie/homelab').resolve()):
        raise ValueError('pool evidence is outside this checkout')
    report = json.loads((evidence / 'report.json').read_text())
    if report.get('exitCode') != 0:
        raise ValueError('pool Clippy build did not succeed')
    for name, digest in report.get('outputs', {}).items():
        relative = Path(name)
        if not name.endswith(DIAGNOSTICS_SUFFIX):
            continue
        if relative.is_absolute() or '..' in relative.parts or relative.parts[0] != 'products':
            raise ValueError('unsafe pool diagnostic path')
        path = (evidence.parent / relative).resolve()
        if not path.is_relative_to(evidence.parent) or hashlib.sha256(path.read_bytes()).hexdigest() != digest:
            raise ValueError('pool diagnostic digest mismatch')
        outputs[diagnostics_key(relative.relative_to('products'))] = path
    return outputs


def count_diagnostics(path):
    counts = dict.fromkeys(PANIC_LINTS, 0)
    other = 0
    for line in path.read_text(errors="replace").splitlines():
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
    return counts, other


def panic_lints_command(args):
    baseline = read_baseline(args.baseline)
    tracked = {key[0] for key in baseline}
    specs = [spec.rstrip("/") for spec in args.patterns] or crate_dirs([], baseline)
    targets = {}
    for spec in specs:
        try:
            targets[spec] = lint_target(spec)
        except ValueError as error:
            if args.patterns:
                print(f"panic-lints: {error}", file=sys.stderr)
                return 2
            print(f"panic-lints: skipping {spec}: {error}", file=sys.stderr)
    if not targets:
        print("panic-lints: no crates to lint", file=sys.stderr)
        return 2
    # The Bazel clippy aspect lints any rust_library, including the vendored
    # luaur crates that cargo clippy skips (they are workspace-excluded path
    # patches), with the pinned toolchain's clippy, so counts do not drift
    # with the host's cargo. Lib targets only; a bare crate folder lints the
    # default feature graph, a `:TARGET` row a named variant, one Bazel build
    # per platform.
    flags = [
        "--aspects=@rules_rust//rust:defs.bzl%rust_clippy_aspect",
        "--output_groups=clippy_output",
        "--@rules_rust//rust/settings:clippy_output_diagnostics=true",
        "--show_result=1000",
        *[f"--@rules_rust//rust/settings:clippy_flag=--force-warn={lint}" for lint in PANIC_LINTS],
    ]
    rows = []
    others = {}
    for platform in sorted({platform or "" for _, _, platform in targets.values()}):
        group = {spec: target for spec, target in targets.items() if (target[2] or "") == platform}
        labels = sorted({label for _, label, _ in group.values()})
        command = [*bazel_command(), "build", *flags, *PLATFORMS.get(platform, []), *labels]
        print("panic-lints: " + " ".join(command), file=sys.stderr)
        result = subprocess.run(command, cwd=ROOT, stderr=subprocess.PIPE, text=True)
        if result.returncode != 0:
            sys.stderr.write(result.stderr[-20000:])
            print("panic-lints: Bazel clippy build failed", file=sys.stderr)
            return 2
        try:
            outputs = clippy_diagnostics(result.stderr)
        except (OSError, ValueError) as error:
            print(f'panic-lints: {error}', file=sys.stderr)
            return 2
        for spec, (_, label, _) in group.items():
            diagnostics = outputs.get(label_key(label))
            if diagnostics is None or not diagnostics.is_file():
                print(f"panic-lints: no clippy diagnostics for {label}", file=sys.stderr)
                return 2
            counts, others[spec] = count_diagnostics(diagnostics)
            rows += [((spec, lint), count) for lint, count in counts.items()]
    rows.sort()
    failed, lowered = judge(rows, baseline, tracked, "crate lint")
    for spec in sorted(targets):
        total = sum(count for (owner, _), count in rows if owner == spec)
        print(f"{spec}: {total} forced panic-lint sites ({others[spec]} other clippy diagnostics, not ratcheted)")
    header = [
        "Forced clippy panic-lint sites per crate and lint (lib target, pinned toolchain clippy",
        "via the Bazel clippy aspect, each lint --force-warn). A bare crate folder is the default",
        "label and feature graph; CRATE_DIR:TARGET@PLATFORM is a named variant built for a platform",
        "(PLATFORMS in tools/ratchets/ratchets.py).",
        "Checked by tools/ratchets/panic-lints.sh (make ratchet-panic-lints).",
        "Counts may only go down; regenerate with tools/ratchets/panic-lints.sh --update.",
    ]
    return finish("panic-lints", args, baseline, rows, header, failed, lowered, set(targets))


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("ratchet", choices=["unsafe", "panic-lints"])
    parser.add_argument("--update", action="store_true", help="rewrite the baseline entries for the crates checked")
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("patterns", nargs="*", help="unsafe: extra crate globs; panic-lints: crate folders, or CRATE_DIR:TARGET@PLATFORM for a named variant (default: baseline rows and vendor/luaur-*)")
    args = parser.parse_args()
    if args.ratchet == "unsafe":
        args.baseline = (args.baseline or ROOT / "tests/ratchets/unsafe-baseline.txt").resolve()
        return unsafe_command(args)
    args.baseline = (args.baseline or ROOT / "tests/ratchets/panic-lints-baseline.txt").resolve()
    return panic_lints_command(args)


if __name__ == "__main__":
    raise SystemExit(main())
