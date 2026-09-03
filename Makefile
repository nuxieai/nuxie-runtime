.PHONY: rust-sources-fresh rust-runner-provenance-test runtime-differential-report-test fixtures schema check test inspect cpp-probe cpp-probe-scripted blob-differential cpp-atlas-mask-oracle cpp-atlas-mask-oracle-preflight golden-runner scripted-golden-runner rust-golden-runner scripted-rust-golden-runner golden-compare scripted-golden-compare e2e-composed-compare silver-corpus silver-corpus-validate silver-corpus-test silver-corpus-manifest-check cpp-oracle-workspace-tests renderer-replay renderer-references renderer-shaders-check renderer-decoder-oracle renderer-rust-replay-release renderer-metal-reference-bootstrap renderer-metal-reference-replay renderer-metal-reference-check renderer-metal-oracle-tracers renderer-metal-atomic-oracle-tracer renderer-native-metal-tracer-binary ore-metal-binding-witness ore-metal-authenticated-gpu-canvas renderer-dawn-reference-bootstrap renderer-dawn-reference-replay renderer-dawn-reference-check renderer-dawn-live-reference-bootstrap renderer-dawn-live-reference-replay renderer-dawn-live-reference-check renderer-golden-same-runner renderer-stub-baseline perf-compare perf-corpus perf-corpus-check perf-runtime-ref-check perf-hot-loop perf-json perf-gate-measure perf-gate perf-gate-tighten capi-smoke nux-capi-layout-contract nux-capi-surface-contract nux-capi-distribution-contract-test nux-capi-distribution-contract-gate nux-capi-pr-gate nux-capi-distribution-plan nux-capi-xcframeworks nux-capi-android-contract-test nux-capi-android-plan nux-capi-android cpp-binary-compare cpp-runtime-compare cpp-compare crate-seams-baseline-check crate-seams-browser-check crate-seams-apple-check crate-seams-full-check
.PHONY: runtime-source-correspondence-check renderer-shader-authority renderer-shader-authority-check
.PHONY: renderer-metal-msaa-contract renderer-metal-msaa-probe renderer-metal-cpp-parity renderer-metal-wgpu-diagnostic renderer-metal-wgpu-parity
.PHONY: renderer-native-metal-replay
.PHONY: renderer-native-metal-platform-matrix renderer-native-metal-v3

RIVE_RUNTIME_DIR ?= /Users/levi/dev/oss/rive-runtime
RIVE_RUNTIME_REF ?= 675703b9fd71e982eaf97c034b313eba9bde63f4
MICROBENCH_TOOL ?= $(CURDIR)/tools/microbench/microbench.py
DEFS_DIR ?= $(RIVE_RUNTIME_DIR)/dev/defs
SILVER_CORPUS_MANIFEST ?= $(CURDIR)/silver-corpus.toml
SILVER_CORPUS_GENERATOR ?= $(CURDIR)/tools/silver-corpus/generate_manifest.py
RUNTIME_DIFFERENTIAL_REPORT_TOOL ?= $(CURDIR)/tools/runtime-differentials/report.py
RUNTIME_DIFFERENTIAL_REPORT_DIR ?= $(CURDIR)/target/runtime-differentials
RUNTIME_DIFFERENTIAL_LOG_DIR ?= $(RUNTIME_DIFFERENTIAL_REPORT_DIR)/diagnostics
PURE_RUNTIME_BOUNDARY_TOOL ?= $(CURDIR)/tools/pure-runtime-boundary/check.py
CPP_CONFIG ?= debug
RUST_PROFILE ?= debug
RUST_GOLDEN_RUNNER_FLAGS = $(if $(filter release,$(RUST_PROFILE)),--release,)
RENDERER_JOBS ?= 1
RENDERER_SAME_RUNNER_JOBS ?= 1
RENDERER_REPLAY_TIMEOUT_SECONDS ?= 60
RENDERER_GOLDEN_TARGET_DIR ?= $(CURDIR)/target/renderer-golden
RENDERER_GOLDEN_RUST_REPLAY ?= $(RENDERER_GOLDEN_TARGET_DIR)/release/renderer-replay
RENDERER_METAL_REFERENCE_BUILD_DIR ?= $(CURDIR)/target/renderer-metal-reference-build
RENDERER_METAL_REFERENCE_DIR ?= $(CURDIR)/target/renderer-metal-reference
RENDERER_METAL_REFERENCE_REPLAY ?= $(RENDERER_METAL_REFERENCE_DIR)/renderer-replay
RENDERER_METAL_UPSTREAM_STAMP ?= $(RIVE_RUNTIME_DIR)/tests/out/release/.nuxie-metal-upstream-ref
RENDERER_METAL_REFERENCE_INPUT_MANIFEST ?= $(RENDERER_METAL_REFERENCE_DIR)/upstream-inputs.sha256
RENDERER_METAL_REFERENCE_BINDING ?= $(RENDERER_METAL_REFERENCE_DIR)/renderer-replay.inputs.sha256
RENDERER_METAL_ARCHIVE_PATHS = $(addprefix $(RIVE_RUNTIME_DIR)/tests/out/release/,librive.a librive_pls_renderer.a librive_decoders.a liblibwebp.a liblibpng.a libzlib.a liblibjpeg.a librive_harfbuzz.a librive_sheenbidi.a librive_yoga.a)
RENDERER_METAL_BRIDGE_PATHS = $(addprefix $(CURDIR)/,Cargo.lock tools/renderer-replay/Cargo.toml tools/renderer-replay/src/main.rs crates/nuxie-renderer-ffi/Cargo.toml crates/nuxie-renderer-ffi/build.rs crates/nuxie-renderer-ffi/cpp/rive_renderer_ffi.cpp crates/nuxie-renderer-ffi/cpp/rive_renderer_ffi.h crates/nuxie-renderer-ffi/cpp/rive_renderer_ffi_private.hpp crates/nuxie-renderer-ffi/cpp/rive_renderer_ffi_metal.mm)
RENDERER_METAL_CANDIDATE_BUILD_DIR ?= $(CURDIR)/target/renderer-native-metal
RENDERER_METAL_CANDIDATE_REPLAY ?= $(RENDERER_METAL_CANDIDATE_BUILD_DIR)/release/renderer-replay
RENDERER_METAL_CANDIDATE_BACKEND ?= rust-metal
RENDERER_METAL_TRACER_MANIFEST ?= $(CURDIR)/tools/renderer-tracers/tracer-corpus.toml
RENDERER_METAL_ATOMIC_TRACER_MANIFEST ?= $(CURDIR)/tools/renderer-tracers/tracer-corpus-atomic.toml
RENDERER_METAL_WGPU_TRACER_MANIFEST ?= $(CURDIR)/tools/renderer-tracers/tracer-corpus-wgpu-secondary.toml
RENDERER_METAL_TRACER_OUTPUT_DIR ?= $(CURDIR)/target/renderer-metal-tracers
RENDERER_METAL_WGPU_OUTPUT_DIR ?= $(RENDERER_METAL_TRACER_OUTPUT_DIR)/rust-wgpu-secondary
RENDERER_METAL_WGPU_PARITY_MANIFEST ?= $(CURDIR)/target/renderer-metal-wgpu-parity/clockwise-atomic.toml
RENDERER_METAL_CPP_PARITY_OUTPUT_DIR ?= $(CURDIR)/target/renderer-metal-cpp-parity/results
RENDERER_METAL_WGPU_PARITY_OUTPUT_DIR ?= $(CURDIR)/target/renderer-metal-wgpu-parity/results
RENDERER_METAL_WGPU_PARITY_EXPECTED_ROWS ?= 736
RENDERER_METAL_ORACLE_ENTRIES ?= --entry native-metal-first-light-rectangle --entry native-metal-first-light-gradient-cubic --entry native-metal-first-light-atlas-feather-stroke --entry native-metal-first-light-two-atlas-feather-strokes
RENDERER_METAL_ATOMIC_ORACLE_ENTRIES ?= --entry native-metal-first-light-triangle-generic-atomic --entry native-metal-first-light-gradient-cubic-generic-atomic --entry native-metal-gm-rect-grad-generic-atomic --entry native-metal-gm-gamma-correction-clip-generic-atomic --entry native-metal-gm-overfill-opaque-generic-atomic --entry native-metal-first-light-nested-clip-generic-atomic --entry native-metal-riv-deterministic-mode-mixed-gradient-generic-atomic --entry native-metal-gm-overfill-blendmodes-generic-atomic
RENDERER_DAWN_REFERENCE_BUILD_DIR ?= $(CURDIR)/target/renderer-dawn-reference-build
RENDERER_DAWN_REFERENCE_DIR ?= $(CURDIR)/target/renderer-dawn-reference
RENDERER_DAWN_REFERENCE_REPLAY ?= $(RENDERER_DAWN_REFERENCE_DIR)/renderer-replay
RENDERER_DAWN_LIVE_REFERENCE_BUILD_DIR ?= $(CURDIR)/target/renderer-dawn-live-reference-build
RENDERER_DAWN_LIVE_REFERENCE_DIR ?= $(CURDIR)/target/renderer-dawn-live-reference
RENDERER_DAWN_LIVE_REFERENCE_REPLAY ?= $(RENDERER_DAWN_LIVE_REFERENCE_DIR)/renderer-replay
RENDERER_SAME_RUNNER_OUTPUT_DIR ?= $(CURDIR)/target/renderer-same-runner-corpus
RENDERER_CORPUS_MANIFEST ?= $(CURDIR)/corpus-r.toml
RENDERER_CORPUS_EXPECTED_ROWS ?= 1469
CPP_PROBE ?= $(CURDIR)/tools/cpp-probe/build/$(shell uname -s | tr A-Z a-z | sed 's/darwin/macosx/')/bin/$(CPP_CONFIG)/rive_cpp_probe
SCRIPTED_CPP_PROBE ?= $(CURDIR)/tools/cpp-probe/build/$(shell uname -s | tr A-Z a-z | sed 's/darwin/macosx/')/bin/$(CPP_CONFIG)/rive_cpp_probe_scripted
PROMISE_CPP_ORACLE ?= $(CURDIR)/target/promise-oracle/rive_cpp_promise_oracle
GOLDEN_RUNNER ?= $(CURDIR)/tools/golden-runner/build/$(shell uname -s | tr A-Z a-z | sed 's/darwin/macosx/')/bin/$(CPP_CONFIG)/rive_golden_runner
SCRIPTED_GOLDEN_RUNNER ?= $(CURDIR)/tools/golden-runner/build/$(shell uname -s | tr A-Z a-z | sed 's/darwin/macosx/')/bin/$(CPP_CONFIG)/rive_golden_runner_scripted
RUST_GOLDEN_RUNNER ?= $(CURDIR)/target/$(RUST_PROFILE)/rust-golden-runner
SCRIPTED_RUST_GOLDEN_RUNNER ?= $(CURDIR)/target/$(RUST_PROFILE)/rust-golden-runner-scripted
E2E_COMPOSED_CORPUS ?= $(CURDIR)/e2e-composed.toml
PERF_FILE ?= $(RIVE_RUNTIME_DIR)/tests/unit_tests/assets/shapetest.riv
PERF_SAMPLES ?= 0
PERF_ITERATIONS ?= 10
PERF_WARMUPS ?= 3
PERF_RUNNER_ORDER ?= cpp-first
PERF_CORPUS ?= corpus.toml
PERF_GATE_MANIFEST ?= perf-corpus.toml
PERF_GATE_TOOL ?= tools/perf-gate/perf_gate.py
PERF_GATE_PINNER ?= tools/perf-gate/run-pinned.sh
PERF_GATE_COMPARE ?= $(CURDIR)/target/release/perf-compare
PERF_GATE_REPORT ?= $(CURDIR)/target/perf-gate.json
PERF_GATE_TIGHTEN_REPORT_2 ?= $(CURDIR)/target/perf-gate-tighten-2.json
PERF_GATE_TIGHTEN_REPORT_3 ?= $(CURDIR)/target/perf-gate-tighten-3.json
PERF_GATE_ITERATIONS ?= 5
PERF_GATE_WARMUPS ?= 0
PERF_GATE_FRAMES ?= 100
PERF_GATE_HZ ?= 60
PERF_CORPUS_LIMIT ?= 10
PERF_CORPUS_IDS ?= advance_blend_mode,ai_assitant,align_target,animated_clipping,animation_reset_cases,spotify_kids_demo
PERF_CORPUS_SELECTION = $(if $(strip $(PERF_CORPUS_IDS)),--corpus-ids "$(PERF_CORPUS_IDS)",--corpus-limit "$(PERF_CORPUS_LIMIT)")
PERF_AGGREGATE ?= min
PERF_MAX_RATIO ?= 1.0
PERF_BENCHMARK_REPEAT ?= 10000
PERF_JSON_OUT ?= $(CURDIR)/target/perf-compare.json
PERF_JSON_META ?= --meta build_profile=release --meta git_sha=$(shell git rev-parse HEAD 2>/dev/null || echo unknown) --meta timestamp=$(shell date -u +%Y-%m-%dT%H:%M:%SZ)
WASM_PERF_LIMIT ?= 5
WASM_PERF_IDS ?=
WASM_PERF_REPEAT ?= 100
WASM_PERF_RUNS ?= 5
WASM_PERF_WARMUPS ?= 1
WASM_PERF_OUTPUT ?= $(CURDIR)/target/wasm-perf.json
WASM_PERF_MARKDOWN ?= $(CURDIR)/target/wasm-perf.md
PERF_EXPECTED_RIVE_RUNTIME_REF ?= 675703b9fd71e982eaf97c034b313eba9bde63f4
RENDERER_PERF_TARGET_DIR ?= $(CURDIR)/target/renderer-perf
RENDERER_PERF_CPP_RUNNER ?= $(RENDERER_PERF_TARGET_DIR)/release/renderer-perf-cpp-runner
RENDERER_PERF_RUST_RUNNER ?= $(RENDERER_PERF_TARGET_DIR)/release/renderer-perf-rust-runner
# A single report is capture input. The five-report parity gate owns the 1.0x verdict.
RENDERER_PERF_MAX_RATIO ?= 1000
RENDERER_PERF_BASELINE_SOURCE_ID ?=
RENDERER_PERF_CANDIDATE_SOURCE_ID ?=
RENDERER_PERF_JSON ?= $(CURDIR)/target/renderer-perf.json
RENDERER_PERF_MARKDOWN ?= $(CURDIR)/target/renderer-perf.md
RENDERER_PERF_PARITY_REPORT_1 ?=
RENDERER_PERF_PARITY_REPORT_2 ?=
RENDERER_PERF_PARITY_REPORT_3 ?=
RENDERER_PERF_PARITY_REPORT_4 ?=
RENDERER_PERF_PARITY_REPORT_5 ?=
RENDERER_PERF_PARITY_MAX_RATIO ?= 1.0
RENDERER_PERF_PARITY_JSON ?= $(CURDIR)/target/renderer-perf-parity-gate.json
RENDERER_PERF_PARITY_MARKDOWN ?= $(CURDIR)/target/renderer-perf-parity-gate.md
RENDERER_TIMING_GATE_OUT_DIR ?=
RENDERER_TIMING_GATE_RENDERER_PERF ?= $(CURDIR)/target/release/renderer-perf
RENDERER_TIMING_GATE_COMPARATOR ?= $(CURDIR)/target/release/renderer-timing-compare
RENDERER_TIMING_GATE_MANIFEST ?= $(CURDIR)/tools/perf-compare/renderer-scenes.toml
RENDERER_TIMING_GATE_BASELINE_RUNNER ?= $(RENDERER_PERF_CPP_RUNNER)
RENDERER_TIMING_GATE_A_RUNNER ?=
RENDERER_TIMING_GATE_B_RUNNER ?=
RENDERER_TIMING_GATE_RENDERER_PERF_MAX_RATIO ?= 1.0
RENDERER_TIMING_GATE_CAPTURE_MAX_RATIO ?= 1000
RENDERER_TIMING_GATE_MAX_B_OVER_A ?= 1.0
RENDERER_TIMING_GATE_MAX_CONTROL_DRIFT ?= 1.05
RENDERER_TIMING_GATE_MAX_REPEAT_DRIFT ?= 1.05
RENDERER_TIMING_GATE_HOST_SAMPLER ?=
RENDERER_TIMING_GATE_BASELINE_SOURCE_ID ?=
RENDERER_TIMING_GATE_A_SOURCE_ID ?=
RENDERER_TIMING_GATE_B_SOURCE_ID ?=

