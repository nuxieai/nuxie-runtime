//! Dirt/collapse reentry regressions for the pinned 820cc19 runtime contracts.
//! These fixtures stop after lifecycle wiring: no fonts, rendering, or updates
//! are needed to observe synchronous Component::addDirt/collapse callbacks.
use nuxie_runtime::source::{
    artboard::Artboard,
    bones::{root_bone::RootBone, skin::Skin, tendon::Tendon},
    component_dirt::ComponentDirt,
    constraints::rotation_constraint::RotationConstraint,
    core::{CoreArena, CoreHandle, CoreObject},
    core_context::CoreContext,
    generated::{
        bones::tendon_base::TendonBase,
        component_base::ComponentBase,
        core_registry::{CoreCapabilities, CoreRegistry},
        layout_component_base::LayoutComponentBase,
        text::{
            text_base::TextBase, text_modifier_group_base::TextModifierGroupBase,
            text_variation_modifier_base::TextVariationModifierBase,
        },
    },
    layout::layout_component_style::LayoutComponentStyle,
    layout_component::LayoutComponent,
    node::Node,
    scripted::scripted_transition::ScriptedTransition,
    shapes::{points_path::PointsPath, shape::Shape},
    status_code::StatusCode,
    text::{
        text::Text, text_follow_path_modifier::TextFollowPathModifier,
        text_modifier_group::TextModifierGroup, text_variation_modifier::TextVariationModifier,
    },
};

struct Context {
    arena: CoreArena,
    objects: Vec<CoreHandle>,
}

impl CoreContext for Context {
    fn core_arena(&self) -> &CoreArena {
        &self.arena
    }

    fn resolve_handle(&self, id: u32) -> Option<CoreHandle> {
        self.objects.get(id as usize).cloned()
    }
}

impl Context {
    fn new() -> Self {
        let arena = CoreArena::default();
        let root = arena.insert(Artboard::default());
        let mut context = Self {
            arena,
            objects: vec![root.clone()],
        };
        // Install the root Component's artboard identity without requiring a
        // renderer/layout import. Child owners use their actual virtual lifecycle.
        assert_eq!(
            root.with_mut(|object| object
                .as_component_mut()
                .unwrap()
                .on_added_dirty(&mut context)),
            Some(StatusCode::Ok)
        );
        context
    }

    fn insert<T: CoreObject>(&mut self, object: T, parent: u32) -> CoreHandle {
        let handle = self.arena.insert(object);
        uint(&handle, ComponentBase::PARENT_ID_PROPERTY_KEY, parent);
        self.objects.push(handle.clone());
        handle
    }

    fn added(&mut self, handle: &CoreHandle) {
        assert_eq!(
            handle.with_mut(|object| object.on_added_dirty(self)),
            Some(StatusCode::Ok)
        );
    }

    fn clear_dirt(&self) {
        for handle in &self.objects {
            set_dirt(handle, ComponentDirt::NONE);
        }
    }

    fn fixed_text(&mut self) -> CoreHandle {
        let text = self.insert(Text::default(), 0);
        uint(&text, TextBase::SIZING_VALUE_PROPERTY_KEY, 2); // TextSizing::Fixed
        self.added(&text);
        text
    }
}

fn uint(owner: &CoreHandle, key: u16, value: u32) {
    assert!(CoreRegistry::set_uint_handle(owner, key.into(), value));
}

fn number(owner: &CoreHandle, key: u16, value: f32) {
    assert!(CoreRegistry::set_double_handle(owner, key.into(), value));
}

fn set_dirt(owner: &CoreHandle, value: ComponentDirt) {
    assert_eq!(
        owner.with_mut(|object| object.component_set_dirt(value)),
        Some(true)
    );
}

fn dirt(owner: &CoreHandle) -> ComponentDirt {
    owner
        .with(|object| object.component_dirt().unwrap())
        .unwrap()
}

fn add_dependent(owner: &CoreHandle, dependent: &CoreHandle) {
    assert_eq!(
        owner.with_mut(|object| object.component_add_dependent(dependent.clone())),
        Some(true)
    );
}

