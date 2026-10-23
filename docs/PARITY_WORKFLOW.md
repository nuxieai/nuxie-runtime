# Source parity workflow

## Product scope (2026-10-23)

Nuxie is a web-like engine with Taffy/CSS layout, CSS-oriented text behavior,
and Luau scripting. These are intentional product differences, not parity defects.
Upstream is authoritative only outside these approved differences.

Exclude upstream-specific layout semantics and Wasm/AssemblyScript/AnimaScript
guest execution and dependent engine work. Browser renderer Wasm compilation
remains in scope. Consider small layout optimizations and clear fixes only when
compatible with our CSS contract; never restore Yoga/Rive semantics. Text fixes
outside the deliberate CSS divergence remain eligible.

For mixed commits, translate and review applicable portions and explicitly record
excluded portions in the existing sync map. Do not stall unrelated fixes or ask
again about settled exclusions. Ask only about genuinely new product tradeoffs.
Specifically, skip upstream 868da86c's stack-layout change and retain its independent
state-machine fix as in scope. This supersedes older ask-before-layout wording.

The single mechanical translation method is
[runtime-bun-style-source-port-plan.md](runtime-bun-style-source-port-plan.md).
Apply it to the upstream files being changed; the completed port is not an
invitation to restart a whole-runtime campaign.

1. Read the pinned upstream header and implementation and translate the complete
   owner into its corresponding Rust file. Parallelize disjoint owners.
2. After translation, perform the source-equivalence adversarial pass against
   the upstream pair, without using the old Rust behavior as authority.
3. Perform the separate Rust-integration adversarial pass with the approved
   Taffy, audio, scripting, text, ownership, and platform boundaries in view.
4. Integrate the reviewed owners as the only implementation.
5. Run existing translated tests and differential, Golden, Silver, rendering,
   lifecycle, and platform harnesses applicable to the change.
6. Correct failures at their upstream/Rust owner or approved host boundary.
   Do not invent behavior to fit tests or retain the old implementation as a
   fallback. Review any resulting semantic corrections before landing.

Translation and review are separate passes, not a test-first per-file loop.
The upstream tree and mirrored Rust files establish correspondence. Do not
reintroduce campaign ledgers, receipts, promotion gates, or certification rows.
Compilation and passing corpus samples are evidence, not proof that untested
branches were translated.

Preserve upstream defaults, arithmetic, iteration and callback order,
construction/destruction, cloning, identity, failure paths, and conditional
compilation. Approved Rust adaptations are bounded exceptions, not permission
to redesign adjacent behavior. [PORTING.md](PORTING.md) records adaptation
constraints; its legacy path examples are historical, not source authority.

For renderer changes, compare the same backend, inputs, device, execution mode,
and build configuration. Record source/toolchain identity with results.
[METAL_PORTING.md](METAL_PORTING.md) retains native ownership guidance;
[renderer-parity-workflow.md](renderer-parity-workflow.md) points to the live
validation surface. Use the current pinned upstream source whenever an older
guide or test expectation disagrees.

Report what changed, what was reviewed, which validations ran, and what remains
unverified. Do not call waived hardware coverage a pass or reopen translation
solely because a validation finding required a correction.
