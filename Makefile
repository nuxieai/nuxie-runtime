.PHONY: schema check test inspect graph cpp-probe golden-runner rust-golden-runner golden-compare perf-compare perf-corpus perf-hot-loop perf-json capi-smoke cpp-binary-compare cpp-graph-compare cpp-runtime-compare cpp-compare

RIVE_RUNTIME_DIR ?= /Users/levi/dev/oss/rive-runtime
DEFS_DIR ?= $(RIVE_RUNTIME_DIR)/dev/defs
CPP_CONFIG ?= debug
RUST_PROFILE ?= debug
RUST_GOLDEN_RUNNER_FLAGS = $(if $(filter release,$(RUST_PROFILE)),--release,)
CPP_PROBE ?= $(CURDIR)/tools/cpp-probe/build/$(shell uname -s | tr A-Z a-z | sed 's/darwin/macosx/')/bin/$(CPP_CONFIG)/rive_cpp_probe
GOLDEN_RUNNER ?= $(CURDIR)/tools/golden-runner/build/$(shell uname -s | tr A-Z a-z | sed 's/darwin/macosx/')/bin/$(CPP_CONFIG)/rive_golden_runner
RUST_GOLDEN_RUNNER ?= $(CURDIR)/target/$(RUST_PROFILE)/rust-golden-runner
PERF_FILE ?= $(RIVE_RUNTIME_DIR)/tests/unit_tests/assets/shapetest.riv
PERF_SAMPLES ?= 0
PERF_ITERATIONS ?= 5
PERF_WARMUPS ?= 1
PERF_CORPUS ?= corpus.toml
PERF_CORPUS_LIMIT ?= 10
PERF_CORPUS_IDS ?= advance_blend_mode,ai_assitant,align_target,animated_clipping,animation_reset_cases,spotify_kids_demo
PERF_CORPUS_SELECTION = $(if $(strip $(PERF_CORPUS_IDS)),--corpus-ids "$(PERF_CORPUS_IDS)",--corpus-limit "$(PERF_CORPUS_LIMIT)")
PERF_AGGREGATE ?= min
PERF_MAX_RATIO ?= 2.0
PERF_BENCHMARK_REPEAT ?= 1
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

rust-golden-runner:
	cargo build --quiet $(RUST_GOLDEN_RUNNER_FLAGS) -p rust-golden-runner

golden-compare: golden-runner rust-golden-runner
	GOLDEN_RUNNER="$(GOLDEN_RUNNER)" RUST_GOLDEN_RUNNER="$(RUST_GOLDEN_RUNNER)" RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" cargo run --quiet -p golden-compare --bin golden-compare -- --corpus corpus.toml --cpp-runner "$(GOLDEN_RUNNER)" --rust-runner "$(RUST_GOLDEN_RUNNER)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)"

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

cpp-binary-compare: cpp-probe
	RIVE_CPP_PROBE="$(CPP_PROBE)" RIVE_CPP_CORPUS=1 cargo test -p rive-binary --test cpp_import -- --nocapture

cpp-graph-compare: cpp-probe
	RIVE_CPP_PROBE="$(CPP_PROBE)" cargo test -p rive-graph --test cpp_probe -- --nocapture

cpp-runtime-compare: cpp-probe
	RIVE_CPP_PROBE="$(CPP_PROBE)" cargo test -p rive-runtime --test cpp_probe -- --nocapture

cpp-compare: cpp-binary-compare cpp-graph-compare cpp-runtime-compare
