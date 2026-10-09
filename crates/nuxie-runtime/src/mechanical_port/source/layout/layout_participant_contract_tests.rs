use super::*;
use crate::source::{
    artboard::Artboard,
    core::{CoreArena, CoreHandle},
    core_context::{CoreContext, StatusCode},
    generated::{component_base::ComponentBase, core_registry::CoreRegistry},
    layout::grid_item_placement::GridItemPlacement,
    shapes::shape::Shape,
};

struct Context<'a> {
    arena: &'a CoreArena,
    objects: Vec<CoreHandle>,
}
impl CoreContext for Context<'_> {
    fn core_arena(&self) -> &CoreArena { self.arena }
    fn resolve_handle(&self, id: u32) -> Option<CoreHandle> {
        self.objects.get(id as usize).cloned()
    }
}

#[test]
fn placement_cleaned_before_participant_is_registered_when_layout_is_created() {
    let arena = CoreArena::default();
    let root = arena.insert(Artboard::default());
    let host = arena.insert(Shape::default());
    let placement = arena.insert(GridItemPlacement::default());
    let participant = arena.insert(LayoutParticipant::default());
    let mut context = Context {
        arena: &arena,
        objects: vec![root, host.clone(), placement.clone(), participant.clone()],
    };
    for (object, parent_id) in [(&host, 0), (&placement, 1), (&participant, 1)] {
        assert_eq!(object.with_mut(|object| {
            CoreRegistry::set_uint(object, ComponentBase::PARENT_ID_PROPERTY_KEY.into(), parent_id);
            object.as_component_mut().unwrap().on_added_dirty(&mut context)
        }), Some(StatusCode::Ok));
    }
    // File order may clean this sibling before the participant allocates its
    // layout node. The participant's source resync must replay that placement.
    assert_eq!(placement.with_downcast_mut::<GridItemPlacement, _>(|placement| {
        placement.on_added_clean(&mut context)
    }), Some(StatusCode::Ok));
    assert_eq!(participant.with_downcast::<LayoutParticipant, _>(|p| p.layout_data.is_none()), Some(true));
    assert_eq!(LayoutParticipant::on_added_clean_occurrence(&participant, &mut context), StatusCode::Ok);
    let appliers = participant.with_downcast::<LayoutParticipant, _>(|p| {
        p.layout_data.as_ref().unwrap().appliers.as_ref().unwrap().as_ref().clone()
    }).unwrap();
    assert_eq!(appliers, vec![participant.clone(), placement.clone()]);
    // Source appliers are unique; re-establishing participation cannot invoke
    // the placement twice on a later layout solve.
    assert_eq!(LayoutParticipant::on_added_clean_occurrence(&participant, &mut context), StatusCode::Ok);
    assert_eq!(participant.with_downcast::<LayoutParticipant, _>(|p| {
        p.layout_data.as_ref().unwrap().appliers.as_ref().unwrap().as_ref().clone()
    }), Some(appliers));
}

#[test]
fn absent_layout_data_returns_before_accessing_the_host() {
    let arena = CoreArena::default();
    let root = arena.insert(Artboard::default());
    let host = arena.insert(Shape::default());
    let participant = arena.insert(LayoutParticipant::default());
    let mut context = Context { arena: &arena, objects: vec![root, host.clone(), participant.clone()] };
    for (object, parent_id) in [(&host, 0), (&participant, 1)] {
        assert_eq!(object.with_mut(|object| {
            CoreRegistry::set_uint(object, ComponentBase::PARENT_ID_PROPERTY_KEY.into(), parent_id);
            object.as_component_mut().unwrap().on_added_dirty(&mut context)
        }), Some(StatusCode::Ok));
    }
    // The source early return precedes transform/parent lookup. No host data
    // participates in this operation while a layout node does not exist.
    assert_eq!(host.with_mut(|_| participant.with_downcast_mut::<LayoutParticipant, _>(|p| p.sync_style_changes())), Some(Some(false)));
}

#[test]
fn unsolved_layout_is_normalized_before_publishing_an_animated_slot() {
    let arena = CoreArena::default();
    let root = arena.insert(Artboard::default());
    let host = arena.insert(Shape::default());
    let participant = arena.insert(LayoutParticipant::default());
    let mut context = Context { arena: &arena, objects: vec![root, host.clone(), participant.clone()] };
    for (object, parent_id) in [(&host, 0), (&participant, 1)] {
        assert_eq!(object.with_mut(|object| {
            CoreRegistry::set_uint(object, ComponentBase::PARENT_ID_PROPERTY_KEY.into(), parent_id);
            object.as_component_mut().unwrap().on_added_dirty(&mut context)
        }), Some(StatusCode::Ok));
    }
    assert_eq!(LayoutParticipant::on_added_clean_occurrence(&participant, &mut context), StatusCode::Ok);
    participant.with_downcast_mut::<LayoutParticipant, _>(|p| {
        assert!(p.native_layout_data().unwrap().solved_layout.width().is_nan());
        assert!(p.cascade_layout_style(LayoutStyleInterpolation::Linear, None, 0.1, LayoutDirection::Inherit));
        assert_eq!(p.resolved_width(), 0.0);
    }).unwrap();
    LayoutParticipant::update_layout_bounds_occurrence(&participant, false);
    assert_eq!(participant.with_downcast::<LayoutParticipant, _>(|p| (p.resolved_width(), p.resolved_height())), Some((0.0, 0.0)));
}

#[test]
fn non_transform_container_parent_does_not_create_a_layout_node() {
    let arena = CoreArena::default();
    let root = arena.insert(Artboard::default());
    let host = arena.insert(crate::source::custom_property_group::CustomPropertyGroup::default());
    let participant = arena.insert(LayoutParticipant::default());
    let mut context = Context { arena: &arena, objects: vec![root, host.clone(), participant.clone()] };
    for (object, parent_id) in [(&host, 0), (&participant, 1)] {
        assert_eq!(object.with_mut(|object| {
            CoreRegistry::set_uint(object, ComponentBase::PARENT_ID_PROPERTY_KEY.into(), parent_id);
            object.as_component_mut().unwrap().on_added_dirty(&mut context)
        }), Some(StatusCode::Ok));
    }
    assert_eq!(participant.with_mut(|p| p.as_component_mut().unwrap().validate(&mut context)), Some(true));
    assert_eq!(LayoutParticipant::on_added_clean_occurrence(&participant, &mut context), StatusCode::Ok);
    assert_eq!(participant.with_downcast::<LayoutParticipant, _>(|p| p.is_participating_in_layout()), Some(false));
}
