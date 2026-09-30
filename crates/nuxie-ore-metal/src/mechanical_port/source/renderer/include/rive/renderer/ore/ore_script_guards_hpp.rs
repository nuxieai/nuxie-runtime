//! Guard messages shared by the Luau binding and the Wasm host.
//! Source: renderer/include/rive/renderer/ore/ore_script_guards.hpp.
#![allow(non_upper_case_globals)]

pub const kGuardSetPipelineBeforeDraw: &str = "setPipeline must be called before draw";
pub const kGuardVertexSlotRangeFormat: &str = "setVertexBuffer: slot must be 0-%u (got %u)";
pub const kGuardBaseVertexFormat: &str = "%s: baseVertex=%d requires the drawBaseInstance feature, which the active backend does not support";
pub const kGuardFirstInstanceFormat: &str = "%s: firstInstance=%u requires the drawBaseInstance feature, which the active backend does not support";

// Typed substitutions for the source's fixed printf templates; these do not
// interpret arbitrary script-provided format strings.
pub fn vertex_slot_range_message(max_slot: u32, slot: u32) -> String {
    kGuardVertexSlotRangeFormat
        .replacen("%u", &max_slot.to_string(), 1)
        .replacen("%u", &slot.to_string(), 1)
}

pub fn base_vertex_message(operation: &str, base_vertex: i32) -> String {
    kGuardBaseVertexFormat
        .replacen("%d", &base_vertex.to_string(), 1)
        .replacen("%s", operation, 1)
}

pub fn first_instance_message(operation: &str, first_instance: u32) -> String {
    kGuardFirstInstanceFormat
        .replacen("%u", &first_instance.to_string(), 1)
        .replacen("%s", operation, 1)
}
