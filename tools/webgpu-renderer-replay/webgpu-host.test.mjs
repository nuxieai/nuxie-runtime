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
    queue,
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
  return { adapter, canvas, context, device, submitted };
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

test("replay schedules explicit capture before yielding the surface texture", () => {
  const captureIndex = replayHtml.indexOf(
    "const capture = captureWebGpuPixels(sessionId)",
  );
  const waitIndex = replayHtml.indexOf("await waitForWebGpu(sessionId)");
  assert.ok(captureIndex >= 0);
  assert.ok(waitIndex > captureIndex);
});
