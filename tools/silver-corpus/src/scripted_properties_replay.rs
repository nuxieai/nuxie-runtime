//! Literal Silver producers from scripting_{properties,context,dependency}_test.cpp.
//! Programs are the original embedded Luau assets, imported by the File-owned VM.
use crate::{Case, scripting::import_file};
use anyhow::{Context, ensure};
use nuxie_render_api::{PersistentFactory, SerializingFactory};
use nuxie_runtime::{
    RuntimeFactoryHandle,
    source::{
        math::vec2d::Vec2D, pointer_button::PointerButton,
        viewmodel::runtime::viewmodel_instance_runtime::ViewModelInstanceRuntime,
    },
};
use std::path::Path;

fn selection(id: &str) -> Option<(&'static str, &'static str)> {
    Some(match id {
        "viewmodel_access" => ("viewmodel_access.riv", "default"),
        "viewmodel_from_instance" => ("viewmodel_from_instance.riv", "default"),
        "replace_view_model" => ("replace_view_model.riv", "default"),
        "remove_from_list" => ("remove_from_list.riv", "default"),
        "list_index_script_access" => ("list_index_script_access.riv", "default"),
        "scripted_property_image" => ("scripted_property_image.riv", "default"),
        "image_scripting_property_value" => ("image_scripting_property_value.riv", "default"),
        "reset_shared_viewmodel_instance_test" => {
            ("reset_shared_viewmodel_instance_test.riv", "default")
        }
        "script_create_viewmodel_instance" => ("script_create_viewmodel_instance.riv", "main"),
        "viewmodel_from_context" => ("viewmodel_from_context.riv", "main"),
        "scripting_root_viewmodel" => ("scripting_root_viewmodel.riv", "parent"),
        "scripted_data_context" => ("scripted_data_context.riv", "Main"),
        "viewmodel_instance_to_artboard" => ("viewmodel_instance_to_artboard.riv", "default"),
        "script_converter_with_dependency" => ("script_dependency_test.riv", "Artboard"),
        "script_converter_with_dependency_2" => ("script_dependency_test2.riv", "Artboard"),
        "script_converter_with_dependency_with_library" => {
            ("script_dependency_test_using_library.riv", "Artboard")
        }
        "script_converter_with_dependency_with_library_with_update" => {
            ("script_dependency_test_using_library_v2.riv", "Artboard")
        }
        "script_namespace_test" => ("script_namespace_test.riv", "Artboard"),
        _ => return None,
    })
}

pub(crate) fn supports(id: &str) -> bool {
    selection(id).is_some()
}

