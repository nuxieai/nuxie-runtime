//! Pure model of the native Metal resource-slot allocation performed by the
//! pipeline-layout implementation.
//!
//! Keep authored native-shader validation on this exact allocator instead of
//! maintaining a second approximation in a higher layer.

/// The resource namespace consumed by one bind-group-layout entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingSlotKind {
    Buffer,
    Texture,
    Sampler,
    ExternalTexture,
}

/// The three shader stages represented by Rive's frozen BindingMap schema.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum BindingSlotStage {
    Vertex = 0,
    Fragment = 1,
    Compute = 2,
}

impl BindingSlotStage {
    const ALL: [Self; 3] = [Self::Vertex, Self::Fragment, Self::Compute];

    fn flag(self) -> wgt::ShaderStages {
        match self {
            Self::Vertex => wgt::ShaderStages::VERTEX,
            Self::Fragment => wgt::ShaderStages::FRAGMENT,
            Self::Compute => wgt::ShaderStages::COMPUTE,
        }
    }
}

/// Slots assigned to one resource in vertex, fragment, and compute order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BindingSlotAssignment {
    backend_space: u8,
    slots: [Option<u16>; 3],
}

impl BindingSlotAssignment {
    pub fn slots(self) -> [Option<u16>; 3] {
        self.slots
    }

    pub fn backend_space(self) -> u8 {
        self.backend_space
    }

    pub fn for_stage(self, stage: BindingSlotStage) -> Option<u16> {
        self.slots[stage as usize]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Counters {
    buffers: u16,
    textures: u16,
    samplers: u16,
}

/// Stateful allocator used while bind groups are visited in pipeline-layout
/// order. A binding array always occupies one buffer slot because Metal uses
/// an argument buffer for the array, regardless of its element kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetalBindingSlotAllocator {
    counters: [Counters; 3],
}

impl MetalBindingSlotAllocator {
    pub fn new(immediate_size: u32) -> Self {
        let reserved_immediate = u16::from(immediate_size != 0);
        Self {
            counters: [Counters {
                buffers: reserved_immediate,
                ..Counters::default()
            }; 3],
        }
    }

    pub fn allocate(
        &mut self,
        group: u8,
        kind: BindingSlotKind,
        visibility: wgt::ShaderStages,
        is_binding_array: bool,
    ) -> BindingSlotAssignment {
        let mut result = BindingSlotAssignment {
            backend_space: group,
            ..BindingSlotAssignment::default()
        };
        for stage in BindingSlotStage::ALL {
            if !visibility.contains(stage.flag()) {
                continue;
            }
            let counters = &mut self.counters[stage as usize];
            let slot = if is_binding_array {
                let slot = counters.buffers;
                counters.buffers += 1;
                slot
            } else {
                match kind {
                    BindingSlotKind::Buffer => {
                        let slot = counters.buffers;
                        counters.buffers += 1;
                        slot
                    }
                    BindingSlotKind::Texture => {
                        let slot = counters.textures;
                        counters.textures += 1;
                        slot
                    }
                    BindingSlotKind::Sampler => {
                        let slot = counters.samplers;
                        counters.samplers += 1;
                        slot
                    }
                    BindingSlotKind::ExternalTexture => {
                        let slot = counters.textures;
                        counters.textures += 3;
                        counters.buffers += 1;
                        slot
                    }
                }
            };
            result.slots[stage as usize] = Some(slot);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocates_independent_namespaces_across_groups_and_stages() {
        let mut allocator = MetalBindingSlotAllocator::new(0);
        let both = wgt::ShaderStages::VERTEX | wgt::ShaderStages::FRAGMENT;

        assert_eq!(
            allocator
                .allocate(0, BindingSlotKind::Buffer, both, false)
                .slots(),
            [Some(0), Some(0), None]
        );
        assert_eq!(
            allocator
                .allocate(
                    0,
                    BindingSlotKind::Texture,
                    wgt::ShaderStages::FRAGMENT,
                    false
                )
                .slots(),
            [None, Some(0), None]
        );
        assert_eq!(
            allocator
                .allocate(0, BindingSlotKind::Sampler, both, false)
                .slots(),
            [Some(0), Some(0), None]
        );
        // A later group's buffer continues the same per-stage native table.
        assert_eq!(
            allocator
                .allocate(3, BindingSlotKind::Buffer, wgt::ShaderStages::VERTEX, false)
                .slots(),
            [Some(1), None, None]
        );
        assert_eq!(
            allocator
                .allocate(
                    7,
                    BindingSlotKind::Texture,
                    wgt::ShaderStages::COMPUTE,
                    false
                )
                .backend_space(),
            7
        );
    }

    #[test]
    fn arrays_use_one_argument_buffer_slot_for_every_element_kind() {
        let mut allocator = MetalBindingSlotAllocator::new(0);
        let all =
            wgt::ShaderStages::VERTEX | wgt::ShaderStages::FRAGMENT | wgt::ShaderStages::COMPUTE;

        assert_eq!(
            allocator
                .allocate(0, BindingSlotKind::Texture, all, true)
                .slots(),
            [Some(0), Some(0), Some(0)]
        );
        assert_eq!(
            allocator
                .allocate(0, BindingSlotKind::Sampler, all, true)
                .slots(),
            [Some(1), Some(1), Some(1)]
        );
        assert_eq!(
            allocator
                .allocate(0, BindingSlotKind::Buffer, all, false)
                .slots(),
            [Some(2), Some(2), Some(2)]
        );
    }

    #[test]
    fn immediates_reserve_buffer_zero_in_each_stage() {
        let mut allocator = MetalBindingSlotAllocator::new(4);
        assert_eq!(
            allocator
                .allocate(
                    0,
                    BindingSlotKind::Buffer,
                    wgt::ShaderStages::VERTEX | wgt::ShaderStages::COMPUTE,
                    false,
                )
                .slots(),
            [Some(1), None, Some(1)]
        );
    }
}
