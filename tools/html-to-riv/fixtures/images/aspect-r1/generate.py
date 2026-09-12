"""Deterministic authored aspect-ratio controls; never rasterizes HTML/CSS.

Run in place once. Existing encoded source files and manifests are preserved.
"""
import hashlib
import json
from pathlib import Path
from PIL import Image, __version__

root = Path(__file__).resolve().parent
dimensions = {"portrait": (48, 96), "wide": (160, 40), "square": (41, 41),
              "odd": (37, 23), "small": (3, 5)}
colors = [(236, 51, 62), (31, 121, 220), (22, 173, 111), (245, 187, 37),
          (111, 66, 193), (23, 167, 180), (238, 115, 33), (35, 42, 59)]
assert not (root / 'manifest.json').exists()
assert all(not (root / (name + '.png')).exists() for name in dimensions)
rows = []
for name, (width, height) in dimensions.items():
    image = Image.new('RGB', (width, height))
    for y in range(height):
        for x in range(width):
            color = colors[(x * 4 // width + 2 * (y * 4 // height)) % len(colors)]
            if x == 0 and y == 0:
                color = (255, 255, 255)
            elif x == width - 1 and y == height - 1:
                color = (0, 0, 0)
            image.putpixel((x, y), color)
    path = root / (name + '.png')
    image.save(path, format='PNG', compress_level=9)
    rows.append({'name': path.name, 'width': width, 'height': height, 'mode': 'RGB',
                 'bytes': path.stat().st_size, 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()})
(root / 'manifest.json').write_text(json.dumps({
    'scope': 'Programmatic authored RGB image fixtures; no browser screenshot or target-driven pixel adjustment.',
    'pillow': __version__, 'generatorSha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
    'files': rows,
}, indent=2) + '\n')
print(json.dumps(rows, indent=2))
