"use strict";

function checkedElapsed(start, end, label) {
  if (!Number.isFinite(start) || !Number.isFinite(end)) {
    throw new Error(`${label} clock reading was not finite`);
  }
  if (end < start) {
    throw new Error(`${label} monotonic clock moved backwards: ${start} -> ${end}`);
  }
  return end - start;
}

async function withRunner(createRunner, pass, operation) {
  const runner = await createRunner(pass);
  try {
    return await operation(runner);
  } finally {
    runner.free();
  }
}

function readWorkloadIdentity(runner) {
  const identity = JSON.parse(runner.workloadIdentityJson());
  const expectedKeys = [
    "default_state_machine_id",
    "scene_kind",
    "view_model_initialization",
  ];
  assertExactKeys(identity, expectedKeys, "workload identity");
  if (!["static", "state_machine"].includes(identity.scene_kind)) {
    throw new Error(`invalid workload scene_kind ${JSON.stringify(identity.scene_kind)}`);
  }
  if (
    identity.default_state_machine_id !== null &&
    (!Number.isInteger(identity.default_state_machine_id) || identity.default_state_machine_id < 0)
  ) {
    throw new Error("workload default_state_machine_id must be null or a non-negative integer");
  }
  if (identity.scene_kind === "static" && identity.default_state_machine_id !== null) {
    throw new Error("static workload must not identify a default state machine");
  }
  if (identity.scene_kind === "state_machine" && identity.default_state_machine_id === null) {
    throw new Error("state-machine workload must identify its authored default state machine");
  }
  if (!["none", "schema-default"].includes(identity.view_model_initialization)) {
    throw new Error(
      `invalid workload view_model_initialization ${JSON.stringify(identity.view_model_initialization)}`,
    );
  }
  return identity;
}

