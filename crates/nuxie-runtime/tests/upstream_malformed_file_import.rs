//! 6cd5d108 malformed_file_import_test.cpp, using the native import owner.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    artboard::Artboard,
    core::{CoreHandle, binary_data_reader::BinaryDataReader, binary_reader::BinaryReader},
    file::ImportResult,
    nested_artboard::NestedArtboard,
    viewmodel::{
        viewmodel::ViewModel, viewmodel_instance::ViewModelInstance,
        viewmodel_instance_viewmodel::ViewModelInstanceViewModel,
    },
};
use nuxie_runtime::{File, RuntimeFactoryHandle, RuntimeFileHandle};

fn bytes(name: &str) -> Vec<u8> {
    let root = std::path::PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned upstream checkout"),
    );
    std::fs::read(root.join("tests/unit_tests/assets").join(name)).unwrap()
}

fn import(bytes: &[u8]) -> (Option<RuntimeFileHandle>, ImportResult) {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    let mut result = ImportResult::Success;
    let file = File::import(bytes, factory, Some(&mut result), None, None);
    assert_eq!(file.is_none(), result != ImportResult::Success);
    (file, result)
}

fn fixture(name: &str) -> RuntimeFileHandle {
    import(&bytes(name)).0.unwrap()
}

fn name_length(encoded: &[u8]) -> Vec<u8> {
    let bytes = bytes("data_binding_test_2.riv");
    let file = import(&bytes).0.unwrap();
    let name = file
        .with_file(|file| file.artboard_handle(0).unwrap())
        .with_downcast::<Artboard, _>(|artboard| artboard.name().to_owned())
        .unwrap();
    assert!(!name.is_empty() && name.len() < 128);
    let mut needle = vec![name.len() as u8];
    needle.extend_from_slice(name.as_bytes());
    let offset = bytes
        .windows(needle.len())
        .position(|window| window == needle)
        .unwrap();
    let mut patched = bytes[..offset].to_vec();
    patched.extend_from_slice(encoded);
    patched.extend_from_slice(&bytes[offset + 1..]);
    patched
}

#[test]
fn truncated_file_import_never_crashes() {
    let bytes = bytes("data_binding_test_2.riv");
    for length in 0..=bytes.len() {
        import(&bytes[..length]);
    }
    assert!(
        fixture("data_binding_test_2.riv")
            .with_file(File::artboard)
            .is_some()
    );
}

#[test]
fn string_lengths_past_end_and_overlong_varuints_are_refused() {
    for encoded in [
        vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x20],
        vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 1],
        vec![
            0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 1,
        ],
    ] {
        assert_eq!(import(&name_length(&encoded)).1, ImportResult::Malformed);
    }
    let file = fixture("data_binding_test_2.riv");
    let length = file
        .with_file(|file| file.artboard_handle(0).unwrap())
        .with_downcast::<Artboard, _>(|artboard| artboard.name().len())
        .unwrap();
    let mut encoded = vec![0x80; 10];
    encoded[0] = length as u8 | 0x80;
    encoded[9] = 2;
    assert_eq!(import(&name_length(&encoded)).1, ImportResult::Malformed);
    encoded[9] = 0;
    assert_eq!(import(&name_length(&encoded)).1, ImportResult::Success);
}

#[test]
fn primitive_varuint_boundaries() {
    let mut overflowing = [0x80, 0x80, 0x80, 0x80, 0x10];
    let mut reader = BinaryDataReader::new(&mut overflowing, 5);
    reader.read_var_uint32();
    assert!(reader.did_overflow());
    let mut largest = [0xff, 0xff, 0xff, 0xff, 0x0f];
    let mut reader = BinaryDataReader::new(&mut largest, 5);
    assert_eq!(reader.read_var_uint32(), u32::MAX);
    assert!(!reader.did_overflow());
    let largest = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 1];
    let mut reader = BinaryReader::new(&largest);
    assert_eq!(reader.read_var_uint64(), u64::MAX);
    assert!(!reader.did_overflow());
    assert!(reader.position().is_empty());
}

