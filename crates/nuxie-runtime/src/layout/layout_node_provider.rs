use crate::{
    artboard::ArtboardInstance, components::ComponentHandle, objects::InstanceObjectArena,
};

/// Pinned `LayoutNodeProvider::from(Component*)`.
///
/// Rust retains `LayoutParticipant` as the migrated host's authored child, so
/// the Text/Image/Shape cases resolve that child handle instead of a member
/// pointer returned by `layoutParticipant()`.
pub(crate) fn from(
    objects: &InstanceObjectArena,
    component: Option<ComponentHandle>,
) -> Option<ComponentHandle> {
    let component = component?;
    let runtime_component = objects.component(component)?;
    match runtime_component.type_name {
        "LayoutComponent" | "NestedArtboardLayout" | "ArtboardComponentList" => Some(component),
        "Text" | "Image" | "Shape" => runtime_component.children.iter().find_map(|child| {
            objects
                .component(*child)
                .is_some_and(|child| child.type_name == "LayoutParticipant")
                .then_some(*child)
        }),
        _ => None,
    }
}

/// Rust's empty `Vec` has no heap allocation, preserving the handwritten
/// header's lazy `m_layoutConstraints` allocation and shared-empty read
/// behavior while keeping the storage occurrence-local.
pub(crate) fn layout_constraints(
    objects: &InstanceObjectArena,
    provider: ComponentHandle,
) -> &[ComponentHandle] {
    let Some(component) = objects.component(provider) else {
        return &[];
    };
    if let Some(list) = component.concrete.constrainable_list.as_ref() {
        &list.layout_constraints
    } else if let Some(layout) = component.concrete.layout.as_ref() {
        &layout.layout_constraints
    } else if let Some(participant) = component.concrete.participant_layout.as_ref() {
        &participant.layout_constraints
    } else {
        &[]
    }
}

/// Pinned `LayoutNodeProvider::addLayoutConstraint` in source order: reject a
/// duplicate, append to the provider, then call the constraint's reciprocal
/// `addLayoutChild`. The only pinned override is `ScrollConstraint`.
pub(crate) fn add_layout_constraint(
    objects: &mut InstanceObjectArena,
    provider: ComponentHandle,
    constraint: ComponentHandle,
) {
    let component = objects
        .component_mut(provider)
        .expect("LayoutNodeProvider handle must remain live");
    let constraints = if let Some(list) = component.concrete.constrainable_list.as_mut() {
        &mut list.layout_constraints
    } else if let Some(layout) = component.concrete.layout.as_mut() {
        &mut layout.layout_constraints
    } else if let Some(participant) = component.concrete.participant_layout.as_mut() {
        &mut participant.layout_constraints
    } else {
        unreachable!("LayoutNodeProvider::from returned a non-provider")
    };
    assert!(!constraints.contains(&constraint));
    constraints.push(constraint);

    if let Some(scroll) = objects
        .component_mut(constraint)
        .and_then(|component| component.concrete.scroll.as_mut())
    {
        scroll.layout_children.push(provider);
    }
}

/// Live compatibility seam for callers of the formerly packed Node body. The
/// behavior itself now resides in the primary Rust `Node` owner.
pub(crate) fn mark_layout_node_dirty(
    instance: &mut ArtboardInstance,
    node_local_id: usize,
) -> bool {
    instance.runtime_node_mark_layout_node_dirty(node_local_id)
}