export RENDERER_TIMING_GATE_OUT_DIR RENDERER_TIMING_GATE_RENDERER_PERF RENDERER_TIMING_GATE_COMPARATOR RENDERER_TIMING_GATE_MANIFEST
export RENDERER_TIMING_GATE_BASELINE_RUNNER RENDERER_TIMING_GATE_A_RUNNER RENDERER_TIMING_GATE_B_RUNNER
export RENDERER_TIMING_GATE_RENDERER_PERF_MAX_RATIO RENDERER_TIMING_GATE_MAX_B_OVER_A RENDERER_TIMING_GATE_MAX_CONTROL_DRIFT
export RENDERER_TIMING_GATE_CAPTURE_MAX_RATIO
export RENDERER_TIMING_GATE_MAX_REPEAT_DRIFT
export RENDERER_TIMING_GATE_HOST_SAMPLER
export RENDERER_TIMING_GATE_BASELINE_SOURCE_ID RENDERER_TIMING_GATE_A_SOURCE_ID RENDERER_TIMING_GATE_B_SOURCE_ID
RENDERER_COUNTER_TARGET_DIR ?= $(CURDIR)/target/renderer-counter
RENDERER_COUNTER_CPP_RUNNER ?= $(RENDERER_COUNTER_TARGET_DIR)/release/renderer-perf-cpp-runner
RENDERER_COUNTER_RUST_RUNNER ?= $(RENDERER_COUNTER_TARGET_DIR)/release/renderer-perf-rust-runner
RENDERER_COUNTER_BASELINE_SOURCE_ID ?= $(RENDERER_PERF_BASELINE_SOURCE_ID)
RENDERER_COUNTER_CANDIDATE_SOURCE_ID ?= $(RENDERER_PERF_CANDIDATE_SOURCE_ID)
RENDERER_COUNTER_JSON ?= $(CURDIR)/target/renderer-work-counters.json
RENDERER_COUNTER_MARKDOWN ?= $(CURDIR)/target/renderer-work-counters.md
CAPI_SMOKE_FIXTURE ?= fixtures/animation/smi_test.riv
CC ?= cc

fixtures:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" tools/fetch-test-assets.sh

schema:
	cargo run -p nuxie-codegen -- --defs "$(DEFS_DIR)" --out crates/nuxie-schema/src/generated/schema.rs
	cargo fmt --all

