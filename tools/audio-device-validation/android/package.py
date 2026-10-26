#!/usr/bin/env python3
"""Compile/package the fixture APK on the selected build-pool host.

Invoke through the :package Bazel run target with declared pool runtime IO.
The library argument is the separately pool-built arm64 Android cdylib.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import tempfile
import zipfile


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--library", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    library = args.library.resolve(strict=True)
    header = library.read_bytes()[:20]
    if header[:5] != b"\x7fELF\x02" or struct.unpack("<H", header[18:20])[0] != 183:
        raise RuntimeError("expected an arm64 Android ELF library")
    sdk = next((path for path in [
        Path(os.environ.get("ANDROID_SDK_ROOT", "/nonexistent")),
        Path(os.environ.get("ANDROID_HOME", "/nonexistent")),
        Path("/opt/homebrew/share/android-commandlinetools"),
        Path("/opt/android-sdk"),
    ] if (path / "platforms/android-36/android.jar").is_file()), None)
    if sdk is None:
        raise RuntimeError("pool host lacks Android API36 SDK")
    candidates = sorted((sdk / "build-tools").glob("*"), reverse=True)
    buildtools = next((path for path in candidates if all(
        (path / tool).is_file() for tool in ("d8", "aapt2", "zipalign", "apksigner"))), None)
    if buildtools is None:
        raise RuntimeError("pool host lacks complete Android build tools: " + str(candidates))
    print("Android build tools: " + str(buildtools), flush=True)
    android_jar = sdk / "platforms/android-36/android.jar"
    environment = dict(os.environ)
    java_homes = [Path(environment.get("JAVA_HOME", "/nonexistent"))]
    java_homes += [Path("/opt/homebrew/opt") / name / "libexec/openjdk.jdk/Contents/Home"
                   for name in ("openjdk@17", "openjdk@21", "openjdk")]
    if Path("/usr/libexec/java_home").is_file():
        detected = subprocess.run(["/usr/libexec/java_home"], capture_output=True, text=True)
        if detected.returncode == 0:
            java_homes.append(Path(detected.stdout.strip()))
    java_home = next((path for path in java_homes if (path / "bin/javac").is_file()), None)
    if java_home is None:
        raise RuntimeError("pool host lacks a Java development kit")
    environment["JAVA_HOME"] = str(java_home)
    java_bin = java_home / "bin"
    environment["PATH"] = str(java_bin) + os.pathsep + environment.get("PATH", os.defpath)
    here = Path(__file__).resolve().parent
    output = args.output.resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="nuxie-audio-apk-") as temporary:
        root = Path(temporary)
        classes = root / "classes"
        dex = root / "dex"
        classes.mkdir()
        dex.mkdir()
        def run(*command):
            subprocess.run([str(x) for x in command], env=environment, check=True)
        run(java_bin / "javac", "-source", "17", "-target", "17", "-classpath", android_jar,
            "-d", classes, here / "ai/nuxie/audiovalidation/MainActivity.java")
        jar = root / "classes.jar"
        with zipfile.ZipFile(jar, "w", zipfile.ZIP_DEFLATED) as archive:
            for path in sorted(classes.rglob("*.class")):
                archive.write(path, path.relative_to(classes).as_posix())
        run(buildtools / "d8", "--lib", android_jar, "--min-api", "26", "--output", dex, jar)
        unsigned = root / "unsigned.apk"
        run(buildtools / "aapt2", "link", "-o", unsigned, "-I", android_jar,
            "--manifest", here / "AndroidManifest.xml")
        with zipfile.ZipFile(unsigned, "a", zipfile.ZIP_DEFLATED, strict_timestamps=False) as archive:
            archive.write(dex / "classes.dex", "classes.dex")
            archive.write(library, "lib/arm64-v8a/libaudio_device_validation.so")
        aligned = root / "aligned.apk"
        run(buildtools / "zipalign", "-f", "-p", "4", unsigned, aligned)
        key = root / "debug.keystore"
        # Disposable validation-only identity, never a release signing key.
        run(java_bin / "keytool", "-genkeypair", "-keystore", key, "-storepass", "android",
            "-keypass", "android", "-alias", "androiddebugkey", "-dname", "CN=Nuxie Audio Validation",
            "-keyalg", "RSA", "-keysize", "2048", "-validity", "3650")
        run(buildtools / "apksigner", "sign", "--ks", key, "--ks-pass", "pass:android",
            "--out", output, aligned)
        run(buildtools / "apksigner", "verify", output)
    print(json.dumps({"apk": str(output), "sha256": hashlib.sha256(output.read_bytes()).hexdigest(),
                      "librarySha256": hashlib.sha256(library.read_bytes()).hexdigest()}))


if __name__ == "__main__":
    main()
