.PHONY: schema check test inspect graph cpp-probe golden-runner scripted-golden-runner rust-golden-runner scripted-rust-golden-runner golden-compare scripted-golden-compare perf-compare perf-corpus perf-hot-loop perf-json capi-smoke size-report cpp-binary-compare cpp-graph-compare cpp-runtime-compare cpp-compare

RIVE_RUNTIME_DIR ?= /Users/levi/dev/oss/rive-runtime
DEFS_DIR ?= $(RIVE_RUNTIME_DIR)/dev/defs
CPP_CONFIG ?= debug
RUST_PROFILE ?= debug
RUST_GOLDEN_RUNNER_FLAGS = $(if $(filter release,$(RUST_PROFILE)),--release,)
CPP_PROBE ?= $(CURDIR)/tools/cpp-probe/build/$(shell uname -s | tr A-Z a-z | sed 's/darwin/macosx/')/bin/$(CPP_CONFIG)/rive_cpp_probe
GOLDEN_RUNNER ?= $(CURDIR)/tools/golden-runner/build/$(shell uname -s | tr A-Z a-z | sed 's/darwin/macosx/')/bin/$(CPP_CONFIG)/rive_golden_runner
SCRIPTED_GOLDEN_RUNNER ?= $(CURDIR)/tools/golden-runner/build/$(shell uname -s | tr A-Z a-z | sed 's/darwin/macosx/')/bin/$(CPP_CONFIG)/rive_golden_runner_scripted
RUST_GOLDEN_RUNNER ?= $(CURDIR)/target/$(RUST_PROFILE)/rust-golden-runner
SCRIPTED_RUST_GOLDEN_RUNNER ?= $(CURDIR)/target/$(RUST_PROFILE)/rust-golden-runner-scripted
PERF_FILE ?= $(RIVE_RUNTIME_DIR)/tests/unit_tests/assets/shapetest.riv
PERF_SAMPLES ?= 0
PERF_ITERATIONS ?= 10
PERF_WARMUPS ?= 1
PERF_CORPUS ?= corpus.toml
PERF_CORPUS_LIMIT ?= 10
PERF_CORPUS_IDS ?= advance_blend_mode,ai_assitant,align_target,animated_clipping,animation_reset_cases,spotify_kids_demo
PERF_CORPUS_SELECTION = $(if $(strip $(PERF_CORPUS_IDS)),--corpus-ids "$(PERF_CORPUS_IDS)",--corpus-limit "$(PERF_CORPUS_LIMIT)")
PERF_AGGREGATE ?= min
PERF_MAX_RATIO ?= 2.0
PERF_BENCHMARK_REPEAT ?= 100
PERF_JSON_OUT ?= $(CURDIR)/target/perf-compare.json
PERF_JSON_META ?= --meta build_profile=release --meta git_sha=$(shell git rev-parse HEAD 2>/dev/null || echo unknown) --meta timestamp=$(shell date -u +%Y-%m-%dT%H:%M:%SZ)
CAPI_SMOKE_FIXTURE ?= fixtures/animation/smi_test.riv
CC ?= cc

schema:
	cargo run -p rive-codegen -- --defs "$(DEFS_DIR)" --out crates/rive-schema/src/generated/schema.rs
	cargo fmt --all

check:
	cargo check --workspace

test:
	cargo test --workspace

inspect:
	@cargo run --quiet -p rive-binary --bin riv-inspect -- fixtures/graph/dependency_test.riv

graph:
	@cargo run --quiet -p rive-graph --bin graph-inspect -- fixtures/graph/dependency_test.riv

cpp-probe:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" tools/cpp-probe/build.sh "$(CPP_CONFIG)"

golden-runner:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" tools/golden-runner/build.sh "$(CPP_CONFIG)"

scripted-golden-runner:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" RIVE_GOLDEN_WITH_SCRIPTING=1 RIVE_GOLDEN_RUNNER_NAME=rive_golden_runner_scripted tools/golden-runner/build.sh "$(CPP_CONFIG)"

rust-golden-runner:
	cargo build --quiet $(RUST_GOLDEN_RUNNER_FLAGS) -p rust-golden-runner

scripted-rust-golden-runner:
	cargo build --quiet $(RUST_GOLDEN_RUNNER_FLAGS) -p rust-golden-runner --features scripting
	cp "$(RUST_GOLDEN_RUNNER)" "$(SCRIPTED_RUST_GOLDEN_RUNNER)"

golden-compare: golden-runner rust-golden-runner
	GOLDEN_RUNNER="$(GOLDEN_RUNNER)" RUST_GOLDEN_RUNNER="$(RUST_GOLDEN_RUNNER)" RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" cargo run --quiet -p golden-compare --bin golden-compare -- --corpus corpus.toml --cpp-runner "$(GOLDEN_RUNNER)" --rust-runner "$(RUST_GOLDEN_RUNNER)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)"

scripted-golden-compare: CPP_CONFIG=release
scripted-golden-compare: scripted-golden-runner scripted-rust-golden-runner
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" cargo run --quiet -p golden-compare --bin golden-compare -- --corpus corpus.toml --milestone M8 --verify-unsupported-cpp --verify-divergent-rust --verify-scripted-diagnostics --cpp-runner "$(SCRIPTED_GOLDEN_RUNNER)" --rust-runner "$(SCRIPTED_RUST_GOLDEN_RUNNER)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)"