.PHONY: fmt fmt-check
# Keep formatting rooted at the current workspace so linked worktrees use the
# same manifest and toolchain configuration.
fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

check:
	cargo check --workspace

# Independently selectable runtime package cuts. The Apple target compiles the
# actual five-slice nux-capi distribution root instead of a host-only cfg shell.
crate-seams-baseline-check:
	cargo check -p nuxie-runtime --no-default-features --lib

crate-seams-browser-check:
	RUSTC="$$(rustup which --toolchain stable rustc)" \
		"$$(rustup which --toolchain stable cargo)" check --locked \
		-p webgpu-renderer-replay -p webgl2-renderer-replay \
		--target wasm32-unknown-unknown --all-targets

crate-seams-apple-check:
	tools/check-nux-capi-apple.sh
	cargo check --locked -p nuxie-renderer --no-default-features --features renderer-metal

crate-seams-full-check:
	cargo check --workspace

test: fixtures
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" cargo test --workspace

# --- Tool-check gates: the tool's unit tests and the check it performs are
# independent verdicts, so neither is allowed to hide the other -------------
# These checks used to be written `X-check: X-test`, which makes the unit tests
# a precondition for the check: a red test suite meant the check never ran at
# all, and any real drift underneath stayed invisible until the tests were
# fixed. That masked two separate live failures in #272 alone. The `-check`
# targets now stand alone, and the `-gate` targets run the tests and the check
# in one pass through tools/report-all.sh, which reports every failure.
.PHONY: pure-runtime-boundary-test pure-runtime-boundary-check pure-runtime-boundary-gate

renderer-native-metal-v3:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" MTL_DEBUG_LAYER=1 MTL_SHADER_VALIDATION=1 NUXIE_REQUIRE_LIVE_METAL_TESTS=1 cargo test --locked -p nuxie-renderer --no-default-features --features renderer-metal,native-ore-metal-experimental --lib -- --skip deferred::gm::image_paint::image_paint --test-threads=1
	# Shader validation instruments Metal execution and perturbs image_paint by one
	# channel value. Run its exact oracle under the C++ capture environment instead.
	env -u MTL_SHADER_VALIDATION RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" MTL_DEBUG_LAYER=1 NUXIE_REQUIRE_LIVE_METAL_TESTS=1 cargo test --locked -p nuxie-renderer --no-default-features --features renderer-metal,native-ore-metal-experimental deferred::gm::image_paint::image_paint --lib -- --exact --test-threads=1
	MTL_DEBUG_LAYER=1 MTL_SHADER_VALIDATION=1 NUXIE_REQUIRE_LIVE_METAL_TESTS=1 cargo test --locked -p nuxie-renderer --no-default-features --features renderer-metal,native-ore-metal-experimental --test native_metal_resource_shaders -- --test-threads=1
	NUXIE_REQUIRE_LIVE_METAL_TESTS=1 cargo test --locked -p nuxie-ore-metal --no-default-features --features metal-backend -- --test-threads=1
	NUXIE_REQUIRE_LIVE_METAL_TESTS=1 cargo test --locked -p nuxie-ore-metal --no-default-features --features tools,metal-backend -- --test-threads=1

runtime-source-correspondence-check:
	PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools/test_runtime_source_correspondence.py tools/test_buildkite_pipeline.py
	PYTHONDONTWRITEBYTECODE=1 python3 tools/check_runtime_source_correspondence.py --repo-root "$(CURDIR)" --upstream-root "$(RIVE_RUNTIME_DIR)" --upstream-ref "$(RIVE_RUNTIME_REF)"

renderer-shader-authority:
	PYTHONDONTWRITEBYTECODE=1 python3 tools/backend-port/build_shader_authority_translations.py --repo-root "$(CURDIR)" --upstream-root "$(RIVE_RUNTIME_DIR)"

renderer-shader-authority-check:
	PYTHONDONTWRITEBYTECODE=1 python3 tools/backend-port/build_shader_authority_translations.py --repo-root "$(CURDIR)" --upstream-root "$(RIVE_RUNTIME_DIR)" --check

renderer-native-metal-tracer-binary:
	PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools/test_check_native_metal_product_dependencies.py
	tools/check-native-metal-tracer-binary.sh

ore-metal-binding-witness:
	tools/check-ore-metal-binding-witness.sh

ore-metal-authenticated-gpu-canvas:
	tools/check-ore-metal-authenticated-gpu-canvas.sh

renderer-native-metal-replay:
	MACOSX_DEPLOYMENT_TARGET=12.0 CARGO_TARGET_DIR="$(RENDERER_METAL_CANDIDATE_BUILD_DIR)" cargo build --quiet --locked --release -p renderer-replay --no-default-features --features native-metal --bin renderer-replay

pure-runtime-boundary-test:
	PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools/pure-runtime-boundary -p 'test_*.py' -v

pure-runtime-boundary-check:
	PYTHONDONTWRITEBYTECODE=1 python3 "$(PURE_RUNTIME_BOUNDARY_TOOL)" --repo-root "$(CURDIR)"

pure-runtime-boundary-gate:
	@tools/report-all.sh "pure-runtime-boundary" \
		"pure-runtime boundary tool unit tests" "$(MAKE) --no-print-directory pure-runtime-boundary-test" \
		"workspace dependency and source debt check" "$(MAKE) --no-print-directory pure-runtime-boundary-check"

# --- Pinned upstream microbenchmark mirror ----------------------------------
.PHONY: microbench-test microbench-check microbench-upstream-check microbench-gate microbench-extract
microbench-test:
	PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools/microbench -p 'test_*.py' -v

microbench-check:
	PYTHONDONTWRITEBYTECODE=1 python3 "$(MICROBENCH_TOOL)" --repo-root "$(CURDIR)" check

microbench-upstream-check:
	PYTHONDONTWRITEBYTECODE=1 python3 "$(MICROBENCH_TOOL)" --repo-root "$(CURDIR)" check-upstream --upstream "$(RIVE_RUNTIME_DIR)"

microbench-gate:
	@tools/report-all.sh "upstream-microbenchmarks" \
		"microbenchmark tool unit tests" "$(MAKE) --no-print-directory microbench-test" \
		"20-case upstream inventory and fixture hashes" "$(MAKE) --no-print-directory microbench-check" \
		"pinned upstream registry, sources, and fixture provenance" "$(MAKE) --no-print-directory microbench-upstream-check"

microbench-extract:
	PYTHONDONTWRITEBYTECODE=1 python3 "$(MICROBENCH_TOOL)" --repo-root "$(CURDIR)" extract --upstream "$(RIVE_RUNTIME_DIR)"

# --- Clippy lint gate (panic-freedom discipline) ------------------------------
# The runtime crates opt into the panic-freedom clippy lints
# (clippy::unwrap_used / indexing_slicing / arithmetic_side_effects):
# - LINT_GATE_DENY_CRATES are fully clean and pin the lints at DENY in their
#   own [lints.clippy] tables -- clippy alone fails the gate on a violation.
# - LINT_GATE_WARN_CRATES inherit the workspace table at WARN (root
#   Cargo.toml); their remaining own-src warning counts are printed here so
#   regressions are visible in review. Move a crate into the deny list (and
#   switch its lints table to the deny form) once its counts reach zero.
# Library targets only (src/); deliberately NOT tests and NOT tools/ or
# nuxie-scripting. NOTE: `cargo clippy -- -D lint` is not used because trailing
# flags leak to dependency crates; the per-crate lints tables scope correctly.
LINT_GATE_DENY_CRATES = nuxie nuxie-schema
LINT_GATE_WARN_CRATES = nuxie-audio nuxie-runtime nuxie-binary nuxie-ore-metal nux-capi

.PHONY: lint-gate
lint-gate:
	@set -e; \
	for crate in $(LINT_GATE_DENY_CRATES); do \
		echo "== lint-gate (deny): $$crate =="; \
		cargo clippy -p $$crate --lib --quiet; \
	done; \
	for crate in $(LINT_GATE_WARN_CRATES); do \
		count=$$(cargo clippy -p $$crate --lib --quiet 2>&1 \
			| grep -c -- "--> crates/$$crate/src" || true); \
		echo "== lint-gate (warn): $$crate -- $$count own-src warning sites =="; \
	done

# --- Feature compile gate ---------------------------------------------------
# Code behind a Cargo feature that no CI job builds does not compile in CI, and
# a `#[cfg(feature = ...)]` module that nothing compiles rots silently.
#
# This gate type-checks -- `cargo check`, no linking, no fixtures beyond the
# pinned assets -- every first-party feature that no other CI job builds. New
# feature declarations belong here unless some existing job already compiles
# them; `git grep -- --features Makefile .github` shows what does.
#
# Two tiers because two hosts:
# - PORTABLE runs anywhere and is wired into the ubuntu Clippy lint gate job.
# - APPLE needs an Apple target because the measurement roots include the
#   renderer-owned opaque Metal presenter, so it is wired into macOS ahead of
#   that job's expensive reference-runtime build.
# Both tiers report every failing entry rather than stopping at the first.
.PHONY: feature-compile-gate feature-compile-gate-portable feature-compile-gate-apple
feature-compile-gate-portable:
	@tools/report-all.sh "feature-compile-gate (portable)" \
		"nuxie-runtime --features threading" "cargo check -p nuxie-runtime --features threading --lib --test work_pool" \
		"nuxie-runtime --features tools" "cargo check -p nuxie-runtime --features tools --lib --tests" \
		"nuxie-ore-metal --features tools" "cargo test -p nuxie-ore-metal --features tools --lib" \
		"nuxie-renderer --features with-rive-path-query tests" "cargo test -p nuxie-renderer --no-default-features --features with-rive-path-query --lib" \
		"nuxie-renderer exact Vulkan" "cargo check --locked -p nuxie-renderer --no-default-features --features renderer-vulkan" \
		"nuxie-renderer exact WebGPU" "cargo check --locked -p nuxie-renderer --no-default-features --features renderer-webgpu" \
		"nux-capi Android authored WGSL" "cargo check --locked -p nux-capi --no-default-features --features android-vulkan,scripting,android-authored-wgsl" \
		"rust-golden-runner --features coverage-trace" "cargo check -p rust-golden-runner --features coverage-trace --all-targets" \
		"nuxie-scripting --no-default-features" "cargo check -p nuxie-scripting --no-default-features --lib" \
		"nuxie --no-default-features" "cargo check -p nuxie --no-default-features --lib" \
		"riv-inspect --features inspect" "cargo check -p nuxie-binary --features inspect --bin riv-inspect"

