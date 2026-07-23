.PHONY: fixtures schema check test inspect graph cpp-probe cpp-atlas-mask-oracle cpp-atlas-mask-oracle-preflight golden-runner scripted-golden-runner rust-golden-runner scripted-rust-golden-runner golden-compare scripted-golden-compare cpp-oracle-workspace-tests renderer-replay renderer-references renderer-shaders-check renderer-wgpu-backend-check renderer-wgpu-consumer-check renderer-decoder-oracle renderer-fuzz-replay renderer-golden renderer-rust-replay-release renderer-dawn-reference-bootstrap renderer-dawn-reference-replay renderer-dawn-reference-check renderer-dawn-live-reference-bootstrap renderer-dawn-live-reference-replay renderer-dawn-live-reference-check renderer-golden-same-runner renderer-stub-baseline renderer-perf-runners renderer-perf renderer-perf-parity-gate r4-timing-gate r4-timing-gate-tools renderer-counter-runners perf-counter-compare perf-compare perf-corpus perf-runtime-ref-check perf-hot-loop perf-json browser-renderer-build browser-renderer-smoke browser-renderer-gpu-smoke capi-smoke apple-runtime-check apple-runtime-header-smoke apple-runtime-release-panic-smoke apple-runtime-xcframework size-report parity-scorecard parity-scorecard-test cpp-binary-compare cpp-graph-compare cpp-runtime-compare cpp-compare runtime-drawing-port-test runtime-drawing-port-check runtime-drawing-port-closed

