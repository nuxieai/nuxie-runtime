#!/usr/bin/env python3
"""Process-compatible client for the persistent exact WebGL2 candidate."""

from __future__ import annotations

import argparse
import json
import os
import sys
import urllib.error
import urllib.request
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stream", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--backend", required=True)
    parser.add_argument("--mode", required=True)
    parser.add_argument("--frame", type=int, default=0)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    endpoint = os.environ.get("WEBGL2_CANDIDATE_ENDPOINT", "http://127.0.0.1:8879")
    payload = json.dumps(
        {
            "stream": str(args.stream.resolve()),
            "output": str(args.output.resolve()),
            "backend": args.backend,
            "mode": args.mode,
            "frame": args.frame,
        }
    ).encode()
    request = urllib.request.Request(
        f"{endpoint}/render",
        data=payload,
        headers={"content-type": "application/json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=120) as response:
            result = json.load(response)
    except urllib.error.HTTPError as error:
        body = error.read().decode("utf8", errors="replace")
        print(f"WebGL2 candidate request failed: {body}", file=sys.stderr)
        return 1
    except OSError as error:
        print(f"WebGL2 candidate request failed: {error}", file=sys.stderr)
        return 1
    print(f"adapter={result['adapter']}")
    print(
        f"backend={args.backend} frame={args.frame} "
        f"size={result['width']}x{result['height']} output={args.output}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
