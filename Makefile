.PHONY: rust-sources-fresh rust-runner-provenance-test fixtures schema check test inspect graph cpp-probe cpp-probe-scripted blob-differential cpp-atlas-mask-oracle cpp-atlas-mask-oracle-preflight golden-runner scripted-golden-runner rust-golden-runner scripted-rust-golden-runner golden-compare scripted-golden-compare e2e-composed-compare silver-corpus silver-corpus-validate silver-corpus-test silver-corpus-manifest-check cpp-oracle-workspace-tests renderer-replay renderer-references renderer-shaders-check renderer-wgpu-backend-check renderer-wgpu-consumer-check renderer-decoder-oracle renderer-fuzz-replay renderer-golden renderer-rust-replay-release renderer-dawn-reference-bootstrap renderer-dawn-reference-replay renderer-dawn-reference-check renderer-dawn-live-reference-bootstrap renderer-dawn-live-reference-replay renderer-dawn-live-reference-check renderer-golden-same-runner renderer-stub-baseline renderer-perf-runners renderer-perf renderer-perf-parity-gate renderer-timing-gate renderer-timing-gate-tools renderer-counter-runners perf-counter-compare perf-compare perf-corpus perf-corpus-check perf-runtime-ref-check perf-hot-loop perf-json perf-gate-measure perf-gate perf-gate-tighten wasm-perf wasm-perf-test browser-renderer-build browser-renderer-smoke browser-renderer-gpu-smoke capi-smoke size-report parity-scorecard parity-scorecard-snapshot parity-scorecard-test cpp-binary-compare cpp-graph-compare cpp-runtime-compare cpp-compare runtime-drawing-port-test runtime-drawing-port-check runtime-drawing-port-closed runtime-drawing-port-gate runtime-frame-loop-trace-runners runtime-frame-loop-trace runtime-frame-loop-port-test runtime-frame-loop-port-check runtime-frame-loop-port-closed runtime-frame-loop-port-gate b6-audit-check crate-seams-baseline-check crate-seams-product-check crate-seams-browser-check crate-seams-apple-check crate-seams-full-check

RIVE_RUNTIME_DIR ?= /Users/levi/dev/oss/rive-runtime
MICROBENCH_TOOL ?= $(CURDIR)/tools/microbench/microbench.py
MICROBENCH_RUN_DIR ?= $(CURDIR)/target/microbench/run
MICROBENCH_RUN_MANIFEST ?= $(MICROBENCH_RUN_DIR)/run.json
MICROBENCH_REPORT ?= $(CURDIR)/target/microbench/comparison.md
MICROBENCH_CPP_DURATION ?= 5
MICROBENCH_WARM_UP ?= 3
MICROBENCH_MEASUREMENT ?= 10
MICROBENCH_SAMPLE_SIZE ?= 20
DEFS_DIR ?= $(RIVE_RUNTIME_DIR)/dev/defs
PORT_MANIFEST ?= $(CURDIR)/port-manifest.toml
PORT_MANIFEST_TOOL ?= $(CURDIR)/tools/port-manifest/port_manifest.py
PORT_MANIFEST_UPSTREAM_REF ?= $(shell git -C "$(RIVE_RUNTIME_DIR)" rev-parse HEAD 2>/dev/null)
RUNTIME_DRAWING_PORT_TOOL ?= $(CURDIR)/tools/runtime-drawing-port/check.py
RUNTIME_DRAWING_OWNERSHIP ?= $(CURDIR)/docs/runtime-drawing-ownership.toml
RUNTIME_DRAWING_GAPS ?= $(CURDIR)/docs/runtime-drawing-gaps.toml
RUNTIME_FRAME_LOOP_PORT_TOOL ?= $(CURDIR)/tools/runtime-frame-loop-port/check.py
TEST_CORRESPONDENCE_TOOL ?= $(CURDIR)/tools/runtime-frame-loop-port/check_test_correspondence.py
LAYOUT_STYLE_HANDLER_TOOL ?= $(CURDIR)/tools/runtime-frame-loop-port/check_layout_style_handlers.py
RUNTIME_FRAME_LOOP_OWNERSHIP ?= $(CURDIR)/docs/runtime-frame-loop-ownership.toml
RUNTIME_FRAME_LOOP_GAPS ?= $(CURDIR)/docs/runtime-frame-loop-gaps.toml
TEST_CORRESPONDENCE_MANIFEST ?= $(CURDIR)/test-correspondence-manifest.toml
RUNTIME_FRAME_LOOP_TRACE_DIR ?= $(CURDIR)/target/runtime-frame-loop-trace/$(shell git rev-parse --short HEAD 2>/dev/null || echo unknown)
RUNTIME_FRAME_LOOP_TRACE_EVIDENCE ?= $(CURDIR)/docs/runtime-frame-loop-trace.json
SILVER_CORPUS_MANIFEST ?= $(CURDIR)/silver-corpus.toml
SILVER_CORPUS_GENERATOR ?= $(CURDIR)/tools/silver-corpus/generate_manifest.py
FILE_CORRESPONDENCE_MANIFEST ?= $(CURDIR)/file-correspondence-manifest.toml
RUST_ADDITIONS ?= $(CURDIR)/rust-additions.toml
RUST_ATTRIBUTION_TOOL ?= $(CURDIR)/tools/b6-audit/rust_attribution.py
PURE_RUNTIME_BOUNDARY_TOOL ?= $(CURDIR)/tools/pure-runtime-boundary/check.py
PARITY_SCORECARD_TOOL ?= $(CURDIR)/tools/parity-scorecard/parity_scorecard.py
PARITY_SCORECARD_DOC ?= $(CURDIR)/docs/parity-scorecard.md
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
PERF_EXPECTED_RIVE_RUNTIME_REF ?= 4ac7b32798da0482e441ef09304dc3b480ed3ee5
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
# `cargo fmt --all` formats the workspace members *and* their local path-based
# dependencies, so it reaches the workspace-excluded vendored wgpu packages.
# Each of those manifests carries an empty `[workspace]` table so cargo stops
# its workspace search at the package itself; without it, a git worktree rooted
# inside the main checkout (`.claude/worktrees/<name>`) makes cargo walk up past
# the worktree root into the parent checkout's `Cargo.toml` and reject the
# workspace mismatch. See vendor/wgpu-30.0.0/NUXIE_PATCH.md.
fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