feature-compile-gate-apple:
	@tools/report-all.sh "feature-compile-gate (apple)" \
		"nuxie-audio --features audio-device" "cargo check -p nuxie-audio --features audio-device --all-targets" \
		"nuxie --features renderer-metal" "cargo check --locked -p nuxie --no-default-features --features renderer-metal --lib" \
		"nuxie-renderer --features renderer-metal" "cargo check --locked -p nuxie-renderer --no-default-features --features renderer-metal --lib" \
		"nux-capi --features apple-metal" "cargo check --locked -p nux-capi --no-default-features --features apple-metal" \
		"Darwin renderer measurement seam" "$(MAKE) --no-print-directory crate-seams-apple-check"

feature-compile-gate:
	@tools/report-all.sh "feature-compile-gate" \
		"portable tier" "$(MAKE) --no-print-directory feature-compile-gate-portable" \
		"apple tier" "$(MAKE) --no-print-directory feature-compile-gate-apple"

inspect:
	@cargo run --quiet -p nuxie-binary --features inspect --bin riv-inspect -- fixtures/graph/dependency_test.riv

cpp-probe:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" tools/cpp-probe/build.sh "$(CPP_CONFIG)"

cpp-probe-scripted:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" RIVE_CPP_PROBE_WITH_SCRIPTING=1 RIVE_CPP_PROBE_RUNNER_NAME=rive_cpp_probe_scripted tools/cpp-probe/build.sh "$(CPP_CONFIG)"

.PHONY: capi-player-step-oracle
# Live three-way ABI conformance check: C entry point and direct Rust facade
# against the provenance-checked pinned C++ Scene oracle.
capi-player-step-oracle: cpp-probe
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" RIVE_CPP_PROBE="$(CPP_PROBE)" cargo test -p nux-capi --test player_step live_pinned_cpp_player_step_oracle_matches_c_and_rust -- --exact

blob-differential: cpp-probe-scripted
	NUXIE_CPP_BLOB_ORACLE="$(SCRIPTED_CPP_PROBE)" cargo test -p nuxie-scripting vm::view_model::tests::context_blob_positive_lookup_matches_live_cpp_oracle --lib -- --ignored --exact

.PHONY: promise-oracle promise-differential

promise-oracle:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" bash tools/promise-oracle/build.sh release

promise-differential: promise-oracle
	NUXIE_CPP_PROMISE_ORACLE="$(PROMISE_CPP_ORACLE)" cargo test -p nuxie-scripting --test promise_scenarios promise_scenarios_match_live_cpp_oracle -- --ignored --exact

cpp-atlas-mask-oracle-preflight:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" tools/cpp-atlas-mask-oracle/build.sh --preflight

cpp-atlas-mask-oracle:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" tools/cpp-atlas-mask-oracle/build.sh

golden-runner:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" tools/golden-runner/build.sh "$(CPP_CONFIG)"

scripted-golden-runner:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" RIVE_GOLDEN_WITH_SCRIPTING=1 RIVE_GOLDEN_RUNNER_NAME=rive_golden_runner_scripted tools/golden-runner/build.sh "$(CPP_CONFIG)"

# The Rust runners are built through the content-provenance guard: cargo's
# mtime-based freshness cannot see a source rewritten without a newer mtime
# (e.g. regenerated schema.rs racing a concurrent cargo build), so the guard
# hashes workspace sources, invalidates poisoned members, and binds the gate
# binaries to the verified content. See tools/golden-runner/rust_runner_provenance.py.
rust-golden-runner:
	PYTHONDONTWRITEBYTECODE=1 python3 tools/golden-runner/rust_runner_provenance.py ensure --variant ordinary --profile "$(RUST_PROFILE)"

scripted-rust-golden-runner:
	PYTHONDONTWRITEBYTECODE=1 python3 tools/golden-runner/rust_runner_provenance.py ensure --variant scripted --profile "$(RUST_PROFILE)"

rust-sources-fresh:
	PYTHONDONTWRITEBYTECODE=1 python3 tools/golden-runner/rust_runner_provenance.py ensure-sources

rust-runner-provenance-test:
	PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools/golden-runner -p 'test_*.py' -v

golden-compare: fixtures golden-runner rust-golden-runner
	@mkdir -p "$(RUNTIME_DIFFERENTIAL_LOG_DIR)"; \
	log="$(RUNTIME_DIFFERENTIAL_LOG_DIR)/golden-ordinary.log"; \
	set +e; GOLDEN_RUNNER="$(GOLDEN_RUNNER)" RUST_GOLDEN_RUNNER="$(RUST_GOLDEN_RUNNER)" RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" cargo run --quiet -p golden-compare --bin golden-compare -- --corpus corpus.toml --side-channel --verify-divergent-rust --cpp-runner "$(GOLDEN_RUNNER)" --rust-runner "$(RUST_GOLDEN_RUNNER)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)" >"$$log" 2>&1; gate_rc=$$?; set -e; \
	cat "$$log"; report_rc=0; \
	PYTHONDONTWRITEBYTECODE=1 python3 "$(RUNTIME_DIFFERENTIAL_REPORT_TOOL)" golden --manifest corpus.toml --runtime-dir "$(RIVE_RUNTIME_DIR)" --repo-root "$(CURDIR)" --mode ordinary --cpp-ref "$(RIVE_RUNTIME_REF)" --rust-commit "$$(git rev-parse HEAD)" --runner "cpp=$(GOLDEN_RUNNER)" --runner "rust=$(RUST_GOLDEN_RUNNER)" --diagnostics "$$log" --gate-rc "$$gate_rc" --output "$(RUNTIME_DIFFERENTIAL_REPORT_DIR)/golden-ordinary.json" || report_rc=$$?; \
	if [ "$$gate_rc" -ne 0 ]; then exit "$$gate_rc"; fi; exit "$$report_rc"

scripted-golden-compare: CPP_CONFIG=release
scripted-golden-compare: fixtures scripted-golden-runner scripted-rust-golden-runner
	@mkdir -p "$(RUNTIME_DIFFERENTIAL_LOG_DIR)"; \
	log="$(RUNTIME_DIFFERENTIAL_LOG_DIR)/golden-scripted.log"; \
	set +e; RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" cargo run --quiet -p golden-compare --bin golden-compare -- --corpus corpus.toml --side-channel --verify-unsupported-cpp --verify-divergent-rust --verify-scripted-diagnostics --cpp-runner "$(SCRIPTED_GOLDEN_RUNNER)" --rust-runner "$(SCRIPTED_RUST_GOLDEN_RUNNER)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)" >"$$log" 2>&1; gate_rc=$$?; set -e; \
	cat "$$log"; report_rc=0; \
	PYTHONDONTWRITEBYTECODE=1 python3 "$(RUNTIME_DIFFERENTIAL_REPORT_TOOL)" golden --manifest corpus.toml --runtime-dir "$(RIVE_RUNTIME_DIR)" --repo-root "$(CURDIR)" --mode scripted --cpp-ref "$(RIVE_RUNTIME_REF)" --rust-commit "$$(git rev-parse HEAD)" --runner "cpp=$(SCRIPTED_GOLDEN_RUNNER)" --runner "rust=$(SCRIPTED_RUST_GOLDEN_RUNNER)" --diagnostics "$$log" --gate-rc "$$gate_rc" --output "$(RUNTIME_DIFFERENTIAL_REPORT_DIR)/golden-scripted.json" || report_rc=$$?; \
	if [ "$$gate_rc" -ne 0 ]; then exit "$$gate_rc"; fi; exit "$$report_rc"

e2e-composed-compare: CPP_CONFIG=release
e2e-composed-compare: RUST_PROFILE=release
e2e-composed-compare: fixtures scripted-golden-runner scripted-rust-golden-runner
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" cargo run --quiet -p golden-compare --bin golden-compare -- --corpus "$(E2E_COMPOSED_CORPUS)" --side-channel --require-composed-session --verify-scripted-diagnostics --cpp-runner "$(SCRIPTED_GOLDEN_RUNNER)" --rust-runner "$(SCRIPTED_RUST_GOLDEN_RUNNER)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)"

silver-corpus-test:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" cargo test -p silver-corpus
	PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools/silver-corpus -p 'test_*.py' -v

runtime-differential-report-test:
	PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools/runtime-differentials -p 'test_*.py' -v

silver-corpus-manifest-check:
	PYTHONDONTWRITEBYTECODE=1 python3 "$(SILVER_CORPUS_GENERATOR)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)" --output "$(SILVER_CORPUS_MANIFEST)" --check

