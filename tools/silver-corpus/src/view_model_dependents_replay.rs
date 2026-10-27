//! Literal three-artboard producer from serialized_rendering_test.cpp.
//! Creation and binding order are observable in the shared Silver factory.
use crate::{Case, scripting::import_file};
use anyhow::Context;
use nuxie_render_api::{PersistentFactory, SerializingFactory};
use nuxie_runtime::source::viewmodel::runtime::viewmodel_instance_runtime::ViewModelInstanceRuntime;
use nuxie_runtime::{CoreHandle, RuntimeFactoryHandle};
use std::path::Path;

fn reference(owner: &CoreHandle, property: &str) -> anyhow::Result<CoreHandle> {
    ViewModelInstanceRuntime::new(owner.clone())
        .property_view_model(property)
        .map(|instance| instance.instance())
        .with_context(|| format!("missing reference {property}"))
}

fn counts(owner: &CoreHandle) -> anyhow::Result<(usize, usize)> {
    owner
        .with(|owner| {
            owner
                .as_view_model_instance()
                .map(|instance| (instance.parents().len(), instance.dependents().len()))
        })
        .flatten()
        .context("live view-model instance")
}

fn replace(owner: &CoreHandle, property: &str, replacement: &CoreHandle) -> anyhow::Result<()> {
    anyhow::ensure!(
        ViewModelInstanceRuntime::new(owner.clone()).replace_view_model(
            property,
            ViewModelInstanceRuntime::new(replacement.clone()).into_handle()
        ),
        "failed to replace {property}"
    );
    Ok(())
}

pub(crate) fn run(case: &Case, runtime_dir: &Path) -> anyhow::Result<Vec<u8>> {
    let bytes = std::fs::read(
        runtime_dir
            .join("tests/unit_tests/assets")
            .join(&case.source),
    )?;
    let mut factory = PersistentFactory::new(SerializingFactory::new());
    let retained =
        RuntimeFactoryHandle::from_factory(&mut factory).context("retained serializing factory")?;
    let file = import_file(&bytes, retained)?;
    // All artboards precede all machines, which precede all authored VMIs.
    let artboards = ["main-1", "main-2", "common"].map(|name| {
        file.with_file(|file| file.artboard_named(name))
            .with_context(|| format!("missing artboard {name}"))
    });
    let [main1, main2, common] = artboards;
    let artboards = [main1?, main2?, common?];
    let (width, height) = artboards[0].with_artboard(|a| (a.width(), a.height()));
    factory.borrow_mut().frame_size(width as u32, height as u32);
    let machines = artboards
        .each_ref()
        .map(|artboard| artboard.state_machine_at(0).context("state machine 0"));
    let [first, second, third] = machines;
    let machines = [first?, second?, third?];
    let instances = artboards.each_ref().map(|artboard| {
        let id = artboard.with_artboard(|a| a.base.view_model_id());
        file.with_file_mut(|file| file.create_view_model_instance_at(id as usize, 0))
            .context("authored view-model instance 0")
    });
    let [first, second, third] = instances;
    let instances = [first?, second?, third?];
    for (machine, instance) in machines.iter().zip(&instances) {
        machine.with_instance_mut(|machine| machine.bind_view_model_instance(instance.clone()));
        machine.advance_and_apply(0.1);
    }
    let mut renderer = factory.borrow().make_renderer();
    for (index, artboard) in artboards.iter().enumerate() {
        if index != 0 {
            factory.borrow_mut().add_frame();
        }
        artboard.draw(&mut renderer);
    }
    let mut round = || {
        for (machine, artboard) in machines.iter().zip(&artboards) {
            factory.borrow_mut().add_frame();
            machine.advance_and_apply(0.1);
            artboard.draw(&mut renderer);
        }
    };
    round();
    let common1 = reference(&instances[0], "common-1")?;
    let child = reference(&common1, "ch")?;
    // Retain the actual property across both replacements, as the C++ does.
    let string = ViewModelInstanceRuntime::new(child.clone())
        .property_string("str")
        .context("child string property")?;
    string.set_value("update-1".to_owned());
    round();
    let common2 = reference(&instances[1], "common-2")?;
    replace(&common2, "ch", &child)?;
    round();
    string.set_value("update-2".to_owned());
    round();
    let detached = reference(&instances[2], "ch")?;
    anyhow::ensure!(
        counts(&detached)?.0 == 1,
        "old common child must have one parent"
    );
    replace(&instances[2], "ch", &child)?;
    anyhow::ensure!(
        counts(&detached)?.0 == 0,
        "detached common child must have no parents"
    );
    anyhow::ensure!(
        counts(&instances[0])?.1 == 1,
        "main1 must have one dependent before rebinding"
    );
    anyhow::ensure!(
        counts(&instances[1])?.1 == 1,
        "main2 must have one dependent before rebinding"
    );
    machines[1].with_instance_mut(|machine| machine.bind_view_model_instance(instances[0].clone()));
    anyhow::ensure!(
        counts(&instances[0])?.1 == 2,
        "main1 must have two dependents after rebinding"
    );
    anyhow::ensure!(
        counts(&instances[1])?.1 == 0,
        "main2 must have no dependents after rebinding"
    );
    round();
    drop(round);
    drop(renderer);
    let bytes = factory.borrow().bytes().to_vec();
    Ok(bytes)
}
