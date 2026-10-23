//! Picture measurement across lifecycle attachment and hosted instance reuse.
use super::*;

#[test]
fn same_mode_attachment_does_not_visit_picture_descendants() {
    for css in [false, true] {
        let (arena, root, image) = content_picture_fixture(false, css, true);
        let group = image
            .with(|image| image.component_parent_handle())
            .flatten()
            .unwrap();
        let mut context = ObjectContext {
            arena,
            objects: vec![Some(root.clone()), Some(root)],
        };
        image.with_mut(|_| {
            assert_eq!(
                group.with_mut(|group| group.on_added_dirty(&mut context)),
                Some(StatusCode::Ok)
            );
        });
        measured_picture(&image, css);
    }
}

fn measured_picture(image: &CoreHandle, css: bool) {
    let size = image
        .with_mut(|image| {
            image
                .as_intrinsically_sizeable_mut()
                .unwrap()
                .measure_layout(
                    f32::NAN,
                    LayoutMeasureMode::Undefined,
                    f32::NAN,
                    LayoutMeasureMode::Undefined,
                )
        })
        .unwrap();
    near(size.x, if css { 480.0 } else { 160.0 });
    near(size.y, if css { 270.0 } else { 90.0 });
}

use crate::source::{
    artboard_component_list::ArtboardComponentList,
    data_bind::data_bind::DataBind,
    generated::{
        artboard_base::ArtboardBase, backboard_base::BackboardBase, component_base::ComponentBase,
        layout::layout_component_style_base::LayoutComponentStyleBase,
        layout_component_base::LayoutComponentBase, nested_artboard_base::NestedArtboardBase,
        nested_artboard_layout_base::NestedArtboardLayoutBase,
        nested_artboard_leaf_base::NestedArtboardLeafBase,
    },
    viewmodel::viewmodel_instance_artboard::ViewModelInstanceArtboard,
};
use crate::{
    Artboard, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
};
use nuxie_render_api::{PersistentFactory, RecordingFactory};

struct ObjectContext {
    arena: CoreArena,
    objects: Vec<Option<CoreHandle>>,
}
impl CoreContext for ObjectContext {
    fn core_arena(&self) -> &CoreArena {
        &self.arena
    }
    fn resolve_handle(&self, id: u32) -> Option<CoreHandle> {
        self.objects.get(id as usize).cloned().flatten()
    }
}

// Append through the same dirty and clean lifecycle used by import and copy.
fn append_to_artboard(root: &CoreHandle, object: CoreHandle) {
    let mut context = root
        .with_downcast_mut::<Artboard, _>(|root| {
            root.add_object(Some(object.clone()));
            ObjectContext {
                arena: root.core_arena().clone(),
                objects: root.objects().to_vec(),
            }
        })
        .unwrap();
    object.with_mut(|object| {
        object
            .as_component_mut()
            .unwrap()
            .base
            .set_parent_id_value(0);
        assert_eq!(object.on_added_dirty(&mut context), StatusCode::Ok);
        assert_eq!(object.on_added_clean(&mut context), StatusCode::Ok);
    });
}

fn append_picture(root: &CoreHandle, css: bool) {
    if css {
        let record = root
            .insert_sibling(crate::css_layout::LayoutCssStyle::default())
            .unwrap();
        append_to_artboard(root, record);
    }
    let image = root.insert_sibling(Image::default()).unwrap();
    image.with_mut(|image| image.set_string(CoreField::ComponentName, "Mode picture".into()));
    append_to_artboard(root, image.clone());
    Image::set_asset_occurrence(&image, image.insert_sibling(ImageAsset::default()));
    declare(&image, 480.0, 270.0);
    decode(&image, 160, 90);
}

fn check_hosted_picture(instance: &RuntimeArtboardInstanceHandle, css: bool) {
    // Measurement deliberately runs while the owning artboard is borrowed.
    instance.with_artboard_mut(|root| {
        let image = root.find_handle::<Image>("Mode picture").unwrap();
        measured_picture(&image, css);
    });
}