check:
	cargo check --workspace

# Independently selectable package cuts for the product-surface extraction.
# The platform targets compile their real target-gated interfaces rather than
# succeeding through an empty host cfg. `crate-seams-full-check` remains the
# ordinary whole-workspace verdict.
crate-seams-baseline-check:
	cargo check -p nuxie-runtime --no-default-features --lib

crate-seams-product-check:
	cargo check -p nuxie-product --all-targets
	cargo test -p nuxie-product --features scripting --lib
	@$(MAKE) --no-print-directory crate-seams-product-host-free-check \
		PRODUCT_FEATURES=scripting
	@if $(MAKE) --no-print-directory crate-seams-product-host-free-check \
		PRODUCT_FEATURES=scripting,js-host-seed >/dev/null 2>&1; then \
		echo "host-free feature ratchet missed its js-host-seed positive control" >&2; \
		exit 1; \
	fi

crate-seams-product-host-free-check:
	@set -e; \
	feature_tree="$$(cargo tree --target wasm32-unknown-unknown \
		-p nuxie-product --no-default-features --features "$(PRODUCT_FEATURES)" \
		-e normal,build --format '{p} [{f}]')"; \
	if printf '%s\n' "$$feature_tree" | \
		grep -Eq 'nuxie-(runtime|scripting) v.*\[.*js-host-seed'; then \
		echo "host-free nuxie-product scripting unexpectedly enables js-host-seed" >&2; \
		exit 1; \
	fi

crate-seams-browser-check:
	RUSTC="$$(rustup which --toolchain stable rustc)" \
		"$$(rustup which --toolchain stable cargo)" check \
		-p nuxie-browser-adapter --target wasm32-unknown-unknown --all-targets

crate-seams-apple-check:
	cargo check -p nuxie-apple-adapter --all-targets

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
.PHONY: port-manifest-generate port-manifest-test port-manifest-check port-manifest-gate rust-attribution-test rust-attribution-check rust-attribution-gate pure-runtime-boundary-test pure-runtime-boundary-check pure-runtime-boundary-gate
port-manifest-generate:
	python3 "$(PORT_MANIFEST_TOOL)" generate --rive-runtime-dir "$(RIVE_RUNTIME_DIR)" --upstream-ref "$(PORT_MANIFEST_UPSTREAM_REF)" --output "$(PORT_MANIFEST)"

