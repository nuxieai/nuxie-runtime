#!/usr/bin/env python3
"""Compare a compiler with previously bound public bytes/maps, retaining failures.
Usage: SCRIPT COMPILER PRIOR_MANIFEST FRESH_OUTPUT
This checks deterministic artifacts, not pixels or source/build provenance.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def check(binary, prior, out):
    binary, prior, out = map(lambda p: Path(p).resolve(), (binary, prior, out))
    script_hash = sha(__file__)
    prior_hash = sha(prior)
    references = json.loads(prior.read_text())["results"]
    if not references:
        raise ValueError("Reference manifest contains no comparisons")
    # Validate the entire reference set before starting any compilation.
    for row in references:
        for path_key, hash_key in [("request", "requestSha256"), ("priorRiv", "rivSha256"), ("priorMap", "mapSha256")]:
            if sha(row[path_key]) != row[hash_key]:
                raise ValueError(f"Changed reference {row[path_key]}")
    out.mkdir(parents=True, exist_ok=False)
    frozen = out / "html-to-riv"
    shutil.copy2(binary, frozen)
    binary_hash = sha(frozen)
    rows = []
    for index, row in enumerate(references):
        dest = out / str(index)
        dest.mkdir()
        request = dest / "request.json"
        shutil.copy2(row["request"], request)
        if sha(request) != row["requestSha256"]:
            raise ValueError("Request changed while copying")
        command = [str(frozen), str(request), str(dest / "scene.riv")]
        result = subprocess.run(command, capture_output=True, text=True)
        (dest / "compile.log").write_text(result.stdout + result.stderr)
        actual_riv, actual_map = dest / "scene.riv", dest / "scene.map.json"
        actual_riv_hash = sha(actual_riv) if actual_riv.exists() else None
        actual_map_hash = sha(actual_map) if actual_map.exists() else None
        rows.append({**row, "command": command, "result": str(dest),
                     "exitCode": result.returncode,
                     "actualRivSha256": actual_riv_hash,
                     "actualMapSha256": actual_map_hash,
                     "exact": result.returncode == 0 and actual_riv_hash == row["rivSha256"] and actual_map_hash == row["mapSha256"]})
    if sha(__file__) != script_hash:
        raise ValueError("Validation script changed during comparison")
    if sha(prior) != prior_hash:
        raise ValueError("Reference manifest changed during comparison")
    if sha(frozen) != binary_hash:
        raise ValueError("Frozen compiler changed during comparison")
    passed = sum(row["exact"] for row in rows)
    receipt = {"scope": "Exact bound public outputs; no rerender or visual qualification",
               "compiler": str(frozen), "compilerSha256": binary_hash,
               "scriptSha256": script_hash, "priorManifest": str(prior),
               "priorManifestSha256": prior_hash, "total": len(rows), "passed": passed,
               "results": rows}
    (out / "manifest.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps({"total": len(rows), "passed": passed, "manifest": str(out / "manifest.json")}))
    return passed == len(rows)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("compiler")
    parser.add_argument("prior_manifest")
    parser.add_argument("fresh_output")
    args = parser.parse_args()
    raise SystemExit(0 if check(args.compiler, args.prior_manifest, args.fresh_output) else 1)
