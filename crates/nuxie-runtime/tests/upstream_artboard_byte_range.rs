//! All nine artboard_byte_range_test.cpp cases from upstream 1e979c62.
//! Replacement and byte-range access are the upstream tools-only surface.
#![cfg(feature = "tools")]

use std::path::PathBuf;

use nuxie_render_api::{PersistentFactory, RecordingFactory, RecordingRenderer};
use nuxie_runtime::{
    Artboard, CoreHandle, File, ImportResult, RuntimeFactoryHandle, RuntimeFileHandle,
    source::{
        core::binary_reader::BinaryReader, dependency_sorter::DependencySorter,
        runtime_header::RuntimeHeader, shapes::image::Image,
    },
};

fn load(name: &str) -> (Vec<u8>, RuntimeFileHandle, RecordingRenderer) {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()));
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let renderer = factory.borrow().make_renderer();
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let mut result = ImportResult::Malformed;
    let file = File::import(&bytes, retained, Some(&mut result), None, None)
        .unwrap_or_else(|| panic!("{name} imports: {result:?}"));
    assert_eq!(result, ImportResult::Success);
    (bytes, file, renderer)
}

// Read the header independently; do not derive its size from the ranges.
fn header_size_of(bytes: &[u8]) -> usize {
    let mut reader = BinaryReader::new(bytes);
    let mut header = RuntimeHeader::default();
    assert!(RuntimeHeader::read(&mut reader, &mut header));
    bytes.len() - reader.position().len()
}

fn run<'a>(file: &RuntimeFileHandle, bytes: &'a [u8], index: usize) -> &'a [u8] {
    let range = file.with_file(|file| file.artboard_byte_range(index));
    let header = header_size_of(bytes);
    &bytes[header + range.start..header + range.end]
}

fn objects(file: &RuntimeFileHandle, index: usize) -> Vec<CoreHandle> {
    file.with_file(|file| file.artboard_at_source(index))
        .expect("source artboard")
        .with_downcast::<Artboard, _>(|artboard| {
            artboard.objects().iter().flatten().cloned().collect()
        })
        .expect("Artboard")
}

#[test]
fn artboard_byte_ranges_are_contiguous_and_cover_the_artboard_stream() {
    let (bytes, file, _) = load("artboardclipping.riv");
    let count = file.with_file(File::artboard_count);
    assert!(count > 1);
    let mut previous_end = 0;
    for i in 0..count {
        let range = file.with_file(|file| file.artboard_byte_range(i));
        assert!(range.end > range.start);
        if i > 0 {
            assert_eq!(range.start, previous_end);
        }
        previous_end = range.end;
    }
    assert!(previous_end > 0);
    assert_eq!(header_size_of(&bytes) + previous_end, bytes.len());
    assert!(file.with_file(|file| file.artboard_byte_range(0).start) < previous_end);
}

#[test]
fn artboard_byte_ranges_hold_across_files() {
    for name in ["juice.riv", "entry.riv", "artboardclipping.riv"] {
        let (_, file, _) = load(name);
        let mut previous_end = 0;
        for i in 0..file.with_file(File::artboard_count) {
            let range = file.with_file(|file| file.artboard_byte_range(i));
            assert!(range.end > range.start);
            if i > 0 {
                assert_eq!(range.start, previous_end);
            }
            previous_end = range.end;
        }
    }
}

#[test]
fn an_out_of_range_artboard_index_reports_an_empty_byte_range() {
    let (_, file, _) = load("artboardclipping.riv");
    let range = file.with_file(|file| file.artboard_byte_range(file.artboard_count() + 10));
    assert_eq!(range.start, 0);
    assert_eq!(range.end, 0);
}

#[test]
fn an_artboard_can_be_replaced_in_place_from_its_own_bytes() {
    let (bytes, file, mut renderer) = load("artboardclipping.riv");
    let count_before = file.with_file(File::artboard_count);
    assert!(count_before > 1);
    let target = 1;
    let name_before = file.with_file(|file| file.artboard_name_at(target));
    let untouched = file
        .with_file(|file| file.artboard_at(0))
        .expect("untouched instance");
    let untouched_width = untouched.with_artboard(|artboard| artboard.width());
    let range = file.with_file(|file| file.artboard_byte_range(target));
    assert!(range.end > range.start);
    let bytes = run(&file, &bytes, target);
    assert_eq!(
        file.with_file_mut(|file| file.replace_artboard(target, bytes)),
        ImportResult::Success
    );
    assert_eq!(file.with_file(File::artboard_count), count_before);
    assert_eq!(
        file.with_file(|file| file.artboard_name_at(target)),
        name_before
    );
    let replaced = file
        .with_file(|file| file.artboard_at(target))
        .expect("replacement instance");
    replaced.advance_default(0.0);
    replaced.draw(&mut renderer);
    assert_eq!(
        untouched.with_artboard(|artboard| artboard.width()),
        untouched_width
    );
    untouched.advance_default(0.0);
    untouched.draw(&mut renderer);
}

