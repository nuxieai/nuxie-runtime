//! Generated computed setters in pinned160085: getter/equality, virtual setter,
//! changed hook, then the computed property's own observer notification.
use nuxie_runtime::source::{
    artboard::Artboard,
    component_dirt::ComponentDirt,
    constraints::scrolling::{
        clamped_scroll_physics::ClampedScrollPhysics,
        scroll_constraint::ScrollConstraint,
        scroll_physics::ScrollPhysicsRuntime,
    },
    core::{CoreArena, CoreHandle, PropertySetterCompletion},
    core_context::{CoreContext, StatusCode},
    data_bind::data_bind::DataBind,
    generated::{
        core_registry::CoreRegistry,
        node_base::NodeBase,
        constraints::scrolling::scroll_constraint_base::ScrollConstraintBase,
    },
    layout_component::LayoutComponent,
    node::Node,
};


// Scroll percent/index getters require real content and viewport owners in C++.
// Hydrate those public Component links before exercising the generated setter.
fn bound_scroll(arena: &CoreArena) -> CoreHandle {
    struct Context<'a> { arena: &'a CoreArena, objects: Vec<CoreHandle> }
    impl CoreContext for Context<'_> {
        fn core_arena(&self) -> &CoreArena { self.arena }
        fn resolve_handle(&self, id: u32) -> Option<CoreHandle> {
            self.objects.get(id as usize).cloned()
        }
    }
    let artboard = arena.insert(Artboard::default());
    let mut viewport = LayoutComponent::default();
    viewport.set_layout(0.0, 0.0, 100.0, 100.0);
    let viewport = arena.insert(viewport);
    let mut content = LayoutComponent::default();
    content.set_layout(0.0, 0.0, 500.0, 600.0);
    let content = arena.insert(content);
    let owner = arena.insert(ScrollConstraint::default());
    let mut context = Context { arena, objects: vec![artboard, viewport.clone(), content.clone(), owner.clone()] };
    for (handle, parent_id) in [(&viewport, 0), (&content, 1), (&owner, 2)] {
        assert_eq!(handle.with_mut(|object| {
            CoreRegistry::set_uint(object, nuxie_runtime::source::generated::component_base::ComponentBase::PARENT_ID_PROPERTY_KEY.into(), parent_id);
            object.as_component_mut().unwrap().on_added_dirty(&mut context)
        }), Some(StatusCode::Ok));
    }
    owner
}

fn observer(arena: &CoreArena, owner: &CoreHandle, key: u16) -> CoreHandle {
    let observed = arena.insert(DataBind::new(0, key.into(), 0));
    owner.with_mut(|object| {
        observed.with_mut(|binding| {
            object.core_mut().add_property_observer(binding.as_data_bind_mut().unwrap());
        }).unwrap();
    }).unwrap();
    observed
}
fn dirt(observed: &CoreHandle) -> u32 {
    observed.with(|binding| binding.as_data_bind().unwrap().dirt()).unwrap()
}
fn clear(observed: &CoreHandle) {
    observed.with_mut(|binding| binding.as_data_bind_mut().unwrap().set_dirt(0)).unwrap();
}
fn write_double(owner: &CoreHandle, key: u16, value: f32, occurrence: bool) {
    if occurrence {
        assert!(CoreRegistry::set_double_handle(owner, key.into(), value));
    } else {
        owner.with_mut(|object| CoreRegistry::set_double(object, key.into(), value)).unwrap();
    }
}
fn target_dirt() -> u32 { ComponentDirt::BINDINGS_TARGET.0.into() }

#[test]
fn node_computed_noop_setters_still_notify_the_requested_computed_property() {
    // These six getters are valid without an Artboard; root-coordinate getters
    // deliberately remain outside this detached-owner test.
    let keys = [NodeBase::COMPUTED_LOCAL_X_PROPERTY_KEY, NodeBase::COMPUTED_LOCAL_Y_PROPERTY_KEY,
        NodeBase::COMPUTED_WORLD_X_PROPERTY_KEY, NodeBase::COMPUTED_WORLD_Y_PROPERTY_KEY,
        NodeBase::COMPUTED_WIDTH_PROPERTY_KEY, NodeBase::COMPUTED_HEIGHT_PROPERTY_KEY];
    let mut failures = Vec::new();
    for occurrence in [false, true] {
        for key in keys {
            let arena = CoreArena::default();
            let owner = arena.insert(Node::default());
            let observed = observer(&arena, &owner, key);
            write_double(&owner, key, 1.0, occurrence);
            if dirt(&observed) != target_dirt() { failures.push((key, occurrence, dirt(&observed))); }
            assert_eq!(owner.with_mut(|object| CoreRegistry::get_double(object, key.into())), Some(0.0));
        }
    }
    assert!(failures.is_empty(), "missing computed-key notifications: {failures:?}");
}

#[test]
fn node_computed_equality_retains_signed_zero_and_repeated_nan_semantics() {
    for occurrence in [false, true] {
        let arena = CoreArena::default();
        let owner = arena.insert(Node::default());
        let key = NodeBase::COMPUTED_WIDTH_PROPERTY_KEY;
        let observed = observer(&arena, &owner, key);
        write_double(&owner, key, -0.0, occurrence);
        assert_eq!(dirt(&observed), 0, "signed zero must compare equal");
        for _ in 0..2 {
            clear(&observed);
            write_double(&owner, key, f32::NAN, occurrence);
            assert_eq!(dirt(&observed), target_dirt(), "NaN must not compare equal");
        }
    }
}