fn import(bytes: &[u8]) -> RuntimeFileHandle {
    let mut factory = PersistentFactory::new(RecordingFactory::default());
    File::import(
        bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap()
}

#[test]
fn css_picture_list_row_keeps_mode_when_recycled_and_rebound() {
    let path = std::path::PathBuf::from(std::env::var_os("RIVE_RUNTIME_DIR").unwrap())
        .join("tests/unit_tests/assets/component_list_virtualized.riv");
    let file = import(&std::fs::read(path).unwrap());
    for source in file.with_file(File::artboards) {
        if source
            .with_downcast::<Artboard, _>(|source| source.base.name() != "Main")
            .unwrap()
        {
            append_picture(&source, true);
        }
    }
    let main = file.with_file(|file| file.artboard_named("Main")).unwrap();
    let model = file
        .with_file(|file| file.create_default_view_model_instance_for_artboard(main.core_handle()))
        .unwrap();
    main.bind_view_model_instance(Some(model));
    main.advance_default(0.0);
    main.advance_default(0.0);
    let list = main
        .with_artboard(|main| main.find_handle::<ArtboardComponentList>("List"))
        .unwrap();
    let (first, second, row) = list
        .with_downcast::<ArtboardComponentList, _>(|list| {
            assert!(list.virtualization_enabled());
            (0..list.artboard_count())
                .find_map(|first| {
                    let row = list.artboard_instance(first as i32)?;
                    let source = row.with_artboard(|row| row.base.artboard_source_handle())?;
                    let second = (first + 1..list.artboard_count()).find(|second| {
                        list.artboard_instance(*second as i32).is_some_and(|other| {
                            other
                                .with_artboard(|other| other.base.artboard_source_handle())
                                .as_ref()
                                == Some(&source)
                        })
                    })?;
                    Some((first as i32, second as i32, row))
                })
                .unwrap()
        })
        .unwrap();
    check_hosted_picture(&row, true);
    list.with_downcast_mut::<ArtboardComponentList, _>(|list| {
        assert_ne!(list.list_item(first), list.list_item(second));
        list.remove_virtualizable(second);
        list.remove_virtualizable(first);
    });
    ArtboardComponentList::add_virtualizable_occurrence(&list, second);
    let recycled = list
        .with_downcast::<ArtboardComponentList, _>(|list| list.artboard_instance(second))
        .flatten()
        .unwrap();
    assert_eq!(
        recycled.core_handle(),
        row.core_handle(),
        "the exact instance is reused for another item"
    );
    check_hosted_picture(&recycled, true);
    let copy = recycled.instance().unwrap();
    check_hosted_picture(&copy, true);
}

fn nested_sources_file() -> RuntimeFileHandle {
    use crate::source::layout_component::layout_measured_content_tests::Riv;
    let mut riv = Riv::default();
    riv.object(BackboardBase::TYPE_KEY).end();
    // Main owns copies using each nested host implementation.
    riv.object(ArtboardBase::TYPE_KEY)
        .string(ComponentBase::NAME_PROPERTY_KEY, "Main")
        .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, 800.0)
        .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 600.0)
        .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, 4)
        .end();
    for kind in [
        NestedArtboardBase::TYPE_KEY,
        NestedArtboardLayoutBase::TYPE_KEY,
        NestedArtboardLeafBase::TYPE_KEY,
    ] {
        riv.object(kind)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
            .uint(NestedArtboardBase::ARTBOARD_ID_PROPERTY_KEY, 1)
            .end();
    }
    riv.object(LayoutComponentStyleBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .end();
    for name in ["Translated", "CSS"] {
        riv.object(ArtboardBase::TYPE_KEY)
            .string(ComponentBase::NAME_PROPERTY_KEY, name)
            .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, 800.0)
            .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 600.0)
            .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, 1)
            .end();
        riv.object(LayoutComponentStyleBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
            .end();
    }
    let file = import(&riv.bytes());
    for source in file.with_file(File::artboards).into_iter().skip(1) {
        let css = source
            .with_downcast::<Artboard, _>(|source| source.base.name() == "CSS")
            .unwrap();
        append_picture(&source, css);
    }
    file
}

