Stopped without implementing a fix because the audio-content/engine hypothesis was disproved.

Even after temporarily:

- Removing Rust artboard audio content and engine
- Disabling the Lua `Audio` global
- Invalidating embedded FLAC data before import

…the exact mismatch remained `y=500` versus `y=0`. All probes were removed; no runtime or runner source changes remain. The requested broad checks were not run due to the stop condition.

Full findings: [FIX-report.md](/Users/levi/dev/worktrees/nuxie-p1c-importers/FIX-report.md)

No commit was created.