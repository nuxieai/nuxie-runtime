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

async function measureFixture({ createRunner, now, repeat, sampleSeconds }) {
  if (!Number.isInteger(repeat) || repeat <= 0) {
    throw new Error("repeat must be a positive integer");
  }
  if (!Number.isFinite(sampleSeconds) || sampleSeconds < 0) {
    throw new Error("sampleSeconds must be a finite non-negative number");
  }

  const elapsedMs = await withRunner(createRunner, "total", async (runner) => {
    const start = now();
    for (let index = 0; index < repeat; index += 1) {
      runner.advanceAndDraw(sampleSeconds);
    }
    return checkedElapsed(start, now(), "total pass");
  });

  const phases = await withRunner(createRunner, "phases", async (runner) => {
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
    return { advanceMs, drawMs, phaseElapsedMs };
  });

  const accountedMs = phases.advanceMs + phases.drawMs;
  return {
    schema: "rive-golden-benchmark-v1",
    elapsed_ms: elapsedMs,
    total_ms: elapsedMs,
    advance_ms: phases.advanceMs,
    input_ms: 0,
    prepare_ms: 0,
    draw_ms: phases.drawMs,
    accounted_ms: accountedMs,
    bookkeeping_ms: Math.max(phases.phaseElapsedMs - accountedMs, 0),
    segments: repeat,
  };
}

const api = { checkedElapsed, measureFixture };
if (typeof module !== "undefined" && module.exports) {
  module.exports = api;
}
if (typeof globalThis !== "undefined") {
  globalThis.WasmPerfDriver = api;
}
