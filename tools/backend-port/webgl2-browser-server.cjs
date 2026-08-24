"use strict";

const crypto = require("crypto");
const fs = require("fs");
const http = require("http");
const path = require("path");

const repo = path.resolve(process.argv[2] || ".");
const port = Number(process.argv[3] || "8878");
const role = process.argv[4] || "source";
if (!new Set(["source", "candidate"]).has(role)) {
  throw new Error(`unsupported WebGL2 browser replay role ${role}`);
}
const isSource = role === "source";
const pagePath = path.join(
  repo,
  isSource
    ? "tools/backend-port/webgl2-source-oracle.html"
    : "tools/webgl2-renderer-replay/index.html",
);
const artifactDir = isSource
  ? "target/renderer-webgl2-live-reference-1.91/wasm32-unknown-emscripten/release"
  : "tools/webgl2-renderer-replay/pkg";
const jsPath = path.join(
  repo,
  artifactDir,
  isSource ? "renderer-replay.js" : "webgl2_renderer_replay.js",
);
const wasmPath = path.join(
  repo,
  artifactDir,
  isSource ? "renderer_replay.wasm" : "webgl2_renderer_replay_bg.wasm",
);
const acceptedBackend = isSource ? "ffi-webgl2" : "rust-webgl2-exact";
const workerPath = isSource
  ? "/tools/backend-port/webgl2-source-oracle.html?source-worker"
  : "/tools/webgl2-renderer-replay/index.html?candidate-worker";

function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}

function sha256Buffer(bytes) {
  return crypto.createHash("sha256").update(bytes).digest("hex");
}

const artifactIdentity = {
  harness_html_sha256: sha256(pagePath),
  replay_js_sha256: sha256(jsPath),
  replay_wasm_sha256: sha256(wasmPath),
};
let browserIdentity;
let nextJobId = 1;
let pendingJob;
let assignedJob;
let renderQueue = Promise.resolve();

const mime = new Map([
  [".html", "text/html; charset=utf-8"],
  [".js", "text/javascript; charset=utf-8"],
  [".wasm", "application/wasm"],
]);

function sendJson(response, status, payload) {
  const bytes = Buffer.from(JSON.stringify(payload));
  response.writeHead(status, {
    "content-type": "application/json",
    "content-length": bytes.length,
  });
  response.end(bytes);
}

function readJson(request, maximumBytes = 32 * 1024 * 1024) {
  return new Promise((resolve, reject) => {
    const chunks = [];
    let length = 0;
    request.on("data", (chunk) => {
      length += chunk.length;
      if (length > maximumBytes) {
        reject(new Error(`request body exceeds ${maximumBytes} bytes`));
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

function parseReplayLine(stdout) {
  const replay = stdout.find((line) => line.startsWith("backend="));
  if (!replay) throw new Error("source replay did not report backend metadata");
  const fields = Object.fromEntries(replay.split(/\s+/).map((part) => part.split("=", 2)));
  const dimensions = fields.size?.match(/^(\d+)x(\d+)$/);
  if (!dimensions) throw new Error(`source replay reported invalid size ${fields.size}`);
  return { width: Number(dimensions[1]), height: Number(dimensions[2]) };
}

async function render(payload) {
  if (!browserIdentity) throw new Error("browser WebGL2 worker is not ready");
  if (!payload || payload.backend !== acceptedBackend) {
    throw new Error(`${role} worker only accepts backend ${acceptedBackend}`);
  }
  if (!new Set(["msaa", "clockwise-atomic"]).has(payload.mode)) {
    throw new Error(`unsupported mode ${payload.mode}`);
  }
  if (pendingJob || assignedJob) throw new Error("browser worker already owns a render");
  const streamPath = withinRepo(payload.stream);
  const outputPath = withinRepo(payload.output);
  const result = await new Promise((resolve, reject) => {
    pendingJob = {
      id: nextJobId++,
      stream: fs.readFileSync(streamPath, "utf8"),
      mode: payload.mode,
      frame: payload.frame || 0,
      resolve,
      reject,
    };
  });
  if (isSource && (result.exitCode !== 0 || result.stderr.length !== 0)) {
    throw new Error(
      `browser replay failed: exit=${result.exitCode} stderr=${result.stderr.join("\n")}`,
    );
  }
  const png = Buffer.from(result.pngBase64, "base64");
  if (png.length !== result.pngBytes || sha256Buffer(png) !== result.sha256) {
    throw new Error("browser PNG transfer identity mismatch");
  }
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, png);
  const normalized = isSource
    ? {
        adapter:
          result.stdout.find((line) => line.startsWith("adapter="))?.slice(8) || "",
        ...parseReplayLine(result.stdout),
      }
    : { adapter: result.adapter, width: result.width, height: result.height };
  return {
    ...result,
    pngBase64: undefined,
    ...normalized,
    artifacts: artifactIdentity,
    browser: browserIdentity,
  };
}

const server = http.createServer(async (request, response) => {
  const requestUrl = new URL(request.url, `http://127.0.0.1:${port}`);
  if (request.method === "GET" && requestUrl.pathname === "/status") {
    sendJson(response, browserIdentity ? 200 : 503, {
      ready: Boolean(browserIdentity),
      worker_url: `http://127.0.0.1:${port}${workerPath}`,
      artifacts: artifactIdentity,
      browser: browserIdentity,
    });
    return;
  }
  if (request.method === "POST" && requestUrl.pathname === "/render") {
    try {
      const payload = await readJson(request, 1024 * 1024);
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
  if (request.method === "POST" && requestUrl.pathname === "/worker/register") {
    try {
      const identity = await readJson(request, 64 * 1024);
      if (!identity.webgl2) throw new Error("browser worker does not expose WebGL2");
      browserIdentity = identity;
      sendJson(response, 200, { ready: true });
    } catch (error) {
      sendJson(response, 500, { error: error.stack || String(error) });
    }
    return;
  }
  if (request.method === "GET" && requestUrl.pathname === "/worker/next") {
    if (!pendingJob) {
      response.writeHead(204).end();
      return;
    }
    assignedJob = pendingJob;
    pendingJob = undefined;
    sendJson(response, 200, {
      id: assignedJob.id,
      stream: assignedJob.stream,
      mode: assignedJob.mode,
      frame: assignedJob.frame,
    });
    return;
  }
  if (request.method === "POST" && requestUrl.pathname === "/worker/complete") {
    try {
      const completion = await readJson(request);
      if (!assignedJob || completion.id !== assignedJob.id) {
        throw new Error(`completion does not own active job ${completion.id}`);
      }
      const job = assignedJob;
      assignedJob = undefined;
      if (completion.error) job.reject(new Error(completion.error));
      else job.resolve(completion.result);
      sendJson(response, 200, { accepted: true });
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

server.listen(port, "127.0.0.1", () => {
  console.log(JSON.stringify({ ready: true, port, role, artifacts: artifactIdentity }));
});