# validate reads the manifest, so the manifest check stays a genuine
# precondition of it. The unit tests do not: they were only a prerequisite, and
# as one a red suite stopped both the manifest check and the validation from
# running at all.
silver-corpus-validate: silver-corpus-manifest-check
	@mkdir -p "$(RUNTIME_DIFFERENTIAL_LOG_DIR)"; \
	log="$(RUNTIME_DIFFERENTIAL_LOG_DIR)/silver.log"; \
	set +e; cargo run --quiet -p silver-corpus -- validate --manifest "$(SILVER_CORPUS_MANIFEST)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)" --lane runtime >"$$log" 2>&1; gate_rc=$$?; set -e; \
	cat "$$log"; report_rc=0; \
	PYTHONDONTWRITEBYTECODE=1 python3 "$(RUNTIME_DIFFERENTIAL_REPORT_TOOL)" silver --manifest "$(SILVER_CORPUS_MANIFEST)" --runtime-dir "$(RIVE_RUNTIME_DIR)" --repo-root "$(CURDIR)" --rust-commit "$$(git rev-parse HEAD)" --runner "validator=$(CURDIR)/target/debug/silver-corpus" --diagnostics "$$log" --gate-rc "$$gate_rc" --output "$(RUNTIME_DIFFERENTIAL_REPORT_DIR)/silver.json" || report_rc=$$?; \
	if [ "$$gate_rc" -ne 0 ]; then exit "$$gate_rc"; fi; exit "$$report_rc"

silver-corpus:
	@tools/report-all.sh "silver-corpus" \
		"silver corpus unit tests" "$(MAKE) --no-print-directory silver-corpus-test" \
		"runtime differential report tests" "$(MAKE) --no-print-directory runtime-differential-report-test" \
		"silver corpus manifest check and validation" "$(MAKE) --no-print-directory silver-corpus-validate"

cpp-oracle-workspace-tests: fixtures golden-runner cpp-probe cpp-probe-scripted
	@test -x "$(GOLDEN_RUNNER)" || { echo "missing executable pinned C++ golden runner: $(GOLDEN_RUNNER)" >&2; exit 2; }
	@test -x "$(CPP_PROBE)" || { echo "missing executable pinned C++ probe: $(CPP_PROBE)" >&2; exit 2; }
	@test -x "$(SCRIPTED_CPP_PROBE)" || { echo "missing executable pinned scripted C++ probe: $(SCRIPTED_CPP_PROBE)" >&2; exit 2; }
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" RIVE_GOLDEN_RUNNER="$(GOLDEN_RUNNER)" RIVE_CPP_PROBE="$(CPP_PROBE)" RIVE_CPP_PROBE_SCRIPTED="$(SCRIPTED_CPP_PROBE)" cargo test --workspace

renderer-replay:
	cargo build --quiet -p renderer-replay

renderer-references:
	CARGO_TARGET_DIR="$(CURDIR)/target/renderer-ffi" cargo build --quiet -p renderer-replay --features ffi
	CARGO_TARGET_DIR="$(CURDIR)/target/renderer-ffi" cargo run --quiet -p pixel-compare --bin capture-corpus-r-references -- --replay "$(CURDIR)/target/renderer-ffi/debug/renderer-replay"

renderer-shaders-check:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" tools/check-renderer-shaders.sh

renderer-decoder-oracle:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" tools/check-renderer-decoder-provenance.sh
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" CARGO_INCREMENTAL=0 cargo test -p nuxie-renderer-ffi --features decode-oracle --test decode_oracle -- --nocapture

# The same-runner gate deliberately keeps the live reference and candidate
# builds separate. CI may restore only RENDERER_DAWN_LIVE_REFERENCE_REPLAY from
# its exact pinned-input cache; the Rust candidate below is always compiled
# from HEAD. The historical RENDERER_DAWN_REFERENCE_REPLAY remains isolated for
# the immutable renderer-port pixel oracle and is never relabeled as current-runtime output.
renderer-rust-replay-release:
	CARGO_TARGET_DIR="$(RENDERER_GOLDEN_TARGET_DIR)" cargo build --quiet --locked --release -p renderer-replay --no-default-features --features native-webgpu-exact --bin renderer-replay

# Build the upstream archives and bind them to the exact source revision. The
# stamp prevents a checkout change from silently reusing ABI-incompatible
# archives.
renderer-metal-reference-bootstrap:
	@test "$$(git -C "$(RIVE_RUNTIME_DIR)" rev-parse HEAD)" = "$(RIVE_RUNTIME_REF)" || { echo "C++ Metal oracle checkout does not match RIVE_RUNTIME_REF=$(RIVE_RUNTIME_REF)" >&2; exit 2; }
	@git -C "$(RIVE_RUNTIME_DIR)" diff --quiet --ignore-submodules -- && git -C "$(RIVE_RUNTIME_DIR)" diff --cached --quiet --ignore-submodules -- || { echo "C++ Metal oracle checkout has tracked changes" >&2; exit 2; }
	cd "$(RIVE_RUNTIME_DIR)/tests" && ../build/build_rive.sh release -- rive rive_pls_renderer rive_decoders libwebp libpng zlib libjpeg rive_harfbuzz rive_sheenbidi rive_yoga
	git -C "$(RIVE_RUNTIME_DIR)" rev-parse HEAD > "$(RENDERER_METAL_UPSTREAM_STAMP)"
	mkdir -p "$(RENDERER_METAL_REFERENCE_DIR)"
	rm -f "$(RENDERER_METAL_REFERENCE_BINDING)"
	@{ printf 'runtime_revision=%s\n' "$(RIVE_RUNTIME_REF)"; shasum -a 256 $(RENDERER_METAL_ARCHIVE_PATHS); } > "$(RENDERER_METAL_REFERENCE_INPUT_MANIFEST)"

# Build a C++ native-Metal-only oracle replay. This deliberately disables the
# exact WebGPU renderer and does not select a fallback backend.
renderer-metal-reference-replay:
	@test -f "$(RENDERER_METAL_UPSTREAM_STAMP)" || { echo "missing pinned upstream Metal archive stamp; run make renderer-metal-reference-bootstrap" >&2; exit 2; }
	@test "$$(cat "$(RENDERER_METAL_UPSTREAM_STAMP)")" = "$(RIVE_RUNTIME_REF)" || { echo "stale upstream Metal archives; run make renderer-metal-reference-bootstrap" >&2; exit 2; }
	MACOSX_DEPLOYMENT_TARGET=12.0 RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" CARGO_TARGET_DIR="$(RENDERER_METAL_REFERENCE_BUILD_DIR)" cargo build --quiet --locked --release -p renderer-replay --no-default-features --features ffi --bin renderer-replay
	mkdir -p "$(RENDERER_METAL_REFERENCE_DIR)"
	cp "$(RENDERER_METAL_REFERENCE_BUILD_DIR)/release/renderer-replay" "$(RENDERER_METAL_REFERENCE_REPLAY)"
	chmod 0755 "$(RENDERER_METAL_REFERENCE_REPLAY)"
	@shasum -a 256 "$(RENDERER_METAL_REFERENCE_INPUT_MANIFEST)" "$(RENDERER_METAL_REFERENCE_REPLAY)" $(RENDERER_METAL_BRIDGE_PATHS) > "$(RENDERER_METAL_REFERENCE_BINDING)"

renderer-metal-reference-check:
	@test -x "$(RENDERER_METAL_REFERENCE_REPLAY)" || { echo "missing executable C++ Metal reference replay: $(RENDERER_METAL_REFERENCE_REPLAY)" >&2; exit 2; }
	@test "$$(git -C "$(RIVE_RUNTIME_DIR)" rev-parse HEAD)" = "$(RIVE_RUNTIME_REF)" || { echo "C++ Metal oracle checkout moved from $(RIVE_RUNTIME_REF)" >&2; exit 2; }
	@git -C "$(RIVE_RUNTIME_DIR)" diff --quiet --ignore-submodules -- && git -C "$(RIVE_RUNTIME_DIR)" diff --cached --quiet --ignore-submodules -- || { echo "C++ Metal oracle checkout has tracked changes" >&2; exit 2; }
	@test -f "$(RENDERER_METAL_UPSTREAM_STAMP)" && test "$$(cat "$(RENDERER_METAL_UPSTREAM_STAMP)")" = "$(RIVE_RUNTIME_REF)" || { echo "missing or stale upstream Metal archive stamp" >&2; exit 2; }
	@test -f "$(RENDERER_METAL_REFERENCE_INPUT_MANIFEST)" && grep -Fqx 'runtime_revision=$(RIVE_RUNTIME_REF)' "$(RENDERER_METAL_REFERENCE_INPUT_MANIFEST)" || { echo "missing or stale Metal reference input manifest" >&2; exit 2; }
	@tail -n +2 "$(RENDERER_METAL_REFERENCE_INPUT_MANIFEST)" | shasum -a 256 -c - >/dev/null || { echo "C++ Metal oracle archive identity changed; rerun renderer-metal-reference-bootstrap and rebuild the replay" >&2; exit 2; }
	@test -f "$(RENDERER_METAL_REFERENCE_BINDING)" && shasum -a 256 -c "$(RENDERER_METAL_REFERENCE_BINDING)" >/dev/null || { echo "C++ Metal reference replay or its bridge inputs changed; rerun renderer-metal-reference-replay" >&2; exit 2; }
	@if otool -L "$(RENDERER_METAL_REFERENCE_REPLAY)" | tail -n +2 | grep -Eiq 'dawn|webgpu'; then echo "C++ Metal reference replay unexpectedly requires a Dawn/WebGPU dynamic library" >&2; exit 2; fi