pub(crate) fn run(case: &Case, runtime: &Path) -> anyhow::Result<Vec<u8>> {
    let id = case.id.as_str();
    let (source, artboard_name) =
        selection(id).context("unknown scripted property/context/dependency producer")?;
    ensure!(
        case.source == source && case.artboard == artboard_name,
        "{id}: producer source/artboard differs from upstream"
    );
    let bytes = std::fs::read(runtime.join("tests/unit_tests/assets").join(source))?;
    let mut factory = PersistentFactory::new(SerializingFactory::new());
    let retained =
        RuntimeFactoryHandle::from_factory(&mut factory).context("retained serializing factory")?;
    let file = import_file(&bytes, retained)?;
    let artboard = file
        .with_file(|file| {
            if artboard_name == "default" {
                file.artboard_default()
            } else {
                file.artboard_named(artboard_name)
            }
        })
        .context("selected artboard")?;
    let (width, height) = artboard.with_artboard(|a| (a.width(), a.height()));
    factory.borrow_mut().frame_size(width as u32, height as u32);
    let machine = artboard.state_machine_at(0).context("state machine 0")?;
    let dependency = artboard_name == "Artboard";
    let authored = dependency
        || matches!(
            id,
            "viewmodel_access"
                | "remove_from_list"
                | "script_create_viewmodel_instance"
                | "viewmodel_from_context"
        );
    let owner = file
        .with_file_mut(|file| {
            if authored {
                let model_id = artboard.with_artboard(|a| a.view_model_id());
                if model_id == u32::MAX {
                    file.create_view_model_instance_for_artboard(artboard.core_handle())
                } else {
                    file.create_view_model_instance_at(model_id as usize, 0)
                }
            } else {
                file.create_default_view_model_instance_for_artboard(artboard.core_handle())
            }
        })
        .context("source-selected view model")?;
    let view_model = ViewModelInstanceRuntime::new(owner.clone());
    // C++ keeps this property from before binding through the complete reset sequence.
    let reset_trigger = if id == "reset_shared_viewmodel_instance_test" {
        Some(
            view_model
                .property_trigger("tri1")
                .context("tri1 trigger")?,
        )
    } else {
        None
    };
    let before_bind = matches!(
        id,
        "list_index_script_access" | "reset_shared_viewmodel_instance_test"
    );
    let before_advance = matches!(
        id,
        "viewmodel_access"
            | "viewmodel_from_instance"
            | "replace_view_model"
            | "scripted_data_context"
            | "viewmodel_instance_to_artboard"
    );
    let mut renderer = before_bind.then(|| factory.borrow().make_renderer());
    machine.with_instance_mut(|m| m.bind_view_model_instance(owner.clone()));
    if before_advance {
        renderer = Some(factory.borrow().make_renderer());
    }
    let initial = if dependency
        || matches!(
            id,
            "remove_from_list"
                | "list_index_script_access"
                | "script_create_viewmodel_instance"
                | "viewmodel_from_context"
                | "scripting_root_viewmodel"
        ) {
        0.1
    } else if matches!(
        id,
        "image_scripting_property_value" | "reset_shared_viewmodel_instance_test"
    ) {
        0.0
    } else {
        0.016
    };
    machine.advance_and_apply(initial);
    let mut renderer = renderer.unwrap_or_else(|| factory.borrow().make_renderer());
    artboard.draw(&mut renderer);
    let mut draw_after = |seconds| {
        machine.advance_and_apply(seconds);
        artboard.draw(&mut renderer);
    };
    let click = |x, y| {
        machine.with_instance_mut(|m| {
            m.pointer_down(Vec2D::new(x, y), 0, PointerButton::Primary);
            m.pointer_up(Vec2D::new(x, y), 0, PointerButton::Primary);
        })
    };
    match id {
        "replace_view_model" => {
            factory.borrow_mut().add_frame();
            let x = artboard.with_artboard(|a| a.width()) / 2.0;
            machine.with_instance_mut(|m| {
                m.pointer_down(Vec2D::new(x, 480.0), 0, PointerButton::Primary);
            });
            let x = artboard.with_artboard(|a| a.width()) / 2.0;
            machine.with_instance_mut(|m| {
                m.pointer_up(Vec2D::new(x, 480.0), 0, PointerButton::Primary);
            });
            draw_after(0.016);
        }
        "remove_from_list" | "viewmodel_instance_to_artboard" => {
            let count = if id == "remove_from_list" {
                10
            } else {
                (1.0_f32 / 0.016_f32) as usize
            };
            for _ in 0..count {
                factory.borrow_mut().add_frame();
                draw_after(0.016);
            }
        }
        "image_scripting_property_value" => {
            factory.borrow_mut().add_frame();
            draw_after(0.25);
        }
        "reset_shared_viewmodel_instance_test" => {
            let trigger = reset_trigger.as_ref().expect("retained reset trigger");
            factory.borrow_mut().add_frame();
            draw_after(0.016);
            factory.borrow_mut().add_frame();
            trigger.trigger();
            draw_after(0.016);
            factory.borrow_mut().add_frame();
            trigger.trigger();
            click(45.0, 165.0);
            draw_after(0.016);
            factory.borrow_mut().add_frame();
            draw_after(0.016);
            factory.borrow_mut().add_frame();
            click(45.0, 165.0);
            draw_after(0.016);
        }
        "viewmodel_from_context" | "scripting_root_viewmodel" => {
            factory.borrow_mut().add_frame();
            draw_after(0.1);
        }
        "script_create_viewmodel_instance" => {
            for (button, count) in [
                ("newButton", 1),
                ("newAtButton", 1),
                ("swapButton", 1),
                ("shiftButton", 1),
                ("popButton", 1),
                ("popButton", 4),
                ("newButton", 2),
            ] {
                factory.borrow_mut().add_frame();
                // Resolve each live nested instance anew. The C++ scoped rcp stays
                // alive through advance/draw, then drops before the next block.
                let root = ViewModelInstanceRuntime::new(owner.clone());
                let instance = root
                    .property_view_model(button)
                    .with_context(|| format!("view model {button}"))?;
                let trigger = instance
                    .property_trigger("onClick")
                    .context("onClick trigger")?;
                for _ in 0..count {
                    trigger.trigger();
                }
                draw_after(0.1);
            }
        }
        "script_converter_with_dependency" => {
            let number = view_model
                .property_number("InputValue1")
                .context("InputValue1 number")?;
            let mut counter = 0;
            for _ in 0..30 {
                number.set_value(counter as f32);
                factory.borrow_mut().add_frame();
                draw_after(0.016);
                counter += 5;
            }
        }
        _ if dependency => {
            let string = view_model
                .property_string("InputString")
                .context("InputString string")?;
            for value in [
                "Hello world!",
                "1,2,3",
                "rive scripting",
                "testing testing testing",
                "Script Data Converter",
            ] {
                string.set_value(value.to_owned());
                factory.borrow_mut().add_frame();
                draw_after(0.016);
            }
        }
        _ => {}
    }
    let bytes = factory.borrow().bytes().to_vec();
    Ok(bytes)
}
