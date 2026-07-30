Implemented the two checker fixes without committing:

- Angle-bracket UFCS paths now detect `notify_events`, `reported_event`, and `flush_deferred_owner_audio_events` in [check.py](/Users/levi/dev/worktrees/nuxie-fl-c/tools/runtime-frame-loop-port/check.py:541).
- Guarded-enum `use … as X` aliases now detect `X::StateMachine(...)`, with identifier-boundary protection against `PrefixX` false positives, in [check.py](/Users/levi/dev/worktrees/nuxie-fl-c/tools/runtime-frame-loop-port/check.py:602).
- Added permanent negative probes, including both W68 forms verbatim, in [test_check.py](/Users/levi/dev/worktrees/nuxie-fl-c/tools/runtime-frame-loop-port/test_check.py:3717).

Verification:

- `make runtime-frame-loop-port-test`: 67/67 passed.
- `make runtime-frame-loop-port-check`: unit suite passed; only the three expected pre-E5 trace failures remained.
- Python syntax and `git diff --check`: clean.
- Final standards/spec reviews: no remaining findings.
- No gaps-ledger bound change. No commit created.

Goal usage: 147,981 tokens over 12m 51s.