#[test]
fn borrowed_shape_reenters_through_its_child_bones_skin_and_path() {
    let mut context = Context::new();
    let shape = context.insert(Shape::default(), 0);
    let bone = context.insert(RootBone::default(), 1);
    let path = context.insert(PointsPath::default(), 1);
    let skin = context.insert(Skin::default(), 3);
    let tendon = context.insert(Tendon::default(), 4);
    uint(&tendon, TendonBase::BONE_ID_PROPERTY_KEY, 2);
    for owner in [&shape, &bone, &path, &skin, &tendon] {
        context.added(owner);
    }
    for owner in [&shape, &bone, &path, &skin, &tendon] {
        assert_eq!(
            owner.with_mut(|object| object.on_added_clean(&mut context)),
            Some(StatusCode::Ok),
        );
    }
    // RootBone explicitly accepts any TransformComponent parent upstream.
    // Normal dependency building installs Shape -> Bone -> Skin -> PointsPath.
    for owner in [&shape, &bone, &skin, &path] {
        assert_eq!(
            owner.with_mut(|object| object.component_build_dependencies()),
            Some(true),
        );
    }
    let composer = shape
        .with_downcast_mut::<Shape, _>(|shape| shape.path_composer_mut().clone())
        .unwrap();
    context.clear_dirt();
    composer.with_mut(|composer| composer.component.set_dirt(ComponentDirt::NONE));

    assert_eq!(
        shape.with_mut(|shape| shape.world_transform_mark_dirty()),
        Some(true),
    );
    // Skin::onDirty calls PointsPath's base markPathDirty, whose notification
    // returns to the initiating Shape before its original cascade continues.
    assert_eq!(dirt(&shape), ComponentDirt::WORLD_TRANSFORM);
    assert_eq!(dirt(&bone), ComponentDirt::WORLD_TRANSFORM);
    assert_eq!(dirt(&skin), ComponentDirt::WORLD_TRANSFORM);
    assert_eq!(
        dirt(&path),
        ComponentDirt::PATH | ComponentDirt::WORLD_TRANSFORM
    );
    assert_eq!(
        composer.with(|composer| composer.component.dirt()),
        ComponentDirt::PATH | ComponentDirt::WORLD_TRANSFORM,
    );
    assert_eq!(dirt(&context.objects[0]), ComponentDirt::COMPONENTS);
}

#[test]
fn fixed_text_width_self_dependency_stops_at_the_published_duplicate_bit() {
    let mut context = Context::new();
    let text = context.fixed_text();
    add_dependent(&text, &text);
    context.clear_dirt();

    // Text::markShapeDirty publishes Path, then WorldTransform recursively.
    // The self edge must see WorldTransform already present and return before
    // virtual dispatch, even while CoreRegistry's setter owns this Text loan.
    number(&text, TextBase::WIDTH_PROPERTY_KEY, 120.0);
    assert_eq!(
        dirt(&text),
        ComponentDirt::PATH | ComponentDirt::WORLD_TRANSFORM
    );
    assert_eq!(dirt(&context.objects[0]), ComponentDirt::COMPONENTS);
    assert_eq!(
        CoreRegistry::get_double_handle(&text, TextBase::WIDTH_PROPERTY_KEY.into()),
        Some(120.0)
    );

    context.clear_dirt();
    number(&text, TextBase::WIDTH_PROPERTY_KEY, 120.0);
    assert_eq!(dirt(&text), ComponentDirt::NONE);
    assert_eq!(dirt(&context.objects[0]), ComponentDirt::NONE);
}

#[test]
fn explicit_text_constraint_dependency_returns_to_the_active_text() {
    let mut context = Context::new();
    let text = context.fixed_text();
    let constraint = context.insert(RotationConstraint::default(), 1);
    context.added(&constraint);
    assert_eq!(
        constraint.with_mut(|object| object.component_build_dependencies()),
        Some(true)
    );
    // TargetedConstraint::buildDependencies does NOT call Constraint's base
    // implementation. A normal targeted child alone does not author this edge.
    assert!(
        text.with(|object| object.as_component().unwrap().dependents().is_empty())
            .unwrap()
    );
    add_dependent(&text, &constraint);
    context.clear_dirt();

    number(&text, TextBase::WIDTH_PROPERTY_KEY, 80.0);
    // Constraint::onDirty marks its parent Transform and WorldTransform. The
    // second world dirt is a duplicate, so the explicitly authored cycle stops.
    assert_eq!(
        dirt(&text),
        ComponentDirt::PATH | ComponentDirt::TRANSFORM | ComponentDirt::WORLD_TRANSFORM
    );
    assert_eq!(dirt(&constraint), ComponentDirt::WORLD_TRANSFORM);
    assert_eq!(dirt(&context.objects[0]), ComponentDirt::COMPONENTS);
}

