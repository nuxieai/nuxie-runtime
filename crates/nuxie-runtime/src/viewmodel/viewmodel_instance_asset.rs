// Direct Rust owner for pinned C++ `src/viewmodel/viewmodel_instance_asset.cpp`.
// Shared serialized asset payload conversion used by concrete asset values.

/// Converts a u64-typed public value into the u32 cell payload the retained
/// core stores (C++ types these properties `uint32_t`). Values beyond u32
/// cannot come from a valid file; a hostile/absurd API write saturates to
/// the C++ `-1` missing sentinel rather than truncating bit patterns.
fn owned_scalar_u32_payload(value: u64) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