port-manifest-test:
	PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools/port-manifest -p 'test_*.py' -v

port-manifest-check:
	python3 "$(PORT_MANIFEST_TOOL)" check --rive-runtime-dir "$(RIVE_RUNTIME_DIR)" --upstream-ref "$(PORT_MANIFEST_UPSTREAM_REF)" --repo-root "$(CURDIR)" --manifest "$(PORT_MANIFEST)"

port-manifest-gate:
	@tools/report-all.sh "port-manifest" \
		"port-manifest tool unit tests" "$(MAKE) --no-print-directory port-manifest-test" \
		"upstream C++ port manifest check" "$(MAKE) --no-print-directory port-manifest-check"

rust-attribution-test:
	PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools/b6-audit -p 'test_rust_attribution.py' -v

rust-attribution-check:
	PYTHONDONTWRITEBYTECODE=1 python3 "$(RUST_ATTRIBUTION_TOOL)" --repo-root "$(CURDIR)" --manifest "$(FILE_CORRESPONDENCE_MANIFEST)" --additions "$(RUST_ADDITIONS)"

rust-attribution-gate:
	@tools/report-all.sh "rust-attribution" \
		"rust attribution tool unit tests" "$(MAKE) --no-print-directory rust-attribution-test" \
		"rust attribution coverage check" "$(MAKE) --no-print-directory rust-attribution-check"

pure-runtime-boundary-test:
	PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools/pure-runtime-boundary -p 'test_*.py' -v

pure-runtime-boundary-check:
	PYTHONDONTWRITEBYTECODE=1 python3 "$(PURE_RUNTIME_BOUNDARY_TOOL)" --repo-root "$(CURDIR)"

pure-runtime-boundary-gate:
	@tools/report-all.sh "pure-runtime-boundary" \
		"pure-runtime boundary tool unit tests" "$(MAKE) --no-print-directory pure-runtime-boundary-test" \
		"workspace dependency and source debt check" "$(MAKE) --no-print-directory pure-runtime-boundary-check"

# --- Pinned upstream microbenchmark mirror ----------------------------------
.PHONY: microbench-test microbench-check microbench-upstream-check microbench-gate microbench-extract microbench-build microbench-run microbench-compare
microbench-test:
	PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools/microbench -p 'test_*.py' -v

microbench-check:
	PYTHONDONTWRITEBYTECODE=1 python3 "$(MICROBENCH_TOOL)" --repo-root "$(CURDIR)" check

microbench-upstream-check:
	PYTHONDONTWRITEBYTECODE=1 python3 "$(MICROBENCH_TOOL)" --repo-root "$(CURDIR)" check-upstream --upstream "$(RIVE_RUNTIME_DIR)"

microbench-gate:
	@tools/report-all.sh "upstream-microbenchmarks" \
		"microbenchmark tool unit tests" "$(MAKE) --no-print-directory microbench-test" \
		"20-case Rust registry and fixture hashes" "$(MAKE) --no-print-directory microbench-check" \
		"pinned upstream registry, sources, and fixture provenance" "$(MAKE) --no-print-directory microbench-upstream-check"

microbench-extract:
	PYTHONDONTWRITEBYTECODE=1 python3 "$(MICROBENCH_TOOL)" --repo-root "$(CURDIR)" extract --upstream "$(RIVE_RUNTIME_DIR)"

microbench-build:
	cargo build -p nuxie-runtime --features upstream-microbenchmarks --bench upstream_microbenchmarks
	cargo build -p nuxie-renderer --features upstream-microbenchmarks --bench upstream_microbenchmarks

microbench-run:
	PYTHONDONTWRITEBYTECODE=1 python3 "$(MICROBENCH_TOOL)" --repo-root "$(CURDIR)" run --upstream "$(RIVE_RUNTIME_DIR)" --run-dir "$(MICROBENCH_RUN_DIR)" --duration "$(MICROBENCH_CPP_DURATION)" --warm-up "$(MICROBENCH_WARM_UP)" --measurement "$(MICROBENCH_MEASUREMENT)" --sample-size "$(MICROBENCH_SAMPLE_SIZE)"

