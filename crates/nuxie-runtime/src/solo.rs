/// Concrete, occurrence-owned members of C++ `Solo`.
///
/// `Solo` inherits its retained `children()` from `ContainerComponent`. The
/// parallel ids below add only the imported Artboard object-table identity
/// needed by generated `activeComponentId`; child identity itself remains
/// solely in the embedded Component base (`src/solo.cpp:8-31,50-81`). There is
/// deliberately no Artboard-side Solo registry or authored-id rediscovery.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeSoloState {
    pub(crate) active_component_property_key: Option<u16>,
    pub(crate) cpp_local_ids: Vec<usize>,
}
impl RuntimeSoloState {
    fn new() -> Self {
        Self {
            active_component_property_key: property_key_for_name("Solo", "activeComponentId"),
            cpp_local_ids: Vec::new(),
        }
    }

    fn clone_for_occurrence(&self) -> Self {
        // Core/generated clone copies activeComponentId, while
        // ContainerComponent::onAddedDirty rebuilds this occurrence's child
        // pointers before Solo::onAddedClean propagates collapse
        // (`src/solo.cpp:38-48`; `src/container_component.cpp:8-37`).
        Self {
            active_component_property_key: self.active_component_property_key,
            cpp_local_ids: Vec::new(),
        }
    }
}
