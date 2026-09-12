#!/usr/bin/env python3
"""Freeze public-only output references before content-owner integration.

Preparation copies existing source-bound outputs; it performs no compile,
render, candidate-file promotion, or new support qualification.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import runpy
from collections import Counter

MODULE = Path(__file__).resolve().parents[1]
OLD_COMPILER_SHA = "034084cd868e9b33ba849bc328ea4c540d63115192eba7918e7d7ec00c3c2b2d"


def read(path):
    return json.loads(Path(path).read_text())


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def bind(path):
    return {"path": str(Path(path).resolve()), "sha256": sha(path)}


def canonical(request):
    return json.dumps(request, sort_keys=True, ensure_ascii=False, separators=(",", ":"))


def prepare(output):
    output = Path(output).resolve()
    assert output.name.startswith("public-content-owner-regression-")
    output.mkdir(parents=True, exist_ok=False)
    old_compiler = MODULE / "output/public-value-build-r1/frozen/html-to-riv"
    old_freeze = old_compiler.parent / "source-bindings.json"
    prior_file = MODULE / "output/public-value-regression-r1/manifest.json"
    prior_refs_file = MODULE / "output/public-value-regression-references-r1/manifest.json"
    prior_receipt_file = MODULE / "validation/public-value-regression-receipt.json"
    prior, prior_refs, prior_receipt = map(read, [prior_file, prior_refs_file, prior_receipt_file])
    assert sha(old_compiler) == OLD_COMPILER_SHA == prior["compilerSha256"]
    assert sha(prior_refs_file) == prior["priorManifestSha256"]
    assert prior["total"] == prior["passed"] == len(prior["results"]) == 731
    assert prior_receipt["total"] == prior_receipt["exactRivFiles"] == prior_receipt["exactSourceMaps"] == 731
    assert len(prior_refs["results"]) == 731
    frozen_dir = output / "frozen"
    frozen_dir.mkdir()
    shutil.copy2(old_compiler, frozen_dir / "html-to-riv")
    shutil.copy2(Path(__file__), frozen_dir / "preparation.py")
    freeze = read(old_freeze)
    frozen_sources = []
    for index, item in enumerate(freeze["files"]):
        source = Path(item.get("snapshot", item["path"]))
        assert sha(source) == item["sha256"]
        target = frozen_dir / "inputs" / str(index) / source.name
        target.parent.mkdir(parents=True)
        shutil.copy2(source, target)
        assert sha(target) == item["sha256"]
        frozen_sources.append({"original": item, "snapshot": bind(target)})
    assert len(frozen_sources) == 118
    (frozen_dir / "source-bindings.json").write_text(json.dumps({"originalFreeze": bind(old_freeze), "files": frozen_sources}, indent=2) + "\n")
    rows, aliases, sources, known, groups = [], [], [], {}, []

    def add(request, riv, mapping, expected, origin, allow_duplicate):
        files = list(map(lambda p: Path(p).resolve(), [request, riv, mapping]))
        assert all("candidate" not in str(p) for p in files), "Private output cannot become an old public reference"
        actual = [sha(p) for p in files]
        assert actual == expected, (origin, actual, expected)
        sources.extend(bind(p) for p in files)
        key = canonical(read(files[0]))
        duplicate = key in known
        if duplicate:
            assert allow_duplicate, "Every historical row remains independently represented"
            prior_row = rows[known[key]]
            assert actual[1:] == [prior_row["rivSha256"], prior_row["mapSha256"]], "Identical requests have conflicting public output"
            destination = output / "aliases" / str(len(aliases))
        else:
            destination = output / "references" / str(len(rows))
        destination.mkdir(parents=True)
        copies = [destination / n for n in ["request.json", "scene.riv", "scene.map.json"]]
        for source, target, expected_hash in zip(files, copies, actual):
            shutil.copy2(source, target)
            assert sha(target) == expected_hash
        row = {"referenceIndex": known[key] if duplicate else len(rows),
               "request": str(copies[0]), "requestSha256": actual[0],
               "priorRiv": str(copies[1]), "rivSha256": actual[1],
               "priorMap": str(copies[2]), "mapSha256": actual[2],
               "canonicalRequestSha256": hashlib.sha256(key.encode()).hexdigest(),
               "origin": origin, "sourceArtifacts": [bind(p) for p in files]}
        if duplicate:
            aliases.append({**row, "sameCompleteRequestObject": True, "sameRivAndMap": True})
        else:
            known[key] = len(rows)
            rows.append(row)

    for index, (result, reference) in enumerate(zip(prior["results"], prior_refs["results"])):
        assert result["referenceIndex"] == reference["referenceIndex"] == index
        assert result["exact"] and result["exitCode"] == 0
        for path_key, hash_key in [("request", "requestSha256"), ("priorRiv", "rivSha256"), ("priorMap", "mapSha256")]:
            assert sha(reference[path_key]) == reference[hash_key] == result[hash_key]
        base = Path(result["result"])
        add(base / "request.json", base / "scene.riv", base / "scene.map.json",
            [result["requestSha256"], result["actualRivSha256"], result["actualMapSha256"]],
            {"suite": "public-value-regression-r1", "historicalIndex": index,
             "resultManifest": bind(prior_file), "originalReferenceOrigin": reference["origin"]}, False)
    assert len(rows) == 731

    for name, directory, expected in [
        ("value-native", MODULE / "output/public-value-native-r1/render", 35),
        ("content-owner-public-core", MODULE / "output/content-owner-public-r1/render", 28),
        ("content-owner-public-stretch", MODULE / "output/content-owner-public-r1/stretch/render", 6),
    ]:
        receipt_file = directory / "receipt.json"
        receipt = read(receipt_file)
        assert receipt["toolHashes"]["compiler"] == OLD_COMPILER_SHA
        assert len(receipt["artifacts"]) == expected
        start, alias_start = len(rows), len(aliases)
        for artifact in receipt["artifacts"]:
            base = directory / artifact["name"]
            add(base / "request.json", base / "scene.riv", base / "scene.map.json",
                [artifact["requestSha256"], artifact["rivSha256"], artifact["mapSha256"]],
                {"suite": name, "name": artifact["name"], "renderReceipt": bind(receipt_file),
                 "priorNativeStatus": receipt["status"],
                 "priorFailingFrames": [{"frame": r["frame"], "geometry": r["geometryFailures"], "pixels": r["pixelFailures"]}
                                        for r in receipt["rows"] if r["name"] == artifact["name"] and (r["geometryFailures"] or r["pixelFailures"])]}, True)
        groups.append({"name": name, "receipt": bind(receipt_file), "scenesConsidered": expected,
                       "referencesAdded": len(rows) - start, "aliases": len(aliases) - alias_start,
                       "priorNativeStatus": receipt["status"]})

    for item in sources:
        assert sha(item["path"]) == item["sha256"]
    for row in rows + aliases:
        for path_key, hash_key in [("request", "requestSha256"), ("priorRiv", "rivSha256"), ("priorMap", "mapSha256")]:
            assert sha(row[path_key]) == row[hash_key]
    bindings = [bind(p) for p in [prior_file, prior_refs_file, prior_receipt_file, old_freeze, old_compiler]]
    manifest = {"scope": "Frozen previous PUBLIC request/bytes/maps only. Known native failures remain labelled; no candidate output or new visual qualification is included.",
                "referenceCompiler": bind(frozen_dir / "html-to-riv"),
                "referenceSourceFreeze": bind(frozen_dir / "source-bindings.json"),
                "preparationScript": bind(frozen_dir / "preparation.py"),
                "command": [sys.executable, str(Path(__file__).resolve()), "prepare", str(output)],
                "historicalRowsRetained": 731, "additionalScenesConsidered": 69,
                "additionalScenesAdded": len(rows) - 731, "nativeGroups": groups,
                "deduplicationRule": "Preserve all 731 historical rows. Added scenes alias only an exactly equal complete parsed request with identical Rive bytes and map. JSON formatting/key order are ignored; source spelling and all request fields are preserved. Alias artifacts and origins are copied independently. Never deduplicate by output hash alone.",
                "total": len(rows), "aliases": aliases, "results": rows, "bindings": bindings,
                "nextStep": "Await new frozen public CLI, then compare every reference and classify every changed file/map/diagnostic. Separate native evidence must qualify newly emitted files."}
    (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(json.dumps({"manifest": bind(output / "manifest.json"), "total": len(rows),
                      "historicalRowsRetained": 731, "additionalScenesConsidered": 69,
                      "additionalScenesAdded": len(rows) - 731, "aliases": len(aliases), "nativeGroups": groups}))


def compare(compiler, references_file, output, source_freeze):
    """Compile all references and inventory changes; semantic review stays explicit."""
    compiler, references_file, output, source_freeze = map(lambda p: Path(p).resolve(),
                                                          [compiler, references_file, output, source_freeze])
    assert "candidate" not in str(compiler), "New compiler must be a frozen public build"
    assert output.name.startswith("public-content-owner-regression-") and not output.exists()
    references = read(references_file)
    assert references["total"] == len(references["results"]) == 794
    assert references["referenceCompiler"]["sha256"] == OLD_COMPILER_SHA
    assert sha(references["referenceCompiler"]["path"]) == OLD_COMPILER_SHA
    freeze = read(source_freeze)
    frozen_sources = []
    for item in freeze["files"]:
        for key in ["path", "snapshot"]:
            if key in item:
                assert sha(item[key]) == item["sha256"], item[key]
        frozen_sources.append(item)
    compiler_binding, freeze_binding = bind(compiler), bind(source_freeze)
    checker = MODULE / "validation/check-output-regression.py"
    command = [sys.executable, str(checker), str(compiler), str(references_file), str(output)]
    execution = subprocess.run(command, capture_output=True, text=True)
    assert output.exists(), execution.stderr
    (output / "driver.log").write_text(execution.stdout + execution.stderr)
    (output / "comparison-command.json").write_text(json.dumps({"command": command, "exitCode": execution.returncode,
        "log": bind(output / "driver.log"), "compiler": compiler_binding, "sourceFreeze": freeze_binding}, indent=2) + "\n")
    result_file = output / "manifest.json"
    result = read(result_file)
    assert result["compilerSha256"] == compiler_binding["sha256"]
    assert result["priorManifestSha256"] == sha(references_file)
    assert result["total"] == len(result["results"]) == 794
    assert execution.returncode in [0, 1]
    frozen = output / "analysis-inputs"
    frozen.mkdir()
    for source, name in [(Path(__file__), "regression.py"), (checker, "check-output-regression.py"),
                         (source_freeze, "source-bindings.json"),
                         (MODULE / "validation/content-box-rounding-wire.py", "wire.py")]:
        shutil.copy2(source, frozen / name)
        assert sha(source) == sha(frozen / name)
    records = runpy.run_path(str(frozen / "wire.py"))["records"]
    classifications, changed, exact, source_map_errors = [], [], [], []
    for before, after in zip(references["results"], result["results"]):
        index = before["referenceIndex"]
        assert after["referenceIndex"] == index
        base = Path(after["result"])
        assert sha(base / "request.json") == before["requestSha256"]
        for path_key, hash_key in [("request", "requestSha256"), ("priorRiv", "rivSha256"), ("priorMap", "mapSha256")]:
            assert sha(before[path_key]) == before[hash_key] == after[hash_key]
        riv, mapping = base / "scene.riv", base / "scene.map.json"
        for path, key in [(riv, "actualRivSha256"), (mapping, "actualMapSha256")]:
            assert (sha(path) if path.exists() else None) == after[key]
        row = {"referenceIndex": index, "origin": before["origin"], "request": bind(base / "request.json"),
               "exitCode": after["exitCode"], "compileLog": bind(base / "compile.log"),
               "oldRiv": bind(before["priorRiv"]), "oldMap": bind(before["priorMap"]),
               "newRiv": bind(riv) if riv.exists() else None, "newMap": bind(mapping) if mapping.exists() else None}
        if after["exact"]:
            row["classification"] = "exact-public-bytes-and-map"
            exact.append(index)
        elif after["exitCode"] != 0:
            row["classification"] = "new-diagnostic" if not riv.exists() and not mapping.exists() else "failed-compile-with-partial-output"
            text = (base / "compile.log").read_text()
            try:
                row["diagnosticJson"] = json.loads(text)
            except json.JSONDecodeError:
                row["diagnosticText"] = text
            changed.append(index)
        else:
            assert riv.exists() and mapping.exists()
            riv_changed = after["actualRivSha256"] != before["rivSha256"]
            map_changed = after["actualMapSha256"] != before["mapSha256"]
            row["classification"] = "rive-and-map-changed" if riv_changed and map_changed else "rive-only-changed" if riv_changed else "map-only-changed"
            old_map, new_map = read(before["priorMap"]), read(mapping)
            identity = lambda values: sorted((r["id"], r["path"]) for r in values)
            row["sameAuthoredSourceIdsAndPaths"] = identity(old_map) == identity(new_map)
            if not row["sameAuthoredSourceIdsAndPaths"]:
                source_map_errors.append(index)
            old_records, new_records = records(Path(before["priorRiv"]).read_bytes()), records(riv.read_bytes())
            normalize = lambda record: {"kind": record["kind"], "properties": {key: {"field": value["field"], "value": value["value"]} for key, value in record["properties"].items()}}
            wire_file = base / "wire-comparison.json"
            wire = {"scope": "Read-only decoded ordinary records. Differences require semantic review; no automatic correctness classification.",
                    "before": [normalize(r) for r in old_records], "after": [normalize(r) for r in new_records],
                    "beforeSourceMap": old_map, "afterSourceMap": new_map}
            wire_file.write_text(json.dumps(wire, indent=2) + "\n")
            row.update({"oldRecordCount": len(old_records), "newRecordCount": len(new_records),
                        "oldRecordKinds": dict(Counter(r["kind"] for r in old_records)),
                        "newRecordKinds": dict(Counter(r["kind"] for r in new_records)),
                        "wireComparison": bind(wire_file)})
            changed.append(index)
        classifications.append(row)
    assert len(exact) == result["passed"]
    assert sha(compiler) == compiler_binding["sha256"] and sha(source_freeze) == freeze_binding["sha256"]
    summary = {"status": "all-exact" if not changed else "changes-await-semantic-review",
               "scope": "Complete public output comparison and change inventory. Changed files require explicit review and independent new native evidence.",
               "total": 794, "exact": len(exact), "changed": len(changed),
               "classifications": dict(Counter(r["classification"] for r in classifications)),
               "sourceMapIdentityErrors": source_map_errors, "changedReferenceIndices": changed,
               "results": classifications, "frozenSourceBindingsVerified": len(frozen_sources),
               "bindings": [bind(p) for p in [result_file, references_file, output / "comparison-command.json", output / "html-to-riv",
                                                frozen / "regression.py", frozen / "check-output-regression.py", frozen / "wire.py", frozen / "source-bindings.json"]]}
    (output / "classification.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps({key: summary[key] for key in ["status", "total", "exact", "changed", "classifications", "sourceMapIdentityErrors"]}))


def review_changes(output):
    """Audit every decoded delta without treating structure as native equivalence."""
    output = Path(output).resolve()
    classification = read(output / "classification.json")
    assert (classification["total"], classification["exact"], classification["changed"]) == (794, 719, 75)
    reviewed, fixtures, fixture_bindings = [], [], []
    layout_changes, style_changes = Counter(), Counter()
    for result in classification["results"]:
        if result["classification"] == "exact-public-bytes-and-map":
            continue
        assert result["classification"] in ["rive-and-map-changed", "rive-only-changed"]
        assert result["exitCode"] == 0 and result["sameAuthoredSourceIdsAndPaths"]
        wire = read(result["wireComparison"]["path"])
        assert sha(result["wireComparison"]["path"]) == result["wireComparison"]["sha256"]
        before, after = wire["before"], wire["after"]
        old_map = {m["id"]: m for m in wire["beforeSourceMap"]}
        new_map = {m["id"]: m for m in wire["afterSourceMap"]}
        old_kinds, new_kinds = [Counter(r["kind"] for r in records) for records in [before, after]]
        delta = {key: new_kinds[key] - old_kinds[key] for key in old_kinds.keys() | new_kinds.keys() if new_kinds[key] != old_kinds[key]}
        assert set(delta) == {409, 420} and delta[409] == delta[420] in [1, 2]
        # The header host, artboard and its ordinary white paint remain exact.
        assert before[:5] == after[:5]
        fields = lambda records: {(key, value["field"]) for record in records for key, value in record["properties"].items()}
        assert fields(before) == fields(after), "No file-level property/type vocabulary additions permitted"
        assert set(old_kinds) == set(new_kinds), "No new schema or asset object types"

        def parent_source(records, mapping, object_id):
            by_id = {r["object_id"]: key for key, r in mapping.items()}
            seen = set()
            current = records[object_id + 1]["properties"]["5"]["value"]
            while current not in by_id and current != 0:
                assert current not in seen
                seen.add(current)
                current = records[current + 1]["properties"]["5"]["value"]
            return by_id.get(current, "#artboard")

        def paints(records, mapping):
            by_id = {r["object_id"]: key for key, r in mapping.items()}
            result = []
            for record in records:
                if record["kind"] != 18:
                    continue
                fill = records[record["properties"]["5"]["value"] + 1]
                assert fill["kind"] == 20
                owner = fill["properties"]["5"]["value"]
                assert owner in by_id or owner == 0, "Ordinary helpers must stay unpainted"
                result.append((by_id.get(owner, "#artboard"), record["properties"]["37"]))
            return result

        assert paints(before, old_map) == paints(after, new_map), "Paint ownership, color and order must remain exact"
        node_deltas = []
        for name, old_node in old_map.items():
            new_node = new_map[name]
            assert old_node["path"] == new_node["path"]
            old_layout = before[old_node["object_id"] + 1]
            new_layout = after[new_node["object_id"] + 1]
            assert old_layout["kind"] == new_layout["kind"] == 409
            op, np = old_layout["properties"], new_layout["properties"]
            layout_keys = [k for k in op.keys() | np.keys() if op.get(k) != np.get(k)]
            assert set(layout_keys) <= {"5", "494"}, "Only ordinary parent/style links may change on authored layout records"
            assert bytes.fromhex(np["4"]["value"]).decode() == name
            old_style = before[op["494"]["value"] + 1]
            new_style = after[np["494"]["value"] + 1]
            assert old_style["kind"] == new_style["kind"] == 420
            os, ns = old_style["properties"], new_style["properties"]
            style_keys = [k for k in os.keys() | ns.keys() if os.get(k) != ns.get(k)]
            assert set(style_keys) <= {"598", "632"}, "Only outer packing direction/alignment may change"
            assert parent_source(before, old_map, old_node["object_id"]) == parent_source(after, new_map, new_node["object_id"])
            layout_changes.update(layout_keys)
            style_changes.update(style_keys)
            if layout_keys or style_keys:
                node_deltas.append({"sourceId": name, "path": old_node["path"], "layoutFieldsChanged": sorted(layout_keys),
                                    "styleFieldsChanged": sorted(style_keys), "sameNearestAuthoredParent": True})
        base = Path(result["request"]["path"]).parent
        assert not (base / "scene.requirements.json").exists()
        request = read(base / "request.json")
        assert set(request) == {"html", "css", "width", "height"}
        name = f"content-owner-regression-{result['referenceIndex']}"
        fixtures.append({"name": name, "html": request["html"], "css": request["css"],
                         "compileViewport": [request["width"], request["height"]], "referenceIndex": result["referenceIndex"]})
        fixture_bindings.append({"name": name, **{k: result[k] for k in ["referenceIndex", "request", "oldRiv", "oldMap", "newRiv", "newMap", "classification", "origin"]}})
        reviewed.append({"referenceIndex": result["referenceIndex"], "disposition": "ordinary-content-owner-composition; native-qualification-separate",
                         "addedLayoutComponents": delta[409], "addedLayoutStyles": delta[420],
                         "allOtherObjectCountsUnchanged": True, "schemaAndFieldVocabularyUnchanged": True,
                         "authoredSourceIdentityAndAncestorsPreserved": True, "paintOwnershipColorsAndOrderExact": True,
                         "authoredSizesBoundsPaddingMarginsUnchanged": True, "helpersUnpainted": True,
                         "noRequirementsSidecar": True, "authoredNodeDeltas": node_deltas,
                         "wireComparison": result["wireComparison"], "newRiv": result["newRiv"], "newMap": result["newMap"]})
    assert len(reviewed) == 75
    cases_file, bindings_file = output / "changed-native-cases.json", output / "changed-native-bindings.json"
    cases_text = json.dumps(fixtures, indent=2) + "\n"
    if cases_file.exists():
        assert cases_file.read_text() == cases_text, "Do not change already handed-off native sources"
    else:
        cases_file.write_text(cases_text)
    fixture_receipt = {"scope": "Exact complete successful changed reference requests for separate native validation. Only compileViewport transports request width/height; HTML/CSS remain unchanged. No native qualification claimed.",
                       "compiler": read(output / "comparison-command.json")["compiler"],
                       "classification": bind(output / "classification.json"), "fixtures": bind(cases_file), "cases": fixture_bindings}
    bindings_text = json.dumps(fixture_receipt, indent=2) + "\n"
    if bindings_file.exists():
        assert read(bindings_file) == fixture_receipt, "Do not change already handed-off native bindings"
    else:
        bindings_file.write_text(bindings_text)
    snapshot = output / "analysis-inputs/semantic-review.py"
    shutil.copy2(Path(__file__), snapshot)
    # Bind the exact immutable schema vocabulary used to name audited key deltas.
    schema = MODULE.parents[1] / "crates/nuxie-schema/src/generated/schema.rs"
    result = {"status": "all-75-changes-classified-as-ordinary-content-owner-composition",
              "scope": "Record-level change disposition only. Native geometry, pixels and retained failures are independently qualified by the native receipt.",
              "reviewed": 75, "addedLayoutComponents": sum(r["addedLayoutComponents"] for r in reviewed),
              "addedLayoutStyles": sum(r["addedLayoutStyles"] for r in reviewed),
              "changedAuthoredLayoutFieldCounts": dict(layout_changes), "changedAuthoredStyleFieldCounts": dict(style_changes),
              "fieldNames": {"5": "parentId", "494": "styleId", "598": "flexDirectionValue", "632": "layoutAlignmentType"},
              "newDiagnostics": 0, "assetOrSchemaVocabularyAdditions": 0, "nativeQualification": "separate",
              "results": reviewed, "bindings": [bind(output / "classification.json"), bind(snapshot), bind(schema), bind(cases_file), bind(bindings_file)]}
    (output / "semantic-review.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({k: result[k] for k in ["status", "reviewed", "addedLayoutComponents", "addedLayoutStyles", "changedAuthoredLayoutFieldCounts", "changedAuthoredStyleFieldCounts"]}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="mode", required=True)
    freeze_parser = commands.add_parser("prepare")
    freeze_parser.add_argument("fresh_output")
    compare_parser = commands.add_parser("compare")
    compare_parser.add_argument("compiler")
    compare_parser.add_argument("references_manifest")
    compare_parser.add_argument("fresh_output")
    compare_parser.add_argument("--source-freeze", required=True)
    review_parser = commands.add_parser("review")
    review_parser.add_argument("output")
    args = parser.parse_args()
    if args.mode == "prepare":
        prepare(args.fresh_output)
    elif args.mode == "compare":
        compare(args.compiler, args.references_manifest, args.fresh_output, args.source_freeze)
    elif args.mode == "review":
        review_changes(args.output)
