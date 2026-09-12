#!/usr/bin/env python3
"""Prepare source-bound byte/map references for the CSS value recovery repair.

This does not compile or render anything. Use check-output-regression.py with
the emitted manifest once the new compiler has been frozen by the parent task.
"""
import argparse
import hashlib
import html
import json
from pathlib import Path
import re
import shutil
import sys


MODULE = Path(__file__).resolve().parent.parent
CSS_WHITESPACE = "\t\n\f\r "
CSS_ESCAPE = re.compile(r"\\(?:[0-9a-fA-F]{1,6}(?:\r\n|[\t\n\f\r ])?|[^\n\r\f0-9a-fA-F])")
HTML_ENTITY = re.compile(r"&(?:#[xX][0-9a-fA-F]+|#[0-9]+|[a-zA-Z][a-zA-Z0-9]+);?")


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def read(path):
    return json.loads(Path(path).read_text())


def binding(path):
    path = Path(path).resolve()
    return {"path": str(path), "sha256": sha(path)}


def canonical(request):
    # Only JSON representation is canonicalized. HTML/CSS string contents and
    # every request field remain exact, including viewport and source spelling.
    return json.dumps(request, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def suspicious(character):
    return (character.isspace() and character not in CSS_WHITESPACE) or character in "\u200b\ufeff"


def whitespace_findings(request):
    findings = []
    for field in ("html", "css"):
        text = request[field]
        for index, character in enumerate(text):
            if suspicious(character):
                findings.append({"field": field, "offset": index, "form": "literal",
                                 "codePoint": f"U+{ord(character):04X}"})
        # Scan escapes in both fields so inline style attributes are included.
        # This deliberately over-reports escapes in HTML text or quoted data;
        # any finding requires review, never automatic exclusion or recovery.
        escape_views = [("CSS escape", text)]
        if field == "html" and html.unescape(text) != text:
            escape_views.append(("CSS escape after HTML entity decoding", html.unescape(text)))
        for form, view in escape_views:
            for match in CSS_ESCAPE.finditer(view):
                value = match.group()[1:]
                digits = re.match(r"[0-9a-fA-F]{1,6}", value)
                if digits:
                    number = int(digits.group(), 16)
                    character = chr(number) if 0 < number <= 0x10ffff else "\ufffd"
                else:
                    character = value
                if len(character) == 1 and suspicious(character):
                    findings.append({"field": field, "viewOffset": match.start(), "form": form,
                                     "source": match.group(), "codePoint": f"U+{ord(character):04X}"})
        if field == "html":
            for match in HTML_ENTITY.finditer(text):
                for character in html.unescape(match.group()):
                    if suspicious(character):
                        findings.append({"field": field, "offset": match.start(), "form": "HTML entity",
                                         "source": match.group(), "codePoint": f"U+{ord(character):04X}"})
    return findings


def prepare(output):
    output = Path(output).resolve()
    prior_file = MODULE / "output/numeric-token-prior-regression-r1/manifest.json"
    prior = read(prior_file)
    assert (prior["total"], prior["passed"], len(prior["results"])) == (694, 691, 694)
    compiler = binding(prior["compiler"])
    assert compiler["sha256"] == prior["compilerSha256"]
    old_freeze = MODULE / "output/public-numeric-token-r3/frozen/source-bindings.json"
    inputs = [binding(__file__), binding(prior_file), binding(old_freeze), compiler]
    output.mkdir(parents=True, exist_ok=False)
    rows, aliases, promoted, flags = [], [], [], []
    known = {}

    def add(request, riv, mapping, expected, origin, allow_duplicate):
        files = [Path(request).resolve(), Path(riv).resolve(), Path(mapping).resolve()]
        hashes = [sha(file) for file in files]
        assert hashes == expected, (origin, hashes, expected)
        inputs.extend(binding(file) for file in files)
        data = read(files[0])
        key = canonical(data)
        if key in known:
            assert allow_duplicate, "Every historical row must remain independently represented"
            existing = rows[known[key]]
            assert hashes[1:] == [existing["rivSha256"], existing["mapSha256"]], "Conflicting output for identical source request"
            alias_path = output / "aliases" / f"{len(aliases)}.request.json"
            alias_path.parent.mkdir(exist_ok=True)
            shutil.copy2(files[0], alias_path)
            assert sha(alias_path) == hashes[0]
            aliases.append({"origin": origin, "request": binding(alias_path),
                            "originalRequest": binding(files[0]), "referenceIndex": existing["referenceIndex"],
                            "sameRequestObject": True, "rivSha256": hashes[1], "mapSha256": hashes[2]})
            return
        index = len(rows)
        dest = output / "references" / str(index)
        dest.mkdir(parents=True)
        copies = [dest / "request.json", dest / "scene.riv", dest / "scene.map.json"]
        for source, target, expected_hash in zip(files, copies, hashes):
            shutil.copy2(source, target)
            assert sha(target) == expected_hash
        row = {"referenceIndex": index, "request": str(copies[0]), "requestSha256": hashes[0],
               "priorRiv": str(copies[1]), "rivSha256": hashes[1],
               "priorMap": str(copies[2]), "mapSha256": hashes[2],
               "canonicalRequestSha256": hashlib.sha256(key.encode()).hexdigest(),
               "origin": origin, "sourceArtifacts": [binding(file) for file in files]}
        rows.append(row)
        known[key] = index
        findings = whitespace_findings(data)
        if findings:
            flags.append({"referenceIndex": index, "request": binding(copies[0]), "findings": findings,
                          "disposition": "Retained in regression; any changed result requires explicit review"})

    for index, row in enumerate(prior["results"]):
        assert row["exitCode"] == 0
        result = Path(row["result"])
        origin = {"suite": "numeric-token-prior-regression-r1", "manifest": binding(prior_file),
                  "historicalIndex": index, "priorComparisonExact": row["exact"]}
        add(result / "request.json", result / "scene.riv", result / "scene.map.json",
            [row["requestSha256"], row["actualRivSha256"], row["actualMapSha256"]], origin, False)
        if not row["exact"]:
            assert row["actualMapSha256"] == row["mapSha256"]
            promoted.append({"historicalIndex": index, "referenceIndex": len(rows) - 1,
                             "supersededRivSha256": row["rivSha256"],
                             "correctedRivSha256": row["actualRivSha256"],
                             "mapSha256": row["actualMapSha256"]})
    assert len(rows) == 694 and [row["historicalIndex"] for row in promoted] == [557, 558, 560]

    numeric_root = MODULE / "output/public-numeric-native-r1"
    native_groups = []
    for group in ("visible", "fractional", "large"):
        directory = numeric_root / group / "render"
        receipt_file = directory / "receipt.json"
        receipt = read(receipt_file)
        assert receipt["toolHashes"]["compiler"] == compiler["sha256"]
        inputs.append(binding(receipt_file))
        native_groups.append({"group": group, "receipt": binding(receipt_file),
                              "priorRenderStatus": receipt["status"], "sceneCount": len(receipt["artifacts"])})
        for artifact in receipt["artifacts"]:
            scene = directory / artifact["name"]
            origin = {"suite": "public-numeric-native-r1", "group": group, "name": artifact["name"],
                      "receipt": binding(receipt_file), "priorRenderStatus": receipt["status"]}
            add(scene / "request.json", scene / "scene.riv", scene / "scene.map.json",
                [artifact["requestSha256"], artifact["rivSha256"], artifact["mapSha256"]], origin, True)

    assert len(rows) == 731 and len(aliases) == 1
    assert aliases[0]["referenceIndex"] == 665
    # Recheck every original input and copied reference after preparation.
    for item in inputs:
        assert sha(item["path"]) == item["sha256"], item["path"]
    for row in rows:
        for path_key, hash_key in (("request", "requestSha256"), ("priorRiv", "rivSha256"), ("priorMap", "mapSha256")):
            assert sha(row[path_key]) == row[hash_key]
    manifest = {"scope": "Exact deterministic request/bytes/maps regression only; no new compile, rendering, or visual qualification",
                "referenceCompiler": compiler, "referenceSourceFreeze": binding(old_freeze),
                "preparationScript": binding(__file__), "command": [sys.executable, str(Path(__file__).resolve()), str(output)],
                "historicalManifest": binding(prior_file),
                "historicalRowsRetained": 694, "correctedNumericOutputsPromoted": promoted,
                "nativeGroups": native_groups, "nativeScenesConsidered": 38, "nativeScenesAdded": 37,
                "deduplicationRule": "Retain all 694 historical rows. Add a native scene unless its complete parsed request object is exactly equal to an existing request (JSON formatting/key order ignored; HTML/CSS string spelling and every request field preserved). A duplicate must have identical bytes/maps and is recorded as an alias. Never deduplicate solely by output hash.",
                "aliases": aliases,
                "omittedArtifacts": [{"path": str(numeric_root / "preflight"),
                                      "reason": "26 admission preflights used the earlier pre-numeric-token-repair compiler; latest native render receipts supply their current outputs instead."}],
                "whitespaceAudit": {"scope": "Conservative direct Unicode whitespace, CSS-escaped code point, and HTML entity scan over all stored HTML/CSS sources. Findings stay in the reference set; a finding is a review flag, not proof of invalid CSS.",
                                    "flaggedRequests": len(flags), "findings": flags},
                "total": len(rows), "results": rows}
    file = output / "manifest.json"
    file.write_text(json.dumps(manifest, indent=2) + "\n")
    print(json.dumps({"manifest": binding(file), "total": len(rows), "historical": 694,
                      "nativeAdded": 37, "aliases": len(aliases), "whitespaceFlags": len(flags)}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("fresh_output")
    prepare(parser.parse_args().fresh_output)