RIVE_RUNTIME_DIR ?= /Users/levi/dev/oss/rive-runtime
DEFS_DIR ?= $(RIVE_RUNTIME_DIR)/dev/defs
PORT_MANIFEST ?= $(CURDIR)/port-manifest.toml
PORT_MANIFEST_TOOL ?= $(CURDIR)/tools/port-manifest/port_manifest.py
PORT_MANIFEST_UPSTREAM_REF ?= $(shell git -C "$(RIVE_RUNTIME_DIR)" rev-parse HEAD 2>/dev/null)
RUNTIME_DRAWING_PORT_TOOL ?= $(CURDIR)/tools/runtime-drawing-port/check.py
RUNTIME_DRAWING_OWNERSHIP ?= $(CURDIR)/docs/runtime-drawing-ownership.toml
RUNTIME_DRAWING_GAPS ?= $(CURDIR)/docs/runtime-drawing-gaps.toml
PARITY_SCORECARD_TOOL ?= $(CURDIR)/tools/parity-scorecard/parity_scorecard.py
PARITY_SCORECARD_EVIDENCE_DIR ?= $(CURDIR)/target/parity-scorecard/evidence
PARITY_SCORECARD_JSON ?= $(CURDIR)/target/parity-scorecard/scorecard.json
CPP_CONFIG ?= debug
RUST_PROFILE ?= debug
RUST_GOLDEN_RUNNER_FLAGS = $(if $(filter release,$(RUST_PROFILE)),--release,)
RENDERER_JOBS ?= 1
RENDERER_SAME_RUNNER_JOBS ?= 1
RENDERER_REPLAY_TIMEOUT_SECONDS ?= 60
RENDERER_GOLDEN_TARGET_DIR ?= $(CURDIR)/target/renderer-golden
RENDERER_GOLDEN_RUST_REPLAY ?= $(RENDERER_GOLDEN_TARGET_DIR)/release/renderer-replay
RENDERER_DAWN_REFERENCE_BUILD_DIR ?= $(CURDIR)/target/renderer-dawn-reference-build
RENDERER_DAWN_REFERENCE_DIR ?= $(CURDIR)/target/renderer-dawn-reference
RENDERER_DAWN_REFERENCE_REPLAY ?= $(RENDERER_DAWN_REFERENCE_DIR)/renderer-replay
RENDERER_DAWN_LIVE_REFERENCE_BUILD_DIR ?= $(CURDIR)/target/renderer-dawn-live-reference-build
RENDERER_DAWN_LIVE_REFERENCE_DIR ?= $(CURDIR)/target/renderer-dawn-live-reference
RENDERER_DAWN_LIVE_REFERENCE_REPLAY ?= $(RENDERER_DAWN_LIVE_REFERENCE_DIR)/renderer-replay
RENDERER_SAME_RUNNER_OUTPUT_DIR ?= $(CURDIR)/target/renderer-same-runner-corpus
RENDERER_CORPUS_MANIFEST ?= $(CURDIR)/corpus-r.toml
RENDERER_CORPUS_EXPECTED_ROWS ?= 1468
CPP_PROBE ?= $(CURDIR)/tools/cpp-probe/build/$(shell uname -s | tr A-Z a-z | sed 's/darwin/macosx/')/bin/$(CPP_CONFIG)/rive_cpp_probe
GOLDEN_RUNNER ?= $(CURDIR)/tools/golden-runner/build/$(shell uname -s | tr A-Z a-z | sed 's/darwin/macosx/')/bin/$(CPP_CONFIG)/rive_golden_runner
SCRIPTED_GOLDEN_RUNNER ?= $(CURDIR)/tools/golden-runner/build/$(shell uname -s | tr A-Z a-z | sed 's/darwin/macosx/')/bin/$(CPP_CONFIG)/rive_golden_runner_scripted
RUST_GOLDEN_RUNNER ?= $(CURDIR)/target/$(RUST_PROFILE)/rust-golden-runner
SCRIPTED_RUST_GOLDEN_RUNNER ?= $(CURDIR)/target/$(RUST_PROFILE)/rust-golden-runner-scripted
PERF_FILE ?= $(RIVE_RUNTIME_DIR)/tests/unit_tests/assets/shapetest.riv
PERF_SAMPLES ?= 0
PERF_ITERATIONS ?= 10
PERF_WARMUPS ?= 3
PERF_RUNNER_ORDER ?= cpp-first
PERF_CORPUS ?= corpus.toml
PERF_CORPUS_LIMIT ?= 10
PERF_CORPUS_IDS ?= advance_blend_mode,ai_assitant,align_target,animated_clipping,animation_reset_cases,spotify_kids_demo
PERF_CORPUS_SELECTION = $(if $(strip $(PERF_CORPUS_IDS)),--corpus-ids "$(PERF_CORPUS_IDS)",--corpus-limit "$(PERF_CORPUS_LIMIT)")
PERF_AGGREGATE ?= min
PERF_MAX_RATIO ?= 1.0
PERF_BENCHMARK_REPEAT ?= 10000
PERF_JSON_OUT ?= $(CURDIR)/target/perf-compare.json
PERF_JSON_META ?= --meta build_profile=release --meta git_sha=$(shell git rev-parse HEAD 2>/dev/null || echo unknown) --meta timestamp=$(shell date -u +%Y-%m-%dT%H:%M:%SZ)
PERF_EXPECTED_RIVE_RUNTIME_REF ?= d788e8ec6e8b598526607d6a1e8818e8b637b60c
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
R4_TIMING_GATE_OUT_DIR ?=
R4_TIMING_GATE_RENDERER_PERF ?= $(CURDIR)/target/release/renderer-perf
R4_TIMING_GATE_COMPARATOR ?= $(CURDIR)/target/release/r4-timing-compare
R4_TIMING_GATE_MANIFEST ?= $(CURDIR)/tools/perf-compare/renderer-scenes.toml
R4_TIMING_GATE_BASELINE_RUNNER ?= $(RENDERER_PERF_CPP_RUNNER)
R4_TIMING_GATE_A_RUNNER ?=
R4_TIMING_GATE_B_RUNNER ?=
R4_TIMING_GATE_RENDERER_PERF_MAX_RATIO ?= 1.0
R4_TIMING_GATE_CAPTURE_MAX_RATIO ?= 1000
R4_TIMING_GATE_MAX_B_OVER_A ?= 1.0
R4_TIMING_GATE_MAX_CONTROL_DRIFT ?= 1.05
R4_TIMING_GATE_MAX_REPEAT_DRIFT ?= 1.05
R4_TIMING_GATE_HOST_SAMPLER ?=
R4_TIMING_GATE_BASELINE_SOURCE_ID ?=
R4_TIMING_GATE_A_SOURCE_ID ?=
R4_TIMING_GATE_B_SOURCE_ID ?=

