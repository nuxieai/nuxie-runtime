"""Check solid interior group-opacity samples, independently of area metrics.

Usage: SCRIPT IMAGE_DIRECTORY [--suffix .native.png]
Expected names are CASE-WIDTH.png by default. Uses only unambiguous interior
pixels; border/clipping cases require separate contour and geometry validation.
"""
import argparse
import hashlib
import json
from pathlib import Path
from PIL import Image


def close(actual, expected):
    return max(abs(a - b) for a, b in zip(actual[:3], expected)) <= 1


def check(directory, suffix='.png'):
    rows = []
    for opacity, name in [(0, '0'), (0.5, '0-5'), (1, '1')]:
        for variant in ['overlap', 'nested']:
            for width in [240, 390, 768]:
                path = directory / f'group-opacity-{variant}-{name}-{width}{suffix}'
                # Group is rendered first, then composited over opaque white.
                base = [(255, 0, 0), (0, 255, 0), (0, 0, 255)]
                if variant == 'nested':
                    base[0] = (127.5, 0, 127.5)
                expected = [tuple(int(c * opacity + 255 * (1 - opacity)) for c in color) for color in base]
                expected.append((51, 51, 51))  # Sibling must retain its own alpha.
                points = [(40, 40), (80, 80), (25, 175), (25, 185)]
                with Image.open(path) as image:
                    assert image.size == (width, 320)
                    actual = [image.getpixel(point)[:3] for point in points]
                failures = [dict(point=point, actual=a, expected=e) for point, a, e in zip(points, actual, expected) if not close(a, e)]
                rows.append(dict(name=path.name, sha256=hashlib.sha256(path.read_bytes()).hexdigest(), samples=actual, failures=failures))
    return dict(frames=len(rows), failed=sum(bool(r['failures']) for r in rows), rows=rows)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory', type=Path)
    parser.add_argument('--suffix', default='.png')
    args = parser.parse_args()
    # A common incorrect implementation accumulates alpha on every draw.
    assert not close((95, 159, 63), (127, 255, 127))
    result = check(args.directory, args.suffix)
    print(json.dumps(result, indent=2))
    raise SystemExit(bool(result['failed']))
