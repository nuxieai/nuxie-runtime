#!/usr/bin/env python3
"""Run the installed optimized 720p fixture app on a chosen physical device."""
import argparse
from pathlib import Path
import subprocess
import time

parser = argparse.ArgumentParser()
parser.add_argument('--device', required=True)
args = parser.parse_args()
root = Path(__file__).resolve().parents[3]
log = root / 'target' / 'video-720p-ios-release-live.log'
command = ['xcrun', 'devicectl', 'device', 'process', 'launch', '--device', args.device,
           '--terminate-existing', '--console', 'com.nuxie.video-proof', '--benchmark']
with log.open('w') as output:
    console = subprocess.Popen(command, stdout=output, stderr=subprocess.STDOUT)
    try:
        deadline = time.monotonic() + 60
        while time.monotonic() < deadline:
            text = log.read_text()
            if 'NUX_VIDEO_BENCHMARK status=' in text:
                print(text[-6000:])
                raise SystemExit(0 if 'NUX_VIDEO_BENCHMARK status=1 ' in text else 1)
            if console.poll() is not None:
                print(text[-6000:])
                raise SystemExit('device console ended before qualification')
            time.sleep(.2)
        print(log.read_text()[-6000:])
        raise SystemExit('benchmark qualification timed out')
    finally:
        if console.poll() is None:
            console.kill()
        console.wait(timeout=5)
