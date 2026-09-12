#!/usr/bin/env python3
"""Verify preserved artifacts and bind the completed numeric native review."""
import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VALIDATION = ROOT / "validation"
OUT = ROOT / "output/public-numeric-native-r1"
FREEZE = ROOT / "output/public-numeric-token-r3/frozen"


def read(path):
    return json.loads(path.read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def bind(path):
    return {"path": str(path), "sha256": sha(path)}


def check(path, expected):
    assert sha(path) == expected, str(path)


freeze = read(FREEZE / "source-bindings.json")
verified_frozen_paths = 0
for item in freeze["files"]:
    for key in ["path", "snapshot"]:
        if key in item:
            check(Path(item[key]), item["sha256"])
            verified_frozen_paths += 1
check(Path(freeze["buildChecks"]["path"]), freeze["buildChecks"]["sha256"])
build = read(Path(freeze["buildChecks"]["path"]))
for item in build:
    assert item["exitCode"] == 0
    check(Path(item["log"]), item["logSha256"])
transport = read(FREEZE.parent / "transport-check.json")
assert transport["exitCode"] == 0
for key in ["log", "testSource", "sourceFreeze"]:
    check(Path(transport[key]["path"]), transport[key]["sha256"])
rust_count = sum(map(int, re.findall(r"test result: ok\. (\d+) passed;", (FREEZE.parent / "full-rust.log").read_text())))
assert rust_count == 247
assert "tests 41" in (FREEZE.parent / "full-node.log").read_text()
assert read(OUT / "runtime-guard.json")["status"] == "pass"

totals, retained_failures, inspected_sheets = {}, [], []
for name in ["visible", "large", "fractional"]:
    run = OUT / name
    render = read(run / "render/receipt.json")
    coverage, sheets = read(run / "visual-coverage.json"), read(run / "review-sheets.json")
    assert render["browser"] == "153.0.8010.12"
    assert render["effectiveMode"] == "RasterOrdering"
    assert render["status"] == "failed-public-baseline"
    check(FREEZE / "html-to-riv", render["toolHashes"]["compiler"])
    for key, path in render["tools"].items():
        check(Path(path), render["toolHashes"][key])
    for item in render["sourceBindings"]:
        check(Path(item["source"]), item["sha256"])
        check(run / "render" / item["snapshot"], item["sha256"])
    for row in render["rows"]:
        for role in ["chrome", "native"]:
            check(Path(row["prefix"] + "." + role + ".png"), row[role + "Sha256"])
        for clear in row["clearChecks"]:
            check(Path(clear["path"]), clear["sha256"])
            assert clear["samePixels"]
        base = Path(row["prefix"]).parent
        for file, key in [("request.json", "requestSha256"), ("scene.riv", "rivSha256"), ("scene.map.json", "sourceMapSha256")]:
            check(base / file, row[key])
        # Stream/geometry paths are recorded in the immutable probe manifest.
        frame = next(f for f in read(base / "probe/frames.json")["frames"] if f["frame"] == row["frame"])
        check(base / "probe" / frame["geometry"], row["geometrySha256"])
        check(base / "probe" / frame["stream"], row["streamSha256"])
        if row["geometryFailures"] or row["pixelFailures"]:
            retained_failures.append({"corpus": name, "name": row["name"], "frame": row["frame"], "geometry": row["geometryFailures"], "pixels": row["pixelFailures"]})
    assert all(row["nativeIdentical"] and row["chromeIdentical"] for row in render["repeated"])
    assert len(coverage["coverage"]) == len(render["rows"])
    check(run / "render/receipt.json", coverage["renderReceiptSha256"])
    check(run / "visual-coverage.json", sheets["coverageSha256"])
    assert sheets["directPairs"] == coverage["directPairs"]
    for sheet in sheets["sheets"]:
        check(Path(sheet["path"]), sheet["sha256"])
        inspected_sheets.append({"corpus": name, **sheet})
    totals[name] = {
        "cases": len(render["artifacts"]), "frames": len(render["rows"]),
        "geometryPass": sum(not r["geometryFailures"] for r in render["rows"]),
        "pixelPass": sum(not r["pixelFailures"] for r in render["rows"]),
        "clearPass": sum(c["samePixels"] for r in render["rows"] for c in r["clearChecks"]),
        "directVisualPairs": coverage["directPairs"], "exactFullRgbaTransferredPairs": coverage["transferredPairs"],
        "reviewedSheets": len(sheets["sheets"]), "allRepeatedViewportRoleImagesIdentical": True,
    }
assert [totals[n]["pixelPass"] for n in totals] == [204, 80, 4]
assert [totals[n]["geometryPass"] for n in totals] == [208, 48, 16]
assert len(inspected_sheets) == 22
colors = read(OUT / "color-interiors.json")
assert len(colors["rows"]) == 48
check(VALIDATION / "content-box-rounding-wire.py", colors["decoderSha256"])
for name in ["browser", "browser-r2"]:
    browser_run = read(OUT / name / "receipt.json")
    check(OUT / name / "cases.json", browser_run["fixturesSha256"])
    check(OUT / name / "reset.css", browser_run["resetSha256"])
    assert len(browser_run["rows"]) == 78
    assert sum(bool(r["paintFailures"]) for r in browser_run["rows"]) == (9 if name == "browser" else 0)
color_summary = []
for name in dict.fromkeys(r["name"] for r in colors["rows"]):
    rows = [r for r in colors["rows"] if r["name"] == name]
    assert len(rows) == 8
    assert len({json.dumps(r["rgbaHistograms"], sort_keys=True) for r in rows}) == 1
    color_summary.append({k: rows[0][k] for k in ["name", "ordinaryArgb", "priorOrdinaryArgb", "cssom", "interiorRegion", "rgbaHistograms"]})

files = [Path(__file__), VALIDATION / "public-numeric-native-review.md", FREEZE / "source-bindings.json",
    FREEZE.parent / "build-checks.json", FREEZE.parent / "transport-check.json", FREEZE.parent / "runtime-guard.json",
    VALIDATION / "public-numeric-regression-receipt.json", ROOT / "output/numeric-token-prior-regression-r1/manifest.json",
    VALIDATION / "content-box-rounding-review.md", ROOT / "output/content-box-rounding-r1/receipt.json",
    ROOT / "output/content-box-rounding-r2/receipt.json", VALIDATION / "flex-fixed-controls-review.md",
    VALIDATION / "flex-fixed-controls-receipt.json", ROOT / "output/flex-fixed-controls-r2/edges-render/receipt.json",
    VALIDATION / "public-empty-variable-visual.py", VALIDATION / "check-public-baseline.mjs", VALIDATION / "pixels.mjs",
    VALIDATION / "public-numeric-token-cases.json", VALIDATION / "public-numeric-token-rejections.json",
    VALIDATION / "content-box-rounding-wire.py", ROOT / "output/public-content-box-r2/html-to-riv"]
files += sorted((OUT / "preflight").glob("*/*"))
files += [OUT / name / file for name in ["browser", "browser-r2"] for file in ["cases.json", "reset.css"]]
files += [p for p in VALIDATION.glob("public-numeric-native-*") if p.suffix in [".py", ".mjs", ".json"] and p.name != "public-numeric-native-receipt.json"]
files += [OUT / p for p in ["native-commands.json", "preflight.json", "browser/receipt.json", "browser-command.json", "browser-r2/receipt.json", "browser-r2-command.json", "color-interior-initial-failure.json", "color-interiors.json", "freeze-check.json", "runtime-guard.json", "large/measure/receipt.json", "large/measure-command.json", "fractional/native-command.json", "fractional/comparisons.json"]]
for name in totals:
    files += [OUT / name / p for p in ["render/receipt.json", "visual-coverage.json", "review-sheets.json"]]
receipt = {
    "status": "bounded-numeric-native-evidence-with-retained-failures",
    "scope": "Shared token preservation on the immutable runtime; visible numeric consumers, exact original large sources, and supplemental fractional controls. No blanket browser or color equality claim.",
    "baseline": "6c7ac16617835b5f581784ff08a9e779bb52faf3",
    "browser": "153.0.8010.12", "backend": "rust-metal", "effectiveMode": "RasterOrdering",
    "compiler": bind(FREEZE / "html-to-riv"), "wasm": bind(FREEZE / "compiler.wasm"),
    "verifiedFrozenPaths": verified_frozen_paths, "rustTests": rust_count, "nodeTests": 41,
    "priorOutputs": {"total": 694, "exact": 691, "correctedFilesWithSeparateAudit": 3},
    "totals": totals,
    "frames": 304, "directVisualPairs": 65, "exactFullRgbaTransferredPairs": 239,
    "visualReviewCompleted": True, "inspectedSheets": inspected_sheets,
    "transferScope": "Within the same fixture and bound source/file/map: complete decoded RGBA equality after explicit white-canvas extension, including every region beyond the smaller viewport. All 304 pairs covered.",
    "exactColorInteriors": color_summary,
    "fractionalControl": {"sameSourceReplacementNativeImagesIdentical": 8, "sameSourceReplacementChromeImagesDifferent": 8, "differentChromePixelsPerFrame": 210, "historicalMinimalBothRolesIdentical": 8},
    "limitations": [
        "Four visible fractional pixel failures remain, plus 12 deliberately failing fractional control comparisons.",
        "Large two-million padding retains 32 geometry failures; offscreen children and far edges are not visually qualified.",
        "Left/split padding has a separate 0.0625px Chrome world-rectangle projection difference inside the unchanged 0.1px gate.",
        "All color pixel gates pass but exact one-channel interior differences remain; CSSOM serialization is not a rendered-pixel oracle.",
        "Original browser expectation failures and initial nonuniform-interior observer failure remain preserved.",
        "This receipt is bounded validation; lexical admissions and retained strict diagnostics have independent Rust/CLI/WASM evidence."
    ],
    "retainedFailures": retained_failures,
    "bindings": [bind(p) for p in dict.fromkeys(files)],
}
(VALIDATION / "public-numeric-native-receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
print(json.dumps({"status": receipt["status"], "totals": totals, "bindings": len(receipt["bindings"])}))
