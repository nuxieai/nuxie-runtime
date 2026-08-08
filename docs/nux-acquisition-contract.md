# `.nux` acquisition contract

Status: version 1, canonical owner `nux-container`

A host must fetch a package and any external assets before the package can be
authenticated. Metadata used for that fetch is therefore untrusted. This is a
separate phase from package authentication; it is not a weaker form of trust.

`read_acquisition_metadata` is the canonical Rust implementation. It validates
the bounded v1 container envelope, required member names, the delivery identity,
and content-addressed external-asset descriptors. It returns only:

- `experienceId` and `buildId`, for comparison with the delivery pointer; and
- external image/font fetch descriptors: kind, runtime asset ID, unique name,
  content-addressed key, SHA-256, byte count, and required/optional disposition.

It deliberately does not decode or expose the journey, screens, text inputs,
products, scripts, or other side-effect-bearing content. A host may use the
result only to acquire and hash bytes. It must pass the exact package bytes,
expected identity, candidate keys, and acquired assets to the authenticated
runtime import. Only after `read_package` and `verify_signature` succeed may a
host hydrate the complete manifest and journey or execute the runtime.

The machine-readable contract and error vocabulary are pinned in
`crates/nux-container/tests/fixtures/acquisition-contract-v1.json`. Apple and
other platform SDKs implement the contract independently and copy that fixture
as conformance data; they do not share editor or platform implementation code.
