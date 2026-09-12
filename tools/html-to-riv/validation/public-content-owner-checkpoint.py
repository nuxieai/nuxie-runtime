#!/usr/bin/env python3
"""Read-only verification of the completed, bounded public content-owner checkpoint.

Rechecks existing artifacts; never builds, compiles, probes, renders, or changes
comparison gates. The final source freeze is an explicit list, so unrelated
future experiment files are not inferred to be part of the reviewed build.
Requires Pillow for complete decoded-RGBA identity and contact-sheet checks.
"""
import collections
import hashlib
import json
import math
from pathlib import Path
import re
import subprocess
import sys

from PIL import Image

MODULE = Path(__file__).resolve().parents[1]
OUT = MODULE / "output/public-content-owner-checkpoint-r1"
BUILD = MODULE / "output/public-content-owner-build-r1"
REG = MODULE / "output/public-content-owner-regression-r1"
NATIVE = MODULE / "output/public-content-owner-native-r1"
PRIOR = MODULE / "output/content-owner-visual-review-r1"
CANDIDATE = MODULE / "output/content-owner-candidate-native-r2"
COMPILER = "746caad8f78f251acbf188f75d10bdc9340c6cd7d48a994a844e6b0898b30ad5"
BASELINE = "6c7ac16617835b5f581784ff08a9e779bb52faf3"
hashes = {}
checks = collections.Counter()
inputs = {}


def require(condition, message):
    if not condition:
        raise AssertionError(message)
    checks["assertions"] += 1


def digest(path):
    path = Path(path).resolve()
    if path not in hashes:
        hashes[path] = hashlib.sha256(path.read_bytes()).hexdigest()
    return hashes[path]


def bound(path, expected):
    require(digest(path) == expected, f"SHA mismatch: {path}")
    checks["sha256Bindings"] += 1


def load(path):
    path = Path(path).resolve()
    inputs[str(path)] = digest(path)
    return json.loads(path.read_text())


def bindings(value):
    if isinstance(value, list):
        for child in value:
            bindings(child)
    elif isinstance(value, dict):
        if "path" in value and "sha256" in value:
            bound(value["path"], value["sha256"])
        for key, child in value.items():
            if key == "bindings" and isinstance(child, dict):
                for path, sha in child.items():
                    bound(path, sha)
            else:
                bindings(child)


def image(path):
    with Image.open(path) as source:
        return source.convert("RGBA")


def extended(path):
    source = image(path)
    require(source.width <= 768 and source.height <= 200, "transfer viewport exceeds canvas")
    canvas = Image.new("RGBA", (768, 200), "white")
    canvas.paste(source, (0, 0))
    return hashlib.sha256(canvas.tobytes()).digest()


def source_ids(path):
    return sorted((row["id"], row["path"]) for row in load(path))


def counts(rows):
    return {"total": len(rows), "geometryPass": sum(not r["geometryFailures"] for r in rows),
            "pixelPass": sum(not r["pixelFailures"] for r in rows),
            "clearPass": sum(c["samePixels"] for r in rows for c in r["clearChecks"])}


def verify_native(receipt, directory):
    for role in ("compiler", "probe", "renderer"):
        bound(receipt["tools"][role], receipt["toolHashes"][role])
    for source in receipt["sourceBindings"]:
        bound(directory / source["snapshot"], source["sha256"])
    for row in receipt["rows"]:
        scene = Path(row["prefix"]).parent
        for filename, key in [("request.json", "requestSha256"), ("scene.riv", "rivSha256"),
                              ("scene.map.json", "sourceMapSha256")]:
            bound(scene / filename, row[key])
        require(digest(scene / "scene.riv") == digest(scene / "probe/scene.riv"), "probe input mismatch")
        for suffix, key in [("geometry.json", "geometrySha256"), ("stream", "streamSha256")]:
            bound(scene / "probe" / f"frame-{row['frame']}.{suffix}", row[key])
        for role in ("chrome", "native"):
            bound(f"{row['prefix']}.{role}.png", row[f"{role}Sha256"])
        native = image(f"{row['prefix']}.native.png").tobytes()
        for clear in row["clearChecks"]:
            bound(clear["path"], clear["sha256"])
            require(clear["samePixels"] and image(clear["path"]).tobytes() == native, "clear RGBA mismatch")
    for row in receipt["repeated"]:
        require(row["nativeIdentical"] and row["chromeIdentical"], "original/clone restore differs")
        for role in ("chrome", "native"):
            scene = directory / row["name"]
            require(image(scene / f"frame-{row['frame']}.{role}.png").tobytes() ==
                    image(scene / f"frame-{row['firstFrame']}.{role}.png").tobytes(), "repeat pixels differ")


