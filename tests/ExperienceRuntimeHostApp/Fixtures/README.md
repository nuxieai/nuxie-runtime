# SDK runtime package fixtures

This directory contains the minimal signed package set used by the Apple
runtime's native package and lifecycle tests. `fixture-index.json` is the
authority for the committed fixture IDs and their runtime behavior roles.

The corpus moved here with the Apple runtime from nuxie-ios revision
`1b4cbf9a671f37f302dd9f1dd1e2d0d259c2f537`. Future fixture refreshes belong
to this repository and must preserve the deterministic test-only development
key contract used by `package_lifecycle_cycles.rs`.
