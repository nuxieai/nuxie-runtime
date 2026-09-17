#!/usr/bin/env python3
"""Drive the installed fixture app through Settings and back on a chosen device."""
import argparse
from pathlib import Path
import subprocess
import time

parser = argparse.ArgumentParser()
parser.add_argument('--device', required=True)
parser.add_argument('--embedded', action='store_true')
parser.add_argument('--audible', action='store_true')
args = parser.parse_args()
root = Path(__file__).resolve().parents[3]
mode = 'embedded' if args.embedded else 'external'
if args.audible:
    mode = 'audible-' + mode
log = root / 'target' / f'video-os-lifecycle-ios-{mode}.log'
base = ['xcrun', 'devicectl', 'device', 'process', 'launch', '--device', args.device]
command = base + ['--terminate-existing', '--console', 'com.nuxie.video-proof', '--lifecycle']
if args.embedded:
    command.append('--embedded')
if args.audible:
    command.append('--audible')
with log.open('w') as output:
    console = subprocess.Popen(command, stdout=output, stderr=subprocess.STDOUT)
    try:
        deadline = time.monotonic() + 40
        activated = False
        while time.monotonic() < deadline:
            text = log.read_text()
            if 'opening app Settings' in text and not activated:
                time.sleep(1)
                subprocess.run(base + ['com.nuxie.video-proof'], check=True, timeout=15)
                activated = True
            if 'NUX_VIDEO_PROOF status=' in text:
                print(text[-6000:])
                raise SystemExit(0 if 'NUX_VIDEO_PROOF status=1 ' in text and
                                 'NUX_VIDEO_OS_LIFECYCLE Apple paused' in text else 1)
            if console.poll() is not None:
                print(text[-6000:])
                raise SystemExit('device console ended before qualification')
            time.sleep(.2)
        print(log.read_text()[-6000:])
        raise SystemExit('lifecycle qualification timed out')
    finally:
        # Kill only this console observer without forwarding a catchable signal
        # to the fixture app; retain the on-device result for inspection.
        if console.poll() is None:
            console.kill()
        console.wait(timeout=5)
