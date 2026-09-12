"""Bind all 794 latest public outputs, then require exact new CLI bytes/maps.

Usage: SCRIPT NEW_FROZEN_DIRECTORY FRESH_OUTPUT_DIRECTORY
The prior corpus uses actual content-owner checkpoint output, including all
75 files changed at that checkpoint. It never reuses stale pre-owner bytes.
"""
from pathlib import Path
import hashlib
import json
import shutil
import subprocess
import sys

module = Path(__file__).resolve().parents[1]
frozen, output = map(lambda value: Path(value).resolve(), sys.argv[1:3])
assert str(output).startswith(str(module / "output/public-transport-malformed-"))
output.mkdir(parents=True, exist_ok=False)
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest()
read = lambda path: json.loads(Path(path).read_text())
def write(path, value):
    Path(path).write_text(json.dumps(value, indent=2) + "\n")

prior_file = module / "output/public-content-owner-regression-r1/manifest.json"
prior = read(prior_file)
assert prior["compilerSha256"] == "746caad8f78f251acbf188f75d10bdc9340c6cd7d48a994a844e6b0898b30ad5"
assert len(prior["results"]) == prior["total"] == 794
bindings = read(frozen / "source-bindings.json")["files"]
for binding in bindings:
    assert sha(binding.get("snapshot", binding["path"])) == binding["sha256"]
inputs = output / "frozen"
inputs.mkdir()
for source, name in [(Path(__file__), "regression.py"), (prior_file, "prior-manifest.json"),
                     (frozen / "source-bindings.json", "source-bindings.json"), (frozen / "html-to-riv", "html-to-riv")]:
    shutil.copy2(source, inputs / name)
input_hashes = {name: sha(inputs / name) for name in ["regression.py", "prior-manifest.json", "source-bindings.json", "html-to-riv"]}
references = []
for index, row in enumerate(prior["results"]):
    assert row["exitCode"] == 0
    source = Path(row["result"])
    directory = output / "references" / str(index)
    directory.mkdir(parents=True)
    hashes = {}
    for name, expected in [("request.json", row["requestSha256"]), ("scene.riv", row["actualRivSha256"]), ("scene.map.json", row["actualMapSha256"])]:
        assert sha(source / name) == expected
        shutil.copy2(source / name, directory / name)
        assert sha(directory / name) == expected
        hashes[name] = expected
    references.append({"index": index, "origin": row.get("origin"), "reference": str(directory), "hashes": hashes})
write(output / "references.json", references)
results = []
for reference in references:
    index = reference["index"]
    directory = output / "results" / str(index)
    directory.mkdir(parents=True)
    request = Path(reference["reference"]) / "request.json"
    command = [str(inputs / "html-to-riv"), str(request), str(directory / "scene.riv")]
    try:
        process = subprocess.run(command, capture_output=True, timeout=10)
        code, stdout, stderr = process.returncode, process.stdout, process.stderr
        termination = None
    except subprocess.TimeoutExpired as error:
        code, stdout, stderr, termination = None, error.stdout or b"", error.stderr or b"", "timeout"
    (directory / "stdout.log").write_bytes(stdout)
    (directory / "stderr.log").write_bytes(stderr)
    actual = {name: sha(directory / name) if (directory / name).exists() else None for name in ["scene.riv", "scene.map.json"]}
    exact = code == 0 and not stdout and not stderr and all(actual[name] == reference["hashes"][name] for name in actual)
    result = {"index": index, "command": command, "exitCode": code, "termination": termination, "exact": exact,
              "reference": reference, "actualHashes": actual, "stdoutSha256": sha(directory / "stdout.log"), "stderrSha256": sha(directory / "stderr.log")}
    write(directory / "result.json", result)
    results.append(result)
assert all(sha(inputs / name) == expected for name, expected in input_hashes.items())
assert all(sha(binding.get("snapshot", binding["path"])) == binding["sha256"] for binding in bindings)
passed = sum(result["exact"] for result in results)
receipt = {"scope": "794 latest actual public request/Rive/map triples; exact byte equality only, no new native or browser qualification",
           "command": [sys.executable, str(Path(__file__).resolve()), str(frozen), str(output)], "total": 794, "passed": passed,
           "allExact": passed == 794, "inputHashes": input_hashes, "priorManifestSha256": sha(prior_file),
           "sourceBindingsVerified": len(bindings), "referencesSha256": sha(output / "references.json"), "results": results}
write(output / "receipt.json", receipt)
print(json.dumps({"total": 794, "passed": passed, "receipt": str(output / "receipt.json")}))
raise SystemExit(0 if passed == 794 else 1)