microbench-compare:
	PYTHONDONTWRITEBYTECODE=1 python3 "$(MICROBENCH_TOOL)" --repo-root "$(CURDIR)" compare --run-manifest "$(MICROBENCH_RUN_MANIFEST)" --output "$(MICROBENCH_REPORT)"

b6-audit-check:
	PYTHONDONTWRITEBYTECODE=1 python3 tools/b6-audit/check.py

runtime-drawing-port-test:
	PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools/runtime-drawing-port -p 'test_*.py' -v

runtime-drawing-port-check:
	PYTHONDONTWRITEBYTECODE=1 python3 "$(RUNTIME_DRAWING_PORT_TOOL)" --repo-root "$(CURDIR)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)" --ledger "$(RUNTIME_DRAWING_OWNERSHIP)" --gaps "$(RUNTIME_DRAWING_GAPS)"

runtime-drawing-port-closed:
	PYTHONDONTWRITEBYTECODE=1 python3 "$(RUNTIME_DRAWING_PORT_TOOL)" --repo-root "$(CURDIR)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)" --ledger "$(RUNTIME_DRAWING_OWNERSHIP)" --gaps "$(RUNTIME_DRAWING_GAPS)" --require-closed

runtime-drawing-port-gate:
	@tools/report-all.sh "runtime-drawing-port" \
		"runtime drawing port tool unit tests" "$(MAKE) --no-print-directory runtime-drawing-port-test" \
		"runtime drawing port ledger check" "$(MAKE) --no-print-directory runtime-drawing-port-check"

runtime-frame-loop-port-test:
	PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools/runtime-frame-loop-port -p 'test_*.py' -v

runtime-frame-loop-trace-runners:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" tools/runtime-frame-loop-port/build-trace-runners.sh

runtime-frame-loop-trace: runtime-frame-loop-trace-runners
	PYTHONDONTWRITEBYTECODE=1 python3 tools/runtime-frame-loop-port/capture_trace.py \
		--repo-root "$(CURDIR)" \
		--upstream "$(RIVE_RUNTIME_DIR)" \
		--output-dir "$(RUNTIME_FRAME_LOOP_TRACE_DIR)" \
		--output "$(RUNTIME_FRAME_LOOP_TRACE_EVIDENCE)"

# The three correspondence checks below are independent verdicts over the same
# ledger, so they run through report-all.sh rather than as a `set -e` chain
# that would report only the first drift.
runtime-frame-loop-port-check:
	@tools/report-all.sh "runtime-frame-loop-port-check" \
		"test correspondence" 'PYTHONDONTWRITEBYTECODE=1 python3 "$(TEST_CORRESPONDENCE_TOOL)" --repo-root "$(CURDIR)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)" --manifest "$(TEST_CORRESPONDENCE_MANIFEST)"' \
		"layout style handler" 'PYTHONDONTWRITEBYTECODE=1 python3 "$(LAYOUT_STYLE_HANDLER_TOOL)" --repo-root "$(CURDIR)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)" --file-manifest "$(FILE_CORRESPONDENCE_MANIFEST)"' \
		"frame loop ownership ledger" 'PYTHONDONTWRITEBYTECODE=1 python3 "$(RUNTIME_FRAME_LOOP_PORT_TOOL)" --repo-root "$(CURDIR)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)" --ledger "$(RUNTIME_FRAME_LOOP_OWNERSHIP)" --gaps "$(RUNTIME_FRAME_LOOP_GAPS)" --file-manifest "$(FILE_CORRESPONDENCE_MANIFEST)"'

runtime-frame-loop-port-closed:
	@tools/report-all.sh "runtime-frame-loop-port-closed" \
		"test correspondence" 'PYTHONDONTWRITEBYTECODE=1 python3 "$(TEST_CORRESPONDENCE_TOOL)" --repo-root "$(CURDIR)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)" --manifest "$(TEST_CORRESPONDENCE_MANIFEST)"' \
		"layout style handler" 'PYTHONDONTWRITEBYTECODE=1 python3 "$(LAYOUT_STYLE_HANDLER_TOOL)" --repo-root "$(CURDIR)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)" --file-manifest "$(FILE_CORRESPONDENCE_MANIFEST)"' \
		"frame loop ownership ledger (closed)" 'PYTHONDONTWRITEBYTECODE=1 python3 "$(RUNTIME_FRAME_LOOP_PORT_TOOL)" --repo-root "$(CURDIR)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)" --ledger "$(RUNTIME_FRAME_LOOP_OWNERSHIP)" --gaps "$(RUNTIME_FRAME_LOOP_GAPS)" --file-manifest "$(FILE_CORRESPONDENCE_MANIFEST)" --require-closed'

