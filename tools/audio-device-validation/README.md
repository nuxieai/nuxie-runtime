# Audio device qualification

Standalone test hosts exercise the shipping `nuxie-audio` mixer without adding
shipping APIs, dependencies, or SDK releases. All compilation runs on the configured
Nuxie pool. Only artifact execution, device installation, and iOS development
signing run locally.

The fixture mixes a decoded 24 kHz mono PCM WAV with a custom renderer, changes
its frequency/gain during playback, detaches it while the WAV continues, then
stops the WAV. It checks finite bounded samples, changing levels, final silence,
renderer callback ordering, and destruction. The Mac host additionally checks
that its device clock stops after sink teardown and reopens with two block sizes.
The mobile hosts consume the same production mixer through AVAudioSourceNode and
AudioTrack. These are standalone hosts, not the shipping SDK/view lifecycle.

This demonstrates samples reaching native output APIs. It does not capture
speaker output, measure acoustic fidelity, test Bluetooth, interruptions, long
background sessions, or certify physical Android hardware. The input WAV is a
synthetic decoded fixture, not a full RIV asset integration test.

## Pool builds

From the repository root, with the installed Nuxie pool client/configuration:

```sh
python3 tools/audio-device-validation/pool.py test
python3 tools/audio-device-validation/pool.py mac
python3 tools/audio-device-validation/pool.py ios
ANDROID_NDK_HOME=/path/to/android/ndk/29.0.14206865 python3 tools/audio-device-validation/pool.py android
```

Build commands print retrieved artifact paths and retain product pointers under
`.nuxie/audio-validation-*-products.json`. They have no local compilation fallback.
The standalone targets use existing pinned dependencies; no Cargo repin is needed.
The iOS host uses the selected pool worker's installed Xcode/iPhoneOS SDK. Android
packaging requires API 36, a complete Android build-tools installation, and Java 17 or newer on the pool.

## Run

Mac: execute the printed `macos_probe` binary on the intended output device.
Expect two JSON records with `pass: true`, for requested blocks 256 and 1024.
The probe uses modest sample amplitudes and does not change system volume.

For iOS, package the pool-produced executable with an existing development identity
and a provisioning profile that authorizes the specific test device:

```sh
python3 tools/audio-device-validation/apple/package-sign.py \
  --binary /path/to/NuxieAudioValidation --profile /path/to/profile.mobileprovision \
  --identity 'Apple Development: ...' --device DEVICE_ID --out /tmp/AudioValidation.app
xcrun devicectl device install app --device DEVICE_ID /tmp/AudioValidation.app
xcrun devicectl device process launch --device DEVICE_ID --console ai.nuxie.audio-validation
```

Pairing, Developer Mode, and an unlocked device are required. Wireless pairing is
supported. The app prints its result and writes `Documents/audio-validation.txt`.
Check both the host result and nested mixer `pass`, not merely successful launch.
Signing credentials/profiles stay local and must not be committed.

For Android, copy the retrieved shared library to `/tmp` (pool runtime inputs cannot
be inside `.nuxie`), then package it on the pool:

```sh
python3 tools/audio-device-validation/pool.py android-apk \
  --library /path/to/libaudio_device_validation.so --output /tmp/audio-validation.apk
adb -s DEVICE install /tmp/audio-validation.apk
adb -s DEVICE shell am start -n ai.nuxie.audiovalidation/.MainActivity
adb -s DEVICE logcat -d -s NuxieAudioValidation
adb -s DEVICE shell run-as ai.nuxie.audiovalidation cat files/audio-validation.json
```

Check `hostOutputPassed`, `teardownCompleted`, and nested `rust.pass`. Force-stop and relaunch the same APK to
exercise recreation. Each packaging run creates a disposable signing identity:
remove only the validation package before installing a newly packaged APK, or
reuse the same APK. Packaging is not byte-for-byte reproducible.
Remove test packages and stop an emulator only if this validation run created them;
do not disturb other apps or devices.

## Luau oscillator qualification

The second probe uses the shipping `ScriptVm`, the real `Audio.createProcessor`
bindings, and a real script instance. Its Lua fixture is `src/luau_fixture.lua`.
Run `pool.py luau-test`, `luau-mac`, `luau-ios`, or `luau-android` to build this
variant; package/run its artifacts with the same device commands above. Do not
link both probe variants into the same host: they intentionally share the test ABI.
For Android the new cdylib is still packaged as `libaudio_device_validation.so`.

The four phases are oscillator at 440Hz/gain0.02, scheduled/ramped change to
880Hz/gain0.01, fade-stop plus a second owned oscillator at330Hz/gain0.005, and
script-instance destruction while its Lua table and processor handle remain alive.
The last phase must be silent. The control VM stays on the constructing thread;
only a separate thread-safe mixer/measurement object reaches audio callbacks.
The meter/status calls are made in the script. Exact sample-offset/ramp semantics
are checked by unit tests, not inferred from hardware timing or acoustic output.

The original decoded-WAV/raw-renderer probe remains available unchanged. Reports
from the Luau variant identify `mode: luau-oscillator`; keep their results separate
from the original qualification records.
