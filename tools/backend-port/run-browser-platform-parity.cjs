"use strict";

const crypto = require("node:crypto");
const fs = require("node:fs");
const http = require("node:http");
const path = require("node:path");
const { spawn, spawnSync } = require("node:child_process");
const { chromium, firefox } = require("./browser-platform/node_modules/playwright");

const repo = path.resolve(__dirname, "../..");
const [backend, browserName] = process.argv.slice(2);
if (!new Set(["webgpu", "webgl2"]).has(backend)) {
  throw new Error("usage: node run-browser-platform-parity.cjs <webgpu|webgl2> <chromium|firefox>");
}
if (!new Set(["chromium", "firefox"]).has(browserName)) {
  throw new Error("usage: node run-browser-platform-parity.cjs <webgpu|webgl2> <chromium|firefox>");
}

const configuration = backend === "webgpu"
  ? {
      server: "tools/backend-port/webgpu-candidate-server.cjs",
      sourcePort: 8881,
      candidatePort: 8878,
      sourceClient: "tools/backend-port/webgpu-source-replay-client.py",
      candidateClient: "tools/backend-port/webgpu-candidate-replay-client.py",
      sourceBackend: "ffi-dawn",
      candidateBackend: "rust-webgpu-exact",
      sourceEndpoint: "WEBGPU_SOURCE_ENDPOINT",
      candidateEndpoint: "WEBGPU_CANDIDATE_ENDPOINT",
    }
  : {
      server: "tools/backend-port/webgl2-browser-server.cjs",
      sourcePort: 8878,
      candidatePort: 8879,
      sourceClient: "tools/backend-port/webgl2-replay-client.py",
      candidateClient: "tools/backend-port/webgl2-candidate-replay-client.py",
      sourceBackend: "ffi-webgl2",
      candidateBackend: "rust-webgl2-exact",
      sourceEndpoint: "WEBGL2_ORACLE_ENDPOINT",
      candidateEndpoint: "WEBGL2_CANDIDATE_ENDPOINT",
    };

const outputRoot = path.join(
  repo,
  process.env.BROWSER_PLATFORM_OUTPUT_DIR ||
    `target/backend-port/platform/${process.platform}-${browserName}-${backend}`,
);
const corpusOutput = path.join(outputRoot, "corpus");
const logPath = path.join(outputRoot, "corpus.log");
const evidencePath = path.join(outputRoot, "evidence.json");
const childProcesses = [];
let browser;

function sha256(bytes) {
  return crypto.createHash("sha256").update(bytes).digest("hex");
}

function launch(command, args, options = {}) {
  const child = spawn(command, args, {
    cwd: repo,
    stdio: ["ignore", "pipe", "pipe"],
    ...options,
  });
  childProcesses.push(child);
  child.stdout.on("data", (bytes) => process.stderr.write(`[${path.basename(command)}] ${bytes}`));
  child.stderr.on("data", (bytes) => process.stderr.write(`[${path.basename(command)}] ${bytes}`));
  return child;
}

function getJson(url) {
  return new Promise((resolve, reject) => {
    const request = http.get(url, { timeout: 5_000 }, (response) => {
      const chunks = [];
      response.on("data", (chunk) => chunks.push(chunk));
      response.on("end", () => {
        if (response.statusCode !== 200) {
          reject(new Error(`${url} returned ${response.statusCode}`));
          return;
        }
        try {
          resolve(JSON.parse(Buffer.concat(chunks).toString("utf8")));
        } catch (error) {
          reject(error);
        }
      });
    });
    request.on("timeout", () => request.destroy(new Error(`${url} timed out`)));
    request.on("error", reject);
  });
}

