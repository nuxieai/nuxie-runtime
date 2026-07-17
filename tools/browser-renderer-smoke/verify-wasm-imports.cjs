const fs = require("node:fs");
const { pathToFileURL } = require("node:url");

async function main() {
  const [javascriptPath, wasmPath] = process.argv.slice(2);
  if (!javascriptPath || !wasmPath) {
    throw new Error("usage: verify-wasm-imports.cjs <javascript> <wasm>");
  }

  const javascript = fs.readFileSync(javascriptPath, "utf8");
  const wasm = fs.readFileSync(wasmPath);
  const imports = WebAssembly.Module.imports(new WebAssembly.Module(wasm));
  const forbiddenImports = imports.filter(
    ({ module, name }) => module === "env" || name === "snprintf",
  );
  const forbiddenJavascript =
    /\bsnprintf\b/.test(javascript) || /\bfrom\s+["']env["']/.test(javascript);

  if (forbiddenImports.length > 0 || forbiddenJavascript) {
    const formatted = forbiddenImports
      .map(({ module, name, kind }) => `${module}.${name} (${kind})`)
      .join(", ");
    throw new Error(
      `browser renderer contains a forbidden C/environment import${
        formatted ? `: ${formatted}` : " in generated JavaScript"
      }`,
    );
  }

  const bindings = await import(pathToFileURL(javascriptPath).href);
  bindings.initSync({ module: wasm });
  const recording = bindings.recording_float_probe();
  const expected =
    "rive-golden-stream-v1\n" + "sample seconds=0.100000001\n";
  if (recording !== expected) {
    throw new Error(
      `browser recording float probe changed: ${JSON.stringify(recording)}`,
    );
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
