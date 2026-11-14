#!/usr/bin/env python3
"""`make luau-abort-gate`: the Luau client path without unwinding.

docs/luaur-client-path-safe.md section 8 (lane F); evidence:
docs/evidence/luaur-client-path-lane-h4-2026-11-14.md.

Three runs, each to its end whatever the others do:
1. nuxie-luau-vm and nuxie-scripting test suites built with -Cpanic=abort
   (NUX_RUNTIME_PANIC=abort, .bazelrc config panic-abort), minus the tests
   in excluded-tests.txt and the corpus test (run 2 prints its table).
2. the hostile corpus (crates/nuxie-scripting/tests/hostile) under abort;
3. the hostile corpus under the default unwind strategy.
Exit status is 1 when any run failed.
"""
import os
import re
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
EXCLUDED = Path(__file__).resolve().parent / "excluded-tests.txt"
PACKAGES = {"nuxie-luau-vm": ROOT / "crates/nuxie-luau-vm", "nuxie-scripting": ROOT / "crates/nuxie-scripting"}


def excluded_tests():
    tests, problems = [], []
    for number, line in enumerate(EXCLUDED.read_text().splitlines(), 1):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        fields = [field.strip() for field in line.split("|", 2)]
        if len(fields) != 3 or not all(fields):
            problems.append(f"{EXCLUDED.name}:{number}: want `package | test | reason`")
            continue
        package, name, _reason = fields
        if package not in PACKAGES:
            problems.append(f"{EXCLUDED.name}:{number}: unknown package {package}")
            continue
        function = name.rsplit("::", 1)[-1]
        pattern = re.compile(r"\bfn\s+" + re.escape(function) + r"\s*\(")
        if not any(pattern.search(path.read_text(errors="replace")) for path in PACKAGES[package].rglob("*.rs")):
            problems.append(f"{EXCLUDED.name}:{number}: no test `{function}` in {package}; remove the line")
        tests.append(name)
    return tests, problems


def run(label, arguments, abort):
    environment = dict(os.environ)
    if abort:
        environment["NUX_RUNTIME_PANIC"] = "abort"
    else:
        environment.pop("NUX_RUNTIME_PANIC", None)
    command = [sys.executable, str(ROOT / "tools/bazel/runtime.py"), "test", *arguments]
    print(f"\n=== {label}\n$ {'NUX_RUNTIME_PANIC=abort ' if abort else ''}{' '.join(command[1:])}", flush=True)
    started = time.monotonic()
    status = subprocess.run(command, cwd=ROOT, env=environment).returncode
    print(f"=== {label}: {'PASS' if status == 0 else 'FAIL'} in {time.monotonic() - started:.0f}s", flush=True)
    return status == 0


def main():
    tests, problems = excluded_tests()
    for problem in problems:
        print(problem, file=sys.stderr)
    skips = ["--exact", "--skip", "hostile_corpus"]
    for name in tests:
        skips += ["--skip", name]
    results = {
        "abort suites": run("abort suites (nuxie-luau-vm, nuxie-scripting)",
                            ["-p", "nuxie-luau-vm", "-p", "nuxie-scripting", "--", *skips], abort=True),
        "abort corpus": run("hostile corpus, panic=abort",
                            ["-p", "nuxie-scripting", "--test", "hostile_corpus", "--", "--nocapture"], abort=True),
        "unwind corpus": run("hostile corpus, panic=unwind",
                             ["-p", "nuxie-scripting", "--test", "hostile_corpus", "--", "--nocapture"], abort=False),
    }
    print("\nluau-abort-gate:", ", ".join(f"{name} {'PASS' if ok else 'FAIL'}" for name, ok in results.items()),
          f"; {len(tests)} tests excluded from abort" + (f"; {len(problems)} exclusion-list problems" if problems else ""))
    return 0 if all(results.values()) and not problems else 1


if __name__ == "__main__":
    sys.exit(main())