export R4_TIMING_GATE_OUT_DIR R4_TIMING_GATE_RENDERER_PERF R4_TIMING_GATE_COMPARATOR R4_TIMING_GATE_MANIFEST
export R4_TIMING_GATE_BASELINE_RUNNER R4_TIMING_GATE_A_RUNNER R4_TIMING_GATE_B_RUNNER
export R4_TIMING_GATE_RENDERER_PERF_MAX_RATIO R4_TIMING_GATE_MAX_B_OVER_A R4_TIMING_GATE_MAX_CONTROL_DRIFT
export R4_TIMING_GATE_CAPTURE_MAX_RATIO
export R4_TIMING_GATE_MAX_REPEAT_DRIFT
export R4_TIMING_GATE_HOST_SAMPLER
export R4_TIMING_GATE_BASELINE_SOURCE_ID R4_TIMING_GATE_A_SOURCE_ID R4_TIMING_GATE_B_SOURCE_ID
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

check:
	cargo check --workspace

test: fixtures
	cargo test --workspace

.PHONY: port-manifest-generate port-manifest-test port-manifest-check
port-manifest-generate:
	python3 "$(PORT_MANIFEST_TOOL)" generate --rive-runtime-dir "$(RIVE_RUNTIME_DIR)" --upstream-ref "$(PORT_MANIFEST_UPSTREAM_REF)" --output "$(PORT_MANIFEST)"

port-manifest-test:
	PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools/port-manifest -p 'test_*.py' -v

port-manifest-check: port-manifest-test
	python3 "$(PORT_MANIFEST_TOOL)" check --rive-runtime-dir "$(RIVE_RUNTIME_DIR)" --upstream-ref "$(PORT_MANIFEST_UPSTREAM_REF)" --repo-root "$(CURDIR)" --manifest "$(PORT_MANIFEST)"

runtime-drawing-port-test:
	PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools/runtime-drawing-port -p 'test_*.py' -v

runtime-drawing-port-check: runtime-drawing-port-test
	PYTHONDONTWRITEBYTECODE=1 python3 "$(RUNTIME_DRAWING_PORT_TOOL)" --repo-root "$(CURDIR)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)" --ledger "$(RUNTIME_DRAWING_OWNERSHIP)" --gaps "$(RUNTIME_DRAWING_GAPS)"

runtime-drawing-port-closed: runtime-drawing-port-test
	PYTHONDONTWRITEBYTECODE=1 python3 "$(RUNTIME_DRAWING_PORT_TOOL)" --repo-root "$(CURDIR)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)" --ledger "$(RUNTIME_DRAWING_OWNERSHIP)" --gaps "$(RUNTIME_DRAWING_GAPS)" --require-closed

# --- Clippy lint gate (panic-freedom discipline, v2-status item 20 #6) -------
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
LINT_GATE_WARN_CRATES = nuxie-runtime nuxie-binary nuxie-graph nux-capi

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

inspect:
	@cargo run --quiet -p nuxie-binary --bin riv-inspect -- fixtures/graph/dependency_test.riv

graph:
	@cargo run --quiet -p nuxie-graph --bin graph-inspect -- fixtures/graph/dependency_test.riv

cpp-probe:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" tools/cpp-probe/build.sh "$(CPP_CONFIG)"

cpp-atlas-mask-oracle-preflight:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" tools/cpp-atlas-mask-oracle/build.sh --preflight

cpp-atlas-mask-oracle:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" tools/cpp-atlas-mask-oracle/build.sh

golden-runner:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" tools/golden-runner/build.sh "$(CPP_CONFIG)"

scripted-golden-runner:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" RIVE_GOLDEN_WITH_SCRIPTING=1 RIVE_GOLDEN_RUNNER_NAME=rive_golden_runner_scripted tools/golden-runner/build.sh "$(CPP_CONFIG)"