#[test]
fn picture_mode_follows_data_bound_nested_swaps_and_component_copies() {
    let file = nested_sources_file();
    for copy in 0..3 {
        if copy == 2 {
            // Hosting changes must not make a translated child inherit a
            // CSS parent's mode. Also exercise factory rerouting in this case.
            let parent = file.with_file(File::artboards)[0].clone();
            let record = parent
                .insert_sibling(crate::css_layout::LayoutCssStyle::default())
                .unwrap();
            append_to_artboard(&parent, record);
        }
        let mut main = file.with_file(|file| file.artboard_named("Main")).unwrap();
        if copy == 1 {
            main = main.instance().unwrap();
        } else if copy == 2 {
            let mut factory = PersistentFactory::new(RecordingFactory::default());
            main = main
                .instance_with_factory(RuntimeFactoryHandle::from_factory(&mut factory))
                .unwrap();
        }
        let hosts = main.with_artboard(|main| main.nested_artboards());
        assert_eq!(hosts.len(), 3);
        for (index, host) in hosts.into_iter().enumerate() {
            let supplied = file.with_file(|file| file.artboard_named("CSS")).unwrap();
            host.with_mut(|host| {
                host.as_nested_artboard_mut()
                    .unwrap()
                    .referenced_artboard_instance(supplied.clone())
            });
            check_hosted_picture(&supplied, true);
            let property = host
                .insert_sibling(ViewModelInstanceArtboard::default())
                .unwrap();
            let bind = host
                .insert_sibling(DataBind::new(
                    0,
                    u32::from(NestedArtboardBase::ARTBOARD_ID_PROPERTY_KEY),
                    0,
                ))
                .unwrap();
            bind.with_downcast_mut::<DataBind, _>(|bind| {
                bind.set_source(property.clone());
                bind.set_target(Some(host.clone()));
            });
            DataBind::bind_handle(&bind);
            for (source, css) in [
                (Some("Translated"), false),
                (Some("CSS"), true),
                (None, false),
                (Some("CSS"), true),
                (Some("Translated"), false),
            ] {
                let asset = source.map(|source| {
                    file.with_file(|file| file.bindable_artboard_named(source))
                        .unwrap()
                });
                property.with_downcast_mut::<ViewModelInstanceArtboard, _>(|property| {
                    property.set_asset(asset)
                });
                DataBind::update_data_bind_handle(&bind, false);
                let instance = host
                    .with(|host| {
                        host.as_nested_artboard()
                            .unwrap()
                            .artboard_instance_handle(0)
                    })
                    .flatten();
                if source.is_none() {
                    assert!(instance.is_none());
                    continue;
                }
                let instance = instance.unwrap();
                check_hosted_picture(&instance, css);
                let measured = instance.with_artboard_mut(|root| {
                    root.base.base.measure_layout(
                        f32::NAN,
                        LayoutMeasureMode::Undefined,
                        f32::NAN,
                        LayoutMeasureMode::Undefined,
                    )
                });
                near(measured.x, if css { 480.0 } else { 160.0 });
                near(measured.y, if css { 270.0 } else { 90.0 });
                // Copy while a CSS or translated replacement is mounted, so
                // clone_references_into rebuilds that child occurrence too.
                let copied = main.instance().unwrap();
                let copied_host =
                    copied.with_artboard(|root| root.nested_artboards())[index].clone();
                let copied_child = copied_host
                    .with(|host| {
                        host.as_nested_artboard()
                            .unwrap()
                            .artboard_instance_handle(0)
                    })
                    .flatten()
                    .unwrap();
                check_hosted_picture(&copied_child, css);
            }
        }
    }
}

#[test]
fn same_mode_artboard_attachment_preserves_picture_measurement() {
    let file = nested_sources_file();
    let root = file.with_file(|file| file.artboard_named("CSS")).unwrap();
    let mut context = root.with_artboard(|root| ObjectContext {
        arena: root.core_arena().clone(),
        objects: root.objects().to_vec(),
    });
    root.with_artboard_mut(|root| {
        assert_eq!(
            crate::source::core::CoreObject::on_added_dirty(&mut root.base, &mut context),
            StatusCode::Ok
        );
    });
    check_hosted_picture(&root, true);
}

#[test]
fn two_container_parent_cycle_completes_attachment() {
    let arena = CoreArena::default();
    let root = arena.insert(Artboard::default());
    let a = arena.insert(crate::source::node::Node::default());
    let b = arena.insert(crate::source::node::Node::default());
    let mut context = ObjectContext {
        arena,
        objects: vec![Some(root), Some(a.clone()), Some(b.clone())],
    };
    for (object, parent_id) in [(&a, 2), (&b, 1)] {
        object.with_mut(|object| {
            object
                .as_component_mut()
                .unwrap()
                .base
                .set_parent_id_value(parent_id);
            assert!(object.validate(&mut context));
            assert_eq!(object.on_added_dirty(&mut context), StatusCode::Ok);
        });
    }
}