#[test]
fn variation_axis_reenters_its_normally_registered_group_through_text() {
    let mut context = Context::new();
    let text = context.fixed_text();
    let group = context.insert(TextModifierGroup::default(), 1);
    context.added(&group);
    let variation = context.insert(TextVariationModifier::default(), 2);
    context.added(&variation);
    assert_eq!(
        text.with_downcast::<Text, _>(Text::have_modifiers),
        Some(true)
    );
    context.clear_dirt();

    // axisValueChanged -> group.shapeModifierChanged -> Text::markShapeDirty
    // revisits every registered group to clear its range maps, even with no ranges.
    number(
        &variation,
        TextVariationModifierBase::AXIS_VALUE_PROPERTY_KEY,
        0.75,
    );
    assert_eq!(
        dirt(&text),
        ComponentDirt::PATH | ComponentDirt::WORLD_TRANSFORM
    );
    assert_eq!(dirt(&group), ComponentDirt::TEXT_COVERAGE);
    assert_eq!(dirt(&variation), ComponentDirt::NONE);
    assert_eq!(dirt(&context.objects[0]), ComponentDirt::COMPONENTS);
    assert_eq!(
        CoreRegistry::get_double_handle(
            &variation,
            TextVariationModifierBase::AXIS_VALUE_PROPERTY_KEY.into()
        ),
        Some(0.75)
    );
}

#[test]
fn coverage_callbacks_do_not_recurse_to_the_groups_dependent() {
    let mut context = Context::new();
    let text = context.fixed_text();
    let group = context.insert(TextModifierGroup::default(), 1);
    context.added(&group);
    let dependent = context.insert(Text::default(), 0);
    context.added(&dependent);
    add_dependent(&group, &dependent);

    let cases: [(fn(&mut TextModifierGroup), ComponentDirt); 3] = [
        (TextModifierGroup::range_type_changed, ComponentDirt::PATH),
        (TextModifierGroup::range_changed, ComponentDirt::PAINT),
        (TextModifierGroup::clear_range_maps, ComponentDirt::NONE),
    ];
    for pending_world_transform in [false, true] {
        for (callback, text_dirt) in cases {
            context.clear_dirt();
            let pending = if pending_world_transform {
                assert_eq!(
                    text.with_mut(|text| text.world_transform_mark_dirty()),
                    Some(true),
                );
                ComponentDirt::WORLD_TRANSFORM
            } else {
                ComponentDirt::NONE
            };
            group
                .with_downcast_mut::<TextModifierGroup, _>(callback)
                .unwrap();
            // All three C++ addDirt(TextCoverage) calls omit recurse (default
            // false). Pending WorldTransform is retained: onDirty must revisit
            // this caller-borrowed Group while processing new Path/Paint dirt.
            assert_eq!(dirt(&group), ComponentDirt::TEXT_COVERAGE);
            assert_eq!(dirt(&dependent), ComponentDirt::NONE);
            assert_eq!(dirt(&text), pending | text_dirt);
            assert_eq!(dirt(&context.objects[0]), ComponentDirt::COMPONENTS);
        }
    }
}

#[test]
fn generated_group_opacity_reenters_with_text_world_transform_pending() {
    let mut context = Context::new();
    let text = context.fixed_text();
    let group = context.insert(TextModifierGroup::default(), 1);
    context.added(&group);
    context.clear_dirt();
    assert_eq!(
        text.with_mut(|text| text.world_transform_mark_dirty()),
        Some(true),
    );

    // The generated setter owns the Group while opacityChanged calls
    // Text::markPaintDirty. Text receives Paint | WorldTransform, not just Paint.
    number(&group, TextModifierGroupBase::OPACITY_PROPERTY_KEY, 0.5);
    assert_eq!(
        dirt(&text),
        ComponentDirt::WORLD_TRANSFORM | ComponentDirt::PAINT,
    );
    assert_eq!(dirt(&group), ComponentDirt::NONE);
    assert_eq!(dirt(&context.objects[0]), ComponentDirt::COMPONENTS);
    assert_eq!(
        CoreRegistry::get_double_handle(&group, TextModifierGroupBase::OPACITY_PROPERTY_KEY.into()),
        Some(0.5),
    );

    context.clear_dirt();
    number(&group, TextModifierGroupBase::OPACITY_PROPERTY_KEY, 0.5);
    assert_eq!(dirt(&text), ComponentDirt::NONE);
    assert_eq!(dirt(&group), ComponentDirt::NONE);
    assert_eq!(dirt(&context.objects[0]), ComponentDirt::NONE);
}

#[test]
fn group_paint_reentry_keeps_the_active_group_through_follow_path_callbacks() {
    for follow_path_parent in [2, 3] {
        let mut context = Context::new();
        let text = context.fixed_text();
        let active = context.insert(TextModifierGroup::default(), 1);
        context.added(&active);
        let other = context.insert(TextModifierGroup::default(), 1);
        context.added(&other);
        context.clear_dirt();
        assert_eq!(
            text.with_mut(|text| text.world_transform_mark_dirty()),
            Some(true),
        );

        // Register the follow-path modifier after WorldTransform is pending.
        // Neither group's registration invents Path dirt; its world callback
        // will add that bit synchronously during the following paint callback.
        let follow = context.insert(TextFollowPathModifier::default(), follow_path_parent);
        context.added(&follow);
        assert_eq!(dirt(&text), ComponentDirt::WORLD_TRANSFORM);
        set_dirt(&context.objects[0], ComponentDirt::NONE);

        // Exercise both the active group's own follow-path callback and one
        // from a different group reentering Text and then the active group.
        number(&active, TextModifierGroupBase::OPACITY_PROPERTY_KEY, 0.5);
        assert_eq!(
            dirt(&text),
            ComponentDirt::WORLD_TRANSFORM | ComponentDirt::PAINT | ComponentDirt::PATH,
        );
        assert_eq!(dirt(&active), ComponentDirt::NONE);
        assert_eq!(dirt(&other), ComponentDirt::NONE);
        assert_eq!(dirt(&context.objects[0]), ComponentDirt::COMPONENTS);
    }
}

