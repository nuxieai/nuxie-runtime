use super::*;
use crate::mechanical_port::source::{
    core::{CoreArena, CoreObject, PropertySetterCompletion},
    data_bind::data_bind::DataBind,
    generated::core_registry::{CoreCapabilities, CoreField, CoreRegistry, CoreRegistryObject},
    shapes::points_path::PointsPath,
};

fn dirty_after_set(key: i32) -> ComponentDirt {
    let mut path = PointsPath::default();
    path.as_path_mut().unwrap().deferred_path_dirt = true;
    path.as_component_mut()
        .unwrap()
        .set_dirt(ComponentDirt::NONE);
    CoreRegistry::set_double(&mut path, key, 1.0);
    path.as_component().unwrap().dirt()
}

#[test]
fn legacy_x_alias_preserves_most_derived_path_dirty_callback() {
    let canonical = dirty_after_set(13);
    assert!(has_dirt(canonical, ComponentDirt::PATH));
    assert_eq!(dirty_after_set(9), canonical);
}

#[test]
fn legacy_y_alias_preserves_most_derived_path_dirty_callback() {
    let canonical = dirty_after_set(14);
    assert!(has_dirt(canonical, ComponentDirt::PATH));
    assert_eq!(dirty_after_set(10), canonical);
}

#[test]
fn every_native_path_retains_derived_dirty_behavior_for_both_aliases() {
    fn verify<T: CoreObject + Default>() {
        for (canonical, alias) in [(13, 9), (14, 10)] {
            let run = |key| {
                let mut path = T::default();
                path.as_path_mut().unwrap().deferred_path_dirt = true;
                path.as_component_mut()
                    .unwrap()
                    .set_dirt(ComponentDirt::NONE);
                CoreRegistry::set_double(&mut path, key, 1.0);
                path.as_component().unwrap().dirt()
            };
            let expected = run(canonical);
            assert!(
                has_dirt(expected, ComponentDirt::PATH),
                "{}",
                std::any::type_name::<T>()
            );
            assert_eq!(
                run(alias),
                expected,
                "{} key{alias}",
                std::any::type_name::<T>()
            );
        }
    }
    verify::<crate::source::shapes::points_path::PointsPath>();
    verify::<crate::source::shapes::rectangle::Rectangle>();
    verify::<crate::source::shapes::triangle::Triangle>();
    verify::<crate::source::shapes::ellipse::Ellipse>();
    verify::<crate::source::shapes::list_path::ListPath>();
    verify::<crate::source::shapes::polygon::Polygon>();
    verify::<crate::source::shapes::star::Star>();
}

#[test]
fn all_native_node_bindings_share_alias_storage_and_same_value_gate() {
    fn verify<T: CoreObject + Default>() {
        let mut object = T::default();
        for (canonical, alias) in [(13, 9), (14, 10)] {
            CoreRegistry::set_double(&mut object, alias, 3.0);
            assert_eq!(CoreRegistry::get_double(&mut object, canonical), 3.0);
            assert_eq!(CoreRegistry::get_double(&mut object, alias), 3.0);
            object
                .as_component_mut()
                .unwrap()
                .set_dirt(ComponentDirt::NONE);
            CoreRegistry::set_double(&mut object, alias, 3.0);
            assert_eq!(
                object.as_component().unwrap().dirt(),
                ComponentDirt::NONE,
                "{}",
                std::any::type_name::<T>()
            );
        }
    }
    verify::<crate::mechanical_port::source::scripted::scripted_transition::ScriptedTransition>();
    verify::<crate::mechanical_port::source::shapes::shape::Shape>();
    verify::<crate::mechanical_port::source::text::text::Text>();
    verify::<crate::mechanical_port::source::node::Node>();
    verify::<crate::mechanical_port::source::foreground_layout_drawable::ForegroundLayoutDrawable>(
    );
    verify::<crate::mechanical_port::source::nested_artboard::NestedArtboard>();
    verify::<crate::mechanical_port::source::artboard_component_list::ArtboardComponentList>();
    verify::<crate::mechanical_port::source::solo::Solo>();
    verify::<crate::mechanical_port::source::scripted::scripted_drawable::ScriptedDrawable>();
    verify::<crate::mechanical_port::source::scripted::scripted_layout::ScriptedLayout>();
    verify::<crate::mechanical_port::source::nested_artboard_layout::NestedArtboardLayout>();
    verify::<crate::mechanical_port::source::layout::n_sliced_node::NSlicedNode>();
    verify::<crate::mechanical_port::source::shapes::points_path::PointsPath>();
    verify::<crate::mechanical_port::source::shapes::rectangle::Rectangle>();
    verify::<crate::mechanical_port::source::shapes::triangle::Triangle>();
    verify::<crate::mechanical_port::source::shapes::ellipse::Ellipse>();
    verify::<crate::mechanical_port::source::shapes::list_path::ListPath>();
    verify::<crate::mechanical_port::source::shapes::polygon::Polygon>();
    verify::<crate::mechanical_port::source::shapes::star::Star>();
    verify::<crate::mechanical_port::source::shapes::image::Image>();
    verify::<crate::mechanical_port::source::layout_component::LayoutComponent>();
    verify::<crate::mechanical_port::source::artboard::Artboard>();
    verify::<crate::mechanical_port::source::nested_artboard_leaf::NestedArtboardLeaf>();
    verify::<crate::mechanical_port::source::text::text_input_cursor::TextInputCursor>();
    verify::<crate::mechanical_port::source::text::text_input_text::TextInputText>();
    verify::<crate::mechanical_port::source::text::text_input_selected_text::TextInputSelectedText>(
    );
    verify::<crate::mechanical_port::source::text::text_input::TextInput>();
    verify::<crate::mechanical_port::source::text::text_input_selection::TextInputSelection>();
}