# Compare the actual Rust Metal candidate against pinned C++ Metal, then run
# the same candidate and stream against the exact WebGPU port as a diagnostic only.
renderer-metal-oracle-tracers: renderer-native-metal-replay renderer-rust-replay-release renderer-metal-reference-check
	cargo run --quiet -p pixel-compare --bin corpus-r -- --manifest "$(RENDERER_METAL_TRACER_MANIFEST)" --replay "$(RENDERER_METAL_CANDIDATE_REPLAY)" --backend "$(RENDERER_METAL_CANDIDATE_BACKEND)" --reference-replay "$(RENDERER_METAL_REFERENCE_REPLAY)" --reference-backend ffi-metal --reference-input-manifest "$(RENDERER_METAL_REFERENCE_INPUT_MANIFEST)" --output-dir "$(RENDERER_METAL_TRACER_OUTPUT_DIR)" --jobs 1 --replay-timeout-seconds "$(RENDERER_REPLAY_TIMEOUT_SECONDS)" $(RENDERER_METAL_ORACLE_ENTRIES)
	cargo run --quiet -p pixel-compare --bin corpus-r -- --manifest "$(RENDERER_METAL_WGPU_TRACER_MANIFEST)" --replay "$(RENDERER_METAL_CANDIDATE_REPLAY)" --backend "$(RENDERER_METAL_CANDIDATE_BACKEND)" --reference-replay "$(RENDERER_GOLDEN_RUST_REPLAY)" --reference-backend rust-webgpu-exact --output-dir "$(RENDERER_METAL_WGPU_OUTPUT_DIR)" --jobs 1 --replay-timeout-seconds "$(RENDERER_REPLAY_TIMEOUT_SECONDS)" $(RENDERER_METAL_ORACLE_ENTRIES)

# Purpose-built UNIV-2088 lane. Keep the established `rust-metal` corpus on
# capability-driven selection; only these bounded tracers force generic atomics.
renderer-metal-atomic-oracle-tracer: renderer-native-metal-replay renderer-metal-reference-check
	cargo run --quiet -p pixel-compare --bin corpus-r -- --manifest "$(RENDERER_METAL_ATOMIC_TRACER_MANIFEST)" --replay "$(RENDERER_METAL_CANDIDATE_REPLAY)" --backend rust-metal-atomic --reference-replay "$(RENDERER_METAL_REFERENCE_REPLAY)" --reference-backend ffi-metal --reference-input-manifest "$(RENDERER_METAL_REFERENCE_INPUT_MANIFEST)" --output-dir "$(RENDERER_METAL_TRACER_OUTPUT_DIR)/generic-atomic" --jobs 1 --replay-timeout-seconds "$(RENDERER_REPLAY_TIMEOUT_SECONDS)" $(RENDERER_METAL_ATOMIC_ORACLE_ENTRIES)

# Complete authoritative product-output differential against the pinned
# upstream C++ Metal renderer. The derived manifest contains every
# Metal-compatible corpus row and carries the predeclared source-manifest
# tolerances unchanged; neither candidate output nor this target can widen
# them. Run serially because both replay processes share one physical adapter.
renderer-metal-cpp-parity: renderer-native-metal-replay renderer-metal-reference-check
	python3 tools/renderer-tracers/derive_clockwise_atomic_manifest.py --input "$(RENDERER_CORPUS_MANIFEST)" --output "$(RENDERER_METAL_WGPU_PARITY_MANIFEST)" --expected "$(RENDERER_METAL_WGPU_PARITY_EXPECTED_ROWS)"
	cargo run --quiet -p pixel-compare --bin corpus-r -- --manifest "$(RENDERER_METAL_WGPU_PARITY_MANIFEST)" --replay "$(RENDERER_METAL_CANDIDATE_REPLAY)" --backend rust-metal-atomic --reference-replay "$(RENDERER_METAL_REFERENCE_REPLAY)" --reference-backend ffi-metal --reference-input-manifest "$(RENDERER_METAL_REFERENCE_INPUT_MANIFEST)" --output-dir "$(RENDERER_METAL_CPP_PARITY_OUTPUT_DIR)" --jobs 1 --replay-timeout-seconds "$(RENDERER_REPLAY_TIMEOUT_SECONDS)"

# Secondary backend differential. This never overrules the pinned C++ Metal
# oracle: completed WebGPU pixel differences are reported as diagnostics, while
# replay crashes, timeouts, and malformed outputs still fail the command.
renderer-metal-wgpu-diagnostic: renderer-native-metal-replay renderer-rust-replay-release
	python3 tools/renderer-tracers/derive_clockwise_atomic_manifest.py --input "$(RENDERER_CORPUS_MANIFEST)" --output "$(RENDERER_METAL_WGPU_PARITY_MANIFEST)" --expected "$(RENDERER_METAL_WGPU_PARITY_EXPECTED_ROWS)"
	cargo run --quiet -p pixel-compare --bin corpus-r -- --manifest "$(RENDERER_METAL_WGPU_PARITY_MANIFEST)" --replay "$(RENDERER_METAL_CANDIDATE_REPLAY)" --backend rust-metal-atomic --reference-replay "$(RENDERER_GOLDEN_RUST_REPLAY)" --reference-backend rust-webgpu-exact --output-dir "$(RENDERER_METAL_WGPU_PARITY_OUTPUT_DIR)" --jobs 1 --replay-timeout-seconds "$(RENDERER_REPLAY_TIMEOUT_SECONDS)" --report-divergences

# Compatibility spelling retained for existing local scripts. This target is
# diagnostic; `renderer-metal-cpp-parity` is the authoritative gate.
renderer-metal-wgpu-parity: renderer-metal-wgpu-diagnostic

# Compile every Apple target configuration represented by the pinned Metal
# source. tvOS and visionOS use nightly build-std because rustup does not ship
# prebuilt standard libraries for those targets.
renderer-native-metal-platform-matrix:
	tools/check-native-metal-platform-matrix.sh

# Native Metal deliberately has no WebGPU-style MSAA execution mode. Keep this
# green negative contract so a future harness cannot silently relabel Dawn
# output or attempt to construct an unsupported native Metal pipeline.
renderer-metal-msaa-contract: renderer-metal-reference-check
	mkdir -p "$(RENDERER_METAL_TRACER_OUTPUT_DIR)"
	@if "$(RENDERER_METAL_REFERENCE_REPLAY)" --stream "$(CURDIR)/fixtures/renderer/streams/first-light-rectangle.rive-stream" --output "$(RENDERER_METAL_TRACER_OUTPUT_DIR)/first-light-rectangle-msaa-reference.png" --backend ffi-metal --mode msaa >"$(RENDERER_METAL_TRACER_OUTPUT_DIR)/msaa-contract.log" 2>&1; then echo "native Metal unexpectedly accepted the WebGPU MSAA mode" >&2; exit 1; fi
	@grep -Fq 'native Metal does not implement `msaa`' "$(RENDERER_METAL_TRACER_OUTPUT_DIR)/msaa-contract.log" || { cat "$(RENDERER_METAL_TRACER_OUTPUT_DIR)/msaa-contract.log" >&2; exit 1; }
	@echo "native Metal correctly rejected the WebGPU MSAA mode"

# Compatibility alias for existing local invocations.
renderer-metal-msaa-probe: renderer-metal-msaa-contract

# Bootstrap the pinned Dawn checkout and the exact static C++ archives consumed
# by nuxie-renderer-ffi. `gclient`, `gn`, `ninja`, `premake5`, and Naga 30 must
# already be on PATH; CI supplies them with pinned revisions.
renderer-dawn-reference-bootstrap:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" RIVE_ATLAS_MASK_JOBS="$(RENDERER_JOBS)" tools/renderer-dawn-reference-bootstrap.sh

renderer-dawn-reference-replay:
	MACOSX_DEPLOYMENT_TARGET=12.0 RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" CARGO_TARGET_DIR="$(RENDERER_DAWN_REFERENCE_BUILD_DIR)" cargo build --quiet --locked --release -p renderer-replay --no-default-features --features perf-dawn --bin renderer-replay
	mkdir -p "$(RENDERER_DAWN_REFERENCE_DIR)"
	cp "$(RENDERER_DAWN_REFERENCE_BUILD_DIR)/release/renderer-replay" "$(RENDERER_DAWN_REFERENCE_REPLAY)"
	chmod 0755 "$(RENDERER_DAWN_REFERENCE_REPLAY)"

renderer-dawn-reference-check:
	@test -x "$(RENDERER_DAWN_REFERENCE_REPLAY)" || { echo "missing executable C++ Dawn reference replay: $(RENDERER_DAWN_REFERENCE_REPLAY)" >&2; exit 2; }
	@if otool -L "$(RENDERER_DAWN_REFERENCE_REPLAY)" | tail -n +2 | grep -Eiq 'dawn|webgpu'; then echo "C++ Dawn reference replay unexpectedly requires a non-system Dawn/WebGPU dynamic library" >&2; exit 2; fi

renderer-dawn-live-reference-bootstrap:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" RIVE_DAWN_LIVE_JOBS="$(RENDERER_JOBS)" tools/renderer-dawn-live-reference-bootstrap.sh

