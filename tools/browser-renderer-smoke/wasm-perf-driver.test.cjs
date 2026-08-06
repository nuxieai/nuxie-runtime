const test = require("node:test");
const assert = require("node:assert/strict");

const { measureFixture, measureRuns } = require("./wasm-perf-driver-lib.cjs");

test("measures fresh total and phase passes with setup outside the clock", async () => {
  const events = [];
  const clockValues = [10, 14, 20, 21, 23, 23, 25, 25, 27, 27, 30, 30];
  const createRunner = async (pass) => {
    events.push(`create:${pass}`);
    return {
      workloadIdentityJson: () =>
        JSON.stringify({
          scene_kind: "state_machine",
          default_state_machine_id: 0,
          view_model_initialization: "schema-default",
        }),
      advanceAndDraw: () => events.push(`total:${pass}`),
      advance: () => events.push(`advance:${pass}`),
      draw: () => events.push(`draw:${pass}`),
      free: () => events.push(`free:${pass}`),
    };
  };

  const report = await measureFixture({
    createRunner,
    now: () => clockValues.shift(),
    repeat: 2,
    sampleSeconds: 0,
  });

  assert.deepEqual(events, [
    "create:total",
    "total:total",
    "total:total",
    "free:total",
    "create:phases",
    "advance:phases",
    "draw:phases",
    "advance:phases",
    "draw:phases",
    "free:phases",
  ]);
  assert.equal(report.elapsed_ms, 4);
  assert.equal(report.advance_ms, 4);
  assert.equal(report.draw_ms, 5);
  assert.equal(report.bookkeeping_ms, 1);
  assert.equal(report.segments, 2);
  assert.deepEqual(report.workload_identity, {
    scene_kind: "state_machine",
    default_state_machine_id: 0,
    view_model_initialization: "schema-default",
  });
});

test("rejects fresh runners that do not describe the same workload", async () => {
  await assert.rejects(
    measureFixture({
      createRunner: async (pass) => ({
        workloadIdentityJson: () =>
          JSON.stringify({
            scene_kind: pass === "total" ? "state_machine" : "static",
            default_state_machine_id: pass === "total" ? 0 : null,
            view_model_initialization: "schema-default",
          }),
        advanceAndDraw() {},
        advance() {},
        draw() {},
        free() {},
      }),
      now: () => 1,
      repeat: 1,
      sampleSeconds: 0,
    }),
    /fresh runner workload identity mismatch/,
  );
});

test("records a static no-default-state-machine workload", async () => {
  const identity = {
    scene_kind: "static",
    default_state_machine_id: null,
    view_model_initialization: "none",
  };
  const report = await measureFixture({
    createRunner: async () => ({
      workloadIdentityJson: () => JSON.stringify(identity),
      advanceAndDraw() {},
      advance() {},
      draw() {},
      free() {},
    }),
    now: () => 1,
    repeat: 1,
    sampleSeconds: 0,
  });

  assert.deepEqual(report.workload_identity, identity);
});

test("fails closed when the monotonic browser clock moves backwards", async () => {
  const runner = {
    workloadIdentityJson: () =>
      JSON.stringify({
        scene_kind: "static",
        default_state_machine_id: null,
        view_model_initialization: "none",
      }),
    advanceAndDraw() {},
    advance() {},
    draw() {},
    free() {},
  };

  await assert.rejects(
    measureFixture({
      createRunner: async () => runner,
      now: (() => {
        const values = [10, 9];
        return () => values.shift();
      })(),
      repeat: 1,
      sampleSeconds: 0,
    }),
    /monotonic clock moved backwards/,
  );
});

test("rejects invalid repeat counts before constructing a runner", async () => {
  await assert.rejects(
    measureFixture({
      createRunner: async () => assert.fail("runner should not be constructed"),
      now: () => 0,
      repeat: 0,
      sampleSeconds: 0,
    }),
    /repeat must be a positive integer/,
  );
});

test("discards warmups and returns only independent measured runs", async () => {
  let invocation = 0;
  const reports = await measureRuns({
    measure: async () => ++invocation,
    warmups: 1,
    runs: 3,
  });

  assert.deepEqual(reports, [2, 3, 4]);
});
