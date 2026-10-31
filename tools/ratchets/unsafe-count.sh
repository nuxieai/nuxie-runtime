#!/bin/sh
# Ratchet: `unsafe` sites per crate may only go down. See ratchets.py for details.
# Usage: tools/ratchets/unsafe-count.sh [--update] [--baseline FILE] [GLOB...]
#   Checks every crate in tests/ratchets/unsafe-baseline.txt, every vendor/luaur-*
#   crate, and every crate folder matching a GLOB (for example 'crates/nuxie-*').
# Exit: 0 no count went up, 1 a count went up, 2 usage error.
exec python3 "$(dirname "$0")/ratchets.py" unsafe "$@"
