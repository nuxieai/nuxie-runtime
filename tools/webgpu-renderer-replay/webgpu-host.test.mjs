import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const hostUrl = new URL("./webgpu-host.js", import.meta.url);
const hostSource = await readFile(hostUrl, "utf8");
const replayHtml = await readFile(new URL("./index.html", import.meta.url), "utf8");

async function loadFreshHost() {
  return import(
    `data:text/javascript;base64,${Buffer.from(hostSource).toString("base64")}#${crypto.randomUUID()}`
  );
}

function fakeSession(label) {
  const submitted = [];
  const shaderModules = [];
  const context = {
    currentTextureCalls: 0,
    getCurrentTexture() {
      this.currentTextureCalls += 1;
      return { label: `${label}-surface-texture` };
    },
  };
  const queue = {
    async onSubmittedWorkDone() {
      submitted.push(label);
    },
  };
  const device = {
    destroyed: false,
    features: new Set(),
    limits: {},
    lost: new Promise(() => {}),
    queue,
    addEventListener() {},
    createShaderModule(descriptor) {
      shaderModules.push(descriptor);
      return descriptor;
    },
    destroy() {
      this.destroyed = true;
    },
  };
  const adapter = {
    features: new Set(),
    async requestDevice() {
      return device;
    },
  };
  const canvas = {
    width: 32,
    height: 24,
    getContext(kind) {
      assert.equal(kind, "webgpu");
      return context;
    },
  };
  return { adapter, canvas, context, device, shaderModules, submitted };
}

test("keeps prepared WebGPU canvases isolated by session", async () => {
  const first = fakeSession("first");
  const second = fakeSession("second");
  const adapters = [first.adapter, second.adapter];
  Object.defineProperty(globalThis, "navigator", {
    configurable: true,
    value: {
      gpu: {
        async requestAdapter() {
          return adapters.shift() ?? null;
        },
      },
    },
  });
  const host = await loadFreshHost();
  const firstSessionId = await host.prepareWebGpu(first.canvas);
  const secondSessionId = await host.prepareWebGpu(second.canvas);
  assert.notEqual(firstSessionId, secondSessionId);

  const memory = new WebAssembly.Memory({ initial: 1 });
  const imports = host.createWebGpuImports(() => ({
    memory,
    __wbindgen_free() {},
    __wbindgen_malloc() {
      return 1024;
    },
  }));
  const firstInstance = imports.wgpuCreateInstance();
  const secondInstance = imports.wgpuCreateInstance();
  const firstSurface = imports.wgpuInstanceCreateSurface(firstInstance, 0);
  const secondSurface = imports.wgpuInstanceCreateSurface(secondInstance, 0);
  imports.wgpuSurfaceGetCurrentTexture(firstSurface, 16);
  imports.wgpuSurfaceGetCurrentTexture(secondSurface, 32);

  assert.equal(first.context.currentTextureCalls, 1);
  assert.equal(second.context.currentTextureCalls, 1);
  await host.waitForWebGpu(firstSessionId);
  await host.waitForWebGpu(secondSessionId);
  assert.deepEqual(first.submitted, ["first"]);
  assert.deepEqual(second.submitted, ["second"]);

  host.releaseWebGpu(firstSessionId);
  assert.equal(first.device.destroyed, true);
  assert.equal(second.device.destroyed, false);
  await assert.rejects(
    host.waitForWebGpu(firstSessionId),
    /session .* is unavailable/i,
  );
  await host.waitForWebGpu(secondSessionId);
  host.releaseWebGpu(secondSessionId);
  assert.equal(second.device.destroyed, true);
});

test("ordinary queue submission does not allocate a capture buffer", () => {
  const queueSubmit = hostSource.slice(
    hostSource.indexOf("wgpuQueueSubmit:"),
    hostSource.indexOf("wgpuQueueWriteBuffer:"),
  );
  assert.ok(queueSubmit.includes("object(queue).submit(values)"));
  assert.ok(!queueSubmit.includes("createBuffer"));
  assert.ok(!queueSubmit.includes("copyTextureToBuffer"));
});

test("provides exact GPU-canvas ownership and dynamic-state imports", async () => {
  const host = await loadFreshHost();
  const imports = host.createWebGpuImports(() => null);

  assert.equal(typeof imports.wgpuSamplerAddRef, "function");
  assert.equal(
    typeof imports.wgpuRenderPassEncoderSetBlendConstant,
    "function",
  );
});

test("decodes Wagyu shader descriptors with their exact wasm32 layout", async () => {
  const session = fakeSession("shader");
  Object.defineProperty(globalThis, "navigator", {
    configurable: true,
    value: {
      gpu: {
        async requestAdapter() {
          return session.adapter;
        },
      },
    },
  });
  const host = await loadFreshHost();
  const sessionId = await host.prepareWebGpu(session.canvas);
  const memory = new WebAssembly.Memory({ initial: 1 });
  const callbacks = new Map();
  let allocation = 4096;
  const wasm = {
    memory,
    __indirect_function_table: {
      get(index) {
        return callbacks.get(index);
      },
    },
    __wbindgen_free() {},
    __wbindgen_malloc(size, alignment = 1) {
      allocation = Math.ceil(allocation / alignment) * alignment;
      const pointer = allocation;
      allocation += size;
      return pointer;
    },
  };
  const imports = host.createWebGpuImports(() => wasm);
  const words = new Uint32Array(memory.buffer);
  const registerCallback = (callbackInfo, callbackIndex, callback) => {
    callbacks.set(callbackIndex, callback);
    words[(callbackInfo + 8) >>> 2] = callbackIndex;
  };

  let adapterHandle;
  registerCallback(64, 1, (_status, handle) => {
    adapterHandle = handle;
  });
  const instanceHandle = imports.wgpuCreateInstance();
  imports.wgpuInstanceRequestAdapter(instanceHandle, 0, 64);

  let deviceHandle;
  registerCallback(96, 2, (_status, handle) => {
    deviceHandle = handle;
  });
  imports.wgpuAdapterRequestDevice(adapterHandle, 0, 96);

  const descriptor = 256;
  const chain = 512;
  const sourcePointer = 1024;
  const source = "@vertex fn vertex_main() -> @builtin(position) vec4f { return vec4f(); }";
  const sourceBytes = new TextEncoder().encode(source);
  new Uint8Array(memory.buffer).set(sourceBytes, sourcePointer);
  words[descriptor >>> 2] = chain;
  words[(chain + 4) >>> 2] = 393228;
  words[(chain + 8) >>> 2] = sourceBytes.length;
  words[(chain + 12) >>> 2] = sourcePointer;
  words[(chain + 16) >>> 2] = 3;

  imports.wgpuDeviceCreateShaderModule(deviceHandle, descriptor);

  assert.deepEqual(session.shaderModules, [{ label: undefined, code: source }]);
  host.releaseWebGpu(sessionId);
});

test("replay schedules explicit capture before yielding the surface texture", () => {
  const captureIndex = replayHtml.indexOf(
    "const capture = captureWebGpuPixels(sessionId)",
  );
  const waitIndex = replayHtml.indexOf("await waitForWebGpu(sessionId)");
  assert.ok(captureIndex >= 0);
  assert.ok(waitIndex > captureIndex);
});