#[test]
fn replacing_an_artboard_decodes_the_bytes_it_is_given() {
    let (bytes, file, mut renderer) = load("artboardclipping.riv");
    assert!(file.with_file(File::artboard_count) > 1);
    let source_name = file.with_file(|file| file.artboard_name_at(0));
    let target_name = file.with_file(|file| file.artboard_name_at(1));
    assert_ne!(source_name, target_name);
    let source_run = run(&file, &bytes, 0);
    assert_eq!(
        file.with_file_mut(|file| file.replace_artboard(1, source_run)),
        ImportResult::Success
    );
    assert_eq!(file.with_file(|file| file.artboard_name_at(1)), source_name);
    assert_eq!(file.with_file(|file| file.artboard_name_at(0)), source_name);
    assert!(file.with_file(File::artboard_count) > 1);
    for i in 0..2 {
        let instance = file
            .with_file(|file| file.artboard_at(i))
            .expect("usable instance");
        instance.advance_default(0.0);
        instance.draw(&mut renderer);
    }
}

#[test]
fn replacing_an_artboard_rejects_bad_input_without_changing_the_file() {
    let (_, file, _) = load("artboardclipping.riv");
    let count_before = file.with_file(File::artboard_count);
    let name_before = file.with_file(|file| file.artboard_name_at(0));
    assert_eq!(
        file.with_file_mut(|file| file.replace_artboard(count_before + 5, &[0])),
        ImportResult::Malformed
    );
    assert_eq!(
        file.with_file_mut(|file| file.replace_artboard(0, &[])),
        ImportResult::Malformed
    );
    assert_eq!(file.with_file(File::artboard_count), count_before);
    assert_eq!(file.with_file(|file| file.artboard_name_at(0)), name_before);
}

#[test]
fn replacing_a_referenced_artboard_repoints_its_referencers() {
    let (bytes, file, mut renderer) = load("nested_artboard_opacity.riv");
    let count = file.with_file(File::artboard_count);
    assert!(count > 1);
    let (nested, referenced_index) = (0..count)
        .find_map(|i| {
            objects(&file, i).into_iter().find_map(|object| {
                let id = object
                    .with(|object| {
                        object
                            .as_nested_artboard()
                            .map(|nested| nested.referenced_artboard_id())
                    })
                    .flatten()?;
                (id >= 0).then_some((object, id))
            })
        })
        .expect("nested artboard with a referenced index");
    assert!(referenced_index >= 0);
    let index = referenced_index as usize;
    assert!(index < count);
    let run = run(&file, &bytes, index);
    let before = file
        .with_file(|file| file.artboard_at_source(index))
        .expect("old source");
    let source = || {
        nested
            .with(|object| object.as_nested_artboard().unwrap().source_artboard())
            .flatten()
    };
    assert_eq!(source(), Some(before.clone()));
    assert_eq!(
        file.with_file_mut(|file| file.replace_artboard(index, run)),
        ImportResult::Success
    );
    let after = file
        .with_file(|file| file.artboard_at_source(index))
        .expect("new source");
    assert_ne!(after, before);
    assert_eq!(source(), Some(after));
    assert_ne!(source(), Some(before));
    let host = file
        .with_file(File::artboard_default)
        .expect("host instance");
    host.advance_default(0.0);
    host.draw(&mut renderer);
}

#[test]
fn a_replaced_artboard_still_resolves_its_file_assets() {
    let (bytes, file, _) = load("walle.riv");
    let find_image = |index| {
        objects(&file, index)
            .into_iter()
            .find(|object| object.with_downcast::<Image, _>(|_| true).unwrap_or(false))
    };
    let (index, image) = (0..file.with_file(File::artboard_count))
        .find_map(|index| find_image(index).map(|image| (index, image)))
        .expect("image");
    let asset_before = image
        .with_downcast::<Image, _>(Image::image_asset)
        .flatten()
        .expect("image asset");
    let run = run(&file, &bytes, index);
    assert_eq!(
        file.with_file_mut(|file| file.replace_artboard(index, run)),
        ImportResult::Success
    );
    let replaced = find_image(index).expect("replacement image");
    assert_ne!(replaced, image);
    let asset_after = replaced
        .with_downcast::<Image, _>(Image::image_asset)
        .flatten()
        .expect("replacement asset");
    assert_eq!(asset_after, asset_before);
}

