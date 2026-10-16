//! A compiled animation applies exactly what LinearAnimation::apply applies
//! through the keyed cells: every apply below runs the same animation on two
//! identical artboards, one through each path, and compares the targets.

use std::{cell::RefCell, rc::Rc};

use super::*;
use crate::mechanical_port::source::{
    animation::{
        compiled_animation::CompiledAnimation,
        interpolating_keyframe::KeyFrameValueContext,
        keyed_object::KeyedObject,
        keyed_property::KeyedProperty,
        keyframe_double::KeyFrameDouble,
        linear_animation::{LinearAnimation, LinearAnimationOwner},
    },
    core::CoreHandle,
    generated::{
        animation::{
            interpolating_keyframe_base::InterpolatingKeyFrameBase, keyframe_base::KeyFrameBase,
            keyframe_double_base::KeyFrameDoubleBase,
        },
        core_registry::CoreRegistry,
        node_base::NodeBase,
        transform_component_base::TransformComponentBase,
        world_transform_component_base::WorldTransformComponentBase,
    },
    node::Node,
};

const FPS: i32 = 60;
const HOLD: u32 = 0;
const LINEAR: u32 = 1;
const X: u16 = NodeBase::X_PROPERTY_KEY;
const ROTATION: u16 = TransformComponentBase::ROTATION_PROPERTY_KEY;
const OPACITY: u16 = WorldTransformComponentBase::OPACITY_PROPERTY_KEY;

/// An artboard instance whose object table holds two nodes after the root.
struct Scene {
    root: RuntimeArtboardInstanceHandle,
    nodes: [CoreHandle; 2],
}

impl Scene {
    fn new() -> Self {
        let root = RuntimeArtboardInstanceHandle::new(ArtboardInstance::default());
        let arena = root.core_handle().retain_arena().unwrap();
        let nodes = [arena.insert(Node::default()), arena.insert(Node::default())];
        root.with_artboard_mut(|root| {
            for node in &nodes {
                root.base.objects.push(Some(node.clone()));
            }
        });
        Self { root, nodes }
    }

    fn values(&self) -> Vec<Option<f32>> {
        self.nodes
            .iter()
            .flat_map(|node| {
                [X, ROTATION, OPACITY]
                    .map(|key| CoreRegistry::get_double_handle(node, i32::from(key)))
            })
            .collect()
    }
}

fn key(scene: &Scene, frame: u32, value: f32, interpolation: u32) -> CoreHandle {
    let arena = scene.root.core_handle().retain_arena().unwrap();
    let key = arena.insert(KeyFrameDouble::default());
    assert!(CoreRegistry::set_double_handle(
        &key,
        i32::from(KeyFrameDoubleBase::VALUE_PROPERTY_KEY),
        value
    ));
    assert!(CoreRegistry::set_uint_handle(
        &key,
        i32::from(InterpolatingKeyFrameBase::INTERPOLATION_TYPE_PROPERTY_KEY),
        interpolation
    ));
    retime(&key, frame);
    key
}

fn retime(key: &CoreHandle, frame: u32) {
    assert!(CoreRegistry::set_uint_handle(
        key,
        i32::from(KeyFrameBase::FRAME_PROPERTY_KEY),
        frame
    ));
    key.with_mut(|key| key.as_key_frame_mut().unwrap().compute_seconds(FPS))
        .unwrap();
}

/// Node 1's x through linear and held keys and its opacity held; node 2's
/// rotation linear. Object ids are table indices (0 is the root).
fn animation(scene: &Scene) -> (Rc<RefCell<LinearAnimation>>, Vec<CoreHandle>) {
    let arena = scene.root.core_handle().retain_arena().unwrap();
    let mut keys = Vec::new();
    let mut animation = LinearAnimation::default();
    let tracks: [(u32, u16, &[(u32, f32, u32)]); 3] = [
        (1, X, &[(0, 0.0, LINEAR), (12, 30.0, HOLD), (24, 10.0, LINEAR), (48, 90.0, LINEAR)]),
        (1, OPACITY, &[(0, 0.5, HOLD)]),
        (2, ROTATION, &[(6, 1.0, LINEAR), (30, -2.0, LINEAR)]),
    ];
    for (object, property_key, frames) in tracks {
        let mut property = KeyedProperty::default();
        property.base.set_property_key_value(property_key.into());
        for &(frame, value, interpolation) in frames {
            let handle = key(scene, frame, value, interpolation);
            keys.push(handle.clone());
            property.add_key_frame(handle);
        }
        let mut keyed = KeyedObject::default();
        keyed.base.set_object_id_value(object);
        keyed.add_keyed_property(arena.insert(property));
        animation.add_keyed_object(arena.insert(keyed));
    }
    (Rc::new(RefCell::new(animation)), keys)
}