function assertExactKeys(value, expectedKeys, label) {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${label} must be an object`);
  }
  const actualKeys = Object.keys(value).sort();
  const sortedExpected = [...expectedKeys].sort();
  if (JSON.stringify(actualKeys) !== JSON.stringify(sortedExpected)) {
    throw new Error(`${label} keys must be exactly ${sortedExpected.join(", ")}`);
  }
}

function assertLoadedFixtureIdentity(fixtureId, expected, loaded) {
  assertExactKeys(loaded, ["bytes", "sha256"], "browser loaded fixture identity");
  const expectedIdentity = {
    bytes: expected.bytes,
    sha256: expected.sha256,
  };
  if (loaded.bytes !== expectedIdentity.bytes || loaded.sha256 !== expectedIdentity.sha256) {
    throw new Error(
      `browser loaded fixture identity mismatch for ${fixtureId}: ` +
        `expected=${JSON.stringify(expectedIdentity)} loaded=${JSON.stringify(loaded)}`,
    );
  }
}

function assertLoadedArtifactIdentity(artifactName, expected, loaded) {
  assertExactKeys(loaded, ["bytes", "sha256"], "browser loaded artifact identity");
  const expectedIdentity = {
    bytes: expected.bytes,
    sha256: expected.sha256,
  };
  if (loaded.bytes !== expectedIdentity.bytes || loaded.sha256 !== expectedIdentity.sha256) {
    throw new Error(
      `browser loaded artifact identity mismatch for ${artifactName}: ` +
        `expected=${JSON.stringify(expectedIdentity)} loaded=${JSON.stringify(loaded)}`,
    );
  }
}

async function artifactIdentity(bytes) {
  const digest = await globalThis.crypto.subtle.digest("SHA-256", bytes);
  return {
    bytes: bytes.byteLength,
    sha256: Array.from(new Uint8Array(digest), (byte) =>
      byte.toString(16).padStart(2, "0"),
    ).join(""),
  };
}

async function fetchArtifactBytes(url) {
  const response = await fetch(url, { cache: "no-store" });
  if (!response.ok) {
    throw new Error(`artifact fetch failed ${response.status} ${url}`);
  }
  return new Uint8Array(await response.arrayBuffer());
}

async function installWasmPerfArtifacts({ jsUrl, wasmUrl, expectedArtifacts }) {
  const [jsBytes, wasmBytes] = await Promise.all([
    fetchArtifactBytes(jsUrl),
    fetchArtifactBytes(wasmUrl),
  ]);
  const loadedArtifacts = {
    wasm_bindgen_js: await artifactIdentity(jsBytes),
    wasm: await artifactIdentity(wasmBytes),
  };
  assertLoadedArtifactIdentity(
    "wasm_bindgen_js",
    expectedArtifacts.wasm_bindgen_js,
    loadedArtifacts.wasm_bindgen_js,
  );
  assertLoadedArtifactIdentity("wasm", expectedArtifacts.wasm, loadedArtifacts.wasm);

  // Import and initialize the exact buffers whose identities were checked
  // above. No second URL read can substitute different bytes at execution.
  const moduleUrl = URL.createObjectURL(new Blob([jsBytes], { type: "text/javascript" }));
  let module;
  try {
    module = await import(moduleUrl);
  } finally {
    URL.revokeObjectURL(moduleUrl);
  }
  await module.default({ module_or_path: wasmBytes });
  const { WasmPerfRunner } = module;
  globalThis.measureWasmFixture = async ({ bytes, repeat, sampleSeconds }) =>
    measureFixture({
      createRunner: async () => new WasmPerfRunner(bytes),
      now: () => performance.now(),
      repeat,
      sampleSeconds,
    });
  globalThis.measureWasmFixtureRuns = async ({ bytes, repeat, sampleSeconds, warmups, runs }) =>
    measureRuns({
      measure: () => globalThis.measureWasmFixture({ bytes, repeat, sampleSeconds }),
      warmups,
      runs,
    });
  return loadedArtifacts;
}

async function measureFixture({ createRunner, now, repeat, sampleSeconds }) {
  if (!Number.isInteger(repeat) || repeat <= 0) {
    throw new Error("repeat must be a positive integer");
  }
  if (!Number.isFinite(sampleSeconds) || sampleSeconds < 0) {
    throw new Error("sampleSeconds must be a finite non-negative number");
  }

  const total = await withRunner(createRunner, "total", async (runner) => {
    const workloadIdentity = readWorkloadIdentity(runner);
    const start = now();
    for (let index = 0; index < repeat; index += 1) {
      runner.advanceAndDraw(sampleSeconds);
    }
    return {
      elapsedMs: checkedElapsed(start, now(), "total pass"),
      workloadIdentity,
    };
  });

  const phases = await withRunner(createRunner, "phases", async (runner) => {
    const workloadIdentity = readWorkloadIdentity(runner);
    let advanceMs = 0;
    let drawMs = 0;
    const phaseStart = now();
    for (let index = 0; index < repeat; index += 1) {
      let start = now();
      runner.advance(sampleSeconds);
      advanceMs += checkedElapsed(start, now(), "advance");
      start = now();
      runner.draw();
      drawMs += checkedElapsed(start, now(), "draw");
    }
    const phaseElapsedMs = checkedElapsed(phaseStart, now(), "phase pass");
    return { advanceMs, drawMs, phaseElapsedMs, workloadIdentity };
  });

  if (JSON.stringify(total.workloadIdentity) !== JSON.stringify(phases.workloadIdentity)) {
    throw new Error("fresh runner workload identity mismatch between total and phase passes");
  }

  const accountedMs = phases.advanceMs + phases.drawMs;
  return {
    schema: "rive-golden-benchmark-v1",
    elapsed_ms: total.elapsedMs,
    total_ms: total.elapsedMs,
    advance_ms: phases.advanceMs,
    input_ms: 0,
    prepare_ms: 0,
    draw_ms: phases.drawMs,
    accounted_ms: accountedMs,
    bookkeeping_ms: Math.max(phases.phaseElapsedMs - accountedMs, 0),
    segments: repeat,
    workload_identity: total.workloadIdentity,
  };
}

async function measureRuns({ measure, warmups, runs }) {
  if (!Number.isInteger(warmups) || warmups < 0) {
    throw new Error("warmups must be a non-negative integer");
  }
  if (!Number.isInteger(runs) || runs < 2) {
    throw new Error("runs must be an integer of at least 2");
  }
  for (let index = 0; index < warmups; index += 1) {
    await measure();
  }
  const reports = [];
  for (let index = 0; index < runs; index += 1) {
    reports.push(await measure());
  }
  return reports;
}

const api = {
  assertLoadedArtifactIdentity,
  assertLoadedFixtureIdentity,
  checkedElapsed,
  installWasmPerfArtifacts,
  measureFixture,
  measureRuns,
};
if (typeof module !== "undefined" && module.exports) {
  module.exports = api;
}
if (typeof globalThis !== "undefined") {
  globalThis.WasmPerfDriver = api;
}