renderer-dawn-live-reference-replay:
	MACOSX_DEPLOYMENT_TARGET=12.0 RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" CARGO_TARGET_DIR="$(RENDERER_DAWN_LIVE_REFERENCE_BUILD_DIR)" cargo build --quiet --locked --release -p renderer-replay --no-default-features --features perf-dawn --bin renderer-replay
	mkdir -p "$(RENDERER_DAWN_LIVE_REFERENCE_DIR)"
	cp "$(RENDERER_DAWN_LIVE_REFERENCE_BUILD_DIR)/release/renderer-replay" "$(RENDERER_DAWN_LIVE_REFERENCE_REPLAY)"
	chmod 0755 "$(RENDERER_DAWN_LIVE_REFERENCE_REPLAY)"

renderer-dawn-live-reference-check:
	@test -x "$(RENDERER_DAWN_LIVE_REFERENCE_REPLAY)" || { echo "missing executable current-runtime C++ Dawn reference replay: $(RENDERER_DAWN_LIVE_REFERENCE_REPLAY)" >&2; exit 2; }
	@if otool -L "$(RENDERER_DAWN_LIVE_REFERENCE_REPLAY)" | tail -n +2 | grep -Eiq 'dawn|webgpu'; then echo "current-runtime C++ Dawn reference replay unexpectedly requires a non-system Dawn/WebGPU dynamic library" >&2; exit 2; fi

renderer-golden-same-runner: renderer-rust-replay-release renderer-dawn-live-reference-check
	@actual_rows=$$(awk '$$0 == "[[entry]]" { count++ } END { print count + 0 }' "$(RENDERER_CORPUS_MANIFEST)"); test "$$actual_rows" = "$(RENDERER_CORPUS_EXPECTED_ROWS)" || { echo "renderer corpus row count drifted: expected $(RENDERER_CORPUS_EXPECTED_ROWS), got $$actual_rows" >&2; exit 2; }
	cargo run --quiet -p pixel-compare --bin corpus-r -- --manifest "$(RENDERER_CORPUS_MANIFEST)" --replay "$(RENDERER_GOLDEN_RUST_REPLAY)" --backend rust-webgpu-exact --reference-replay "$(RENDERER_DAWN_LIVE_REFERENCE_REPLAY)" --reference-backend ffi-dawn --output-dir "$(RENDERER_SAME_RUNNER_OUTPUT_DIR)" --jobs "$(RENDERER_SAME_RUNNER_JOBS)" --replay-timeout-seconds "$(RENDERER_REPLAY_TIMEOUT_SECONDS)"

renderer-stub-baseline: renderer-replay
	cargo run --quiet -p pixel-compare --bin corpus-r -- --replay "$(CURDIR)/target/debug/renderer-replay" --backend stub --output-dir target/renderer-stub-corpus --jobs "$(RENDERER_JOBS)" --replay-timeout-seconds "$(RENDERER_REPLAY_TIMEOUT_SECONDS)" --expect-all-fail

perf-compare: CPP_CONFIG=release
perf-compare: RUST_PROFILE=release
perf-compare: golden-runner rust-golden-runner
	GOLDEN_RUNNER="$(GOLDEN_RUNNER)" RUST_GOLDEN_RUNNER="$(RUST_GOLDEN_RUNNER)" RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" cargo run --quiet -p perf-compare --bin perf-compare -- --cpp-runner "$(GOLDEN_RUNNER)" --rust-runner "$(RUST_GOLDEN_RUNNER)" --file "$(PERF_FILE)" --samples "$(PERF_SAMPLES)" --iterations "$(PERF_ITERATIONS)" --warmups "$(PERF_WARMUPS)" --runner-order "$(PERF_RUNNER_ORDER)" --aggregate "$(PERF_AGGREGATE)"

perf-corpus: CPP_CONFIG=release
perf-corpus: RUST_PROFILE=release
perf-corpus: golden-runner rust-golden-runner
	GOLDEN_RUNNER="$(GOLDEN_RUNNER)" RUST_GOLDEN_RUNNER="$(RUST_GOLDEN_RUNNER)" RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" cargo run --quiet -p perf-compare --bin perf-compare -- --cpp-runner "$(GOLDEN_RUNNER)" --rust-runner "$(RUST_GOLDEN_RUNNER)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)" --corpus "$(PERF_CORPUS)" $(PERF_CORPUS_SELECTION) --iterations "$(PERF_ITERATIONS)" --warmups "$(PERF_WARMUPS)" --runner-order "$(PERF_RUNNER_ORDER)" --aggregate "$(PERF_AGGREGATE)" --max-ratio "$(PERF_MAX_RATIO)"

perf-corpus-check:
	python3 "$(PERF_GATE_TOOL)" check-manifest --manifest "$(PERF_GATE_MANIFEST)" --corpus "$(PERF_CORPUS)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)"

perf-runtime-ref-check:
	@set -e; \
	actual_ref=$$(git -C "$(RIVE_RUNTIME_DIR)" rev-parse HEAD 2>/dev/null || true); \
	if [ "$$actual_ref" != "$(PERF_EXPECTED_RIVE_RUNTIME_REF)" ]; then \
		echo "perf-hot-loop requires a rive-runtime checkout at $(PERF_EXPECTED_RIVE_RUNTIME_REF); got $${actual_ref:-not-a-git-checkout} from $(RIVE_RUNTIME_DIR)" >&2; \
		exit 1; \
	fi; \
	if ! git -C "$(RIVE_RUNTIME_DIR)" diff --quiet --ignore-submodules -- || \
	   ! git -C "$(RIVE_RUNTIME_DIR)" diff --cached --quiet --ignore-submodules --; then \
		echo "perf-hot-loop requires a clean tracked rive-runtime checkout: $(RIVE_RUNTIME_DIR)" >&2; \
		exit 1; \
	fi

perf-hot-loop: CPP_CONFIG=release
perf-hot-loop: RUST_PROFILE=release
perf-hot-loop: perf-runtime-ref-check golden-runner rust-golden-runner
	GOLDEN_RUNNER="$(GOLDEN_RUNNER)" RUST_GOLDEN_RUNNER="$(RUST_GOLDEN_RUNNER)" RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" cargo run --quiet -p perf-compare --bin perf-compare -- --cpp-runner "$(GOLDEN_RUNNER)" --rust-runner "$(RUST_GOLDEN_RUNNER)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)" --corpus "$(PERF_CORPUS)" $(PERF_CORPUS_SELECTION) --iterations "$(PERF_ITERATIONS)" --warmups "$(PERF_WARMUPS)" --runner-order "$(PERF_RUNNER_ORDER)" --aggregate "$(PERF_AGGREGATE)" --max-ratio "$(PERF_MAX_RATIO)" --runner-benchmark --benchmark-repeat "$(PERF_BENCHMARK_REPEAT)" --json "$(PERF_JSON_OUT)" $(PERF_JSON_META)

perf-json: CPP_CONFIG=release
perf-json: RUST_PROFILE=release
perf-json: golden-runner rust-golden-runner
	GOLDEN_RUNNER="$(GOLDEN_RUNNER)" RUST_GOLDEN_RUNNER="$(RUST_GOLDEN_RUNNER)" RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" cargo run --quiet -p perf-compare --bin perf-compare -- --cpp-runner "$(GOLDEN_RUNNER)" --rust-runner "$(RUST_GOLDEN_RUNNER)" --file "$(PERF_FILE)" --samples "$(PERF_SAMPLES)" --iterations "$(PERF_ITERATIONS)" --warmups "$(PERF_WARMUPS)" --runner-order "$(PERF_RUNNER_ORDER)" --aggregate "$(PERF_AGGREGATE)" --runner-benchmark --benchmark-repeat "$(PERF_BENCHMARK_REPEAT)" --json "$(PERF_JSON_OUT)" $(PERF_JSON_META)
	@echo "perf-json wrote $(PERF_JSON_OUT)"

perf-gate-measure: CPP_CONFIG=release
perf-gate-measure: RUST_PROFILE=release
perf-gate-measure: perf-runtime-ref-check perf-corpus-check scripted-golden-runner scripted-rust-golden-runner
	@set -e; \
	tools/perf-gate/wait-for-quiet.sh; \
	cargo build --quiet --release -p perf-compare --bin perf-compare; \
	ids=$$(python3 "$(PERF_GATE_TOOL)" ids --manifest "$(PERF_GATE_MANIFEST)" --corpus "$(PERF_CORPUS)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)"); \
	mkdir -p "$(dir $(PERF_GATE_REPORT))"; \
	"$(PERF_GATE_PINNER)" "$(PERF_GATE_COMPARE)" --cpp-runner "$(SCRIPTED_GOLDEN_RUNNER)" --rust-runner "$(SCRIPTED_RUST_GOLDEN_RUNNER)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)" --corpus "$(PERF_CORPUS)" --corpus-ids "$$ids" --iterations "$(PERF_GATE_ITERATIONS)" --warmups "$(PERF_GATE_WARMUPS)" --aggregate median --runner-order cpp-first --runner-benchmark --benchmark-frames "$(PERF_GATE_FRAMES)" --benchmark-hz "$(PERF_GATE_HZ)" --rust-execute-scripts --json "$(PERF_GATE_REPORT)" $(PERF_JSON_META)

perf-gate: perf-gate-measure
	python3 "$(PERF_GATE_TOOL)" check-report --manifest "$(PERF_GATE_MANIFEST)" --corpus "$(PERF_CORPUS)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)" --report "$(PERF_GATE_REPORT)"

