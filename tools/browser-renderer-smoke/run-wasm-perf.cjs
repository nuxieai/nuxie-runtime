"use strict";

const fs = require("node:fs");
const crypto = require("node:crypto");
const { isDeepStrictEqual } = require("node:util");
const { chromium } = require("playwright");
const { assertLoadedFixtureIdentity } = require("./wasm-perf-driver-lib.cjs");

const [baseUrl, configPath, outputPath, sealPath, expectedSealSha256] = process.argv.slice(2);
if (!baseUrl || !configPath || !outputPath || !sealPath || !expectedSealSha256) {
  throw new Error(
    "usage: node run-wasm-perf.cjs <base-url> <config-json> <output-json> <seal-json> <seal-sha256>",
  );
}
const config = JSON.parse(fs.readFileSync(configPath, "utf8"));
const sealBytes = fs.readFileSync(sealPath);
const sealSha256 = crypto.createHash("sha256").update(sealBytes).digest("hex");
if (sealSha256 !== expectedSealSha256) {
  throw new Error(
    `run seal sha256 mismatch: expected=${expectedSealSha256} current=${sealSha256}`,
  );
}
const seal = JSON.parse(sealBytes.toString("utf8"));
if (
  seal.schema !== "nuxie-wasm-perf-seal-v1" ||
  !isDeepStrictEqual(config.provenance, seal.provenance)
) {
  throw new Error("config provenance differs from anchored seal");
}
const browserMode = process.env.BROWSER_RENDERER_BROWSER || "chrome";
const launchOptions = { headless: true };
if (browserMode === "chrome") {
  launchOptions.channel = "chrome";
} else if (browserMode !== "chromium") {
  throw new Error(`unknown BROWSER_RENDERER_BROWSER ${browserMode}`);
}

(async () => {
  const browser = await chromium.launch(launchOptions);
  try {
    const page = await browser.newPage();
    page.on("pageerror", (error) => console.error(`browser page error: ${error.stack || error}`));
    await page.goto(`${baseUrl}wasm-perf.html`, { waitUntil: "networkidle" });
    await page.waitForFunction(
      () => ["ready", "failed"].includes(document.body.dataset.status),
      undefined,
      { timeout: 180_000 },
    );
    const status = await page.getAttribute("body", "data-status");
    if (status !== "ready") {
      throw new Error(`wasm perf page failed: ${await page.textContent("body")}`);
    }

    const fixtures = {};
    const loadedFixtures = {};
    for (const fixture of config.fixtures) {
      console.log(`measuring wasm fixture ${fixture.id}`);
      const loaded = await page.evaluate(async (url) => {
        const response = await fetch(url);
        if (!response.ok) throw new Error(`fixture fetch failed ${response.status} ${url}`);
        const bytes = new Uint8Array(await response.arrayBuffer());
        const digest = await crypto.subtle.digest("SHA-256", bytes);
        const sha256 = Array.from(new Uint8Array(digest), (byte) =>
          byte.toString(16).padStart(2, "0"),
        ).join("");
        return {
          bytes,
          identity: { bytes: bytes.byteLength, sha256 },
        };
      }, fixture.url);
      assertLoadedFixtureIdentity(fixture.id, fixture, loaded.identity);
      loadedFixtures[fixture.id] = loaded.identity;
      const runs = await page.evaluate(
        async ({ bytes, repeat, sampleSeconds, warmups, runs }) =>
          window.measureWasmFixtureRuns({ bytes, repeat, sampleSeconds, warmups, runs }),
        {
          bytes: loaded.bytes,
          repeat: config.repeat,
          sampleSeconds: fixture.sample_seconds,
          warmups: config.warmups,
          runs: config.runs,
        },
      );
      for (const [run, report] of runs.entries()) {
        console.log(
          `wasm ${fixture.id} run ${run + 1}/${config.runs}: ${report.elapsed_ms.toFixed(3)} ms`,
        );
      }
      fixtures[fixture.id] = runs;
    }
    const payload = {
      schema: "nuxie-wasm-perf-browser-raw-v1",
      browser: browserMode,
      browser_version: browser.version(),
      seal_sha256: sealSha256,
      measurement: {
        repeat: config.repeat,
        runs: config.runs,
        warmups: config.warmups,
        fixtures: config.fixtures.map(({ id, bytes, sha256, sample_seconds }) => ({
          id,
          bytes,
          sha256,
          sample_seconds,
        })),
      },
      loaded_fixtures: loadedFixtures,
      fixtures,
    };
    fs.writeFileSync(outputPath, `${JSON.stringify(payload, null, 2)}\n`);
    await page.close();
  } finally {
    await browser.close();
  }
})().catch((error) => {
  console.error(error.stack || String(error));
  process.exitCode = 1;
});
