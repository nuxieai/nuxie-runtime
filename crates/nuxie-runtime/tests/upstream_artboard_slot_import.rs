//! Literal translation of artboard_slot_import_test.cpp at 439ffe0c.
#[path = "support/riv_bytes.rs"]
mod riv_bytes;
#[path = "support/import_439.rs"]
mod support;
use nuxie_runtime::source::{
    generated::{
        component_base::ComponentBase,
        core_registry::CoreRegistry,
        inputs::keyboard_input_base::KeyboardInputBase,
        node_base::NodeBase,
        script_input_number_base::ScriptInputNumberBase,
        scripted::{
            scripted_drawable_base::ScriptedDrawableBase,
            scripted_interpolator_base::ScriptedInterpolatorBase,
        },
    },
    node::Node,
    scripted::scripted_object::ScriptedObject,
};
use riv_bytes::{RivBytes, write_artboard};
use support::*;
fn write_nodes(riv: &mut RivBytes, first: u64) {
    for (name, parent) in [("first", 0), ("second", 0), ("child", first)] {
        riv.object(NodeBase::TYPE_KEY);
        riv.prop_string(ComponentBase::NAME_PROPERTY_KEY, name);
        riv.prop_uint(ComponentBase::PARENT_ID_PROPERTY_KEY, parent);
        riv.end();
    }
}
fn require_nodes(file: &RuntimeFileHandle) {
    let child = artboard(file)
        .with_downcast::<Artboard, _>(|a| a.find_handle::<Node>("child"))
        .flatten()
        .expect("child");
    let parent = child
        .with(|o| o.as_component().unwrap().parent_handle())
        .flatten()
        .expect("parent");
    assert_eq!(
        parent
            .with(|o| o.as_component().unwrap().name().to_owned())
            .unwrap(),
        "first"
    );
}
fn unknown(riv: &mut RivBytes) {
    assert!(CoreRegistry::make_core_box(16000).is_none());
    riv.object(16000);
    riv.end();
}
fn input(riv: &mut RivBytes, parent: u64) {
    riv.object(ScriptInputNumberBase::TYPE_KEY);
    riv.prop_uint(ComponentBase::PARENT_ID_PROPERTY_KEY, parent);
    riv.end();
}
#[test]
fn a_script_input_of_an_unreadable_interpolator_takes_no_slot() {
    let mut riv = RivBytes::default();
    write_artboard(&mut riv);
    unknown(&mut riv);
    input(&mut riv, 0);
    write_nodes(&mut riv, 2);
    let file = import(&riv.bytes());
    require_nodes(&file);
    let objects = objects(&file);
    assert_eq!(objects.len(), 5);
    assert!(objects[1].is_none());
}
#[test]
fn a_script_input_of_an_unreadable_component_keeps_its_slot() {
    let mut riv = RivBytes::default();
    write_artboard(&mut riv);
    unknown(&mut riv);
    input(&mut riv, 1);
    write_nodes(&mut riv, 3);
    let file = import(&riv.bytes());
    require_nodes(&file);
    let objects = objects(&file);
    assert_eq!(objects.len(), 6);
    assert!(objects[1].is_none());
    assert!(objects[2].is_none());
}
#[test]
fn a_script_input_of_an_unreadable_owner_ignores_the_previous_one() {
    for component_owner in [false, true] {
        let mut riv = RivBytes::default();
        write_artboard(&mut riv);
        riv.object(ScriptedDrawableBase::TYPE_KEY);
        riv.prop_uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0);
        riv.end();
        unknown(&mut riv);
        input(&mut riv, if component_owner { 2 } else { 0 });
        let next = if component_owner { 4 } else { 3 };
        write_nodes(&mut riv, next);
        let file = import(&riv.bytes());
        require_nodes(&file);
        let objects = objects(&file);
        assert_eq!(objects.len(), next as usize + 3);
        let drawable = objects[1].as_ref().unwrap();
        assert!(drawable.with(|o| o.as_scripted_object().is_some()).unwrap());
        assert!(ScriptedObject::custom_properties(drawable).is_empty());
    }
}
#[test]
fn a_keyboard_input_with_no_listener_keeps_its_artboard_slot() {
    let mut riv = RivBytes::default();
    write_artboard(&mut riv);
    riv.object(KeyboardInputBase::TYPE_KEY);
    riv.end();
    write_nodes(&mut riv, 2);
    let file = import(&riv.bytes());
    require_nodes(&file);
    assert!(objects(&file)[1].is_none());
}
#[test]
fn a_scripted_interpolators_input_takes_no_artboard_slot() {
    let mut riv = RivBytes::default();
    write_artboard(&mut riv);
    riv.object(ScriptedInterpolatorBase::TYPE_KEY);
    riv.end();
    riv.object(ScriptInputNumberBase::TYPE_KEY);
    riv.end();
    write_nodes(&mut riv, 2);
    let file = import(&riv.bytes());
    require_nodes(&file);
    let objects = objects(&file);
    assert_eq!(objects.len(), 5);
    assert_eq!(
        objects[1]
            .as_ref()
            .unwrap()
            .with(|o| o.core_type())
            .unwrap(),
        ScriptedInterpolatorBase::TYPE_KEY
    );
}