#[test]
fn an_artboard_instances_dependency_order_matches_a_real_sort() {
    for name in ["juice.riv", "entry.riv", "artboardclipping.riv"] {
        let (_, file, _) = load(name);
        for i in 0..file.with_file(File::artboard_count) {
            let instance = file
                .with_file(|file| file.artboard_at(i))
                .expect("instance");
            let recipe = instance.with_artboard(|artboard| artboard.dependency_order().to_vec());
            let mut sorted = Vec::new();
            DependencySorter::default().sort(instance.core_handle().into(), &mut sorted);
            assert_eq!(recipe.len(), sorted.len(), "{name} artboard {i}");
            for (p, (actual, expected)) in recipe.iter().zip(&sorted).enumerate() {
                assert!(actual == expected, "{name} artboard {i} position {p}");
            }
        }
    }
}

// Supplemental Rust arena check, not an additional upstream test case. A
// replacement must retire the old source's owned records, not merely detach
// them from File while their weak handles remain usable.
#[test]
fn replacement_retires_source_owned_arena_records() {
    use nuxie_runtime::source::animation::{
        keyed_object::KeyedObject, keyed_property::KeyedProperty,
        linear_animation::LinearAnimation, state_machine::StateMachine,
        state_machine_layer::StateMachineLayer,
    };
    let mut keyed_records = 0;
    let mut state_records = 0;
    for name in ["juice.riv", "entry.riv"] {
        let (bytes, file, _) = load(name);
        for index in 0..file.with_file(File::artboard_count) {
            let source = file
                .with_file(|file| file.artboard_at_source(index))
                .unwrap();
            let (mut owned, animations, machines) = source
                .with_downcast::<Artboard, _>(|artboard| {
                    let owned = artboard
                        .objects()
                        .iter()
                        .flatten()
                        .filter(|handle| {
                            handle
                                .with(|object| {
                                    object.as_view_model_instance().is_none()
                                        && object.as_view_model_instance_value().is_none()
                                })
                                .unwrap()
                        })
                        .cloned()
                        .collect::<Vec<_>>();
                    (
                        owned,
                        artboard.animation_handles().to_vec(),
                        artboard.state_machine_handles().to_vec(),
                    )
                })
                .unwrap();
            for animation in animations {
                let objects = animation
                    .with_downcast::<LinearAnimation, _>(|animation| {
                        animation.keyed_objects().to_vec()
                    })
                    .unwrap();
                for object in objects {
                    let properties = object
                        .with_downcast::<KeyedObject, _>(|object| {
                            object.keyed_properties().to_vec()
                        })
                        .unwrap();
                    for property in properties {
                        let frames = property
                            .with_downcast::<KeyedProperty, _>(|property| {
                                property.keyframes().to_vec()
                            })
                            .unwrap();
                        keyed_records += frames.len();
                        owned.extend(frames);
                        owned.push(property);
                    }
                    owned.push(object);
                }
                owned.push(animation);
            }
            for machine in machines {
                let layers = machine
                    .with_downcast::<StateMachine, _>(|machine| {
                        (0..machine.layer_count())
                            .map(|i| machine.layer(i).unwrap())
                            .collect::<Vec<_>>()
                    })
                    .unwrap();
                for layer in layers {
                    let states = layer
                        .with_downcast::<StateMachineLayer, _>(|layer| layer.states().to_vec())
                        .unwrap();
                    state_records += states.len();
                    owned.extend(states);
                    owned.push(layer);
                }
                owned.push(machine);
            }
            assert!(owned.iter().all(CoreHandle::is_alive));
            let replacement_bytes = run(&file, &bytes, index);
            assert_eq!(
                file.with_file_mut(|file| file.replace_artboard(index, replacement_bytes)),
                ImportResult::Success
            );
            assert!(
                owned.iter().all(|handle| !handle.is_alive()),
                "{name} artboard {index}: outgoing owned occurrence remains live"
            );
            let replacement = file
                .with_file(|file| file.artboard_at_source(index))
                .unwrap();
            assert!(replacement.is_alive());
            assert_ne!(replacement, source);
        }
    }
    assert!(keyed_records > 0, "exercise owned keyframes");
    assert!(state_records > 0, "exercise owned states");
}
