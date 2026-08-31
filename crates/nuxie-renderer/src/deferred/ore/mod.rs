//! ORE deferred owners; shared wire/replay implementations live with Context.
#[cfg(test)]
mod ore_deferred_alias_test;
pub mod ore_deferred_context;
#[cfg(test)]
mod ore_deferred_device_state_test;
#[cfg(test)]
mod ore_source_equivalence_test;
pub use nuxie_ore_metal::ore_cmd::ore_command_silver;
pub use nuxie_ore_metal::ore_cmd::ore_deferred_render_pass;
pub use nuxie_ore_metal::ore_cmd::{
    ore_command_buffer, ore_commands, ore_deferred_resource, ore_handle, ore_make_recording,
    ore_make_replay, ore_render_pass_recording, ore_replay, ore_resource_commands,
};
