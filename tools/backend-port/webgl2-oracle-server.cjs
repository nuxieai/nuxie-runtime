"use strict";

const crypto = require("crypto");
const fs = require("fs");
const http = require("http");
const path = require("path");
const { chromium } = require("playwright");

const repo = path.resolve(process.argv[2] || ".");
const port = Number(process.argv[3] || "8878");
const pagePath = path.join(repo, "tools/backend-port/webgl2-source-oracle.html");
const jsPath = path.join(
  repo,
  "target/renderer-webgl2-live-reference-1.91/wasm32-unknown-emscripten/release/renderer-replay.js",
);
const wasmPath = path.join(
  repo,
  "target/renderer-webgl2-live-reference-1.91/wasm32-unknown-emscripten/release/renderer_replay.wasm",
);

function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}

const artifactIdentity = {
  harness_html_sha256: sha256(pagePath),
  replay_js_sha256: sha256(jsPath),
  replay_wasm_sha256: sha256(wasmPath),
};
let browser;
let page;
let browserIdentity;
let renderQueue = Promise.resolve();
let rendersSinceReload = 0;
const pageUrl = `http://127.0.0.1:${port}/tools/backend-port/webgl2-source-oracle.html`;

const mime = new Map([
  [".html", "text/html; charset=utf-8"],
  [".js", "text/javascript; charset=utf-8"],
  [".wasm", "application/wasm"],
  [".rive-stream", "text/plain; charset=utf-8"],
]);

function sendJson(response, status, payload) {
  const bytes = Buffer.from(JSON.stringify(payload));
  response.writeHead(status, {
    "content-type": "application/json",
    "content-length": bytes.length,
  });
  response.end(bytes);
}

function readJson(request) {
  return new Promise((resolve, reject) => {
    const chunks = [];
    let length = 0;
    request.on("data", (chunk) => {
      length += chunk.length;
      if (length > 1024 * 1024) {
        reject(new Error("request body exceeds 1 MiB"));
        request.destroy();
        return;
      }
      chunks.push(chunk);
    });
    request.on("end", () => {
      try {
        resolve(JSON.parse(Buffer.concat(chunks).toString("utf8")));
      } catch (error) {
        reject(error);
      }
    });
    request.on("error", reject);
  });
}

function withinRepo(file) {
  const resolved = path.resolve(file);
  if (resolved !== repo && !resolved.startsWith(`${repo}${path.sep}`)) {
    throw new Error(`path escapes repository: ${file}`);
  }
  return resolved;
}

async function render(payload) {
  if (!page) throw new Error("browser source oracle is not ready");
  if (!payload || payload.backend !== "ffi-webgl2") {
    throw new Error("source oracle only accepts backend ffi-webgl2");
  }
  if (!["msaa", "clockwise-atomic"].includes(payload.mode)) {
    throw new Error(`unsupported mode ${payload.mode}`);
  }
  const streamPath = withinRepo(payload.stream);
  const outputPath = withinRepo(payload.output);
  const stream = fs.readFileSync(streamPath, "utf8");
  if (rendersSinceReload >= 16) {
    await loadOraclePage();
  }
  const browserErrors = [];
  const onPageError = (error) => browserErrors.push(error.stack || String(error));
  page.on("pageerror", onPageError);
  try {
    const result = await page.evaluate(
      async ({ stream, mode, frame }) =>
        window.runWebgl2Oracle({ stream, mode, frame }),
      { stream, mode: payload.mode, frame: payload.frame || 0 },
    );
    if (result.exitCode !== 0 || result.stderr.length !== 0) {
      throw new Error(
        `browser replay failed: exit=${result.exitCode} stderr=${result.stderr.join("\n")}`,
      );
    }
    if (browserErrors.length !== 0) {
      throw new Error(`browser page errors:\n${browserErrors.join("\n")}`);
    }
    const png = Buffer.from(result.pngBase64, "base64");
    if (png.length !== result.pngBytes || sha256Buffer(png) !== result.sha256) {
      throw new Error("browser PNG transfer identity mismatch");
    }
    fs.mkdirSync(path.dirname(outputPath), { recursive: true });
    fs.writeFileSync(outputPath, png);
    rendersSinceReload += 1;
    return {
      ...result,
      pngBase64: undefined,
      artifacts: artifactIdentity,
      browser: browserIdentity,
    };
  } finally {
    page.off("pageerror", onPageError);
  }
}

async function loadOraclePage() {
  await page.goto(pageUrl, { waitUntil: "networkidle" });
  await page.evaluate(() => window.webgl2OracleReady);
  rendersSinceReload = 0;
}

function sha256Buffer(bytes) {
  return crypto.createHash("sha256").update(bytes).digest("hex");
}

const server = http.createServer(async (request, response) => {
  const requestUrl = new URL(request.url, `http://127.0.0.1:${port}`);
  if (request.method === "GET" && requestUrl.pathname === "/status") {
    sendJson(response, page ? 200 : 503, {
      ready: Boolean(page),
      artifacts: artifactIdentity,
      browser: browserIdentity,
    });
    return;
  }
  if (request.method === "POST" && requestUrl.pathname === "/render") {
    try {
      const payload = await readJson(request);
      const run = () => render(payload);
      const resultPromise = renderQueue.then(run, run);
      renderQueue = resultPromise.then(
        () => undefined,
        () => undefined,
      );
      sendJson(response, 200, await resultPromise);
    } catch (error) {
      sendJson(response, 500, { error: error.stack || String(error) });
    }
    return;
  }
  if (request.method !== "GET") {
    response.writeHead(405).end();
    return;
  }
  try {
    const relative = decodeURIComponent(requestUrl.pathname).replace(/^\/+/, "");
    const file = withinRepo(path.join(repo, relative || "index.html"));
    const bytes = fs.readFileSync(file);
    response.writeHead(200, {
      "content-type": mime.get(path.extname(file)) || "application/octet-stream",
      "content-length": bytes.length,
    });
    response.end(bytes);
  } catch (_) {
    response.writeHead(404).end();
  }
});

async function close() {
  server.close();
  if (browser) await browser.close();
}

for (const signal of ["SIGINT", "SIGTERM"]) {
  process.on(signal, () => close().finally(() => process.exit(0)));
}

(async () => {
  browser = await chromium.launch({ channel: "chrome", headless: true });
  page = await browser.newPage();
  await loadOraclePage();
  browserIdentity = {
    name: "chrome",
    version: browser.version(),
    user_agent: await page.evaluate(() => navigator.userAgent),
  };
  console.log(
    JSON.stringify({ ready: true, port, artifacts: artifactIdentity, browser: browserIdentity }),
  );
})().catch((error) => {
  console.error(error.stack || String(error));
  process.exitCode = 1;
});

server.listen(port, "127.0.0.1");
