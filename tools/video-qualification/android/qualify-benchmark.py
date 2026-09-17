#!/usr/bin/env python3
"""Run the installed 720p Vulkan fixture on an explicitly selected Android host."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import time

parser = argparse.ArgumentParser()
parser.add_argument('--serial', required=True)
args = parser.parse_args()
sdk = os.environ.get('ANDROID_SDK_ROOT') or os.environ.get('ANDROID_HOME')
adb = str(Path(sdk) / 'platform-tools/adb') if sdk else shutil.which('adb')
if not adb:
    raise SystemExit('Set ANDROID_HOME or put adb on PATH')
base = [adb, '-s', args.serial]
package = 'ai.nuxie.videoqualification'
subprocess.run(base + ['shell', 'run-as', package, 'rm', '-f', 'files/video-proof-result.txt'],
               check=True, timeout=15)
subprocess.run(base + ['shell', 'am', 'start', '-S', '-n', package + '/.MainActivity',
                      '--ez', 'benchmark', 'true'], check=True, timeout=15)
log = Path(__file__).resolve().parents[3] / 'target/video-720p-android-release-live.log'
deadline = time.monotonic() + 60
while time.monotonic() < deadline:
    result = subprocess.run(base + ['shell', 'run-as', package, 'cat', 'files/video-proof-result.txt'],
                            capture_output=True, text=True, timeout=15)
    if result.returncode == 0 and result.stdout:
        log.write_text(result.stdout + '\n')
        print(result.stdout)
        raise SystemExit(0 if result.stdout.startswith('PASS: 720p Vulkan benchmark;') else 1)
    time.sleep(.5)
raise SystemExit('Android benchmark qualification timed out')
