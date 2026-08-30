#!/usr/bin/env node
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const target = process.env.CARGO_TARGET_DIR ?? path.join(root, "target");
const rustTool = (name) => execFileSync("rustup", ["which", "--toolchain", "stable", name], { encoding: "utf8" }).trim();
const cargoEnv = { ...process.env, RUSTC: rustTool("rustc"), RUSTDOC: rustTool("rustdoc") };
const cargo = (args, encoding = "utf8") => execFileSync("rustup", ["run", "stable", "cargo", ...args], {
  cwd: root, env: cargoEnv, encoding, stdio: ["ignore", "pipe", "inherit"],
});
const vectors = JSON.parse(cargo(["run", "--quiet", "-p", "nuxie-script-signature", "--example", "native_vectors"]));
cargo(["build", "--release", "-p", "nuxie-script-signature", "--example", "wasm_probe", "--target", "wasm32-unknown-unknown"]);
const wasm = readFileSync(path.join(target, "wasm32-unknown-unknown/release/examples/wasm_probe.wasm"));
const module = new WebAssembly.Module(wasm);
assert.deepEqual(WebAssembly.Module.imports(module), [], "verifier must have zero host/WASI/RNG imports");
const { exports } = new WebAssembly.Instance(module, {});
let checks = 0;
function verify(vector, expected) {
  const allocations = [];
  const put = (bytes) => {
    const pointer = exports.signature_probe_alloc(bytes.length);
    allocations.push([pointer, bytes.length]);
    new Uint8Array(exports.memory.buffer, pointer, bytes.length).set(bytes);
    return pointer;
  };
  try {
    const signature = put(vector.signature);
    const message = put(vector.message);
    const context = put(vector.context);
    const publicKey = put(vector.publicKey);
    assert.equal(exports.signature_probe_verify(signature, message, vector.message.length, context, publicKey), Number(expected));
    checks += 1;
  } finally {
    for (const [pointer, length] of allocations) exports.signature_probe_free(pointer, length);
  }
}
for (const vector of vectors) {
  verify(vector, true);
  for (let index = 0; index < 64; index += 1) {
    const signature = [...vector.signature];
    signature[index] ^= 1;
    verify({ ...vector, signature }, false);
  }
  verify({ ...vector, message: [...vector.message, 1] }, false);
  verify({ ...vector, context: vector.context.map((byte, index) => index === 0 ? byte ^ 1 : byte) }, false);
  verify({ ...vector, publicKey: vector.publicKey.map((byte, index) => index === 0 ? byte ^ 1 : byte) }, false);
  verify({ ...vector, signature: Array(64).fill(0) }, false);
}
console.log(`WASM signature verifier: ${checks} differential checks passed; ${wasm.length} bytes; zero imports`);
