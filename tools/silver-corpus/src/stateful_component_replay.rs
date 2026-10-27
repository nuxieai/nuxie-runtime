//! Literal component_test.cpp and data_binding_test.cpp Silver producers.
use crate::{Case, scripting::import_file};
use anyhow::Context;
use nuxie_render_api::{PersistentFactory, SerializingFactory, SerializingRenderer};
use nuxie_runtime::source::{
    bindable_artboard::RuntimeBindableArtboardHandle,
    math::vec2d::Vec2D,
    pointer_button::PointerButton,
    viewmodel::{
        runtime::viewmodel_instance_runtime::ViewModelInstanceRuntime,
        viewmodel_instance::ViewModelInstance,
        viewmodel_instance_artboard::ViewModelInstanceArtboard,
        viewmodel_instance_list::ViewModelInstanceList,
        viewmodel_instance_list_item::ViewModelInstanceListItem,
    },
};
use nuxie_runtime::{
    CoreHandle, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
    RuntimeStateMachineInstanceHandle,
};
use std::path::Path;

struct Fixture {
    file: RuntimeFileHandle,
    artboard: RuntimeArtboardInstanceHandle,
    machine: RuntimeStateMachineInstanceHandle,
    root: CoreHandle,
    factory: PersistentFactory<SerializingFactory>,
    renderer: SerializingRenderer,
}
impl Fixture {
    fn new(
        case: &Case,
        runtime: &Path,
        artboard_name: Option<&str>,
        initial: f32,
    ) -> anyhow::Result<Self> {
        let bytes = std::fs::read(runtime.join("tests/unit_tests/assets").join(&case.source))?;
        let mut factory = PersistentFactory::new(SerializingFactory::new());
        let retained =
            RuntimeFactoryHandle::from_factory(&mut factory).context("retained factory")?;
        let file = import_file(&bytes, retained)?;
        let artboard = file
            .with_file(|file| match artboard_name {
                Some(name) => file.artboard_named(name),
                None => file.artboard_default(),
            })
            .context("selected artboard")?;
        let (width, height) = artboard.with_artboard(|a| (a.width(), a.height()));
        factory.borrow_mut().frame_size(width as u32, height as u32);
        let machine = artboard.state_machine_at(0).context("state machine 0")?;
        let id = artboard.with_artboard(|a| a.base.view_model_id());
        let root = file
            .with_file_mut(|file| {
                if id == u32::MAX {
                    file.create_view_model_instance_for_artboard(artboard.core_handle())
                } else {
                    file.create_view_model_instance_at(id as usize, 0)
                }
            })
            .context("authored view model")?;
        machine.with_instance_mut(|machine| machine.bind_view_model_instance(root.clone()));
        machine.advance_and_apply(initial);
        let mut renderer = factory.borrow().make_renderer();
        artboard.draw(&mut renderer);
        Ok(Self {
            file,
            artboard,
            machine,
            root,
            factory,
            renderer,
        })
    }
    fn frames(&mut self, count: usize) {
        for _ in 0..count {
            self.factory.borrow_mut().add_frame();
            self.machine.advance_and_apply(0.016);
            self.artboard.draw(&mut self.renderer);
        }
    }
    fn source(&self, name: &str) -> anyhow::Result<RuntimeBindableArtboardHandle> {
        self.file
            .with_file(|file| file.bindable_artboard_named(name))
            .with_context(|| format!("bindable artboard {name}"))
    }
    fn swapped_nested(&self) -> Option<CoreHandle> {
        self.artboard
            .with_artboard(|a| a.nested_artboards().to_vec())
            .into_iter()
            .find(|nested| {
                source_name(nested).is_some_and(|name| name == "Button" || name == "StrokedButton")
            })
    }
    fn click(&self, y: f32) {
        self.machine.with_instance_mut(|machine| {
            machine.pointer_down(Vec2D::new(50.0, y), 0, PointerButton::Primary);
            machine.pointer_up(Vec2D::new(50.0, y), 0, PointerButton::Primary);
        });
        // The source advances but does not draw/addFrame for this click.
        self.machine.advance_and_apply(0.016);
    }
}
fn source_name(nested: &CoreHandle) -> Option<String> {
    nested
        .with(|n| n.as_nested_artboard()?.source_artboard())
        .flatten()?
        .with(|a| a.as_component().map(|a| a.name().to_owned()))
        .flatten()
}
fn bound_vmi(nested: &CoreHandle) -> anyhow::Result<CoreHandle> {
    nested
        .with(|n| n.as_nested_artboard()?.artboard_instance_default())
        .flatten()
        .context("nested artboard instance")?
        .data_context()
        .context("nested data context")?
        .with_context(|ctx| ctx.main_view_model_instance())
        .context("nested bound view model")
}
fn stateful_child(nested: &CoreHandle) -> Option<CoreHandle> {
    nested
        .with(|n| {
            n.as_container_component()?
                .children_typed::<ViewModelInstance>()
                .first()
        })
        .flatten()
}
fn model_id(vmi: &CoreHandle) -> anyhow::Result<u32> {
    vmi.with_downcast::<ViewModelInstance, _>(|vmi| vmi.base.view_model_id())
        .context("view model id")
}
fn asset_property(root: &CoreHandle, name: &str) -> anyhow::Result<CoreHandle> {
    ViewModelInstanceRuntime::new(root.clone())
        .property_artboard(name)
        .map(|property| property.value_runtime().handle())
        .with_context(|| format!("artboard property {name}"))
}
fn set_asset(
    property: &CoreHandle,
    source: Option<RuntimeBindableArtboardHandle>,
) -> anyhow::Result<()> {
    property
        .with_downcast_mut::<ViewModelInstanceArtboard, _>(|property| property.set_asset(source))
        .context("artboard property")?;
    Ok(())
}
fn button_shape(vmi: &CoreHandle) -> anyhow::Result<()> {
    let runtime = ViewModelInstanceRuntime::new(vmi.clone());
    anyhow::ensure!(
        runtime.property("count").is_some(),
        "ButtonVM count missing"
    );
    anyhow::ensure!(
        runtime.property("strokeWidth").is_none(),
        "ButtonVM retained strokeWidth"
    );
    Ok(())
}
fn require_variant(f: &Fixture, name: &str) -> anyhow::Result<CoreHandle> {
    let nested = f.swapped_nested().context("swap-target nested artboard")?;
    anyhow::ensure!(
        source_name(&nested).as_deref() == Some(name),
        "wrong source after swap to {name}"
    );
    Ok(nested)
}
fn artboard_swap(f: &mut Fixture) -> anyhow::Result<()> {
    let property = asset_property(&f.root, "buttonArtboard")?;
    let button = f.source("Button")?;
    let stroked = f.source("StrokedButton")?;
    f.frames(2);
    anyhow::ensure!(f.swapped_nested().is_none(), "swap target must start empty");
    set_asset(&property, Some(button.clone()))?;
    f.frames(5);
    let _after_button = require_variant(f, "Button")?;
    set_asset(&property, Some(stroked.clone()))?;
    f.frames(5);
    let after_stroked = require_variant(f, "StrokedButton")?;
    let stroked_vmi = bound_vmi(&after_stroked)?;
    let stroke_width = ViewModelInstanceRuntime::new(stroked_vmi.clone())
        .property_number("strokeWidth")
        .context("StrokedButtonVM strokeWidth")?;
    stroke_width.set_value(8.0);
    f.frames(5);
    set_asset(&property, Some(button.clone()))?;
    f.frames(5);
    let after_button2 = require_variant(f, "Button")?;
    let button_vmi = bound_vmi(&after_button2)?;
    button_shape(&button_vmi)?;
    set_asset(&property, None)?;
    f.frames(5);
    anyhow::ensure!(f.swapped_nested().is_none(), "swap target did not clear");
    set_asset(&property, Some(button.clone()))?;
    f.frames(5);
    let after_clear = require_variant(f, "Button")?;
    let button_vmi3 = bound_vmi(&after_clear)?;
    button_shape(&button_vmi3)
}
fn source_switch(f: &mut Fixture) -> anyhow::Result<()> {
    let property = asset_property(&f.root, "sourceArtboard")?;
    let label = ViewModelInstanceRuntime::new(f.root.clone())
        .property_string("labelInput")
        .context("labelInput string")?;
    let matching = f.source("MatchingArtboardA")?;
    let different = f.source("DifferentArtboardB")?;
    let nested = f
        .artboard
        .with_artboard(|a| a.nested_artboards().to_vec())
        .into_iter()
        .find(|n| {
            n.with(|n| n.as_nested_artboard().map(|n| n.base.is_stateful()))
                .flatten()
                .unwrap_or(false)
        })
        .context("stateful nested artboard")?;
    let child = stateful_child(&nested).context("stateful child")?;
    let initial_id = model_id(&child)?;
    f.frames(5);
    set_asset(&property, Some(matching.clone()))?;
    f.frames(5);
    anyhow::ensure!(
        bound_vmi(&nested)? == child,
        "matching source must borrow stateful child"
    );
    label.set_value("Matching A");
    f.frames(10);
    set_asset(&property, Some(different.clone()))?;
    f.frames(5);
    let bound_after_different = bound_vmi(&nested)?;
    anyhow::ensure!(
        bound_after_different != child,
        "different source reused stateful child"
    );
    anyhow::ensure!(
        model_id(&bound_after_different)? != model_id(&child)?,
        "different source has same VM schema"
    );
    anyhow::ensure!(
        stateful_child(&nested) == Some(child.clone()),
        "source switch removed stateful child"
    );
    anyhow::ensure!(
        model_id(&child)? == initial_id,
        "stateful child changed VM schema"
    );
    // C++ observes this VMI through a raw pointer; do not retain it across the swap back.
    drop(bound_after_different);
    label.set_value("Different B");
    f.frames(10);
    set_asset(&property, Some(matching.clone()))?;
    f.frames(5);
    anyhow::ensure!(
        bound_vmi(&nested)? == child,
        "switch back did not restore borrowed child"
    );
    label.set_value("Matching A Again");
    f.frames(10);
    Ok(())
}
fn list_lifecycle(f: &mut Fixture) -> anyhow::Result<()> {
    let buttons = ViewModelInstanceRuntime::new(f.root.clone())
        .property_list("buttons")
        .context("buttons list")?;
    let make_button = |label: &str, tint: u32| -> anyhow::Result<_> {
        let model = f
            .file
            .with_file(|file| file.view_model_named("ButtonVM"))
            .context("ButtonVM")?;
        let owner = f
            .file
            .with_file(|file| file.create_view_model_instance(model))
            .context("fresh ButtonVM")?;
        let runtime = ViewModelInstanceRuntime::new(owner).into_handle();
        runtime
            .property_string("label")
            .context("ButtonVM.label")?
            .set_value(label);
        runtime
            .property_color("tint")
            .context("ButtonVM.tint")?
            .set_value(tint as i32);
        buttons.add_instance(runtime.clone());
        Ok(runtime)
    };
    let _button_a = make_button("Alpha", 0xffff3344)?;
    let button_b = make_button("Beta", 0xff33aaff)?;
    let button_c = make_button("Gamma", 0xff44cc55)?;
    f.frames(3);
    anyhow::ensure!(buttons.size() == 3, "expected three buttons");
    buttons.remove_instance_at(1);
    f.frames(5);
    anyhow::ensure!(buttons.size() == 2, "expected two surviving buttons");
    let clicked_c = button_c
        .property_boolean("clicked")
        .context("Gamma.clicked")?;
    let clicked_b = button_b
        .property_boolean("clicked")
        .context("Beta.clicked")?;
    anyhow::ensure!(!clicked_c.value(), "Gamma starts unclicked");
    f.click(73.0);
    anyhow::ensure!(clicked_c.value(), "surviving Gamma must receive click");
    anyhow::ensure!(!clicked_b.value(), "removed Beta received click");
    f.frames(3);
    buttons.add_instance(button_b.clone());
    f.frames(5);
    anyhow::ensure!(buttons.size() == 3, "readded Beta missing");
    f.click(118.0);
    anyhow::ensure!(clicked_b.value(), "readded Beta must receive click");
    f.frames(3);
    while buttons.size() > 0 {
        buttons.remove_instance_at(0);
    }
    f.frames(5);
    anyhow::ensure!(buttons.size() == 0, "buttons list did not clear");
    Ok(())
}
fn empty_list_item(list: &CoreHandle) -> anyhow::Result<CoreHandle> {
    list.insert_sibling(ViewModelInstanceListItem::default())
        .context("empty list item")
}
fn self_comparator(f: &mut Fixture) -> anyhow::Result<()> {
    let vm = ViewModelInstanceRuntime::new(f.root.clone());
    let number = vm.property_number("num").context("num")?;
    let trigger = vm.property_trigger("tri").context("tri")?;
    let color = vm.property_color("col").context("col")?;
    let boolean = vm.property_boolean("bol").context("bol")?;
    let string = vm.property_string("str").context("str")?;
    let list = vm
        .property_list("lis")
        .context("lis")?
        .value_runtime()
        .handle();
    let finish = |f: &mut Fixture| {
        f.machine.advance_and_apply(0.0);
        f.artboard.draw(&mut f.renderer);
    };
    f.factory.borrow_mut().add_frame();
    number.set_value(20.0);
    finish(f);
    f.factory.borrow_mut().add_frame();
    number.set_value(20.0);
    finish(f);
    f.factory.borrow_mut().add_frame();
    number.set_value(10.0);
    number.set_value(20.0);
    finish(f);
    f.factory.borrow_mut().add_frame();
    number.set_value(10.0);
    trigger.trigger();
    finish(f);
    f.factory.borrow_mut().add_frame();
    color.set_value(0x64000a0f);
    color.set_value(0x65000a0f);
    finish(f);
    f.factory.borrow_mut().add_frame();
    color.set_value(0x66000a0f);
    finish(f);
    f.factory.borrow_mut().add_frame();
    boolean.set_value(true);
    boolean.set_value(false);
    finish(f);
    f.factory.borrow_mut().add_frame();
    boolean.set_value(true);
    finish(f);
    f.factory.borrow_mut().add_frame();
    string.set_value("a");
    string.set_value("b");
    finish(f);
    f.factory.borrow_mut().add_frame();
    string.set_value("c");
    finish(f);
    for count in [2, 1] {
        f.factory.borrow_mut().add_frame();
        for _ in 0..count {
            let item = empty_list_item(&list)?;
            list.with_downcast_mut::<ViewModelInstanceList, _>(|list| list.add_item(item))
                .context("list")?;
        }
        finish(f);
    }
    for index in [0, 10] {
        f.factory.borrow_mut().add_frame();
        let item = empty_list_item(&list)?;
        let inserted = list
            .with_downcast_mut::<ViewModelInstanceList, _>(|list| {
                list.add_item_at(item.clone(), index)
            })
            .context("list")?;
        finish(f);
        // Invalid insert never acquired ownership in C++; release its arena occurrence.
        if !inserted {
            item.remove_occurrence();
        }
    }
    f.factory.borrow_mut().add_frame();
    list.with_downcast_mut::<ViewModelInstanceList, _>(|list| list.swap(0, 1))
        .context("list")?;
    finish(f);
    for index in [0, 10] {
        f.factory.borrow_mut().add_frame();
        list.with_downcast_mut::<ViewModelInstanceList, _>(|list| list.remove_item_at(index))
            .context("list")?;
        finish(f);
    }
    Ok(())
}

