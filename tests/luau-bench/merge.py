#!/usr/bin/env python3
"""Write self-contained copies of the upstream Luau bench scripts.

Each upstream script loads `bench_support.lua` with `require`. The Rive Luau
profile has no `require`, so this inlines the support module ahead of the
script and stubs `require` to return it:

    <bench_support.lua without its final `return bench`>
    __bench_support_module = bench
    local script = nil
    local function require(_) return __bench_support_module end
    <the bench script, unchanged>

Output names are `<folder>__<stem>.lua`, for example `tests__sieve.lua`.
The output folder is build output; do not commit it.

Usage: merge.py --out DIR [--upstream DIR]
"""

from pathlib import Path
import argparse
import shutil
import sys

HERE = Path(__file__).resolve().parent

# Benches left out of the suite. These use Luau's experimental `class` syntax,
# which needs the experimental-classes compiler flag; neither the C++ CLI at
# the default flags nor the vendored luaur compiler accepts them, so they fail
# to compile everywhere (docs/evidence/luau-safe-vm-bakeoff-2026-10-31.md,
# section 2). Listed by name, not matched by a pattern, so a new upstream
# bench is never dropped silently.
EXCLUDED = {
    "micro_tests/test_OOP_constructor_classes.lua",
    "micro_tests/test_OOP_constructor_classes_direct.lua",
    "micro_tests/test_OOP_field_access_classes.lua",
    "micro_tests/test_OOP_field_access_random_classes.lua",
    "micro_tests/test_OOP_method_access_classes.lua",
    "micro_tests/test_OOP_method_call_class.lua",
    "tests/chess-classes.lua",
}

FOLDERS = ("micro_tests", "tests")

STUB = (
    "__bench_support_module = bench\n"
    "\n"
    "local script = nil\n"
    "local function require(_) return __bench_support_module end\n"
)


def support_prelude(upstream):
    text = (upstream / "bench_support.lua").read_text()
    stripped = text.rstrip()
    if not stripped.endswith("return bench"):
        raise SystemExit("bench_support.lua no longer ends with `return bench`; update merge.py")
    return stripped[: -len("return bench")] + "\n" + STUB


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--upstream", type=Path, default=HERE / "upstream")
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()

    prelude = support_prelude(args.upstream)
    found = set()
    written = 0
    if args.out.exists():
        shutil.rmtree(args.out)
    args.out.mkdir(parents=True)
    for folder in FOLDERS:
        for script in sorted((args.upstream / folder).glob("*.lua")):
            relative = f"{folder}/{script.name}"
            found.add(relative)
            if relative in EXCLUDED:
                continue
            body = script.read_bytes().decode("utf-8")
            (args.out / f"{folder}__{script.name}").write_text(prelude + body)
            written += 1
    stale = sorted(EXCLUDED - found)
    if stale:
        raise SystemExit("excluded benches missing from upstream copy: " + ", ".join(stale))
    print(f"luau-bench: wrote {written} merged scripts to {args.out} ({len(EXCLUDED)} excluded)", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