#[test]
fn direct_layout_collapse_constraint_child_reenters_the_active_layout() {
    let mut context = Context::new();
    let layout = context.insert(LayoutComponent::default(), 0);
    let style = context.insert(LayoutComponentStyle::default(), 0);
    uint(&layout, LayoutComponentBase::STYLE_ID_PROPERTY_KEY, 2);
    context.added(&layout);
    context.added(&style);
    let constraint = context.insert(RotationConstraint::default(), 1);
    context.added(&constraint);
    context.clear_dirt();

    assert_eq!(
        layout.with_downcast_mut::<LayoutComponent, _>(|layout| layout.collapse(true)),
        Some(true)
    );
    // Layout collapses its children. Constraint::onDirty(Collapsed) must mark
    // this already-borrowed Layout's transform, not borrow its arena slot again.
    assert_eq!(
        dirt(&layout),
        ComponentDirt::COLLAPSED | ComponentDirt::TRANSFORM | ComponentDirt::WORLD_TRANSFORM
    );
    assert_eq!(dirt(&constraint), ComponentDirt::COLLAPSED);
    assert_eq!(dirt(&style), ComponentDirt::COLLAPSED);
    assert_eq!(dirt(&context.objects[0]), ComponentDirt::COMPONENTS);
    assert_eq!(
        layout.with_downcast_mut::<LayoutComponent, _>(|layout| layout.collapse(true)),
        Some(false)
    );
}

#[test]
fn direct_artboard_collapse_retains_artboard_virtual_on_dirty() {
    let mut context = Context::new();
    let root = context.objects[0].clone();
    assert!(CoreRegistry::set_bool_handle(
        &root,
        LayoutComponentBase::CLIP_PROPERTY_KEY.into(),
        true
    ));
    let constraint = context.insert(RotationConstraint::default(), 0);
    context.added(&constraint);
    context.clear_dirt();

    assert_eq!(
        root.with_downcast_mut::<Artboard, _>(|root| root.component_collapse(true)),
        Some(true)
    );
    // Artboard overrides onDirty and does NOT call LayoutComponent::onDirty.
    // With clip=true, an incorrect base virtual dispatch would also add Path.
    assert_eq!(
        dirt(&root),
        ComponentDirt::COLLAPSED
            | ComponentDirt::COMPONENTS
            | ComponentDirt::TRANSFORM
            | ComponentDirt::WORLD_TRANSFORM
    );
    assert_eq!(dirt(&constraint), ComponentDirt::COLLAPSED);
    assert_eq!(
        root.with_downcast_mut::<Artboard, _>(|root| root.component_collapse(true)),
        Some(false)
    );
}

#[test]
fn direct_artboard_uncollapse_resolves_scripted_transition_selection() {
    let mut context = Context::new();
    let root = context.objects[0].clone();
    let transition = context.insert(ScriptedTransition::default(), 0);
    context.added(&transition);
    let child = context.insert(Node::default(), 1);
    context.added(&child);
    context.clear_dirt();

    assert_eq!(
        root.with_downcast_mut::<Artboard, _>(|root| root.component_collapse(true)),
        Some(true),
    );
    assert_eq!(
        dirt(&root),
        ComponentDirt::COLLAPSED | ComponentDirt::COMPONENTS,
    );
    assert_eq!(dirt(&transition), ComponentDirt::COLLAPSED);
    assert_eq!(dirt(&child), ComponentDirt::COLLAPSED);

    // Retain the collapsed state but clear the root's notification bit. Even
    // with no design child/script asset, uncollapse resolves activeComponentId
    // through the already-borrowed Artboard before visiting this ordinary Node.
    set_dirt(&root, ComponentDirt::COLLAPSED);
    assert_eq!(
        root.with_downcast_mut::<Artboard, _>(|root| root.component_collapse(false)),
        Some(true),
    );
    assert_eq!(dirt(&root), ComponentDirt::COMPONENTS);
    assert_eq!(dirt(&transition), ComponentDirt::NONE);
    assert_eq!(dirt(&child), ComponentDirt::NONE);
}
