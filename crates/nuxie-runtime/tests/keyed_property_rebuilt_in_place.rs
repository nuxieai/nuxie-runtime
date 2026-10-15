//! A KeyedProperty whose keys an editor changes in place applies exactly what
//! a fresh import of the edited keys applies.
//!
//! Authority: pinned KeyedProperty::apply reads m_keyFrames, the list the
//! property itself owns, on every application. The port mirrors that list in
//! cells it shares with the KeyedObject, so the mirror has to follow the
//! property's list when an editor rebuilds it in place: a fresh property takes
//! the old one's base (its key and identity), then adds the keys in their new
//! order. These are local regressions, not upstream-authored test cases.

use nuxie_binary::{
    FixtureProperty, FixtureRecord, FixtureValue, RuntimeFile, encode_runtime_file,
};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::{
        keyed_object::KeyedObject, keyed_property::KeyedProperty, keyframe_double::KeyFrameDouble,
        linear_animation::LinearAnimation,
    },
    artboard::Artboard,
    core::CoreArena,
    generated::{
        animation::{
            interpolating_keyframe_base::InterpolatingKeyFrameBase, keyframe_base::KeyFrameBase,
            keyframe_double_base::KeyFrameDoubleBase,
        },
        core_registry::CoreRegistry,
        node_base::NodeBase,
    },
};
use nuxie_runtime::{ArtboardInstance, CoreHandle, File, RuntimeFactoryHandle, RuntimeFileHandle};

const FPS: u32 = 60;
const HOLD: u32 = 0;
const LINEAR: u32 = 1;
// The keyed node is the artboard's second object, after the artboard itself.
const NODE: usize = 1;
const TIMES: [f32; 10] = [0.0, 0.1, 0.2, 0.25, 0.3, 0.45, 0.6, 0.7, 0.8, 0.9];

/// A key as a file lists it: frame, value and interpolation type.
type Key = (u32, f32, u32);

fn property(owner: &str, name: &str, value: FixtureValue) -> FixtureProperty {
    let definition = nuxie_schema::definition_by_name(owner).unwrap();
    let property = std::iter::once(definition.name)
        .chain(definition.ancestors.iter().copied())
        .filter_map(nuxie_schema::definition_by_name)
        .flat_map(|owner| owner.properties)
        .find(|property| property.name == name)
        .unwrap();
    FixtureProperty {
        key: property.key.int,
        value,
    }
}

fn record(owner: &str, properties: Vec<FixtureProperty>) -> FixtureRecord {
    FixtureRecord {
        type_key: nuxie_schema::definition_by_name(owner)
            .unwrap()
            .type_key
            .int,
        properties,
    }
}

/// One artboard whose node's y is keyed by `keys` in a one-second timeline.
fn import(keys: &[Key]) -> (PersistentFactory<RecordingFactory>, RuntimeFileHandle) {
    let mut records = vec![
        record("Backboard", vec![]),
        record(
            "Artboard",
            vec![
                property("LayoutComponent", "width", FixtureValue::Double(100.0)),
                property("LayoutComponent", "height", FixtureValue::Double(100.0)),
            ],
        ),
        record(
            "Node",
            vec![property("Node", "parentId", FixtureValue::Uint(0))],
        ),
        record(
            "LinearAnimation",
            vec![
                property("LinearAnimation", "fps", FixtureValue::Uint(FPS.into())),
                property(
                    "LinearAnimation",
                    "duration",
                    FixtureValue::Uint(FPS.into()),
                ),
            ],
        ),
        record(
            "KeyedObject",
            vec![property(
                "KeyedObject",
                "objectId",
                FixtureValue::Uint(NODE as u64),
            )],
        ),
        record(
            "KeyedProperty",
            vec![property(
                "KeyedProperty",
                "propertyKey",
                FixtureValue::Uint(NodeBase::Y_PROPERTY_KEY.into()),
            )],
        ),
    ];
    for &(frame, value, interpolation) in keys {
        records.push(record(
            "KeyFrameDouble",
            vec![
                property("KeyFrameDouble", "frame", FixtureValue::Uint(frame.into())),
                property("KeyFrameDouble", "value", FixtureValue::Double(value)),
                property(
                    "KeyFrameDouble",
                    "interpolationType",
                    FixtureValue::Uint(interpolation.into()),
                ),
            ],
        ));
    }
    let bytes = encode_runtime_file(&RuntimeFile::from_fixture_records(records).unwrap()).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    (factory, file)
}

/// The node's y at each of TIMES, applying the timeline on `artboard`.
fn play(artboard: &mut ArtboardInstance) -> Vec<f32> {
    let mut animation = artboard.linear_animation_instance(0).unwrap();
    TIMES
        .iter()
        .map(|&time| {
            artboard
                .apply_linear_animation_instance_at(&mut animation, time, 1.0)
                .unwrap();
            artboard
                .double_property(NODE, NodeBase::Y_PROPERTY_KEY)
                .unwrap()
        })
        .collect()
}

fn instance(file: &RuntimeFileHandle) -> ArtboardInstance {
    ArtboardInstance::from_native(file.clone(), 0).unwrap()
}