runtime-frame-loop-port-gate:
	@tools/report-all.sh "runtime-frame-loop-port" \
		"runtime frame loop port tool unit tests" "$(MAKE) --no-print-directory runtime-frame-loop-port-test" \
		"runtime frame loop correspondence checks" "$(MAKE) --no-print-directory runtime-frame-loop-port-check"

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
LINT_GATE_WARN_CRATES = nuxie-audio nuxie-runtime nuxie-binary nuxie-graph nux-capi

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
# a `#[cfg(feature = ...)]` module that nothing compiles rots silently. That is
# how crates/nux-capi/src/size_report_roots.rs sat broken on main: the only
# consumer of `size-report-roots` is tools/size-report.sh, whose renderer root
# inventory check runs (correctly) ahead of the fat-LTO build, so the compile
# errors were never reached.
#
# This gate type-checks -- `cargo check`, no linking, no fixtures beyond the
# pinned assets -- every first-party feature that no other CI job builds. New
# feature declarations belong here unless some existing job already compiles
# them; `git grep -- --features Makefile .github` shows what does.
#
# Two tiers because two hosts:
# - PORTABLE runs anywhere and is wired into the ubuntu Clippy lint gate job.
# - APPLE needs an Apple target (nuxie re-exports AppleSurface and friends only
#   on ios/macos), so it is wired into the macOS runtime-evidence job ahead of
#   that job's expensive reference-runtime build.
# Both tiers report every failing entry rather than stopping at the first.
.PHONY: feature-compile-gate feature-compile-gate-portable feature-compile-gate-apple
feature-compile-gate-portable:
	@tools/report-all.sh "feature-compile-gate (portable)" \
		"nuxie-runtime --features threading" "cargo check -p nuxie-runtime --features threading --lib --test work_pool" \
		"nuxie-runtime --features tools" "cargo check -p nuxie-runtime --features tools --lib --test cpp_probe" \
		"nuxie-renderer --features perf-diagnostics" "cargo check -p nuxie-renderer --features perf-diagnostics --lib" \
		"nuxie-renderer --features perf-counters" "cargo check -p nuxie-renderer --features perf-counters --lib" \
		"nuxie-runtime upstream microbenchmarks" "cargo check -p nuxie-runtime --features upstream-microbenchmarks --bench upstream_microbenchmarks" \
		"nuxie-renderer upstream microbenchmarks" "cargo check -p nuxie-renderer --features upstream-microbenchmarks --bench upstream_microbenchmarks" \
		"renderer-replay --features perf-diagnostics" "cargo check -p renderer-replay --features perf-diagnostics --bins" \
		"rust-golden-runner --features coverage-trace" "cargo check -p rust-golden-runner --features coverage-trace --all-targets" \
		"nuxie-scripting --no-default-features" "cargo check -p nuxie-scripting --no-default-features --lib" \
		"nuxie --no-default-features" "cargo check -p nuxie --no-default-features --lib" \
		"product and authoring seams" "$(MAKE) --no-print-directory crate-seams-product-check"

feature-compile-gate-apple:
	@tools/report-all.sh "feature-compile-gate (apple)" \
		"nuxie-apple-adapter --features size-report-roots" "cargo check -p nuxie-apple-adapter --features size-report-roots --lib" \
		"nuxie-audio --features audio-device" "cargo check -p nuxie-audio --features audio-device --all-targets" \
		"Apple adapter seam" "$(MAKE) --no-print-directory crate-seams-apple-check"

feature-compile-gate:
	@tools/report-all.sh "feature-compile-gate" \
		"portable tier" "$(MAKE) --no-print-directory feature-compile-gate-portable" \
		"apple tier" "$(MAKE) --no-print-directory feature-compile-gate-apple"

