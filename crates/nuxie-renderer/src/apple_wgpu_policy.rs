//! Apple-specific wgpu instance policy.
//!
//! The slim Apple renderer supplies every runtime-reachable shader itself. Keep
//! wgpu from lazily constructing its own WGSL-backed helper pipelines.

const INTERNAL_SHADER_FLAGS: wgpu::InstanceFlags = wgpu::InstanceFlags::VALIDATION_INDIRECT_CALL
    .union(wgpu::InstanceFlags::AUTOMATIC_TIMESTAMP_NORMALIZATION);

fn without_internal_shader_flags(mut flags: wgpu::InstanceFlags) -> wgpu::InstanceFlags {
    flags.remove(INTERNAL_SHADER_FLAGS);
    flags
}

pub(crate) fn apple_instance_descriptor() -> wgpu::InstanceDescriptor {
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.flags = without_internal_shader_flags(descriptor.flags);
    descriptor
}

#[cfg(test)]
mod tests {
    use super::{apple_instance_descriptor, without_internal_shader_flags, INTERNAL_SHADER_FLAGS};

    #[test]
    fn apple_policy_disables_every_wgpu_internal_shader_trigger() {
        let flags = without_internal_shader_flags(wgpu::InstanceFlags::all());

        assert!(!flags.intersects(INTERNAL_SHADER_FLAGS));
        assert!(flags.contains(wgpu::InstanceFlags::DEBUG));
        assert!(flags.contains(wgpu::InstanceFlags::VALIDATION));
    }

    #[test]
    fn apple_descriptor_is_safe_in_debug_and_release_builds() {
        let descriptor = apple_instance_descriptor();

        assert!(!descriptor.flags.intersects(INTERNAL_SHADER_FLAGS));
    }
}