#[test]
fn alias_completion_notifies_canonical_key_after_derived_dirty_work() {
    for (canonical_key, alias_key) in [(13, 9), (14, 10)] {
        let arena = CoreArena::default();
        let owner = arena.insert(PointsPath::default());
        let canonical = arena.insert(DataBind::new(0, canonical_key, 0));
        let alias = arena.insert(DataBind::new(0, alias_key, 0));
        owner.with_mut(|object| {
            object.as_path_mut().unwrap().deferred_path_dirt = true;
            object
                .as_component_mut()
                .unwrap()
                .set_dirt(ComponentDirt::NONE);
            for observer in [&canonical, &alias] {
                observer.with_mut(|observer| {
                    object
                        .core_mut()
                        .add_property_observer(observer.as_data_bind_mut().unwrap())
                });
            }
        });
        let mut completion = PropertySetterCompletion::default();
        owner.with_mut(|object| {
            CoreRegistry::set_double_with_completion(object, alias_key as i32, 7.0, &mut completion)
        });
        assert!(
            owner
                .with(|object| has_dirt(object.as_component().unwrap().dirt(), ComponentDirt::PATH))
                .unwrap()
        );
        for observer in [&canonical, &alias] {
            assert_eq!(
                observer.with(|object| object.as_data_bind().unwrap().dirt()),
                Some(0)
            );
        }
        completion.finish();
        assert_eq!(
            canonical.with(|object| object.as_data_bind().unwrap().dirt()),
            Some(u32::from(ComponentDirt::BINDINGS_TARGET.0))
        );
        assert_eq!(
            alias.with(|object| object.as_data_bind().unwrap().dirt()),
            Some(0)
        );
    }
}

#[test]
fn alias_preserves_nan_and_signed_zero_equality_behavior() {
    let mut path = PointsPath::default();
    path.as_path_mut().unwrap().deferred_path_dirt = true;
    path.as_component_mut()
        .unwrap()
        .set_dirt(ComponentDirt::NONE);
    CoreRegistry::set_double(&mut path, 9, -0.0);
    assert_eq!(path.as_component().unwrap().dirt(), ComponentDirt::NONE);
    assert_eq!(
        CoreRegistry::get_double(&mut path, 13).to_bits(),
        0.0f32.to_bits()
    );
    for key in [9, 13, 9] {
        path.as_component_mut()
            .unwrap()
            .set_dirt(ComponentDirt::NONE);
        CoreRegistry::set_double(&mut path, key, f32::NAN);
        assert!(CoreRegistry::get_double(&mut path, key).is_nan());
        assert!(has_dirt(
            path.as_component().unwrap().dirt(),
            ComponentDirt::PATH
        ));
    }
}

#[derive(Default)]
struct RecordingRegistry {
    fields: Vec<CoreField>,
}
impl CoreCapabilities for RecordingRegistry {}
impl CoreRegistryObject for RecordingRegistry {
    fn as_registry_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_registry_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn is_type_of(&self, _: u16) -> bool {
        false
    }
    fn set_double_with_completion(
        &mut self,
        field: CoreField,
        _: f32,
        _: &mut PropertySetterCompletion,
    ) {
        self.fields.push(field);
    }
    fn set_uint_with_completion(&mut self, _: CoreField, _: u32, _: &mut PropertySetterCompletion) {
    }
    fn set_string_with_completion(
        &mut self,
        _: CoreField,
        _: String,
        _: &mut PropertySetterCompletion,
    ) {
    }
    fn set_color_with_completion(
        &mut self,
        _: CoreField,
        _: i32,
        _: &mut PropertySetterCompletion,
    ) {
    }
    fn set_bool_with_completion(
        &mut self,
        _: CoreField,
        _: bool,
        _: &mut PropertySetterCompletion,
    ) {
    }
    fn set_callback_with_completion(
        &mut self,
        _: CoreField,
        _: crate::source::core::field_types::core_callback_type::CallbackData<'_>,
        _: &mut PropertySetterCompletion,
    ) {
    }
    fn set_int_with_completion(&mut self, _: CoreField, _: i32, _: &mut PropertySetterCompletion) {}
    fn get_uint(&mut self, _: CoreField) -> u32 {
        0
    }
    fn get_string(&mut self, _: CoreField) -> String {
        String::new()
    }
    fn get_color(&mut self, _: CoreField) -> i32 {
        0
    }
    fn get_bool(&mut self, _: CoreField) -> bool {
        false
    }
    fn get_double(&mut self, _: CoreField) -> f32 {
        0.0
    }
    fn get_int(&mut self, _: CoreField) -> i32 {
        0
    }
}

#[test]
fn open_custom_registry_still_receives_original_field_variants() {
    let mut object = RecordingRegistry::default();
    for key in [9, 13, 10, 14] {
        CoreRegistry::set_double(&mut object, key, 1.0);
    }
    assert_eq!(
        object.fields,
        [
            CoreField::NodeXArtboard,
            CoreField::NodeX,
            CoreField::NodeYArtboard,
            CoreField::NodeY
        ]
    );
}