inspect:
	@cargo run --quiet -p nuxie-binary --bin riv-inspect -- fixtures/graph/dependency_test.riv

graph:
	@cargo run --quiet -p nuxie-graph --bin graph-inspect -- fixtures/graph/dependency_test.riv

cpp-probe:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" tools/cpp-probe/build.sh "$(CPP_CONFIG)"

cpp-probe-scripted:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" RIVE_CPP_PROBE_WITH_SCRIPTING=1 RIVE_CPP_PROBE_RUNNER_NAME=rive_cpp_probe_scripted tools/cpp-probe/build.sh "$(CPP_CONFIG)"

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
	GOLDEN_RUNNER="$(GOLDEN_RUNNER)" RUST_GOLDEN_RUNNER="$(RUST_GOLDEN_RUNNER)" RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" cargo run --quiet -p golden-compare --bin golden-compare -- --corpus corpus.toml --side-channel --cpp-runner "$(GOLDEN_RUNNER)" --rust-runner "$(RUST_GOLDEN_RUNNER)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)"

scripted-golden-compare: CPP_CONFIG=release
scripted-golden-compare: fixtures scripted-golden-runner scripted-rust-golden-runner
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" cargo run --quiet -p golden-compare --bin golden-compare -- --corpus corpus.toml --side-channel --verify-unsupported-cpp --verify-divergent-rust --verify-scripted-diagnostics --cpp-runner "$(SCRIPTED_GOLDEN_RUNNER)" --rust-runner "$(SCRIPTED_RUST_GOLDEN_RUNNER)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)"

e2e-composed-compare: CPP_CONFIG=release
e2e-composed-compare: RUST_PROFILE=release
e2e-composed-compare: fixtures scripted-golden-runner scripted-rust-golden-runner
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" cargo run --quiet -p golden-compare --bin golden-compare -- --corpus "$(E2E_COMPOSED_CORPUS)" --side-channel --require-composed-session --verify-scripted-diagnostics --cpp-runner "$(SCRIPTED_GOLDEN_RUNNER)" --rust-runner "$(SCRIPTED_RUST_GOLDEN_RUNNER)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)"

silver-corpus-test:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" cargo test -p silver-corpus
	PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools/silver-corpus -p 'test_*.py' -v

silver-corpus-manifest-check:
	PYTHONDONTWRITEBYTECODE=1 python3 "$(SILVER_CORPUS_GENERATOR)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)" --output "$(SILVER_CORPUS_MANIFEST)" --check

# validate reads the manifest, so the manifest check stays a genuine
# precondition of it. The unit tests do not: they were only a prerequisite, and
# as one a red suite stopped both the manifest check and the validation from
# running at all.
silver-corpus-validate: silver-corpus-manifest-check
	cargo run --quiet -p silver-corpus -- validate --manifest "$(SILVER_CORPUS_MANIFEST)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)" --lane runtime

silver-corpus:
	@tools/report-all.sh "silver-corpus" \
		"silver corpus unit tests" "$(MAKE) --no-print-directory silver-corpus-test" \
		"silver corpus manifest check and validation" "$(MAKE) --no-print-directory silver-corpus-validate"

# b6-audit-check is deliberately NOT a prerequisite here: it is an unrelated
# policy check, and as a prerequisite an audit drift would stop the workspace
# test floor from running at all. CI runs it as its own step.
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
# the immutable renderer-port pixel oracle and is never relabeled as current-runtime output.
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

# Timing-defined renderer acceptance only. The gate invokes the fixed renderer-perf
# executable with pinned baseline, A, and B runner paths; it never evaluates a
# caller-provided shell command.
renderer-timing-gate-tools:
	cargo build --quiet --release -p perf-compare --bin renderer-perf --bin renderer-timing-compare

renderer-timing-gate: renderer-timing-gate-tools
	@test -n "$(strip $(RENDERER_TIMING_GATE_BASELINE_SOURCE_ID))" || { echo "RENDERER_TIMING_GATE_BASELINE_SOURCE_ID is required (identify the baseline source revision)" >&2; exit 2; }
	@test -n "$(strip $(RENDERER_TIMING_GATE_A_SOURCE_ID))" || { echo "RENDERER_TIMING_GATE_A_SOURCE_ID is required (identify the A runner source)" >&2; exit 2; }
	@test -n "$(strip $(RENDERER_TIMING_GATE_B_SOURCE_ID))" || { echo "RENDERER_TIMING_GATE_B_SOURCE_ID is required (identify the B runner source)" >&2; exit 2; }
	tools/renderer-timing-gate.sh

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

