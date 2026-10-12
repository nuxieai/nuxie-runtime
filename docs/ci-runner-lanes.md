# Runtime runner policy and shared cache

The repository keeps its six existing Linux and Apple confidence recipes. The
default `.buildkite/pipeline.yml` uploads the owning JSON graph through
`tools/bazel/pipeline.py`; absent operator settings preserve the full legacy
graph. `NUXIE_CI_RUNNER_POLICY_VERSION=2` selects `linux-14g-v2` and `mac-64g-v2`
capabilities, requests `linux-native` or `macos-apple` admission, and uses the
organization's physical-host concurrency groups with an eager limit of one.
`NUXIE_SDK_SPLIT_LANES=1` requires policy v2 and additionally schedules the
existing Android C ABI contract and distribution entrypoints on Linux.

Both Python Bazel frontends use `tools/bazel/bazel.sh` under policy v2. It verifies
the fixed root-controlled installed policy, obtains ceilings from that helper,
and guards each Bazel invocation with admitted job/cache ownership. Explicit
caller budgets cannot replace the installed ceilings. Checkout outputs stay
private; immutable caches use the installed agent identity. Runtime publication
retains its actual Bazel binary/toolchain and source identity. The owning
post-artifact hook invokes installed cleanup after declared evidence uploads;
uncertain or active outputs are preserved.

Remote caching starts off, including bootstrap. After this repository's action
closure and host toolchain have passed cross-checkout, invalidation, outage and
product qualification, the operator may set
`NUXIE_CI_BAZEL_REMOTE_CACHE_MODE=read` and an exact comma-separated
`NUXIE_CI_BAZEL_CACHE_PROFILES` list. Only matching profiles with
`NUXIE_BAZEL_REMOTE_CACHE_ELIGIBLE=1`, the approved SHA-256 domain, endpoint,
public server-leaf trust file, TLS pin receipt and private reader credential path
receive transport. The helper verifies the public pin and credential origin
before a request. SDK writer activation remains off pending its separate source
closure and credential policy qualification. Resource/transport controls and
credentials do not become runtime test-action environment inputs.

Source delivery is tracked by [UNIV-4312](https://universe.basis.dev/issue/UNIV-4312).
Install and enable these settings only after the affected physical host drains
and its capacity, installed policy and exact native producer frontends qualify.
Another task may continue a native migration in a separate checkout. Rebase
overlapping landed changes and repeat affected checks; preserve that task's
source, products and pins.
