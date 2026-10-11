#!/usr/bin/env node
// Build the probe through tools/bazel/runtime.py and prepare it with wasm-bindgen --target nodejs.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve } from "node:path";
const [bindingsPath, fixturePath] = process.argv.slice(2);
assert.ok(bindingsPath && fixturePath, "usage: node tools/test-text-input-clock-wasm.mjs <bindings.js> <text_input_tray.riv>");
const probe = createRequire(import.meta.url)(resolve(bindingsPath));
const bytes = readFileSync(fixturePath);
// Transfer fixture bytes using a generated wasm-bindgen slice wrapper.
assert.equal(probe.clock_probe_pointer_down_bytes(bytes), 1, "a real text-input pointer-down must finish on wasm32-unknown-unknown");
console.log("PASS: wasm text-input pointer-down with the browser monotonic clock");
