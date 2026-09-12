"""Build unscaled review sheets and prove each transferred pair's full RGBA."""
import hashlib
import json
import pathlib
import sys
from PIL import Image, ImageDraw

out = pathlib.Path(sys.argv[1]).resolve()
receipt_file = out / "render/receipt.json"
receipt = json.loads(receipt_file.read_text())
rows = receipt["rows"]

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def source(row, role):
    path = pathlib.Path(row["prefix"] + "." + role + ".png")
    assert sha(path) == row[role + "Sha256"]
    return Image.open(path).convert("RGBA")

def same_white_canvas(left, right):
    size = (max(left.width, right.width), max(left.height, right.height))
    images = []
    for image in (left, right):
        canvas = Image.new("RGBA", size, "white")
        canvas.paste(image, (0, 0))
        images.append(canvas)
    return images[0].tobytes() == images[1].tobytes()

direct = [row for row in rows if row["frame"] == 0 or
          (row["name"] == "recovery-missing-responsive-width" and row["frame"] in [1, 2])]
coverage = []
for row in rows:
    if row in direct:
        coverage.append({"name": row["name"], "frame": row["frame"], "method": "direct"})
        continue
    for reference in direct:
        if reference["name"] != row["name"]:
            continue
        if row["name"] == "recovery-missing-responsive-width" and (reference["width"], reference["height"]) != (row["width"], row["height"]):
            continue
        assert reference["requestSha256"] == row["requestSha256"]
        assert reference["rivSha256"] == row["rivSha256"]
        assert reference["sourceMapSha256"] == row["sourceMapSha256"]
        if all(same_white_canvas(source(reference, role), source(row, role)) for role in ["chrome", "native"]):
            coverage.append({"name": row["name"], "frame": row["frame"], "method": "complete-rgba-white-canvas", "referenceFrame": reference["frame"]})
            break
    else:
        raise AssertionError("unreviewed distinct pair: " + row["name"] + " " + str(row["frame"]))

# One sheet per at most four pairs, at native image resolution. Large responsive
# viewports get separate sheets to avoid shrinking any source frame.
groups = [[row] for row in direct if row["width"] > 240]
small = [row for row in direct if row["width"] <= 240]
groups += [small[index:index + 4] for index in range(0, len(small), 4)]
sheets = []
placements = []
for index, group in enumerate(groups):
    width = max(row["width"] * 2 + 30 for row in group)
    height = sum(row["height"] + 40 for row in group)
    image = Image.new("RGBA", (width, height), "#e8e8e8")
    draw = ImageDraw.Draw(image)
    y = 0
    for row in group:
        draw.text((8, y + 6), f'{row["name"]} {row["width"]}x{row["height"]}; Chrome left / native right', fill="black")
        for role, x in [("chrome", 10), ("native", row["width"] + 20)]:
            original = source(row, role)
            image.paste(original, (x, y + 30))
            assert image.crop((x, y + 30, x + original.width, y + 30 + original.height)).tobytes() == original.tobytes()
            placements.append({"name": row["name"], "frame": row["frame"], "role": role, "sheet": index, "position": [x, y + 30], "completeRgbaExact": True})
        y += row["height"] + 40
    path = out / f"visual-{index}.png"
    image.save(path)
    sheets.append({"path": str(path), "sha256": sha(path), "pairs": [{"name": row["name"], "frame": row["frame"]} for row in group]})

result = {"scope": "Sheets prepared for direct human/model inspection; transfer proof uses complete RGBA after explicit white-canvas extension. Direct inspection is recorded in the review separately.",
          "renderReceiptSha256": sha(receipt_file), "driverSha256": sha(pathlib.Path(__file__)),
          "directPairs": len(direct), "transferredPairs": len(coverage) - len(direct),
          "coverage": coverage, "sheets": sheets, "placements": placements}
(out / "visual-coverage.json").write_text(json.dumps(result, indent=2) + "\n")
print(json.dumps({"directPairs": len(direct), "transferredPairs": len(coverage) - len(direct), "sheets": len(sheets)}))
