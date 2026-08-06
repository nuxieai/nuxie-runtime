"use strict";

const fs = require("node:fs");
const crypto = require("node:crypto");
const { isDeepStrictEqual } = require("node:util");
const { chromium } = require("playwright");
const {
  assertLoadedArtifactIdentity,
  assertLoadedFixtureIdentity,
} = require("./wasm-perf-driver-lib.cjs");

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
const browserArtifactNames = [
  "wasm_perf_html",
  "wasm_perf_driver_js",
  "wasm_bindgen_js",
  "wasm",
];
const expectedArtifacts = Object.fromEntries(
  browserArtifactNames.map((name) => [
    name,
    {
      bytes: seal.provenance.artifacts[name].bytes,
      sha256: seal.provenance.artifacts[name].sha256,
    },
  ]),
);
const sealedHarness = Object.fromEntries(
  ["wasm_perf_html", "wasm_perf_driver_js"].map((name) => {
    const bytes = fs.readFileSync(seal.provenance.artifacts[name].path);
    const identity = {
      bytes: bytes.byteLength,
      sha256: crypto.createHash("sha256").update(bytes).digest("hex"),
    };
    assertLoadedArtifactIdentity(name, expectedArtifacts[name], identity);
    return [name, { bytes, identity }];
  }),
);
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
    const harnessUrls = {
      wasm_perf_html: `${baseUrl}sealed/${expectedArtifacts.wasm_perf_html.sha256}.html`,
      wasm_perf_driver_js: `${baseUrl}sealed/${expectedArtifacts.wasm_perf_driver_js.sha256}.cjs`,
    };
    await page.route(harnessUrls.wasm_perf_html, (route) =>
      route.fulfill({
        status: 200,
        contentType: "text/html; charset=utf-8",
        body: sealedHarness.wasm_perf_html.bytes,
      }),
    );
    await page.route(harnessUrls.wasm_perf_driver_js, (route) =>
      route.fulfill({
        status: 200,
        contentType: "text/javascript; charset=utf-8",
        body: sealedHarness.wasm_perf_driver_js.bytes,
      }),
    );
    await page.goto(harnessUrls.wasm_perf_html, { waitUntil: "domcontentloaded" });
    await page.addScriptTag({ url: harnessUrls.wasm_perf_driver_js });
    await page.evaluate(() => {
      if (!window.WasmPerfDriver) {
        throw new Error("sealed wasm perf driver did not install");
      }
      document.body.dataset.status = "ready";
    });
    const status = await page.getAttribute("body", "data-status");
    if (status !== "ready") {
      throw new Error(`wasm perf page failed: ${await page.textContent("body")}`);
    }

    const loadedRuntimeArtifacts = await page.evaluate(
      async ({ jsUrl, wasmUrl, expectedArtifacts: expected }) =>
        window.WasmPerfDriver.installWasmPerfArtifacts({
          jsUrl,
          wasmUrl,
          expectedArtifacts: expected,
        }),
      {
        jsUrl: `${baseUrl}pkg/browser_renderer_smoke.js`,
        wasmUrl: `${baseUrl}pkg/browser_renderer_smoke_bg.wasm`,
        expectedArtifacts,
      },
    );
    const loadedArtifacts = {
      wasm_perf_html: sealedHarness.wasm_perf_html.identity,
      wasm_perf_driver_js: sealedHarness.wasm_perf_driver_js.identity,
      ...loadedRuntimeArtifacts,
    };
    for (const name of browserArtifactNames) {
      assertLoadedArtifactIdentity(name, expectedArtifacts[name], loadedArtifacts[name]);
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
      loaded_artifacts: loadedArtifacts,
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
