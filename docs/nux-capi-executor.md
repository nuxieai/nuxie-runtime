# Native command-queue executor

The portable `nux_executor_*` API schedules complete host operations on one
dedicated thread through the runtime's existing `CommandQueue::run_once` and
`CommandServer::serve_until_disconnect`. It does not change scene ownership,
player-step atomicity, presentation revisions, or frame admission policy.

Create an executor before creating its scene/renderer handles. Submit each
operation with an owned context, an invocation callback and a release callback.
`NUX_STATUS_OK` transfers the context to the executor. The worker invokes then
releases it exactly once. If execution is abandoned after worker failure, release
still runs and must resolve any unfinished host continuation. Rejection transfers
no ownership and invokes neither callback. Callbacks must not unwind across C.
Read, copy and free native result handles inside the job before resuming UI work.

The executor handle is thread-safe and reference-counted; scene handles retain
their existing creator-thread checks. Borrowed calls require a live retained
executor reference. `retain`/`release` allow platform adapters to protect calls
against concurrent owner teardown. The last reference closes admission without
blocking; accepted jobs retain the worker's state until they drain.

`close` serializes against submission, rejects subsequent jobs with
`NUX_STATUS_NOT_FOUND`, and places disconnect after accepted work.
`on_terminated` observers run after the server and its resources are destroyed;
multiple observers are supported and late observers run inline on the caller.
They run without executor locks. `wait` includes observers registered before
completion and reports a timeout through its boolean output. Calling `wait`
from the worker returns `NUX_STATUS_WRONG_THREAD`; final-owner release from a
job therefore cannot self-join. Logical termination precedes thread-local OS/JVM
exit hooks. This API exposes callback jobs only, so disconnect cannot discard a
pending coalesced draw.

The server uses a worker-local null factory for bootstrap. Actual graphics
factories remain owned by the ordinary C ABI calls inside jobs; the executor
does not expose the server's stock scene maps or instantiate a GPU.

Swift adapters retain a Sendable closure and resume existing continuations.
JNI adapters retain global callback references, attach the native thread as a
daemon, bound JNI local references, and clear/translate exceptions before
returning to Rust. Platform display clocks, UI dispatch, surfaces and bounded
frame admission remain in the SDKs. The generic command backlog is not bounded.

This follows the public Rive queue's callback extension rather than introducing
a new scheduler. Rive Android's `cppRunOnCommandServer` provides an analogous
Kotlin-lambda-to-`runOnce` bridge. No layout or CSS behavior changes are involved.
