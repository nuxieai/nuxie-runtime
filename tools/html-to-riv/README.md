# HTML/CSS to Rive compiler rework

PR #628 was reverted by [PR #629](https://github.com/nuxieai/nuxie-runtime/pull/629) because the compiler relied on runtime and renderer extensions. This branch starts from the exact restored runtime and currently contains the capability audit and requalification plan. It does not yet publish a replacement compiler.

Read [TARGET.md](TARGET.md) for the immutable-target contract, [BACKLOG.md](BACKLOG.md) for all 99 retained work items, [SUPPORT.md](SUPPORT.md) for support decisions, and [VALIDATION.md](VALIDATION.md) for evidence requirements.

The historical implementation remains in Git at `20248ee6835a7bb071dbf83a364606f5e58aeef9`; its modified-runtime results remain historical evidence. No former qualification automatically transfers.
