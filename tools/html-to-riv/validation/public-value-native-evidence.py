#!/usr/bin/env python3
"""Verify bounded value-recovery evidence and bind its completed visual review."""
import difflib
import hashlib
import json
import re
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
VALIDATION = ROOT / "validation"
OUT = ROOT / "output/public-value-native-r1"
BUILD = ROOT / "output/public-value-build-r1"
FREEZE = BUILD / "frozen"


def read(path):
    return json.loads(path.read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def bind(path):
    return {"path": str(path), "sha256": sha(path)}


verified = 0


def check(path, expected):
    global verified
    assert sha(path) == expected, str(path)
    verified += 1


freeze = read(FREEZE / "source-bindings.json")
for item in freeze["files"]:
    for key in ["path", "snapshot"]:
        if key in item:
            check(Path(item[key]), item["sha256"])
assert len(freeze["files"]) == 118
build = read(BUILD / "summary.json")
assert build["status"] == "public-build-and-transport-pass" and build["sourceUnchanged"]
assert (build["rustTests"], build["nodeTests"]) == (262, 42)
assert sum(map(int, re.findall(r"test result: ok\. (\d+) passed;", (BUILD / "full-rust.log").read_text()))) == 262
assert "tests 42" in (BUILD / "full-node.log").read_text()
for item in read(BUILD / "checks.json"):
    assert item["exitCode"] == 0
    check(Path(item["log"]), item["logSha256"])
check(FREEZE / "html-to-riv", build["compilerSha256"])
check(FREEZE / "compiler.wasm", build["wasmSha256"])
assert read(OUT / "runtime-guard.json")["status"] == "pass"

# The sole integration-source change after the read-only audit is explicit.
audit = read(OUT / "grammar-audit-bindings-r2.json")
audit_deltas = []
for item in audit["sources"]:
    snapshot, live = Path(item["snapshot"]), Path(item["path"])
    check(snapshot, item["sha256"])
    if sha(live) != item["sha256"]:
        assert live.name == "compiler.rs"
        before, after = snapshot.read_text(), live.read_text()
        old = 'unsupported(path, "Text rendering is not yet requalified on the immutable runtime")'
        new = 'unsupported(if path.is_empty() { "html" } else { path }, "Text rendering is not yet requalified on the immutable runtime")'
        assert before.count(old) == 1 and before.replace(old, new) == after
        audit_deltas.append({"before": bind(snapshot), "after": bind(live), "review": "Only top-level unsupported-text diagnostic provenance changes to html; substitution/classification/cascade unchanged.", "diff": "".join(difflib.unified_diff(before.splitlines(True), after.splitlines(True)))})
assert len(audit_deltas) == 1
(OUT / "final-source-audit.json").write_text(json.dumps({"sourceFreeze": bind(FREEZE / "source-bindings.json"), "priorAudit": bind(OUT / "grammar-audit-bindings-r2.json"), "reviewedDeltas": audit_deltas}, indent=2) + "\n")

classifier = read(OUT / "classifier-audit-r2/receipt.json")
assert classifier["cases"] == 594 and not classifier["invalidDespiteBrowserSupport"]
assert len(classifier["grammarValidBrowserRejects"]) == 4
for item in classifier["sources"]:
    for key in ["path", "snapshot"]:
        check(Path(item[key]), item["sha256"])
for item in classifier["bindings"]:
    check(Path(item["path"]), item["sha256"])

browser = read(OUT / "browser-r3/receipt.json")
assert browser["browser"] == "153.0.8010.12"
assert len(browser["rows"]) == 476 and all(not r["failures"] for r in browser["rows"])
assert len(browser["sceneRows"]) == 105
for item in browser["bindings"]:
    check(Path(item["path"]), item["sha256"])
for row in browser["sceneRows"]:
    assert not row["failures"] and not row["paintFailures"]
    for role in ["actual", "control"]:
        check(Path(row[role + "Path"]), row[role + "Sha256"])
    assert row["actualSha256"] == row["controlSha256"]

preflight = read(OUT / "frozen-preflight/receipt.json")
check(Path(preflight["compiler"]["path"]), preflight["compiler"]["sha256"])
assert len(preflight["rows"]) == 105
for row in preflight["rows"]:
    assert row["exactSourceControlBytesAndMap"]
    for role, result in row["results"].items():
        assert result["exitCode"] == 0
        base = Path(result["command"][1]).parent
        for name, key in [("request.json", "requestSha256"), ("scene.riv", "rivSha256"), ("scene.map.json", "mapSha256"), ("compile.log", "logSha256")]:
            check(base / name, result[key])
    assert all(row["results"]["actual"][key] == row["results"]["control"][key] for key in ["rivSha256", "mapSha256"])

render = read(OUT / "render/receipt.json")
assert render["status"] == "passed-public-baseline"
assert render["browser"] == "153.0.8010.12" and render["effectiveMode"] == "RasterOrdering"
assert len(render["artifacts"]) == 35 and len(render["rows"]) == 280
for key, path in render["tools"].items():
    check(Path(path), render["toolHashes"][key])
for item in render["sourceBindings"]:
    check(Path(item["source"]), item["sha256"])
    check(OUT / "render" / item["snapshot"], item["sha256"])
rows = {(r["name"], r["frame"]): r for r in render["rows"]}
for row in render["rows"]:
    assert not row["geometryFailures"] and not row["pixelFailures"]
    for role in ["chrome", "native"]:
        check(Path(row["prefix"] + "." + role + ".png"), row[role + "Sha256"])
    control = next(r for r in browser["sceneRows"] if (r["name"], r["width"], r["height"]) == (row["name"], row["width"], row["height"]))
    assert row["chromeSha256"] == control["actualSha256"]
    for clear in row["clearChecks"]:
        check(Path(clear["path"]), clear["sha256"])
        assert clear["samePixels"]
    base = Path(row["prefix"]).parent
    for name, key in [("request.json", "requestSha256"), ("scene.riv", "rivSha256"), ("scene.map.json", "sourceMapSha256")]:
        check(base / name, row[key])
    frame = next(f for f in read(base / "probe/frames.json")["frames"] if f["frame"] == row["frame"])
    check(base / "probe" / frame["geometry"], row["geometrySha256"])
    check(base / "probe" / frame["stream"], row["streamSha256"])
assert len(render["repeated"]) == 175 and all(r["nativeIdentical"] and r["chromeIdentical"] for r in render["repeated"])

coverage, sheets, review = [read(OUT / name) for name in ["visual-coverage.json", "review-sheets.json", "direct-review.json"]]
check(OUT / "render/receipt.json", coverage["renderReceiptSha256"])
check(OUT / "visual-coverage.json", sheets["coverageSha256"])
check(OUT / "review-sheets.json", review["reviewSheetsSha256"])
assert len(coverage["coverage"]) == 280 and (coverage["directPairs"], coverage["transferredPairs"]) == (37, 243)
assert review["status"] == "complete" and review["sheets"] == sheets["sheets"]
assert len(sheets["sheets"]) == 10 and len(sheets["placements"]) == 74
for sheet in sheets["sheets"]:
    check(Path(sheet["path"]), sheet["sha256"])
assert all(p["completeRgbaExact"] for p in sheets["placements"])
for item in coverage["coverage"]:
    if item["method"] == "direct":
        continue
    assert item["method"] == "complete-rgba-white-canvas"
    row, reference = rows[(item["name"], item["frame"])], rows[(item["name"], item["referenceFrame"])]
    assert all(row[k] == reference[k] for k in ["requestSha256", "rivSha256", "sourceMapSha256"])
    for role in ["chrome", "native"]:
        images = [Image.open(r["prefix"] + "." + role + ".png").convert("RGBA") for r in [row, reference]]
        size = (max(i.width for i in images), max(i.height for i in images))
        canvases = []
        for img in images:
            canvas = Image.new("RGBA", size, "white")
            canvas.paste(img, (0, 0))
            canvases.append(canvas)
        assert canvases[0].tobytes() == canvases[1].tobytes()

regression = read(VALIDATION / "public-value-regression-receipt.json")
assert regression["total"] == regression["exactRivFiles"] == regression["exactSourceMaps"] == 731
assert not regression["failures"] and regression["changedCases"] == 0
assert regression["compiler"]["sha256"] == build["compilerSha256"]
for key in ["compiler", "frozenCompiler", "sourceFreeze", "sourceFreezeCheck", "referenceManifest", "resultManifest", "fullInputs"]:
    check(Path(regression[key]["path"]), regression[key]["sha256"])

files = [Path(__file__), VALIDATION / "public-value-native-review.md", FREEZE / "source-bindings.json", BUILD / "summary.json", BUILD / "checks.json", VALIDATION / "public-value-regression-receipt.json", VALIDATION / "public-value-regression-review.md", VALIDATION / "check-target-runtime.py", VALIDATION / "check-public-baseline.mjs", VALIDATION / "pixels.mjs", VALIDATION / "public-empty-variable-visual.py", VALIDATION / "public-numeric-native-visual.py"]
files += [p for p in VALIDATION.glob("public-value-native-*") if p.is_file() and p.name != "public-value-native-receipt.json"]
files += [OUT / name for name in ["native-command.json", "native.log", "render/receipt.json", "visual-coverage.json", "review-sheets.json", "direct-review.json", "runtime-guard.json", "final-source-audit.json", "grammar-audit-bindings.json", "grammar-audit-bindings-r2.json"]]
files += [OUT / name / "receipt.json" for name in ["browser", "browser-r2", "browser-r3", "old-preflight", "old-preflight-r2", "old-preflight-r3", "frozen-preflight", "grammar-probes-r1", "grammar-probes-r2", "grammar-probes-r3", "classifier-audit-r1", "classifier-audit-r2"]]
files += list((OUT / "paused-handoff").iterdir())
receipt = {
    "status": "bounded-public-value-native-qualification-passed",
    "scope": "Conservative nonempty ordinary-variable recovery and CSS whitespace preservation on the unchanged runtime; complete representative native and visual matrix, not arbitrary CSS grammar or layout support.",
    "baseline": "6c7ac16617835b5f581784ff08a9e779bb52faf3", "browser": render["browser"], "backend": render["backend"], "effectiveMode": render["effectiveMode"],
    "compiler": bind(FREEZE / "html-to-riv"), "wasm": bind(FREEZE / "compiler.wasm"),
    "frozenInputs": 118, "verifiedArtifactBindings": verified, "rustTests": 262, "nodeTests": 42,
    "priorOutputs": {"total": 731, "exactRivFiles": 731, "exactSourceMaps": 731, "changed": 0},
    "chromeCharacterization": {"properties": 33, "forms": 476, "invalidLiteralForms": 317, "validLiteralForms": 159, "paintedScenes": 35, "paintedComparisons": 105, "allPass": True},
    "classifierObservation": {"cases": 594, "invalidDespiteBrowserSupport": 0, "draftValidBrowserUnimplemented": classifier["grammarValidBrowserRejects"], "scope": "Grammar-only observation; no public admission count inferred."},
    "sourceControlPreflight": {"comparisons": 105, "acceptedSources": 105, "acceptedControls": 105, "exactRivAndMaps": 105},
    "native": {"scenes": 35, "frames": 280, "geometryPass": 280, "pixelPass": 280, "clearPass": 560, "repeatedViewportIdentities": 175},
    "visualReviewCompleted": True, "visualReviewer": review["reviewer"], "directVisualPairs": 37, "exactFullRgbaTransferredPairs": 243, "inspectedSheets": sheets["sheets"],
    "transferScope": "Within the same fixture and bound request/Rive file/source map: each role separately has complete decoded RGBA equality after explicit white-canvas extension, including every region outside the smaller viewport. Direct unscaled inspection plus verified transfers covers all 280 frame pairs.",
    "retainedNativeFailures": [],
    "limitations": ["This is a representative 35-scene semantic matrix, not native qualification of every grammar form or modern CSS feature.", "Valid unsupported values and opaque/unclassified functions retain diagnostics; direct invalid literals remain strict diagnostics.", "Unsupported initial outcomes, syntax, context, and resource errors remain diagnostics.", "Passing existing pixel gates is not exact Chrome/native color equality.", "Previously documented fractional, large-coordinate and numeric-color discrepancies remain outside this new matrix.", "Initial invocation errors, unsupported losing-margin candidate, quirks-mode probe, and paused handoff are preserved as history.", "Runtime/renderer changes, editor integration, CSS Grid, scripts and interactions remain excluded."],
    "bindings": [bind(p) for p in dict.fromkeys(files)],
}
(VALIDATION / "public-value-native-receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
print(json.dumps({"status": receipt["status"], "native": receipt["native"], "verifiedArtifactBindings": verified, "bindings": len(receipt["bindings"])}))
