# Cargo configuration watch anchor

This directory is intentionally tracked so the Apple runtime build identity
script can ask Cargo to rerun when a local `.cargo/config.toml` is added or
removed. Local Cargo configuration is included in diagnostic dirty-build
identity and is not permitted in a release artifact.
