"use strict";

const fs = require("node:fs");
const http = require("node:http");
const path = require("node:path");
const { spawn } = require("node:child_process");
const { chromium } = require("./browser-platform/node_modules/playwright");

const defaultRepo = path.resolve(__dirname, "../..");

async function runWebgpuContentProbe(page, repo = defaultRepo) {
  const stream = fs.readFileSync(
    path.join(repo, "fixtures/renderer/streams/solid-fill-content-probe.rive-stream"),
    "utf8",
  );
  const probe = await page.evaluate(async (fixture) => {
    const result = await window.runWebgpuReplay({
      stream: fixture,
      mode: "msaa",
      frame: 0,
    });
    return {
      sampledRgba: result.sampledRgba,
      adapter: result.stdout.find((line) => line.startsWith("adapter="))?.slice(8) || "",
    };
  }, stream);
  const expected = [0x33, 0x66, 0xaa, 0xff];
  const tolerance = 2;
  if (
    Array.isArray(probe.sampledRgba) &&
    probe.sampledRgba.every((channel, index) => channel === [0, 0, 0, 0xff][index])
  ) {
    throw new Error(
      `non-black WebGPU solid fill regressed to opaque black: sampled ${JSON.stringify(probe.sampledRgba)}`,
    );
  }
  if (
    !Array.isArray(probe.sampledRgba) ||
    probe.sampledRgba.length !== expected.length ||
    probe.sampledRgba.some((actual, index) => Math.abs(actual - expected[index]) > tolerance)
  ) {
    throw new Error(
      `WebGPU center pixel ${JSON.stringify(probe.sampledRgba)} did not match authored fill ${JSON.stringify(expected)}`,
    );
  }
  process.stdout.write(
    `WEBGPU_SOLID_FILL_SAMPLE: rgba=${JSON.stringify(probe.sampledRgba)} expected=${JSON.stringify(expected)} adapter=${probe.adapter}\n`,
  );
  return { ...probe, expectedRgba: expected, tolerance };
}

function waitForServer(url, child, timeoutMs = 30_000) {
  return new Promise((resolve, reject) => {
    const deadline = Date.now() + timeoutMs;
    const poll = () => {
      if (child.exitCode !== null) {
        reject(new Error(`WebGPU content-probe server exited ${child.exitCode}`));
        return;
      }
      const request = http.get(url, (response) => {
        response.resume();
        response.on("end", resolve);
      });
      request.on("error", (error) => {
        if (Date.now() >= deadline) {
          reject(new Error(`WebGPU content-probe server did not listen: ${error}`));
        } else {
          setTimeout(poll, 100);
        }
      });
    };
    poll();
  });
}

async function main() {
  const port = Number(process.env.WEBGPU_CONTENT_PROBE_PORT || "8878");
  const server = spawn(
    process.execPath,
    [path.join(__dirname, "webgpu-candidate-server.cjs"), defaultRepo, String(port), "candidate"],
    { cwd: defaultRepo, stdio: ["ignore", "pipe", "inherit"] },
  );
  server.stdout.pipe(process.stderr);
  let browser;
  try {
    await waitForServer(`http://127.0.0.1:${port}/status`, server);
    const channel = process.env.BROWSER_PLATFORM_CHROMIUM_CHANNEL || "chrome";
    const launchOptions = {
      headless: true,
      args: ["--enable-unsafe-webgpu"],
      channel: channel === "bundled" ? "chromium" : channel,
    };
    browser = await chromium.launch(launchOptions);
    const page = await browser.newPage();
    await page.route("**/favicon.ico", (route) => route.fulfill({ status: 204, body: "" }));
    await page.goto(
      `http://127.0.0.1:${port}/tools/webgpu-renderer-replay/index.html?candidate-worker`,
      { waitUntil: "domcontentloaded", timeout: 180_000 },
    );
    await page.waitForFunction(() => typeof window.runWebgpuReplay === "function");
    await runWebgpuContentProbe(page);
  } finally {
    if (browser) await browser.close().catch(() => undefined);
    if (server.exitCode === null) server.kill();
  }
}

if (require.main === module) {
  main().catch((error) => {
    process.stderr.write(`${error.stack || error}\n`);
    process.exitCode = 1;
  });
}

module.exports = { runWebgpuContentProbe };
