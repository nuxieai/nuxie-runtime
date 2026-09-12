#!/usr/bin/env python3
"""Read-only image and sample comparisons for the retained fractional controls."""
import hashlib
import json
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "output/public-numeric-native-r1"


def read(path):
    return json.loads(path.read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def binding(path):
    return {"path": str(path), "sha256": sha(path)}


def pixels(row, role):
    path = Path(row["prefix"] + "." + role + ".png")
    assert sha(path) == row[role + "Sha256"]
    return Image.open(path).convert("RGBA")


def comparison(left, right):
    result = {}
    for role in ["chrome", "native"]:
        a, b = pixels(left, role), pixels(right, role)
        assert a.size == b.size
        aa, bb = a.tobytes(), b.tobytes()
        result[role] = {
            "completeRgbaIdentical": aa == bb,
            "differentPixels": sum(aa[i:i + 4] != bb[i:i + 4] for i in range(0, len(aa), 4)),
            "leftSha256": left[role + "Sha256"],
            "rightSha256": right[role + "Sha256"],
        }
    return result


visible_path = OUT / "visible/render/receipt.json"
control_path = OUT / "fractional/render/receipt.json"
historical_path = ROOT / "output/flex-fixed-controls-r2/edges-render/receipt.json"
visible, control, historical = [read(p) for p in [visible_path, control_path, historical_path]]
original = [r for r in visible["rows"] if r["name"] == "numeric-fractional-height"]
exact_half = [r for r in control["rows"] if r["name"] == "numeric-fractional-height-exact-half"]
minimal = [r for r in control["rows"] if r["name"] == "numeric-fractional-height-historical-minimal"]
old_minimal = [r for r in historical["rows"] if r["name"] == "height-25p5"]
assert len(original) == len(exact_half) == len(minimal) == len(old_minimal) == 8
source_cases = read(ROOT / "validation/public-numeric-native-fractional-cases.json")
old_case = next(f for f in read(ROOT / "validation/flex-fixed-controls-cases.json")["edges"] if f["name"] == "height-25p5")
assert all(source_cases[1][k] == old_case[k] for k in ["html", "css", "viewports"])
source_original = next(f for f in read(ROOT / "validation/public-numeric-native-cases.json") if f["name"] == "numeric-fractional-height")
assert source_cases[0]["html"] == source_original["html"]
assert source_cases[0]["css"] == source_original["css"].replace("25.499998092651367px", "25.5px")

same_scene, historical_comparisons = [], []
for a, b, c, d in zip(original, exact_half, minimal, old_minimal):
    assert a["frame"] == b["frame"] == c["frame"] == d["frame"]
    pair = comparison(a, b)
    assert pair["native"]["completeRgbaIdentical"]
    assert pair["chrome"]["differentPixels"] == 210
    same_scene.append({"frame": a["frame"], "originalChromeBoxes": a["boxes"], "exactHalfChromeBoxes": b["boxes"], **pair})
    pair = comparison(c, d)
    assert all(pair[role]["completeRgbaIdentical"] for role in ["chrome", "native"])
    historical_comparisons.append({"frame": c["frame"], **pair})

sample_points = [(50, 24), (50, 25), (50, 26), (20, 25), (20, 45)]
samples = []
for row in [original[0], exact_half[0], minimal[0]]:
    samples.append({"name": row["name"], "frame": 0, "roles": {
        role: [{"x": x, "y": y, "rgba": list(pixels(row, role).getpixel((x, y)))} for x, y in sample_points]
        for role in ["chrome", "native"]
    }})

receipt = {
    "scope": "Exact-half replacement reproduces all native images but not Chrome images. Independent historical minimal source reproduces both historical image roles exactly. These characterize retained fractional coverage and Chrome subpixel sizing; they do not prove the full source images are equivalent or every ordinary-file paint composition impossible.",
    "sameSceneComparisons": same_scene,
    "historicalMinimalComparisons": historical_comparisons,
    "samples": samples,
    "bindings": [binding(p) for p in [Path(__file__), visible_path, control_path, historical_path,
        ROOT / "validation/public-numeric-native-fractional-cases.json",
        ROOT / "validation/flex-fixed-controls-cases.json",
        ROOT / "validation/flex-fixed-controls-review.md"]],
}
(OUT / "fractional/comparisons.json").write_text(json.dumps(receipt, indent=2) + "\n")
print(json.dumps({"sameSourceNativeIdentical": len(same_scene), "sameSourceChromeDifferent": len(same_scene), "historicalMinimalBothRolesIdentical": len(historical_comparisons)}))
