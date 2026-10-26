#!/usr/bin/env python3
"""Package/sign an already pool-compiled iPhone probe; never compiles locally."""
import argparse
from pathlib import Path
import plistlib
import shutil
import subprocess

parser = argparse.ArgumentParser()
parser.add_argument("--binary", type=Path, required=True)
parser.add_argument("--profile", type=Path, required=True)
parser.add_argument("--identity", required=True)
parser.add_argument("--out", type=Path, required=True)
parser.add_argument("--device", required=True)
args = parser.parse_args()
bundle = "ai.nuxie.audio-validation"
profile = plistlib.loads(subprocess.check_output(["security", "cms", "-D", "-i", str(args.profile)]))
assert args.device in profile.get("ProvisionedDevices", []), "profile does not authorize this device"
entitlements = profile["Entitlements"].copy()
team = profile["TeamIdentifier"][0]
identifier = entitlements["application-identifier"]
assert identifier in (team + ".*", team + "." + bundle), "profile does not authorize probe bundle"
entitlements["application-identifier"] = team + "." + bundle
entitlements["com.apple.developer.team-identifier"] = team
if "keychain-access-groups" in entitlements:
    entitlements["keychain-access-groups"] = [team + "." + bundle]
args.out.mkdir(parents=True, exist_ok=False)
shutil.copy2(args.binary, args.out / "NuxieAudioValidation")
(args.out / "NuxieAudioValidation").chmod(0o755)
shutil.copy2(args.profile, args.out / "embedded.mobileprovision")
info = {
    "CFBundleIdentifier": bundle,
    "CFBundleExecutable": "NuxieAudioValidation",
    "CFBundleName": "NuxieAudioValidation",
    "CFBundleDisplayName": "Audio Validation",
    "CFBundlePackageType": "APPL",
    "CFBundleInfoDictionaryVersion": "6.0",
    "CFBundleSupportedPlatforms": ["iPhoneOS"],
    "LSRequiresIPhoneOS": True,
    "CFBundleVersion": "1",
    "CFBundleShortVersionString": "1.0",
    "MinimumOSVersion": "16.0",
    "UIDeviceFamily": [1, 2],
    "UIRequiredDeviceCapabilities": ["arm64"],
    "UILaunchScreen": {},
    "UIFileSharingEnabled": True,
    "LSSupportsOpeningDocumentsInPlace": True,
    "UISupportedInterfaceOrientations": ["UIInterfaceOrientationPortrait"],
}
(args.out / "Info.plist").write_bytes(plistlib.dumps(info))
entitlement_file = args.out.parent / (args.out.name + ".entitlements.plist")
entitlement_file.write_bytes(plistlib.dumps(entitlements))
subprocess.run(["codesign", "--force", "--sign", args.identity, "--entitlements", str(entitlement_file), str(args.out)], check=True)
subprocess.run(["codesign", "--verify", "--strict", str(args.out)], check=True)
print(args.out)
