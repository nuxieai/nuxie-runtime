#!/bin/sh
# Ratchet: forced clippy panic-lint sites per crate and lint may only go down.
# See ratchets.py for details.
# Usage: tools/ratchets/panic-lints.sh [--update] [--baseline FILE] [CRATE_DIR...]
#   CRATE_DIR is a crate folder such as vendor/luaur-vm-0.1.8 or crates/nuxie;
#   default: every crate in tests/ratchets/panic-lints-baseline.txt and vendor/luaur-*.
#   Lints the lib target through the Bazel clippy aspect with each panic lint
#   forced to warn (unwrap_used, expect_used, indexing_slicing, panic,
#   unreachable, arithmetic_side_effects, unwrap_in_result).
# Exit: 0 no count went up, 1 a count went up, 2 usage or build error.
exec python3 "$(dirname "$0")/ratchets.py" panic-lints "$@"