rust-golden-runner:
	rm -f "$(RUST_GOLDEN_RUNNER)"
	cargo build --quiet $(RUST_GOLDEN_RUNNER_FLAGS) -p rust-golden-runner

scripted-rust-golden-runner:
	rm -f "$(RUST_GOLDEN_RUNNER)"
	cargo build --quiet $(RUST_GOLDEN_RUNNER_FLAGS) -p rust-golden-runner --features scripting
	cp "$(RUST_GOLDEN_RUNNER)" "$(SCRIPTED_RUST_GOLDEN_RUNNER)"

golden-compare: fixtures golden-runner rust-golden-runner
	GOLDEN_RUNNER="$(GOLDEN_RUNNER)" RUST_GOLDEN_RUNNER="$(RUST_GOLDEN_RUNNER)" RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" cargo run --quiet -p golden-compare --bin golden-compare -- --corpus corpus.toml --cpp-runner "$(GOLDEN_RUNNER)" --rust-runner "$(RUST_GOLDEN_RUNNER)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)"

scripted-golden-compare: CPP_CONFIG=release
scripted-golden-compare: fixtures scripted-golden-runner scripted-rust-golden-runner
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" cargo run --quiet -p golden-compare --bin golden-compare -- --corpus corpus.toml --verify-unsupported-cpp --verify-divergent-rust --verify-scripted-diagnostics --cpp-runner "$(SCRIPTED_GOLDEN_RUNNER)" --rust-runner "$(SCRIPTED_RUST_GOLDEN_RUNNER)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)"

cpp-oracle-workspace-tests: fixtures golden-runner cpp-probe
	@test -x "$(GOLDEN_RUNNER)" || { echo "missing executable pinned C++ golden runner: $(GOLDEN_RUNNER)" >&2; exit 2; }
	@test -x "$(CPP_PROBE)" || { echo "missing executable pinned C++ probe: $(CPP_PROBE)" >&2; exit 2; }
	RIVE_GOLDEN_RUNNER="$(GOLDEN_RUNNER)" RIVE_CPP_PROBE="$(CPP_PROBE)" cargo test --workspace

renderer-replay:
	cargo build --quiet -p renderer-replay

renderer-references:
	CARGO_TARGET_DIR="$(CURDIR)/target/renderer-ffi" cargo build --quiet -p renderer-replay --features ffi
	CARGO_TARGET_DIR="$(CURDIR)/target/renderer-ffi" cargo run --quiet -p pixel-compare --bin capture-corpus-r-references -- --replay "$(CURDIR)/target/renderer-ffi/debug/renderer-replay"

renderer-shaders-check:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" tools/check-renderer-shaders.sh

# Exercise the focused invariants and transitive feature wiring across the
# excluded, pinned wgpu packages. Their committed lockfiles keep this check
# reproducible without mutating the vendored source directories.
renderer-wgpu-backend-check:
	CARGO_TARGET_DIR="$(CURDIR)/target" cargo check --locked --manifest-path vendor/wgpu-30.0.0/Cargo.toml --no-default-features --features std,metal,wgsl
	CARGO_TARGET_DIR="$(CURDIR)/target" cargo test --locked --manifest-path vendor/wgpu-hal-30.0.0/Cargo.toml --lib --features metal coalescing
	CARGO_TARGET_DIR="$(CURDIR)/target" cargo test --locked --manifest-path vendor/wgpu-hal-30.0.0/Cargo.toml --lib --features metal invariant_position
	CARGO_TARGET_DIR="$(CURDIR)/target" cargo test --locked --manifest-path vendor/wgpu-core-30.0.0/Cargo.toml --lib command_buffer

renderer-wgpu-consumer-check:
	tools/check-renderer-wgpu-consumer.sh

renderer-decoder-oracle:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" tools/check-renderer-decoder-provenance.sh
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" CARGO_INCREMENTAL=0 cargo test -p nuxie-renderer-ffi --features decode-oracle --test decode_oracle -- --nocapture