/// The imported file's own keyed property, which every instance plays.
fn track(file: &RuntimeFileHandle) -> CoreHandle {
    let artboard = file.with_file(|file| file.artboard_handle(0)).unwrap();
    let timeline = artboard
        .with_downcast::<Artboard, _>(|artboard| artboard.animation_handle_at(0))
        .flatten()
        .unwrap();
    let object = timeline
        .with_downcast::<LinearAnimation, _>(|timeline| timeline.keyed_objects()[0].clone())
        .unwrap();
    object
        .with_downcast::<KeyedObject, _>(|object| object.keyed_properties()[0].clone())
        .unwrap()
}

fn keys(track: &CoreHandle) -> Vec<CoreHandle> {
    track
        .with_downcast::<KeyedProperty, _>(|track| track.keyframes().to_vec())
        .unwrap()
}

/// Rebuild the property in place: a fresh KeyedProperty keeps the old one's
/// base, so its key and the KeyedObject's link to it, and takes `keys` in order.
fn rebuild(track: &CoreHandle, keys: Vec<CoreHandle>) {
    track
        .with_downcast_mut::<KeyedProperty, _>(|track| {
            let mut ordered = KeyedProperty::default();
            ordered.base = std::mem::take(&mut track.base);
            for key in keys {
                ordered.add_key_frame(key);
            }
            *track = ordered;
        })
        .unwrap();
}

fn retime(key: &CoreHandle, frame: u32) {
    assert!(CoreRegistry::set_uint_handle(
        key,
        i32::from(KeyFrameBase::FRAME_PROPERTY_KEY),
        frame,
    ));
    key.with_mut(|key| key.as_key_frame_mut().unwrap().compute_seconds(FPS as i32))
        .unwrap();
}

fn new_key(arena: &CoreArena, (frame, value, interpolation): Key) -> CoreHandle {
    let key = arena.insert(KeyFrameDouble::default());
    assert!(CoreRegistry::set_double_handle(
        &key,
        i32::from(KeyFrameDoubleBase::VALUE_PROPERTY_KEY),
        value,
    ));
    assert!(CoreRegistry::set_uint_handle(
        &key,
        i32::from(InterpolatingKeyFrameBase::INTERPOLATION_TYPE_PROPERTY_KEY),
        interpolation,
    ));
    retime(&key, frame);
    key
}

/// What a fresh import of `keys` plays.
fn fresh(keys: &[Key]) -> Vec<f32> {
    let (_factory, file) = import(keys);
    play(&mut instance(&file))
}

const AUTHORED: [Key; 3] = [(0, 24.0, LINEAR), (18, 12.0, LINEAR), (36, 0.0, LINEAR)];

#[test]
fn replaced_then_retimed_keys_apply_as_a_fresh_import() {
    let (_factory, file) = import(&AUTHORED);
    let mut existing = instance(&file);
    let before = play(&mut existing);
    let track = track(&file);

    // Replace every key with new ones (the middle one now holds), then move
    // the new middle key earlier, keeping the order.
    let arena = CoreArena::default();
    let replaced: Vec<_> = [(0, 60.0, LINEAR), (18, 12.0, HOLD), (36, 0.0, LINEAR)]
        .into_iter()
        .map(|key| new_key(&arena, key))
        .collect();
    rebuild(&track, replaced.clone());
    retime(&replaced[1], 12);
    rebuild(&track, replaced);

    let expected = fresh(&[(0, 60.0, LINEAR), (12, 12.0, HOLD), (36, 0.0, LINEAR)]);
    assert_ne!(expected, before, "the edit changes what plays");
    assert_eq!(
        play(&mut existing),
        expected,
        "an instance made before the edit"
    );
    assert_eq!(
        play(&mut instance(&file)),
        expected,
        "an instance made after it"
    );
}

#[test]
fn a_key_moved_past_its_neighbour_applies_as_a_fresh_import() {
    let (_factory, file) = import(&AUTHORED);
    let mut existing = instance(&file);
    let before = play(&mut existing);
    let track = track(&file);

    let keys = keys(&track);
    retime(&keys[1], 48);
    rebuild(
        &track,
        vec![keys[0].clone(), keys[2].clone(), keys[1].clone()],
    );

    let expected = fresh(&[(0, 24.0, LINEAR), (36, 0.0, LINEAR), (48, 12.0, LINEAR)]);
    assert_ne!(expected, before, "the edit changes what plays");
    assert_eq!(
        play(&mut existing),
        expected,
        "an instance made before the edit"
    );
    assert_eq!(
        play(&mut instance(&file)),
        expected,
        "an instance made after it"
    );
}

#[test]
fn a_rebuild_that_drops_a_key_applies_as_a_fresh_import() {
    let (_factory, file) = import(&AUTHORED);
    let mut existing = instance(&file);
    let before = play(&mut existing);
    let track = track(&file);

    let keys = keys(&track);
    rebuild(&track, vec![keys[1].clone(), keys[2].clone()]);

    let expected = fresh(&[(18, 12.0, LINEAR), (36, 0.0, LINEAR)]);
    assert_ne!(expected, before, "the edit changes what plays");
    assert_eq!(
        play(&mut existing),
        expected,
        "an instance made before the edit"
    );
    assert_eq!(
        play(&mut instance(&file)),
        expected,
        "an instance made after it"
    );
}