#[test]
fn computed_width_equality_uses_most_derived_layout_getter() {
    for occurrence in [false, true] {
        let arena = CoreArena::default();
        let mut layout = LayoutComponent::default();
        layout.set_layout(0.0, 0.0, 123.0, 47.0);
        let owner = arena.insert(layout);
        let key = NodeBase::COMPUTED_WIDTH_PROPERTY_KEY;
        let observed = observer(&arena, &owner, key);
        assert_eq!(owner.with_mut(|object| CoreRegistry::get_double(object, key.into())), Some(123.0));
        write_double(&owner, key, 123.0, occurrence);
        assert_eq!(dirt(&observed), 0, "must not use Node's zero width for equality");
        write_double(&owner, key, 0.0, occurrence);
        assert_eq!(dirt(&observed), target_dirt(), "zero differs from actual Layout width");
        assert_eq!(owner.with_mut(|object| CoreRegistry::get_double(object, key.into())), Some(123.0));
    }
}

#[test]
fn computed_notification_is_deferred_to_completion_and_uses_only_its_key() {
    let arena = CoreArena::default();
    let owner = arena.insert(Node::default());
    let key = NodeBase::COMPUTED_WIDTH_PROPERTY_KEY;
    let observed = observer(&arena, &owner, key);
    let unrelated = observer(&arena, &owner, NodeBase::X_PROPERTY_KEY);
    let mut completion = PropertySetterCompletion::default();
    owner.with_mut(|object| CoreRegistry::set_double_with_completion(object, key.into(), 5.0, &mut completion)).unwrap();
    assert_eq!(dirt(&observed), 0);
    assert_eq!(dirt(&unrelated), 0);
    completion.finish();
    assert_eq!(dirt(&observed), target_dirt());
    assert_eq!(dirt(&unrelated), 0);
}

#[test]
fn equal_scroll_computed_write_must_not_stop_running_physics() {
    let mut failures = Vec::new();
    for occurrence in [false, true] {
        for key in [ScrollConstraintBase::SCROLL_PERCENT_X_PROPERTY_KEY,
            ScrollConstraintBase::SCROLL_PERCENT_Y_PROPERTY_KEY, ScrollConstraintBase::SCROLL_INDEX_PROPERTY_KEY] {
            let arena = CoreArena::default();
            let mut physics = ClampedScrollPhysics::default();
            physics.base.base.run_base();
            let physics = arena.insert(physics);
            let owner = bound_scroll(&arena);
            owner.with_downcast_mut::<ScrollConstraint, _>(|scroll| scroll.set_physics(physics.clone())).unwrap();
            let value = owner.with_mut(|object| CoreRegistry::get_double(object, key.into())).unwrap();
            assert_eq!(value, 0.0);
            assert_eq!(physics.with_downcast::<ClampedScrollPhysics, _>(ScrollPhysicsRuntime::is_running), Some(true));
            write_double(&owner, key, value, occurrence);
            let still_running = physics.with_downcast::<ClampedScrollPhysics, _>(ScrollPhysicsRuntime::is_running).unwrap();
            if !still_running { failures.push((key, occurrence)); }
        }
    }
    assert!(failures.is_empty(), "equal computed writes reset running physics: {failures:?}");
}

#[test]
fn changed_scroll_computed_double_notifies_outer_property_even_with_noop_setter() {
    let mut failures = Vec::new();
    for occurrence in [false, true] {
        for key in [ScrollConstraintBase::SCROLL_PERCENT_X_PROPERTY_KEY,
            ScrollConstraintBase::SCROLL_PERCENT_Y_PROPERTY_KEY, ScrollConstraintBase::SCROLL_INDEX_PROPERTY_KEY,
            ScrollConstraintBase::VELOCITY_X_PROPERTY_KEY, ScrollConstraintBase::VELOCITY_Y_PROPERTY_KEY,
            ScrollConstraintBase::COMPUTED_CONTENT_WIDTH_PROPERTY_KEY, ScrollConstraintBase::COMPUTED_CONTENT_HEIGHT_PROPERTY_KEY] {
            let arena = CoreArena::default();
            let owner = bound_scroll(&arena);
            let observed = observer(&arena, &owner, key);
            write_double(&owner, key, 0.5, occurrence);
            if dirt(&observed) != target_dirt() { failures.push((key, occurrence, dirt(&observed))); }
        }
    }
    assert!(failures.is_empty(), "missing scroll computed-key notifications: {failures:?}");
}

#[test]
fn scroll_active_computed_bool_uses_equality_and_own_notification() {
    for occurrence in [false, true] {
        let arena = CoreArena::default();
        let owner = bound_scroll(&arena);
        let key = ScrollConstraintBase::SCROLL_ACTIVE_PROPERTY_KEY;
        let observed = observer(&arena, &owner, key);
        let write = |value| {
            if occurrence { assert!(CoreRegistry::set_bool_handle(&owner, key.into(), value)); }
            else { owner.with_mut(|object| CoreRegistry::set_bool(object, key.into(), value)).unwrap(); }
        };
        write(false);
        assert_eq!(dirt(&observed), 0);
        write(true);
        assert_eq!(dirt(&observed), target_dirt());
    }
}
