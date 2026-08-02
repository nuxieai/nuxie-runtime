Completed the P2F audio tail without committing.

Key results:

- Optional CPAL device sink behind `audio-device`; headless remains default.
- Lua playback, scheduling, volume, seek, and query APIs wired through scripting.
- Direct upstream `audio_script.riv` fixture coverage added.
- Manifest rows and expected counts updated honestly.
- Final review eliminated allocation/logging from the real-time callback.

All requested gates pass, including both audio configurations, scripting, runtime, frame-loop correspondence, and attribution checks. Formatting and `git diff --check` are clean.

Full details: [P2F3-report.md](/Users/levi/dev/worktrees/nuxie-p2f-audio/P2F3-report.md)