async function waitForBroker(port, child, timeoutMs = 30_000) {
  const deadline = Date.now() + timeoutMs;
  let lastError;
  while (Date.now() < deadline) {
    if (child.exitCode !== null) {
      throw new Error(`browser broker on port ${port} exited ${child.exitCode}`);
    }
    try {
      await new Promise((resolve, reject) => {
        const request = http.get(`http://127.0.0.1:${port}/status`, (response) => {
          response.resume();
          response.on("end", resolve);
        });
        request.on("error", reject);
      });
      return;
    } catch (error) {
      lastError = error;
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error(`browser broker on port ${port} did not listen: ${lastError}`);
}

async function waitForStatus(port, child, timeoutMs = 180_000) {
  const deadline = Date.now() + timeoutMs;
  let lastError;
  while (Date.now() < deadline) {
    if (child.exitCode !== null) {
      throw new Error(`browser broker on port ${port} exited ${child.exitCode}`);
    }
    try {
      const status = await getJson(`http://127.0.0.1:${port}/status`);
      if (status.ready) return status;
    } catch (error) {
      lastError = error;
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error(`browser broker on port ${port} was not ready: ${lastError || "no worker"}`);
}

function pythonExecutable() {
  if (process.env.PYTHON) return process.env.PYTHON;
  for (const candidate of process.platform === "win32" ? ["python", "python3"] : ["python3", "python"]) {
    const probe = spawnSync(candidate, ["--version"], { stdio: "ignore" });
    if (probe.status === 0) return candidate;
  }
  throw new Error("no Python executable found for replay clients");
}

function replayCommand(client, label) {
  const absoluteClient = path.join(repo, client);
  if (process.platform !== "win32") return absoluteClient;
  const launcher = path.join(outputRoot, `${label}-replay.cmd`);
  const python = pythonExecutable();
  fs.writeFileSync(launcher, `@\"${python}\" \"${absoluteClient}\" %*\r\n`);
  return launcher;
}

function runCorpus(sourceReplay, candidateReplay) {
  const args = [
    "run", "--quiet", "--locked", "-p", "pixel-compare", "--bin", "corpus-r", "--",
    "--manifest", path.join(repo, "corpus-r.toml"),
    "--replay", candidateReplay,
    "--backend", configuration.candidateBackend,
    "--reference-replay", sourceReplay,
    "--reference-backend", configuration.sourceBackend,
    "--output-dir", corpusOutput,
    "--jobs", "1",
    "--replay-timeout-seconds", process.env.BROWSER_PLATFORM_REPLAY_TIMEOUT_SECONDS || "300",
  ];
  for (const entry of (process.env.BROWSER_PLATFORM_ENTRIES || "").split(",").filter(Boolean)) {
    args.push("--entry", entry);
  }
  return new Promise((resolve, reject) => {
    const child = spawn(process.env.CARGO || "cargo", args, {
      cwd: repo,
      env: {
        ...process.env,
        [configuration.sourceEndpoint]: `http://127.0.0.1:${configuration.sourcePort}`,
        [configuration.candidateEndpoint]: `http://127.0.0.1:${configuration.candidatePort}`,
      },
      stdio: ["ignore", "pipe", "pipe"],
    });
    childProcesses.push(child);
    const chunks = [];
    for (const stream of [child.stdout, child.stderr]) {
      stream.on("data", (bytes) => {
        chunks.push(Buffer.from(bytes));
        process.stdout.write(bytes);
      });
    }
    child.on("error", reject);
    child.on("close", (code) => {
      const log = Buffer.concat(chunks);
      fs.writeFileSync(logPath, log);
      if (code !== 0) reject(new Error(`corpus-r exited ${code}`));
      else resolve(log);
    });
  });
}

async function cleanup() {
  if (browser) await browser.close().catch(() => undefined);
  for (const child of childProcesses) {
    if (child.exitCode === null) child.kill();
  }
}

async function main() {
  fs.mkdirSync(outputRoot, { recursive: true });
  const sourceBroker = launch("node", [configuration.server, repo, String(configuration.sourcePort), "source"]);
  const candidateBroker = launch("node", [configuration.server, repo, String(configuration.candidatePort), "candidate"]);
  await Promise.all([
    waitForBroker(configuration.sourcePort, sourceBroker),
    waitForBroker(configuration.candidatePort, candidateBroker),
  ]);

  const browserType = browserName === "firefox" ? firefox : chromium;
  const recordedDefaultHeadless = !(backend === "webgpu" && browserName === "firefox");
  const launchOptions = {
    headless: process.env.BROWSER_PLATFORM_HEADLESS === undefined
      ? recordedDefaultHeadless
      : process.env.BROWSER_PLATFORM_HEADLESS === "1",
  };
  if (browserName === "chromium") {
    const channel = process.env.BROWSER_PLATFORM_CHROMIUM_CHANNEL || "chrome";
    if (channel !== "bundled") launchOptions.channel = channel;
    if (backend === "webgpu") launchOptions.args = ["--enable-unsafe-webgpu"];
  } else if (backend === "webgpu") {
    launchOptions.firefoxUserPrefs = {
      "dom.webgpu.enabled": true,
      "gfx.webgpu.ignore-blocklist": true,
    };
  }
  browser = await browserType.launch(launchOptions);
  const context = await browser.newContext();
  const pageErrors = [];
  const openWorker = async (port, role) => {
    const page = await context.newPage();
    await page.route("**/favicon.ico", (route) => route.fulfill({ status: 204, body: "" }));
    page.on("pageerror", (error) => pageErrors.push(`${role}: ${error.stack || error}`));
    page.on("console", (message) => {
      if (message.type() === "error") pageErrors.push(`${role} console: ${message.text()}`);
    });
    const workerUrl = `http://127.0.0.1:${port}/${backend === "webgpu"
        ? role === "source"
          ? "tools/backend-port/webgpu-source-oracle.html?source-worker"
          : "tools/webgpu-renderer-replay/index.html?candidate-worker"
        : role === "source"
          ? "tools/backend-port/webgl2-source-oracle.html?source-worker"
          : "tools/webgl2-renderer-replay/index.html?candidate-worker"}`;
    await page.goto(workerUrl, { waitUntil: "domcontentloaded", timeout: 180_000 });
    return page;
  };
  await Promise.all([
    openWorker(configuration.sourcePort, "source"),
    openWorker(configuration.candidatePort, "candidate"),
  ]);
  const [sourceStatus, candidateStatus] = await Promise.all([
    waitForStatus(configuration.sourcePort, sourceBroker),
    waitForStatus(configuration.candidatePort, candidateBroker),
  ]);
  if (sourceStatus.browser.user_agent !== candidateStatus.browser.user_agent) {
    throw new Error("source and candidate workers do not share one browser identity");
  }
  if (pageErrors.length !== 0) throw new Error(`browser worker errors:\n${pageErrors.join("\n")}`);

  const log = await runCorpus(
    replayCommand(configuration.sourceClient, "source"),
    replayCommand(configuration.candidateClient, "candidate"),
  );
  if (pageErrors.length !== 0) throw new Error(`browser worker errors:\n${pageErrors.join("\n")}`);
  const text = log.toString("utf8");
  const summary = text.match(/renderer-corpus exact=(\d+) byte-exact=(\d+) diverges=(\d+) gated=(\d+) total=(\d+)/);
  if (!summary) throw new Error("corpus log omitted its final summary");
  const counts = Object.fromEntries(
    ["exact", "byte_exact", "diverges", "gated", "total"].map((key, index) => [key, Number(summary[index + 1])]),
  );
  const expectedTotal = process.env.BROWSER_PLATFORM_ENTRIES ?
    process.env.BROWSER_PLATFORM_ENTRIES.split(",").filter(Boolean).length : 1469;
  if (counts.exact !== expectedTotal || counts.diverges !== 0 || counts.gated !== 0 || counts.total !== expectedTotal) {
    throw new Error(`browser parity did not close exactly: ${summary[0]}`);
  }
  const adapters = [...new Set(text.match(/^adapter=.*$/gm) || [])].map((line) => line.slice(8));
  if (adapters.length !== 1 || adapters[0].length === 0) {
    throw new Error(`corpus did not retain one shared nonempty adapter: ${JSON.stringify(adapters)}`);
  }
  const evidence = {
    schema: "nuxie-backend-port-browser-platform-v1",
    platform: process.platform,
    backend,
    browser: browserName,
    browser_identity: sourceStatus.browser,
    adapter: adapters[0],
    source_artifacts: sourceStatus.artifacts,
    candidate_artifacts: candidateStatus.artifacts,
    counts,
    corpus_log: path.relative(repo, logPath).replaceAll(path.sep, "/"),
    corpus_log_sha256: sha256(log),
    no_renderer_fallback: true,
  };
  fs.writeFileSync(evidencePath, `${JSON.stringify(evidence, null, 2)}\n`);
  process.stdout.write(`browser-platform-evidence=${evidencePath}\n`);
}

for (const signal of ["SIGINT", "SIGTERM"]) {
  process.on(signal, () => cleanup().finally(() => process.exit(128)));
}

main()
  .catch((error) => {
    console.error(error.stack || String(error));
    process.exitCode = 1;
  })
  .finally(cleanup);
