//! Literal scripting_text_runs.cpp producer; the embedded Luau mutates the list.
use anyhow::Context;
use nuxie_render_api::{PersistentFactory, SerializingFactory};
use nuxie_runtime::{
    RuntimeFactoryHandle,
    source::viewmodel::runtime::viewmodel_instance_runtime::ViewModelInstanceRuntime,
};
use std::path::Path;

pub(crate) fn run(case: &crate::Case, upstream: &Path) -> anyhow::Result<Vec<u8>> {
    anyhow::ensure!(
        case.id == "script_create_text_runs"
            && case.source == "script_create_text_runs.riv"
            && case.artboard == "main",
        "wrong text-runs producer"
    );
    let bytes =
        std::fs::read(upstream.join("tests/unit_tests/assets/script_create_text_runs.riv"))?;
    let mut factory = PersistentFactory::new(SerializingFactory::new());
    let file = crate::scripting::import_file(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).context("factory")?,
    )?;
    let artboard = file
        .with_file(|file| file.artboard_named("main"))
        .context("main artboard")?;
    let (w, h, id) = artboard.with_artboard(|a| (a.width(), a.height(), a.base.view_model_id()));
    factory.borrow_mut().frame_size(w as u32, h as u32);
    let machine = artboard.state_machine_at(0).context("machine 0")?;
    let vmi = file
        .with_file_mut(|file| {
            if id == u32::MAX {
                file.create_view_model_instance_for_artboard(artboard.core_handle())
            } else {
                file.create_view_model_instance_at(id as usize, 0)
            }
        })
        .context("authored instance 0")?;
    machine.with_instance_mut(|m| m.bind_view_model_instance(vmi.clone()));
    machine.advance_and_apply(0.1);
    let mut renderer = factory.borrow().make_renderer();
    artboard.draw(&mut renderer);

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
        let vm = ViewModelInstanceRuntime::new(vmi.clone());
        let child = vm
            .property_view_model(button)
            .context("button view-model property")?;
        let trigger = child
            .property_trigger("onClick")
            .context("onClick trigger")?;
        for _ in 0..count {
            trigger.trigger();
        }
        machine.advance_and_apply(0.1);
        artboard.draw(&mut renderer);
    }
    drop(renderer);
    let bytes = factory.borrow().bytes().to_vec();
    Ok(bytes)
}