perf-compare: CPP_CONFIG=release
perf-compare: RUST_PROFILE=release
perf-compare: golden-runner rust-golden-runner
	GOLDEN_RUNNER="$(GOLDEN_RUNNER)" RUST_GOLDEN_RUNNER="$(RUST_GOLDEN_RUNNER)" RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" cargo run --quiet -p perf-compare --bin perf-compare -- --cpp-runner "$(GOLDEN_RUNNER)" --rust-runner "$(RUST_GOLDEN_RUNNER)" --file "$(PERF_FILE)" --samples "$(PERF_SAMPLES)" --iterations "$(PERF_ITERATIONS)" --warmups "$(PERF_WARMUPS)" --aggregate "$(PERF_AGGREGATE)"

perf-corpus: CPP_CONFIG=release
perf-corpus: RUST_PROFILE=release
perf-corpus: golden-runner rust-golden-runner
	GOLDEN_RUNNER="$(GOLDEN_RUNNER)" RUST_GOLDEN_RUNNER="$(RUST_GOLDEN_RUNNER)" RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" cargo run --quiet -p perf-compare --bin perf-compare -- --cpp-runner "$(GOLDEN_RUNNER)" --rust-runner "$(RUST_GOLDEN_RUNNER)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)" --corpus "$(PERF_CORPUS)" $(PERF_CORPUS_SELECTION) --iterations "$(PERF_ITERATIONS)" --warmups "$(PERF_WARMUPS)" --aggregate "$(PERF_AGGREGATE)" --max-ratio "$(PERF_MAX_RATIO)"

perf-hot-loop: CPP_CONFIG=release
perf-hot-loop: RUST_PROFILE=release
perf-hot-loop: golden-runner rust-golden-runner
	GOLDEN_RUNNER="$(GOLDEN_RUNNER)" RUST_GOLDEN_RUNNER="$(RUST_GOLDEN_RUNNER)" RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" cargo run --quiet -p perf-compare --bin perf-compare -- --cpp-runner "$(GOLDEN_RUNNER)" --rust-runner "$(RUST_GOLDEN_RUNNER)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)" --corpus "$(PERF_CORPUS)" $(PERF_CORPUS_SELECTION) --iterations "$(PERF_ITERATIONS)" --warmups "$(PERF_WARMUPS)" --aggregate "$(PERF_AGGREGATE)" --max-ratio "$(PERF_MAX_RATIO)" --runner-benchmark --benchmark-repeat "$(PERF_BENCHMARK_REPEAT)"

perf-json: CPP_CONFIG=release
perf-json: RUST_PROFILE=release
perf-json: golden-runner rust-golden-runner
	GOLDEN_RUNNER="$(GOLDEN_RUNNER)" RUST_GOLDEN_RUNNER="$(RUST_GOLDEN_RUNNER)" RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" cargo run --quiet -p perf-compare --bin perf-compare -- --cpp-runner "$(GOLDEN_RUNNER)" --rust-runner "$(RUST_GOLDEN_RUNNER)" --file "$(PERF_FILE)" --samples "$(PERF_SAMPLES)" --iterations "$(PERF_ITERATIONS)" --warmups "$(PERF_WARMUPS)" --aggregate "$(PERF_AGGREGATE)" --runner-benchmark --benchmark-repeat "$(PERF_BENCHMARK_REPEAT)" --json "$(PERF_JSON_OUT)" $(PERF_JSON_META)
	@echo "perf-json wrote $(PERF_JSON_OUT)"

capi-smoke:
	cargo build --quiet -p rive-capi
	mkdir -p target/capi-smoke
	$(CC) -std=c11 -Wall -Wextra -Werror -Icrates/rive-capi/include -o target/capi-smoke/capi_smoke crates/rive-capi/smoke/capi_smoke.c -Ltarget/debug -lrive_capi
	DYLD_LIBRARY_PATH=target/debug LD_LIBRARY_PATH=target/debug target/capi-smoke/capi_smoke "$(CAPI_SMOKE_FIXTURE)"

# SDK binary-size report: builds the `release-size` cdylib (never the perf
# `release` profile) and prints the tracked sizes. Pass SIZE_BASELINE=1 to
# also build the unmodified `release` cdylib and show the delta. See
# docs/SIZE.md.
SIZE_BASELINE ?=
size-report:
	tools/size-report.sh $(if $(SIZE_BASELINE),--baseline,)

cpp-binary-compare: cpp-probe
	RIVE_CPP_PROBE="$(CPP_PROBE)" RIVE_CPP_CORPUS=1 cargo test -p rive-binary --test cpp_import -- --nocapture

cpp-graph-compare: cpp-probe
	RIVE_CPP_PROBE="$(CPP_PROBE)" cargo test -p rive-graph --test cpp_probe -- --nocapture

cpp-runtime-compare: cpp-probe
	RIVE_CPP_PROBE="$(CPP_PROBE)" cargo test -p rive-runtime --test cpp_probe -- --nocapture

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
fuzz-build:
	cd $(FUZZ_DIR) && $(FUZZ_CARGO) fuzz build

# The committed seed corpus lives in fuzz/seeds/<target>/; libFuzzer's writable
# working corpus (fuzz/corpus/<target>/) and any crash artifacts are gitignored.
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
