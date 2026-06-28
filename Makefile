.PHONY: schema check test inspect graph cpp-probe cpp-binary-compare cpp-graph-compare cpp-compare

RIVE_RUNTIME_DIR ?= /Users/levi/dev/oss/rive-runtime
DEFS_DIR ?= $(RIVE_RUNTIME_DIR)/dev/defs
CPP_PROBE ?= $(CURDIR)/tools/cpp-probe/build/$(shell uname -s | tr A-Z a-z | sed 's/darwin/macosx/')/bin/debug/rive_cpp_probe

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

cpp-binary-compare: cpp-probe
	RIVE_CPP_PROBE="$(CPP_PROBE)" RIVE_CPP_CORPUS=1 cargo test -p rive-binary --test cpp_import -- --nocapture

cpp-graph-compare: cpp-probe
	RIVE_CPP_PROBE="$(CPP_PROBE)" cargo test -p rive-graph --test cpp_probe -- --nocapture

cpp-compare: cpp-binary-compare cpp-graph-compare
