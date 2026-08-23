# V2 — compile and configuration matrix

Status: GREEN on 2026-08-22.

Command: `make renderer-native-metal-platform-matrix`.

The exact `-Dwarnings -Aunfulfilled-lint-expectations` matrix passed 9/9:

- `aarch64-apple-darwin`
- `x86_64-apple-darwin`
- `aarch64-apple-ios`
- `aarch64-apple-ios-sim`
- `x86_64-apple-ios`
- `aarch64-apple-tvos`
- `aarch64-apple-tvos-sim`
- `aarch64-apple-visionos`
- `aarch64-apple-visionos-sim`

The tvOS and visionOS lanes used the prescribed nightly `-Z build-std=std,panic_abort` path. All nine per-target logs are empty. The final run log SHA-256 is `6e7d3b26e7c0a5422f5d2465e854ffa44b77213f8e951fb71153c068435469dc`.