def main():
    freeze = load(BUILD / "frozen/source-bindings.json")
    for item in freeze["files"]:
        bound(item["path"], item["sha256"])
        if "snapshot" in item:
            bound(item["snapshot"], item["sha256"])
    require(len(freeze["files"]) == 125, "unexpected frozen source set")
    bound(BUILD / "frozen/html-to-riv", COMPILER)
    build = load(BUILD / "summary.json")
    require(build["sourceUnchanged"] and build["rustTests"] == 270 and build["nodeTests"] == 46,
            "build receipt mismatch")
    for check in load(BUILD / "checks.json"):
        require(check["exitCode"] == 0, "recorded build check failed")
        bound(check["log"], check["logSha256"])
    review = MODULE / "validation/public-content-owner-source-review.md"
    inputs[str(review)] = digest(review)
    table = re.findall(r"\| `([a-z_]+\.rs)` \| `([0-9a-f]{64})` \|", review.read_text())
    require(len(table) == 7, "source review binding table incomplete")
    for filename, sha in table:
        bound(MODULE / "src" / filename, sha)

    documents = [REG / "receipt.json", REG / "semantic-review.json", REG / "classification.json",
                 REG / "changed-native-bindings.json", NATIVE / "receipt.json",
                 NATIVE / "core-transfer-receipt.json", PRIOR / "receipt.json",
                 NATIVE / "parent-visual-review.json"]
    regression, semantic, classification, changed, native, core, prior, parent_visual = map(load, documents)
    for document in (regression, semantic, classification, changed, native, core, prior, parent_visual):
        bindings(document)
    require(parent_visual["complete"] and len(parent_visual["images"]) == 8, "parent visual review incomplete")
    checkpoint_review = MODULE / "validation/public-content-owner-checkpoint-review.md"
    inputs[str(checkpoint_review)] = digest(checkpoint_review)
    require(native["compilerSha256"] == COMPILER and native["immutableBaseline"] == BASELINE,
            "native compiler/runtime mismatch")
    require(native["visualReviewCompleted"] and prior["visualReviewCompleted"], "visual reviews incomplete")
    require(semantic["reviewed"] == 75 and semantic["assetOrSchemaVocabularyAdditions"] == 0,
            "semantic review scope mismatch")
    require(len(classification["results"]) == 794, "regression total mismatch")
    exact = [r for r in classification["results"] if r["oldRiv"]["sha256"] == r["newRiv"]["sha256"]
             and r["oldMap"]["sha256"] == r["newMap"]["sha256"]]
    require(len(exact) == 719 and all(r["exitCode"] == 0 for r in classification["results"]), "regression mismatch")
    changed_by_id = {r["referenceIndex"]: r for r in changed["cases"]}
    coverage = {r["referenceIndex"]: r for r in native["coverage"]}
    require(len(coverage) == len(native["coverage"]) == len(changed_by_id) == 75 and
            set(coverage) == set(classification["changedReferenceIndices"]) == set(changed_by_id),
            "changed-reference coverage mismatch")
    for index, actual in coverage.items():
        reference = changed_by_id[index]
        request = load(reference["request"]["path"])
        require(request == load(actual["request"]["path"]), f"request changed: {index}")
        require(actual["compileViewport"] == [request["width"], request["height"]], "viewport changed")
        for actual_key, reference_key in [("actualRiv", "newRiv"), ("actualMap", "newMap")]:
            require(digest(actual[actual_key]["path"]) == digest(reference[reference_key]["path"]),
                    f"rendered output not exact changed reference: {index}")
        require(source_ids(reference["oldMap"]["path"]) == source_ids(actual["actualMap"]["path"]),
                "authored source IDs/paths changed")
    evidence_counts = collections.Counter(r["evidence"] for r in coverage.values())
    require(evidence_counts == {"exact-core-transfer": 34, "fresh-native-chrome": 41}, "coverage split changed")

    for case in core["cases"]:
        public = NATIVE / "core" / case["name"]
        tested = Path(case["sourceNativeDirectory"])
        require(load(public / "request.json") == load(tested / "request.json") == case["request"], "core request mismatch")
        for filename, key in [("scene.riv", "rivSha256"), ("scene.map.json", "sourceMapSha256")]:
            bound(public / filename, case[key])
            bound(tested / filename, case[key])
    fresh = load(NATIVE / "render/receipt.json")
    candidate = load(CANDIDATE / "receipt.json")
    for key in ("browser", "backend", "effectiveMode", "cliModeToken", "resetSha256", "pixelGateSha256"):
        require(fresh[key] == candidate[key], f"native comparison setup changed: {key}")
    for role in ("probe", "renderer"):
        require(fresh["toolHashes"][role] == candidate["toolHashes"][role] == native["toolHashes"][role],
                f"immutable native tool changed: {role}")
    verify_native(fresh, NATIVE / "render")
    verify_native(candidate, CANDIDATE)
    combined = candidate["rows"] + fresh["rows"]
    public_rows = [r for r in combined if r["name"] != "resource-depth-128"]
    require(counts(public_rows) == native["changedReferenceFrames"] ==
            {"total": 600, "geometryPass": 600, "pixelPass": 594, "clearPass": 1200}, "frame counts mismatch")
    require(counts(combined) == native["withDepthResourceFrames"] ==
            {"total": 608, "geometryPass": 608, "pixelPass": 602, "clearPass": 1216}, "depth frame counts mismatch")
    failures = [{"name": r["name"], "frame": r["frame"], "geometry": r["geometryFailures"],
                 "pixels": r["pixelFailures"]} for r in combined if r["geometryFailures"] or r["pixelFailures"]]
    require(failures == native["retainedFailures"] == prior["retainedFailures"] and len(failures) == 6,
            "retained failures changed or missing")

    visual = load(NATIVE / "visual/coverage.json")
    bound(NATIVE / "render/receipt.json", visual["nativeReceiptSha256"])
    rows = {(r["name"], r["frame"]): r for r in fresh["rows"]}
    require(len(visual["rows"]) == 336 and visual["representatives"] == 94 and visual["transfers"] == 242,
            "fresh visual counts changed")
    require(sum(r["direct"] for r in visual["rows"]) == 94, "fresh direct/transfer split changed")
    for mapping in visual["rows"]:
        row = rows[(mapping["name"], mapping["frame"])]
        rep = visual["representativePairs"][mapping["representative"]]
        require(row["name"] == rep["name"], "cross-source visual transfer")
        for role in ("chrome", "native"):
            require(extended(f"{row['prefix']}.{role}.png") == extended(rep[role]), "incomplete RGBA transfer")
    placement_count = 0
    for sheet in visual["sheets"]:
        bound(sheet["path"], sheet["sha256"])
        pixels = image(sheet["path"])
        for place in sheet["placements"]:
            rep = visual["representativePairs"][place["representative"]]
            for role in ("chrome", "native"):
                x, y = place[f"{role}XY"]
                w, h = place["size"]
                require(pixels.crop((x, y, x+w, y+h)).tobytes() == image(rep[role]).tobytes(), "scaled/cropped sheet pair")
                placement_count += 1
    require(len(visual["sheets"]) == 16 and placement_count == 188, "fresh contact sheets incomplete")
    historical = load(PRIOR / "coverage.json")
    triples = {(r["name"], r["frame"]): r for r in historical["triples"]}
    for mapping in prior["frameMapping"]:
        if mapping["method"] == "direct":
            continue
        row = triples[(mapping["name"], mapping["frame"])]
        rep = triples[(mapping["name"], mapping["referenceFrame"])]
        for key in ("requestSha256", "publicRivSha256", "publicMapSha256", "candidateRivSha256", "candidateMapSha256"):
            require(row[key] == rep[key], "prior visual transfer source mismatch")
        for role in ("chrome", "public", "candidate"):
            require(extended(row["paths"][role]) == extended(rep["paths"][role]), "prior RGBA transfer incomplete")
    require(prior["representativeCount"] == 50 and prior["transferredFrameCount"] == 222 and
            len(prior["inspectedSheets"]) == 15, "prior visual review counts changed")
    for place in historical["placements"]:
        sheet = historical["sheets"][place["sheet"]]
        bound(sheet["path"], sheet["sha256"])
        x, y, w, h = (place[k] for k in ("x", "y", "width", "height"))
        require(image(sheet["path"]).crop((x, y, x+w, y+h)).tobytes() == image(place["source"]["path"]).tobytes(),
                "prior contact sheet incomplete")

    for name, authored_count, layout_count in [("resource-depth-128", 128, 257), ("resource-flat-8192", 8192, 16385)]:
        resource = load(NATIVE / name / "receipt.json")
        bindings(resource)
        directory = NATIVE / ("render/resource-depth-128" if authored_count == 128 else name)
        mapping = load(directory / "scene.map.json")
        require(len(mapping) == len({r["id"] for r in mapping}) == authored_count, "resource authored count mismatch")
        for row in resource["rows"]:
            geometry_path = directory / "probe" / f"frame-{row['frame']}.geometry.json"
            bound(geometry_path, row["geometrySha256"])
            geometry = load(geometry_path)
            require(len(geometry) == layout_count and all(
                math.isfinite(v) for item in geometry for v in [item["width"], item["height"], *item["worldMatrix"]]),
                "resource nonfinite or missing layouts")
            require({r["object_id"] for r in mapping} <= {g["objectId"] for g in geometry}, "resource source IDs absent")
            if "streamSha256" in row:
                bound(directory / "probe" / f"frame-{row['frame']}.stream", row["streamSha256"])
        require(len(resource["rows"]) == 8, "resource clone/resize count mismatch")
    require(native["separateFlatResourceFrames"]["chromeOrPixelQualification"] is False, "flat resource overclaim")

    OUT.mkdir(parents=True, exist_ok=True)
    guard = MODULE / "validation/check-target-runtime.py"
    result = subprocess.run([sys.executable, str(guard)], text=True, capture_output=True, check=True)
    guard_receipt = json.loads(result.stdout)
    require(guard_receipt["status"] == "pass" and guard_receipt["baseline"] == BASELINE, "immutable runtime mismatch")
    (OUT / "immutable-target.json").write_text(result.stdout)
    inputs[str(guard)] = digest(guard)
    inputs[str(OUT / "immutable-target.json")] = digest(OUT / "immutable-target.json")
    inputs[str(Path(__file__).resolve())] = digest(__file__)
    receipt = {"status": "verified-existing-evidence-with-six-retained-pixel-failures", "mismatches": 0,
               "scope": "Existing artifact bindings, exact changed-file coverage, complete RGBA transfers, contact-sheet placements and immutable source guard. No builds, tests, compiler runs or native renders repeated.",
               "compilerSha256": COMPILER, "immutableBaseline": BASELINE,
               "frozenInputs": 125, "buildChecks": 6, "recordedRustTests": 270, "recordedNodeTests": 46,
               "regression": {"total": 794, "exact": 719, "changed": 75, "nativeCovered": 75,
                              "exactCoreTransfers": 34, "freshChangedScenes": 41},
               "changedReferenceFrames": counts(public_rows), "withDepthResourceFrames": counts(combined),
               "retainedFailures": failures, "visualReviewCompleted": True,
               "visual": {"freshDirectPairs": 94, "freshCompleteTransfers": 242, "freshSheets": 16,
                          "coreDirectTriples": 50, "coreCompleteTransfers": 222, "coreSheets": 15,
                          "parentSelectedImages": 8, "newDirectInspectionClaimed": False},
               "resources": {"depth": {"authored": 128, "layouts": 257, "frames": 8, "paintedComplexity": False},
                             "flat": {"authored": 8192, "layouts": 16385, "frames": 8, "chromeOrPixelQualification": False}},
               "checks": dict(checks), "uniqueFilesHashed": len(hashes),
               "limitations": ["Six retained fractional paint mismatch-ratio failures remain.",
                   "Passing geometry/pixel gates do not imply exact equality; alpha and fractional edge differences remain.",
                   "Visual coverage is finite, with complete same-source RGBA transfers; offscreen large-coordinate content is not visually certified.",
                   "128-depth resource scenes are unpainted; 8192-flat resource scope is native import/clone/resize and finite geometry only.",
                   "Content-owner diagnostic descriptors remain a proof barrier, not a complete composition certificate."],
               "bindings": inputs}
    (OUT / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps({k: v for k, v in receipt.items() if k not in ("bindings", "retainedFailures", "limitations")}, indent=2))


if __name__ == "__main__":
    main()
