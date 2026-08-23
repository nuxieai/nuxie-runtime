# V8 — rooted product and no fallback

Status: GREEN on 2026-08-22.

Commands:

- `make renderer-native-metal-tracer-binary`
- `make ore-metal-binding-witness`
- `make ore-metal-authenticated-gpu-canvas`
- `make renderer-shaders-check`

Results:

- Rooted arm64 Mach-O: 986,880 bytes, SHA-256 `fba234ec9ffd4df5a2e7b439d55f4ad94d7791f9cfc4052531b39790828c1481`.
- The native-only normal and build Cargo graphs contain zero forbidden WGPU/Naga/WGSL/WebGPU/Dawn dependency rows.
- The product executable and translated-path/shader-inventory/output assertions exited successfully.
- Forbidden linked-symbol and binary token scans are zero; required `CAMetalLayer`, `nextDrawable`, and `presentDrawable:` markers remain.
- ORE binding witness passed 1/1; authenticated GPU canvas passed 2/2 with Metal API/GPU validation enabled.
- Shader reproducibility regenerated 66 Rust modules at digest `44841b4b740f5a45b91eef19c98a62a57239e4725628d54fc5bc1fbe678732ed` and 56 pinned C++ headers at digest `97312cdab2f0621620d1ad55096464b138aa5ef1dae4a168b0aa223b09f3d64b`.

The final rooted-product log SHA-256 is `40de4312bddd2f2d428f6482300717b3b7cd2fb3d06463179010b03ce626b823`.