wasm-perf: RUST_PROFILE=release
wasm-perf: rust-golden-runner
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" \
	RUST_GOLDEN_RUNNER="$(RUST_GOLDEN_RUNNER)" \
	WASM_PERF_LIMIT="$(WASM_PERF_LIMIT)" \
	WASM_PERF_IDS="$(WASM_PERF_IDS)" \
	WASM_PERF_REPEAT="$(WASM_PERF_REPEAT)" \
	WASM_PERF_RUNS="$(WASM_PERF_RUNS)" \
	WASM_PERF_WARMUPS="$(WASM_PERF_WARMUPS)" \
	WASM_PERF_OUTPUT="$(WASM_PERF_OUTPUT)" \
	WASM_PERF_MARKDOWN="$(WASM_PERF_MARKDOWN)" \
	tools/browser-renderer-smoke/run-wasm-perf.sh

wasm-perf-test:
	cd tools/browser-renderer-smoke && PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s . -p 'test_wasm_perf.py' -v
	node --test tools/browser-renderer-smoke/wasm-perf-driver.test.cjs

browser-renderer-build:
	tools/browser-renderer-smoke/build.sh

browser-renderer-smoke:
	tools/browser-renderer-smoke/run.sh

browser-renderer-gpu-smoke:
	BROWSER_RENDERER_GPU_ONLY=1 tools/browser-renderer-smoke/run.sh

browser-webgpu-only-check: browser-renderer-smoke browser-renderer-gpu-smoke
	tools/check-browser-webgpu-only.sh

capi-smoke: fixtures
	cargo build --quiet -p nux-capi
	mkdir -p target/capi-smoke
	$(CC) -std=c11 -Wall -Wextra -Werror -Icrates/nux-capi/include -o target/capi-smoke/capi_smoke crates/nux-capi/smoke/capi_smoke.c -Ltarget/debug -lnux_capi
	DYLD_LIBRARY_PATH=target/debug LD_LIBRARY_PATH=target/debug target/capi-smoke/capi_smoke "$(CAPI_SMOKE_FIXTURE)"

# SDK binary-size report: builds the post-Phase-R Darwin link closure with the
# renderer retained, for scripting off and on. Pass SIZE_BASELINE=1 to also
# build the opt-level=3 release closure. No budget is enforced until #B-3's
# USER-GATE is decided. See docs/SIZE.md.
SIZE_BASELINE ?=
size-report:
	tools/size-report.sh $(if $(SIZE_BASELINE),--baseline,)

parity-scorecard-test:
	@PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools/parity-scorecard -p 'test_*.py' -v

# The snapshot no longer hides behind the tool's unit tests: a red suite used
# to stop the snapshot from being taken at all.
parity-scorecard-snapshot:
	@PYTHONDONTWRITEBYTECODE=1 python3 "$(PARITY_SCORECARD_TOOL)" snapshot --repo-root "$(CURDIR)" --output "$(PARITY_SCORECARD_DOC)"

parity-scorecard:
	@tools/report-all.sh "parity-scorecard" \
		"parity scorecard tool unit tests" "$(MAKE) --no-print-directory parity-scorecard-test" \
		"parity scorecard snapshot" "$(MAKE) --no-print-directory parity-scorecard-snapshot"

cpp-binary-compare: cpp-probe
	RIVE_CPP_PROBE="$(CPP_PROBE)" RIVE_CPP_CORPUS=1 cargo test -p nuxie-binary --test cpp_import -- --nocapture
	RIVE_CPP_PROBE="$(CPP_PROBE)" cargo test -p nuxie-runtime --test profiler_cpp_probe -- --nocapture

cpp-graph-compare: cpp-probe
	RIVE_CPP_PROBE="$(CPP_PROBE)" cargo test -p nuxie-graph --test cpp_probe -- --nocapture

cpp-runtime-compare: cpp-probe
	RIVE_CPP_PROBE="$(CPP_PROBE)" cargo test -p nuxie-runtime --features tools --test cpp_probe -- --nocapture

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