renderer-fuzz-replay:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" CARGO_TARGET_DIR="$(CURDIR)/target/renderer-ffi" cargo build --quiet -p renderer-replay --features ffi
	cargo run --quiet -p renderer-fuzz-replay -- --replay "$(CURDIR)/target/renderer-ffi/debug/renderer-replay"

renderer-golden: renderer-replay
	cargo run --quiet -p pixel-compare --bin corpus-r -- --replay "$(CURDIR)/target/debug/renderer-replay" --backend rust-wgpu --jobs "$(RENDERER_JOBS)" --replay-timeout-seconds "$(RENDERER_REPLAY_TIMEOUT_SECONDS)"

# The same-runner gate deliberately keeps the live reference and candidate
# builds separate. CI may restore only RENDERER_DAWN_LIVE_REFERENCE_REPLAY from
# its exact pinned-input cache; the Rust candidate below is always compiled
# from HEAD. The historical RENDERER_DAWN_REFERENCE_REPLAY remains isolated for
# the immutable Phase R oracle and is never relabeled as current-runtime output.
renderer-rust-replay-release:
	CARGO_TARGET_DIR="$(RENDERER_GOLDEN_TARGET_DIR)" cargo build --quiet --locked --release -p renderer-replay --bin renderer-replay

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
	cargo run --quiet -p pixel-compare --bin corpus-r -- --manifest "$(RENDERER_CORPUS_MANIFEST)" --replay "$(RENDERER_GOLDEN_RUST_REPLAY)" --backend rust-wgpu --reference-replay "$(RENDERER_DAWN_LIVE_REFERENCE_REPLAY)" --reference-backend ffi-dawn --output-dir "$(RENDERER_SAME_RUNNER_OUTPUT_DIR)" --jobs "$(RENDERER_SAME_RUNNER_JOBS)" --replay-timeout-seconds "$(RENDERER_REPLAY_TIMEOUT_SECONDS)"

renderer-stub-baseline: renderer-replay
	cargo run --quiet -p pixel-compare --bin corpus-r -- --replay "$(CURDIR)/target/debug/renderer-replay" --backend stub --output-dir target/renderer-stub-corpus --jobs "$(RENDERER_JOBS)" --replay-timeout-seconds "$(RENDERER_REPLAY_TIMEOUT_SECONDS)" --expect-all-fail

renderer-perf-runners:
	MACOSX_DEPLOYMENT_TARGET=12.0 RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" CARGO_TARGET_DIR="$(RENDERER_PERF_TARGET_DIR)" cargo build --release -p renderer-replay --features perf-dawn --bin renderer-perf-cpp-runner --bin renderer-perf-rust-runner

renderer-perf: renderer-perf-runners
	@test -n "$(strip $(RENDERER_PERF_BASELINE_SOURCE_ID))" || { echo "RENDERER_PERF_BASELINE_SOURCE_ID is required (identify the baseline source revision)" >&2; exit 2; }
	@test -n "$(strip $(RENDERER_PERF_CANDIDATE_SOURCE_ID))" || { echo "RENDERER_PERF_CANDIDATE_SOURCE_ID is required (use a reconstructable base+dirty source digest)" >&2; exit 2; }
	cargo run --quiet -p perf-compare --bin renderer-perf -- --manifest tools/perf-compare/renderer-scenes.toml --baseline-runner "$(RENDERER_PERF_CPP_RUNNER)" --candidate-runner "$(RENDERER_PERF_RUST_RUNNER)" --baseline-source-id "$(RENDERER_PERF_BASELINE_SOURCE_ID)" --candidate-source-id "$(RENDERER_PERF_CANDIDATE_SOURCE_ID)" --max-ratio "$(RENDERER_PERF_MAX_RATIO)" --json "$(RENDERER_PERF_JSON)" --markdown "$(RENDERER_PERF_MARKDOWN)"

