.PHONY: schema check test inspect graph cpp-probe golden-runner rust-golden-runner golden-compare cpp-binary-compare cpp-graph-compare cpp-runtime-compare cpp-compare

RIVE_RUNTIME_DIR ?= /Users/levi/dev/oss/rive-runtime
DEFS_DIR ?= $(RIVE_RUNTIME_DIR)/dev/defs
CPP_PROBE ?= $(CURDIR)/tools/cpp-probe/build/$(shell uname -s | tr A-Z a-z | sed 's/darwin/macosx/')/bin/debug/rive_cpp_probe
GOLDEN_RUNNER ?= $(CURDIR)/tools/golden-runner/build/$(shell uname -s | tr A-Z a-z | sed 's/darwin/macosx/')/bin/debug/rive_golden_runner
RUST_GOLDEN_RUNNER ?= $(CURDIR)/target/debug/rust-golden-runner

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
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" tools/cpp-probe/build.sh debug

golden-runner:
	RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" tools/golden-runner/build.sh debug

rust-golden-runner:
	cargo build --quiet -p rust-golden-runner

golden-compare: golden-runner rust-golden-runner
	GOLDEN_RUNNER="$(GOLDEN_RUNNER)" RUST_GOLDEN_RUNNER="$(RUST_GOLDEN_RUNNER)" RIVE_RUNTIME_DIR="$(RIVE_RUNTIME_DIR)" cargo run --quiet -p golden-compare -- --corpus corpus.toml --cpp-runner "$(GOLDEN_RUNNER)" --rust-runner "$(RUST_GOLDEN_RUNNER)" --rive-runtime-dir "$(RIVE_RUNTIME_DIR)"

cpp-binary-compare: cpp-probe
	RIVE_CPP_PROBE="$(CPP_PROBE)" RIVE_CPP_CORPUS=1 cargo test -p rive-binary --test cpp_import -- --nocapture

cpp-graph-compare: cpp-probe
	RIVE_CPP_PROBE="$(CPP_PROBE)" cargo test -p rive-graph --test cpp_probe -- --nocapture

cpp-runtime-compare: cpp-probe
	RIVE_CPP_PROBE="$(CPP_PROBE)" cargo test -p rive-runtime --test cpp_probe -- --nocapture

cpp-compare: cpp-binary-compare cpp-graph-compare cpp-runtime-compare