/// Apply `animation` on `cells` through LinearAnimation::apply and on
/// `compiled` through `program`, then compare their targets.
fn apply_both(
    (cells, cells_animation): (&Scene, &Rc<RefCell<LinearAnimation>>),
    (compiled, compiled_animation, program): (&Scene, &Rc<RefCell<LinearAnimation>>, &mut CompiledAnimation),
    time: f32,
    mix: f32,
    context: Option<&dyn KeyFrameValueContext>,
) {
    cells
        .root
        .apply_linear_animation(&mut cells_animation.borrow_mut(), time, mix, context);
    let owner = LinearAnimationOwner::Runtime(Rc::clone(compiled_animation));
    let timing = owner.with(|animation| Rc::clone(animation.base.cells()));
    compiled
        .root
        .apply_linear_animation_compiled(&owner, &timing, time, mix, context, program);
    assert_eq!(cells.values(), compiled.values(), "time {time} mix {mix}");
}

const TIMES: [f32; 9] = [0.0, 0.05, 0.1, 0.2, 0.25, 0.3, 0.45, 0.8, 1.2];

#[test]
fn compiled_applies_match_the_cells_path_across_times_and_mixes() {
    let (a, b) = (Scene::new(), Scene::new());
    let (left, _) = animation(&a);
    let (right, _) = animation(&b);
    let mut program = CompiledAnimation::default();
    for mix in [1.0, 0.6, 0.0] {
        for time in TIMES {
            apply_both((&a, &left), (&b, &right, &mut program), time, mix, None);
        }
    }
}

#[test]
fn compiled_applies_follow_edits_between_applies() {
    let (a, b) = (Scene::new(), Scene::new());
    let (left, left_keys) = animation(&a);
    let (right, right_keys) = animation(&b);
    let mut program = CompiledAnimation::default();
    apply_both((&a, &left), (&b, &right, &mut program), 0.3, 1.0, None);
    // A key's value, then its time, change between applies.
    for keys in [&left_keys, &right_keys] {
        assert!(CoreRegistry::set_double_handle(
            &keys[2],
            i32::from(KeyFrameDoubleBase::VALUE_PROPERTY_KEY),
            -40.0
        ));
    }
    apply_both((&a, &left), (&b, &right, &mut program), 0.45, 1.0, None);
    for keys in [&left_keys, &right_keys] {
        retime(&keys[2], 30);
    }
    for time in TIMES {
        apply_both((&a, &left), (&b, &right, &mut program), time, 1.0, None);
    }
    // The table's second node is replaced between applies.
    for scene in [&a, &b] {
        let replacement = scene
            .root
            .core_handle()
            .retain_arena()
            .unwrap()
            .insert(Node::default());
        scene
            .root
            .with_artboard_mut(|root| root.base.objects[1] = Some(replacement.clone()));
    }
    apply_both((&a, &left), (&b, &right, &mut program), 0.5, 1.0, None);
}

/// Bound keyframe values: number_value answers for every keyframe.
struct Bound;
impl KeyFrameValueContext for Bound {
    fn bool_value(&self, _: &CoreHandle) -> Option<bool> {
        None
    }
    fn string_value(&self, _: &CoreHandle) -> Option<String> {
        None
    }
    fn color_value(&self, _: &CoreHandle) -> Option<i32> {
        None
    }
    fn number_value(&self, _: &CoreHandle) -> Option<f32> {
        Some(7.0)
    }
    fn stateful_interpolator_transform_value(
        &self,
        _: &CoreHandle,
        _: &CoreHandle,
        _: f32,
        _: f32,
        _: f32,
    ) -> Option<f32> {
        None
    }
    fn stateful_interpolator_transform(&self, _: &CoreHandle, _: &CoreHandle, _: f32) -> Option<f32> {
        None
    }
}

#[test]
fn compiled_applies_read_bound_values_like_the_cells_path() {
    let (a, b) = (Scene::new(), Scene::new());
    let (left, _) = animation(&a);
    let (right, _) = animation(&b);
    let mut program = CompiledAnimation::default();
    for time in TIMES {
        apply_both((&a, &left), (&b, &right, &mut program), time, 0.75, Some(&Bound));
    }
}