renderer-perf-parity-gate:
	cargo run --quiet -p perf-compare --bin renderer-perf-parity-gate -- --report "$(RENDERER_PERF_PARITY_REPORT_1)" --report "$(RENDERER_PERF_PARITY_REPORT_2)" --report "$(RENDERER_PERF_PARITY_REPORT_3)" --report "$(RENDERER_PERF_PARITY_REPORT_4)" --report "$(RENDERER_PERF_PARITY_REPORT_5)" --max-ratio "$(RENDERER_PERF_PARITY_MAX_RATIO)" --json "$(RENDERER_PERF_PARITY_JSON)" --markdown "$(RENDERER_PERF_PARITY_MARKDOWN)"

# Timing-defined R4 acceptance only. The gate invokes the fixed renderer-perf
# executable with pinned baseline, A, and B runner paths; it never evaluates a
# caller-provided shell command.
r4-timing-gate-tools:
	cargo build --quiet --release -p perf-compare --bin renderer-perf --bin r4-timing-compare

r4-timing-gate: r4-timing-gate-tools
	@test -n "$(strip $(R4_TIMING_GATE_BASELINE_SOURCE_ID))" || { echo "R4_TIMING_GATE_BASELINE_SOURCE_ID is required (identify the baseline source revision)" >&2; exit 2; }
	@test -n "$(strip $(R4_TIMING_GATE_A_SOURCE_ID))" || { echo "R4_TIMING_GATE_A_SOURCE_ID is required (identify the A runner source)" >&2; exit 2; }
	@test -n "$(strip $(R4_TIMING_GATE_B_SOURCE_ID))" || { echo "R4_TIMING_GATE_B_SOURCE_ID is required (identify the B runner source)" >&2; exit 2; }
	tools/r4-timing-gate.sh

renderer-counter-runners:
	MACOSX_DEPLOYMENT_TARGET=12.0 RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" CARGO_TARGET_DIR="$(RENDERER_COUNTER_TARGET_DIR)" cargo build --release -p renderer-replay --features perf-counters --bin renderer-perf-cpp-runner --bin renderer-perf-rust-runner

perf-counter-compare: renderer-counter-runners
	@test -n "$(strip $(RENDERER_COUNTER_BASELINE_SOURCE_ID))" || { echo "RENDERER_COUNTER_BASELINE_SOURCE_ID is required (identify the baseline source revision)" >&2; exit 2; }
	@test -n "$(strip $(RENDERER_COUNTER_CANDIDATE_SOURCE_ID))" || { echo "RENDERER_COUNTER_CANDIDATE_SOURCE_ID is required (use a reconstructable base+dirty source digest)" >&2; exit 2; }
	cargo run --quiet -p perf-compare --bin perf-counter-compare -- --manifest tools/perf-compare/renderer-scenes.toml --baseline-runner "$(RENDERER_COUNTER_CPP_RUNNER)" --candidate-runner "$(RENDERER_COUNTER_RUST_RUNNER)" --baseline-source-id "$(RENDERER_COUNTER_BASELINE_SOURCE_ID)" --candidate-source-id "$(RENDERER_COUNTER_CANDIDATE_SOURCE_ID)" --json "$(RENDERER_COUNTER_JSON)" --markdown "$(RENDERER_COUNTER_MARKDOWN)"

perf-compare: CPP_CONFIG=release
perf-compare: RUST_PROFILE=release
perf-compare: golden-runner rust-golden-runner
	GOLDEN_RUNNER="$(GOLDEN_RUNNER)" RUST_GOLDEN_RUNNER="$(RUST_GOLDEN_RUNNER)" RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" cargo run --quiet -p perf-compare --bin perf-compare -- --cpp-runner "$(GOLDEN_RUNNER)" --rust-runner "$(RUST_GOLDEN_RUNNER)" --file "$(PERF_FILE)" --samples "$(PERF_SAMPLES)" --iterations "$(PERF_ITERATIONS)" --warmups "$(PERF_WARMUPS)" --runner-order "$(PERF_RUNNER_ORDER)" --aggregate "$(PERF_AGGREGATE)"

