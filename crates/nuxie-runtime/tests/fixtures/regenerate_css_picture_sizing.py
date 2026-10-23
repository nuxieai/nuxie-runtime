#!/usr/bin/env python3
"""Capture CSS picture border boxes from local headless Chrome, offline."""
import argparse
import glob
import html
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--chrome", default=os.environ.get("CHROME"), help="Chrome executable")
    args = parser.parse_args()
    candidates = [args.chrome] if args.chrome else [
        shutil.which("google-chrome"), shutil.which("chromium"),
        *sorted(glob.glob(str(Path.home() / "Library/Caches/ms-playwright/chromium_headless_shell-*/*/chrome-headless-shell")), reverse=True),
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    ]
    chrome = next((p for p in candidates if p and Path(p).is_file()), None)
    if chrome is None:
        parser.error("Pass --chrome or set CHROME to an installed executable; no downloads are made")
    fixtures = Path(__file__).resolve().parent
    # A separate profile leaves existing Chrome sessions untouched. Single
    # process mode allows the oracle to run in the macOS host sandbox.
    with tempfile.TemporaryDirectory(prefix="nuxie-picture-oracle-") as profile:
        result = subprocess.run([
            chrome, "--headless", "--no-sandbox", "--single-process", "--in-process-gpu",
            "--no-zygote", "--disable-gpu", "--disable-background-networking",
            "--disable-component-update", "--disable-sync", "--disable-extensions",
            "--no-first-run", "--no-default-browser-check", "--host-resolver-rules=MAP * ~NOTFOUND",
            f"--user-data-dir={profile}", "--dump-dom", "--virtual-time-budget=2000",
            (fixtures / "css_picture_sizing.html").as_uri(),
        ], capture_output=True, text=True, timeout=60, check=True)
    match = re.search(r'<pre id="results">(.*?)</pre>', result.stdout, re.S)
    if not match or not match[1]:
        raise RuntimeError(f"Chrome did not produce measurements: {result.stderr}")
    captured = json.loads(html.unescape(match[1]))
    (fixtures / "css_picture_sizing_chrome.json").write_text(json.dumps(captured, indent=2) + "\n")
    print(f"Captured {len(captured['cases'])} cases: {captured['userAgent']}")


if __name__ == "__main__":
    main()
