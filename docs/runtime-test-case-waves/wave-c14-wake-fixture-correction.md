# Wave C14 wake fixture correction candidate

Original candidate: `c5d4ef220`

Independent rejection: `11279eb6b`

Pinned upstream: `rive-runtime` `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Corrected topology

The pointer case now authors only `Backboard`, `Artboard`, the real `ScriptedDrawable`, and the real `StateMachine` host required to reach the retained production `HitScriptedDrawable`. It creates no `FocusData`, no `SemanticData`, and performs no focus selection.

The keyboard case authors the same real owners plus its required direct `FocusData`. It selects local focus id 1 as Rust host setup without asserting the selection return. `SemanticData` is absent because neither pinned event route consumes it.

The production `ArtboardInstance`, retained `ScriptedDrawable`, `StateMachineInstance`, exact implemented-method masks, real event paths, script-owned counter getter, literal script, initialization, parking advances, event arguments, and ordered assertions are unchanged. No production behavior changed.

## Forced evidence

Each terminal re-armed advance assertion was independently changed from the pinned expected value 2 to 3 and forced non-incrementally. The pointer and keyboard tests each failed through their live owner at `left: 2, right: 3`. Both pinned assertions were restored before the final suite, hash, and strict audits.

## Validation

- Focused non-incremental suite: 2 passed, zero failed, zero ignored.
- Individual terminal forced failures: pointer and keyboard each failed at the live counter value 2 versus forced 3.
- Strict Wave C14 wake identities and refreshed locators: 2/2 direct passes, zero pending.
- Pinned source SHA-256: `dd704d375bb3c651c9b33e73213898cc22fe93e1f3316a1ec9493e0c7f5d5901`.
- Literal 828-byte script SHA-256: `0a84d4b1ca848fe275a413d814558bdba49d7a1dee45bdba4bd241d6794eb286`.
- Repository correspondence checker: passed for 157 files and 1,404 pinned `TEST_CASE` declarations.
- Correspondence checker unit suite: 24 passed.
- Non-test release LLVM IR: no wake getter, test, or fixture symbols retained.
- Scoped formatting and `git diff --check`: passed.

This is a correction candidate for fresh independent rereview and does not self-accept Wave C14 wake.