perf-corpus: CPP_CONFIG=release
perf-corpus: RUST_PROFILE=release
perf-corpus: golden-runner rust-golden-runner
	GOLDEN_RUNNER="$(GOLDEN_RUNNER)" RUST_GOLDEN_RUNNER="$(RUST_GOLDEN_RUNNER)" RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" cargo run --quiet -p perf-compare --bin perf-compare -- --cpp-runner "$(GOLDEN_RUNNER)" --rust-runner "$(RUST_GOLDEN_RUNNER)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)" --corpus "$(PERF_CORPUS)" $(PERF_CORPUS_SELECTION) --iterations "$(PERF_ITERATIONS)" --warmups "$(PERF_WARMUPS)" --runner-order "$(PERF_RUNNER_ORDER)" --aggregate "$(PERF_AGGREGATE)" --max-ratio "$(PERF_MAX_RATIO)"

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

browser-renderer-build:
	tools/browser-renderer-smoke/build.sh

browser-renderer-smoke:
	tools/browser-renderer-smoke/run.sh

browser-renderer-gpu-smoke:
	BROWSER_RENDERER_GPU_ONLY=1 tools/browser-renderer-smoke/run.sh

capi-smoke: fixtures
	cargo build --quiet -p nux-capi
	mkdir -p target/capi-smoke
	$(CC) -std=c11 -Wall -Wextra -Werror -Icrates/nux-capi/include -o target/capi-smoke/capi_smoke crates/nux-capi/smoke/capi_smoke.c -Ltarget/debug -lnux_capi
	DYLD_LIBRARY_PATH=target/debug LD_LIBRARY_PATH=target/debug target/capi-smoke/capi_smoke "$(CAPI_SMOKE_FIXTURE)"

apple-runtime-header-smoke:
	cargo build --locked -p nux-apple-runtime --features apple-product
	$(CC) -std=c11 -Wall -Wextra -Werror -Icrates/nux-apple-runtime/include -fsyntax-only crates/nux-apple-runtime/smoke/header_smoke.c

apple-runtime-release-panic-smoke:
	cargo test --locked --profile release-apple -p nux-apple-runtime --features apple-product panic_firewall_converts_panics_to_the_declared_fallback

apple-runtime-check: apple-runtime-header-smoke apple-runtime-release-panic-smoke
	cargo test --locked -p nux-apple-runtime --features apple-product
	cargo clippy --locked -p nux-apple-runtime --lib --no-default-features --features apple-product --no-deps --quiet -- -D warnings

apple-runtime-xcframework:
	tools/build-apple-xcframework.sh

# SDK binary-size report: builds the post-Phase-R Darwin link closure with the
# renderer retained, for scripting off and on. Pass SIZE_BASELINE=1 to also
# build the opt-level=3 release closure. No budget is enforced until #B-3's
# USER-GATE is decided. See docs/SIZE.md.
SIZE_BASELINE ?=
size-report:
	tools/size-report.sh $(if $(SIZE_BASELINE),--baseline,)

parity-scorecard-test:
	@PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools/parity-scorecard -p 'test_*.py' -v

parity-scorecard: parity-scorecard-test
	@python3 "$(PARITY_SCORECARD_TOOL)" check --repo-root "$(CURDIR)" --evidence-dir "$(PARITY_SCORECARD_EVIDENCE_DIR)" --json "$(PARITY_SCORECARD_JSON)"

cpp-binary-compare: cpp-probe
	RIVE_CPP_PROBE="$(CPP_PROBE)" RIVE_CPP_CORPUS=1 cargo test -p nuxie-binary --test cpp_import -- --nocapture

cpp-graph-compare: cpp-probe
	RIVE_CPP_PROBE="$(CPP_PROBE)" cargo test -p nuxie-graph --test cpp_probe -- --nocapture

cpp-runtime-compare: cpp-probe
	RIVE_CPP_PROBE="$(CPP_PROBE)" cargo test -p nuxie-runtime --test cpp_probe -- --nocapture

cpp-compare: cpp-binary-compare cpp-graph-compare cpp-runtime-compare

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
# fuzz/regressions/README.md and v2-status item 27), so all targets are back to
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