fn sources(bytes: &[u8]) -> (RuntimeFileHandle, Vec<Option<CoreHandle>>) {
    let file = import(bytes).0.unwrap();
    assert_eq!(file.with_file(File::artboard_count), 2);
    let mut sources = Vec::new();
    for index in 0..2 {
        let artboard = file.with_file(|file| file.artboard_handle(index)).unwrap();
        let nested = artboard
            .with_downcast::<Artboard, _>(|artboard| artboard.find_all_handles::<NestedArtboard>())
            .unwrap();
        assert_eq!(nested.len(), 1);
        sources.push(
            nested[0]
                .with_downcast::<NestedArtboard, _>(NestedArtboard::source_artboard)
                .unwrap(),
        );
        assert!(file.with_file(|file| file.artboard_at(index)).is_some());
    }
    (file, sources)
}

#[test]
fn a_nested_artboard_cycle_loads_without_references_closing_it() {
    let mut bytes = bytes("nested_artboard_cycle.riv");
    let (_, mutual) = sources(&bytes);
    assert!(mutual.iter().all(Option::is_none));
    let offset = bytes
        .windows(4)
        .position(|window| window == [0x5c, 0xc5, 1, 0])
        .unwrap();
    bytes[offset + 3] = 1;
    let (file, references) = sources(&bytes);
    assert_eq!(
        references[0],
        file.with_file(|file| file.artboard_handle(1))
    );
    assert!(references[1].is_none());
}

fn nested(instance: &CoreHandle, name: &str) -> Option<CoreHandle> {
    instance
        .with_downcast::<ViewModelInstance, _>(|instance| instance.property_value_named(name))
        .flatten()?
        .with_downcast::<ViewModelInstanceViewModel, _>(
            ViewModelInstanceViewModel::reference_view_model_instance,
        )
        .flatten()
}

fn depth(instance: CoreHandle) -> usize {
    let mut count = 1;
    let mut next = nested(&instance, "child");
    while let Some(instance) = next {
        count += 1;
        next = nested(&instance, "child");
    }
    count
}

#[test]
fn self_referential_models_create_finite_instances() {
    let file = fixture("viewmodel_self_reference.riv");
    let instance = file
        .with_file(|file| file.create_view_model_instance_for_name("Node"))
        .unwrap();
    assert!(nested(&instance, "child").is_none());
    assert!(
        file.with_file(
            |file| file.create_view_model_instance_for_artboard(file.artboard().unwrap())
        )
        .is_some()
    );
}

#[test]
fn deep_model_and_authored_instance_chains_stop_nesting() {
    let file = fixture("viewmodel_deep_chain.riv");
    assert_eq!(
        depth(
            file.with_file(|file| file.create_view_model_instance_for_name("VM0"))
                .unwrap()
        ),
        256
    );
    let file = fixture("viewmodel_instance_chain.riv");
    assert_eq!(
        depth(
            file.with_file(|file| file
                .create_default_view_model_instance_for_artboard(file.artboard().unwrap()))
                .unwrap()
        ),
        256
    );
    file.with_file(|file| {
        let instance = file
            .view_model_named("Node")
            .unwrap()
            .with_downcast::<ViewModel, _>(|model| model.instance_at(0))
            .flatten()
            .unwrap();
        file.complete_view_model_properties(&instance);
    });
}

#[test]
fn model_graph_fan_out_stops_at_instance_budget() {
    let file = fixture("viewmodel_fan_out.riv");
    let instance = file
        .with_file(|file| {
            file.create_bounded_view_model_instance(file.view_model_named("L0").unwrap(), 1000)
        })
        .unwrap();
    let mut pending = vec![instance];
    let mut count = 0;
    while let Some(instance) = pending.pop() {
        count += 1;
        for name in ["a", "b"] {
            if let Some(child) = nested(&instance, name) {
                pending.push(child);
            }
        }
    }
    assert_eq!(count, 1000);
}