pub(crate) fn run(case: &Case, runtime: &Path, scenario: &str) -> anyhow::Result<Vec<u8>> {
    let (id, source, artboard, initial) = match scenario {
        "artboard-swap" => (
            "stateful_artboard_swap",
            "stateful_artboard_swap.riv",
            Some("Main"),
            0.0,
        ),
        "source-switch" => (
            "stateful_source_switch",
            "stateful_source_switch.riv",
            Some("ParentArtboard"),
            0.0,
        ),
        "list-lifecycle" => (
            "stateful_list_props_lifecycle",
            "stateful_list_props.riv",
            Some("Main"),
            0.0,
        ),
        "self-comparator" => (
            "transition_self_comparator_test",
            "transition_self_comparator_test.riv",
            None,
            0.1,
        ),
        _ => anyhow::bail!("unknown literal stateful component scenario {scenario}"),
    };
    anyhow::ensure!(
        case.id == id && case.source == source && case.artboard == artboard.unwrap_or("default"),
        "stateful replay metadata does not match pinned producer {scenario}"
    );
    let mut f = Fixture::new(case, runtime, artboard, initial)?;
    match scenario {
        "artboard-swap" => artboard_swap(&mut f)?,
        "source-switch" => source_switch(&mut f)?,
        "list-lifecycle" => list_lifecycle(&mut f)?,
        "self-comparator" => self_comparator(&mut f)?,
        _ => unreachable!(),
    }
    drop(f.renderer);
    let bytes = f.factory.borrow().bytes().to_vec();
    Ok(bytes)
}
