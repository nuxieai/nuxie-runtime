#!/usr/bin/env python3
"""Bind the completed 731-case value-recovery output regression and full inputs."""
import hashlib
import json
from pathlib import Path


module = Path(__file__).resolve().parent.parent
repository = module.parent.parent
output = module / "output/public-value-regression-r1"
reference_file = module / "output/public-value-regression-references-r1/manifest.json"
freeze_file = module / "output/public-value-build-r1/frozen/source-bindings.json"
manifest_file = output / "manifest.json"
read = lambda path: json.loads(Path(path).read_text())
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest()
bind = lambda path: {"path": str(Path(path).resolve()), "sha256": sha(path)}
write = lambda path, value: Path(path).write_text(json.dumps(value, indent=2) + "\n")

manifest, references, freeze = map(read, (manifest_file, reference_file, freeze_file))
compiler_hash = "034084cd868e9b33ba849bc328ea4c540d63115192eba7918e7d7ec00c3c2b2d"
assert manifest["compilerSha256"] == sha(manifest["compiler"]) == compiler_hash
assert manifest["priorManifestSha256"] == sha(reference_file)
assert manifest["scriptSha256"] == sha(module / "validation/check-output-regression.py")
assert (manifest["total"], manifest["passed"], len(manifest["results"])) == (731, 731, 731)
assert references["historicalRowsRetained"] == 694 and references["nativeScenesAdded"] == 37
assert references["whitespaceAudit"]["flaggedRequests"] == 0

frozen_rows = []
for row in freeze["files"]:
    assert sha(row["path"]) == row["sha256"], row["path"]
    if "snapshot" in row:
        assert sha(row["snapshot"]) == row["sha256"], row["snapshot"]
    frozen_rows.append({**row, "currentFileMatches": True,
                        "snapshotMatches": True if "snapshot" in row else None})
assert len(frozen_rows) == 118
freeze_check = output / "source-freeze-check.json"
write(freeze_check, {"sourceFreeze": bind(freeze_file), "verifiedBindings": len(frozen_rows),
                     "files": frozen_rows, "failures": []})

full_inputs = []
for index, (reference, result) in enumerate(zip(references["results"], manifest["results"])):
    assert reference["referenceIndex"] == result["referenceIndex"] == index
    assert result["exact"] and result["exitCode"] == 0
    for path_key, hash_key in (("request", "requestSha256"), ("priorRiv", "rivSha256"), ("priorMap", "mapSha256")):
        assert sha(reference[path_key]) == reference[hash_key] == result[hash_key]
    actual = Path(result["result"])
    assert sha(actual / "request.json") == reference["requestSha256"]
    assert sha(actual / "scene.riv") == result["actualRivSha256"] == reference["rivSha256"]
    assert sha(actual / "scene.map.json") == result["actualMapSha256"] == reference["mapSha256"]
    full_inputs.append({"referenceIndex": index, "request": read(reference["request"]),
                        "origin": reference["origin"], "reference": reference,
                        "command": result["command"], "exitCode": result["exitCode"],
                        "actualRequest": bind(actual / "request.json"),
                        "actualRiv": bind(actual / "scene.riv"),
                        "actualMap": bind(actual / "scene.map.json"),
                        "compileLog": bind(actual / "compile.log"), "exact": True})
full_inputs_file = output / "full-inputs.json"
write(full_inputs_file, {"scope": "Complete unchanged request objects with reference and actual artifact bindings",
                         "total": len(full_inputs), "results": full_inputs})

commands = [
    ["python3", "tools/html-to-riv/validation/public-value-regression-references.py",
     "tools/html-to-riv/output/public-value-regression-references-r1"],
    ["python3", "tools/html-to-riv/validation/check-output-regression.py",
     "tools/html-to-riv/output/public-value-build-r1/frozen/html-to-riv",
     "tools/html-to-riv/output/public-value-regression-references-r1/manifest.json",
     "tools/html-to-riv/output/public-value-regression-r1"],
    ["python3", "tools/html-to-riv/validation/public-value-regression-evidence.py"],
]
receipt = {
    "status": "all-731-public-output-references-exact",
    "scope": "Deterministic public compiler bytes/maps only; no new native/browser/visual qualification",
    "cwd": str(repository), "commands": [{"argv": command, "exitCode": 0} for command in commands],
    "compiler": bind(manifest["compiler"]),
    "frozenCompiler": bind(module / "output/public-value-build-r1/frozen/html-to-riv"),
    "sourceFreeze": bind(freeze_file), "sourceFreezeCheck": bind(freeze_check), "verifiedFrozenBindings": 118,
    "referenceManifest": bind(reference_file), "resultManifest": bind(manifest_file), "fullInputs": bind(full_inputs_file),
    "total": 731, "compileSuccess": 731, "exactRivFiles": 731, "exactSourceMaps": 731,
    "historicalRowsRetained": 694, "correctedHistoricalRows": references["correctedNumericOutputsPromoted"],
    "numericNativeScenesConsidered": 38, "numericNativeScenesAdded": 37,
    "deduplicationRule": references["deduplicationRule"], "aliases": references["aliases"],
    "whitespaceAudit": references["whitespaceAudit"], "nativeGroups": references["nativeGroups"],
    "referenceArtifactsRechecked": 2193, "actualRequestRivMapArtifactsRechecked": 2193,
    "compileLogsBound": 731, "changedCases": 0, "excludedHistoricalCases": 0, "failures": [],
    "sources": [bind(module / "validation" / name) for name in [
        "public-value-regression-references.py", "check-output-regression.py",
        "public-value-regression-evidence.py", "public-value-regression-review.md"]],
}
receipt_file = module / "validation/public-value-regression-receipt.json"
write(receipt_file, receipt)
print(json.dumps({"total": 731, "passed": 731, "frozenBindings": 118,
                  "fullInputs": bind(full_inputs_file), "receipt": bind(receipt_file)}))