perf-gate-tighten:
	$(MAKE) perf-gate-measure PERF_GATE_REPORT="$(PERF_GATE_REPORT)"
	$(MAKE) perf-gate-measure PERF_GATE_REPORT="$(PERF_GATE_TIGHTEN_REPORT_2)"
	$(MAKE) perf-gate-measure PERF_GATE_REPORT="$(PERF_GATE_TIGHTEN_REPORT_3)"
	python3 "$(PERF_GATE_TOOL)" tighten --manifest "$(PERF_GATE_MANIFEST)" --corpus "$(PERF_CORPUS)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)" --report "$(PERF_GATE_REPORT)" --report "$(PERF_GATE_TIGHTEN_REPORT_2)" --report "$(PERF_GATE_TIGHTEN_REPORT_3)"

capi-smoke: fixtures
	cargo build --quiet -p nux-capi
	mkdir -p target/capi-smoke
	$(CC) -std=c11 -Wall -Wextra -Werror -Icrates/nux-capi/include -o target/capi-smoke/capi_smoke crates/nux-capi/smoke/capi_smoke.c -Ltarget/debug -lnux_capi
	DYLD_LIBRARY_PATH=target/debug LD_LIBRARY_PATH=target/debug target/capi-smoke/capi_smoke "$(CAPI_SMOKE_FIXTURE)"
	@if [ "$$(uname -s)" = Darwin ]; then \
		$(CC) -std=c11 -Wall -Wextra -Werror -Icrates/nux-capi/include \
			-o target/capi-smoke/capi_smoke_static crates/nux-capi/smoke/capi_smoke.c \
			target/debug/libnux_capi.a -framework CoreFoundation -framework CoreGraphics -framework ImageIO; \
		target/capi-smoke/capi_smoke_static "$(CAPI_SMOKE_FIXTURE)"; \
		xcrun swiftc -I crates/nux-capi/include crates/nux-capi/smoke/capi_lifetime.swift \
			target/debug/libnux_capi.a -framework CoreGraphics -framework ImageIO \
			-o target/capi-smoke/capi_lifetime; \
		target/capi-smoke/capi_lifetime "$(CAPI_SMOKE_FIXTURE)"; \
	else \
		$(CC) -std=c11 -Wall -Wextra -Werror -Icrates/nux-capi/include \
			-o target/capi-smoke/capi_smoke_static crates/nux-capi/smoke/capi_smoke.c \
			target/debug/libnux_capi.a -ldl -lpthread -lm; \
		target/capi-smoke/capi_smoke_static "$(CAPI_SMOKE_FIXTURE)"; \
	fi
	tools/check-nux-capi-exports.sh

nux-capi-layout-contract:
	tools/check-nux-capi-layout.py

nux-capi-surface-contract:
	tools/check-nux-capi-surface.py

nux-capi-distribution-contract-test:
	@tools/report-all.sh "nux-capi-distribution-unit" \
		"distribution tooling tests" "PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools/test_nux_capi_distribution.py" \
		"release contract tests" "PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools/test_apple_runtime_contract.py" \
		"input digest tests" "PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools/test_apple_runtime_input_digest.py" \
		"slim runtime tests" "PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools/test_slim_runtime_distribution.py"

nux-capi-distribution-contract-gate:
	@tools/report-all.sh "nux-capi-distribution-contract" \
		"ABI layout contract" "$(MAKE) --no-print-directory nux-capi-layout-contract" \
		"shipped surface contract" "$(MAKE) --no-print-directory nux-capi-surface-contract" \
		"distribution unit contracts" "$(MAKE) --no-print-directory nux-capi-distribution-contract-test"

nux-capi-pr-gate:
	@tools/report-all.sh "nux-capi-pr" \
		"portable C ABI smoke" "$(MAKE) --no-print-directory capi-smoke" \
		"distribution contracts" "$(MAKE) --no-print-directory nux-capi-distribution-contract-gate"

nux-capi-distribution-plan:
	tools/build-nux-capi-xcframeworks.sh --plan

nux-capi-xcframeworks:
	tools/build-nux-capi-xcframeworks.sh

nux-capi-android-contract-test:
	PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools/test_android_runtime_contract.py
	tools/build-nux-capi-android.sh --plan
	tools/publish-nux-capi-android-release.sh --plan

nux-capi-android-plan:
	tools/build-nux-capi-android.sh --plan

nux-capi-android:
	tools/build-nux-capi-android.sh

cpp-binary-compare: cpp-probe
	RIVE_CPP_PROBE="$(CPP_PROBE)" RIVE_CPP_CORPUS=1 cargo test -p nuxie-binary --test cpp_import -- --nocapture
	RIVE_CPP_PROBE="$(CPP_PROBE)" cargo test -p nuxie-runtime --test profiler_cpp_probe -- --nocapture

cpp-runtime-compare: cpp-probe
	RIVE_CPP_PROBE="$(CPP_PROBE)" cargo test -p nuxie-runtime --features tools --tests -- --nocapture

cpp-compare: cpp-binary-compare cpp-runtime-compare

.PHONY: fuzz-build fuzz-smoke fuzz fuzz-regressions

# --- Negative-input fuzzing (cargo-fuzz, requires the nightly toolchain) -----
# cargo-fuzz spawns its own `cargo`/`rustc`, so pointing it at nightly via a
# `+toolchain` proxy is not enough when the default `cargo` is a non-rustup
# build. FUZZ_CARGO wraps the invocation so the whole tree builds with nightly.
# In CI, where nightly is the default toolchain, override with FUZZ_CARGO=cargo.
FUZZ_DIR ?= fuzz
FUZZ_CARGO ?= rustup run nightly cargo
FUZZ_TARGETS ?= fuzz_import fuzz_runtime fuzz_pointer
FUZZ_SMOKE_SECONDS ?= 30
FUZZ_TARGET ?= fuzz_runtime
FUZZ_SECONDS ?= 300
FUZZ_RSS_LIMIT_MB ?= 4096
# Per-input wall clock. A libFuzzer -timeout is what turns a hang into a
# reported finding; keep it modest so the smoke gate cannot wedge.
FUZZ_TIMEOUT ?= 25

# Build every libfuzzer target (also the CI "build-only" gate).
fuzz-build: fixtures
	cd $(FUZZ_DIR) && $(FUZZ_CARGO) fuzz build

# `make fixtures` materializes the pinned upstream seed corpus under
# fuzz/seeds/<target>/; libFuzzer's writable working corpus
# (fuzz/corpus/<target>/) and crash artifacts are gitignored.
# NOTE: the smoke gate deliberately does NOT replay fuzz/regressions/ -- that
# tree also archives reproducers for KNOWN-OPEN findings (see
# fuzz/regressions/README.md), which would wedge the gate. Fixed-bug
# regressions are checked explicitly via `make fuzz-regressions`.

# Smoke gate: build all targets, then exercise each with a short timed mutation
# burst. This is a gate that proves the harness builds and runs end-to-end -- it
# is NOT a full fuzzing campaign (use `make fuzz` for that).
#
# NOTE: fuzz_runtime/fuzz_pointer were temporarily switched to a deterministic
# seed replay (-runs=0) while the runtime pipeline had open input-dependent HANG
# findings (unbounded parent/reference-chain walks) that a timed mutation run
# rediscovered within seconds. Those cycle-guard findings are now FIXED (see
# fuzz/regressions/README.md), so all targets are back to
# timed mutation. The -timeout guard still turns any residual hang into a hard
# failure.
fuzz-smoke: fuzz-build
	@set -e; for target in $(FUZZ_TARGETS); do \
		echo "== fuzz-smoke: $$target (timed $(FUZZ_SMOKE_SECONDS)s) =="; \
		( cd $(FUZZ_DIR) && mkdir -p corpus/$$target && $(FUZZ_CARGO) fuzz run \
			$$target corpus/$$target seeds/$$target -- \
			-max_total_time=$(FUZZ_SMOKE_SECONDS) -timeout=$(FUZZ_TIMEOUT) \
			-rss_limit_mb=$(FUZZ_RSS_LIMIT_MB) ); \
	done

# Longer local campaign for a single target. Example:
#   make fuzz FUZZ_TARGET=fuzz_runtime FUZZ_SECONDS=1800
# Crash/timeout reproducers land in fuzz/artifacts/<target>/. To keep one as a
# regression, minimize it (`cargo fuzz tmin`) and copy it into
# fuzz/regressions/<target>/ (fixed) or fuzz/regressions/open/ (still failing).
fuzz:
	cd $(FUZZ_DIR) && mkdir -p corpus/$(FUZZ_TARGET) && $(FUZZ_CARGO) fuzz run \
		$(FUZZ_TARGET) corpus/$(FUZZ_TARGET) seeds/$(FUZZ_TARGET) -- \
		-max_total_time=$(FUZZ_SECONDS) -timeout=$(FUZZ_TIMEOUT) \
		-rss_limit_mb=$(FUZZ_RSS_LIMIT_MB)

# Replay committed regression reproducers for FIXED bugs (fuzz/regressions/
# <target>/). Reproducers for still-open findings live under
# fuzz/regressions/open/ and are intentionally excluded here.
fuzz-regressions: fuzz-build
	@set -e; for target in $(FUZZ_TARGETS); do \
		if [ -d "$(FUZZ_DIR)/regressions/$$target" ] && \
			[ -n "$$(ls -A $(FUZZ_DIR)/regressions/$$target 2>/dev/null)" ]; then \
			echo "== fuzz-regressions: $$target =="; \
			( cd $(FUZZ_DIR) && $(FUZZ_CARGO) fuzz run $$target \
				regressions/$$target -- -runs=0 -timeout=$(FUZZ_TIMEOUT) \
				-rss_limit_mb=$(FUZZ_RSS_LIMIT_MB) ); \
		fi; \
	done
