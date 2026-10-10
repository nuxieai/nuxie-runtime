//! Literal translation of component_list_no_state_machine_test.cpp at 439ffe0c.
#[path = "support/import_439.rs"]
mod support;
use nuxie_runtime::source::{
    artboard_component_list::ArtboardComponentList,
    constraints::scrolling::scroll_constraint::ScrollConstraint,
    core::{
        binary_reader::BinaryReader,
        field_types::{
            core_bool_type::CoreBoolType, core_color_type::CoreColorType,
            core_double_type::CoreDoubleType, core_string_type::CoreStringType,
            core_uint_type::CoreUintType,
        },
    },
    generated::{
        animation::{
            linear_animation_base::LinearAnimationBase, state_machine_base::StateMachineBase,
        },
        artboard_base::ArtboardBase,
        core_registry::CoreRegistry,
    },
    runtime_header::RuntimeHeader,
};
use support::*;
fn skip_object(reader: &mut BinaryReader<'_>, header: &RuntimeHeader) -> u16 {
    let type_key = reader.read_var_uint_as::<u16>();
    let mut object = CoreRegistry::make_core_box(type_key.into());
    loop {
        let property_key = reader.read_var_uint_as::<u16>();
        if property_key == 0 || reader.has_error() {
            break;
        }
        if object
            .as_mut()
            .is_some_and(|o| o.deserialize(property_key, reader))
        {
            continue;
        }
        let mut id = CoreRegistry::property_field_id(property_key.into());
        if id == -1 {
            id = header.property_field_id(property_key.into());
        }
        match id {
            CoreUintType::ID => {
                reader.read_var_uint64();
            }
            CoreBoolType::ID => {
                CoreBoolType::deserialize(reader);
            }
            CoreStringType::ID => {
                CoreStringType::deserialize(reader);
            }
            CoreDoubleType::ID => {
                CoreDoubleType::deserialize(reader);
            }
            CoreColorType::ID => {
                CoreColorType::deserialize(reader);
            }
            _ => panic!("unknown property {property_key}"),
        }
    }
    type_key
}
fn uint(out: &mut Vec<u8>, mut value: u64) {
    while value >= 128 {
        out.push((value as u8 & 127) | 128);
        value >>= 7;
    }
    out.push(value as u8);
}
fn without_state_machines(bytes: &[u8], artboard_index: i32) -> Vec<u8> {
    let mut reader = BinaryReader::new(bytes);
    let mut header = RuntimeHeader::default();
    assert!(RuntimeHeader::read(&mut reader, &mut header));
    let mut out = bytes[..bytes.len() - reader.position().len()].to_vec();
    let mut artboard = -1;
    let mut dropping = false;
    let mut replaced = false;
    while !reader.reached_end() {
        let start = bytes.len() - reader.position().len();
        let key = skip_object(&mut reader, &header);
        assert!(!reader.has_error());
        if key == ArtboardBase::TYPE_KEY {
            artboard += 1;
            dropping = false;
        } else if artboard == artboard_index && key == StateMachineBase::TYPE_KEY && !replaced {
            uint(&mut out, LinearAnimationBase::TYPE_KEY.into());
            uint(&mut out, 0);
            dropping = true;
            replaced = true;
        }
        if !dropping {
            out.extend_from_slice(&bytes[start..bytes.len() - reader.position().len()]);
        }
    }
    assert!(replaced);
    out
}
fn import_without_item_state_machines(asset: &str) -> RuntimeFileHandle {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let bytes = std::fs::read(
        std::path::PathBuf::from(root)
            .join("tests/unit_tests/assets")
            .join(asset),
    )
    .expect("asset");
    let file = import(&without_state_machines(&bytes, 1));
    let item = file.with_file(|f| f.artboard_at_source(1)).unwrap();
    item.with_downcast::<Artboard, _>(|a| {
        assert_eq!(a.state_machine_count(), 0);
        assert!(a.animation_count() > 0);
    })
    .unwrap();
    file
}
#[test]
fn rebinding_a_list_whose_rows_have_no_state_machine() {
    let file = import_without_item_state_machines("component_list_1.riv");
    let artboard = file.with_file(|f| f.artboard_named("Main")).unwrap();
    let instance = file
        .with_file_mut(|f| {
            f.create_default_view_model_instance_for_artboard(artboard.core_handle())
        })
        .unwrap();
    artboard.bind_view_model_instance(Some(instance.clone()));
    artboard.advance_default(0.0);
    let list = artboard
        .with_artboard(|a| a.find_handle::<ArtboardComponentList>("List"))
        .unwrap();
    list.with_downcast::<ArtboardComponentList, _>(|list| {
        assert!(list.artboard_count() > 0);
        for i in 0..list.artboard_count() {
            assert!(list.artboard_instance(i as i32).is_some());
            assert!(list.state_machine_instance(i as i32).is_none());
        }
    })
    .unwrap();
    artboard.bind_view_model_instance(Some(instance));
    artboard.advance_default(0.0);
    assert!(
        list.with_downcast::<ArtboardComponentList, _>(|l| l
            .artboard_instance(0)
            .unwrap()
            .data_context()
            .is_some())
            .unwrap()
    );
}
#[test]
fn scrolling_a_virtualized_list_whose_rows_have_no_state_machine() {
    let file = import_without_item_state_machines("component_list_virtualized.riv");
    let artboard = file.with_file(|f| f.artboard_named("Main")).unwrap();
    let instance = file
        .with_file_mut(|f| {
            f.create_default_view_model_instance_for_artboard(artboard.core_handle())
        })
        .unwrap();
    artboard.bind_view_model_instance(Some(instance));
    artboard.advance_default(0.0);
    let scroll = artboard.with_artboard(|a| a.find_all_handles::<ScrollConstraint>()[0].clone());
    for index in [5, 10, 0, 15, 2] {
        scroll
            .with_downcast_mut::<ScrollConstraint, _>(|s| s.set_scroll_index(index as f32))
            .unwrap();
        artboard.advance_default(0.0);
    }
    let list = artboard
        .with_artboard(|a| a.find_handle::<ArtboardComponentList>("List"))
        .unwrap();
    assert!(
        list.with_downcast::<ArtboardComponentList, _>(|l| l.artboard_count() > 0)
            .unwrap()
    );
}
