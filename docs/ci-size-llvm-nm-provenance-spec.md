# CI size-report LLVM provenance repair

## Problem

Hosted `Parity runtime floor evidence` fails before measuring the 9 MiB SDK
budget because its minimal stable Rust toolchain does not install `llvm-nm`.
`tools/size-report.sh` deliberately refuses a system `nm` or a mismatched LLVM
tool: LTO archives must be inspected by the LLVM tools shipped with the exact
`rustc` that built them.

This is an independent CI-environment defect. It is not part of LOC-009 and
must land in its own PR.

## Required behavior

1. In the `parity-runtime-evidence` job only, install stable Rust with the
   official `llvm-tools` component in the existing minimal profile.
2. In the `Record SDK size budget` step, resolve the exact stable `rustc`
   through `rustup which --toolchain "${RUSTUP_TOOLCHAIN}" rustc`.
3. Derive its host triple and sysroot from that exact executable.
4. Export:
   - `RUSTC` as the resolved rustup-managed compiler.
   - `LLVM_NM` as
     `${sysroot}/lib/rustlib/${host}/bin/llvm-nm`.
5. Fail immediately if either the host triple is empty or `LLVM_NM` is not
   executable, then run the unchanged parity-scorecard `make size-report`
   command.

## Non-goals and invariants

- Do not modify `tools/size-report.sh`.
- Do not alter the 9 MiB budget (`9437184`) or scorecard semantics.
- Do not add a fallback to Homebrew/system LLVM.
- Do not touch runtime, renderer, scripting, frame-loop, or Editor code.
- Preserve every other CI command and gate.

## Verification

- `actionlint .github/workflows/ci.yml`
- `make parity-scorecard-test`
- Locally prove the path ritual yields an executable `llvm-nm` whose LLVM major
  matches the resolved stable `rustc`.
- Push the isolated branch, run the hosted `Parity runtime floor evidence`
  job, and require its size-report evidence to contain measured scripting-off
  and scripting-on closures under the unchanged 9 MiB budget.
