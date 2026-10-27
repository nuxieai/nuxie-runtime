//! Literal Luau artboard/path/layout producers from the pinned C++ tests.
use crate::{Case, scripting::import_file};
use anyhow::Context;
use nuxie_render_api::{PersistentFactory, SerializingFactory, SerializingRenderer};
use nuxie_runtime::source::{
    math::vec2d::Vec2D,
    pointer_button::PointerButton,
    viewmodel::runtime::viewmodel_instance_runtime::{
        RuntimeViewModelInstanceHandle, ViewModelInstanceRuntime,
    },
};
use nuxie_runtime::{
    RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
    RuntimeStateMachineInstanceHandle,
};
use std::path::Path;

pub(crate) fn supports(id: &str) -> bool {
    matches!(
        id,
        "scripted_artboard_render"
            | "script_artboards"
            | "script_artboards_origin"
            | "script_artboards_opacity"
            | "script_affects_has_changed"
            | "scripting_linear_animation"
            | "scripted_viewmodel_cache"
            | "script_paths"
            | "script_path_effects"
            | "script_path_opacity"
            | "scripted_as_path"
            | "reuse_path_in_effect"
            | "scripted_path_effect_clip"
            | "script_layout_grid"
    )
}

enum Binding {
    None,
    Fresh,
    Authored,
    Default,
}
struct Fixture {
    _file: RuntimeFileHandle,
    artboard: RuntimeArtboardInstanceHandle,
    machine: RuntimeStateMachineInstanceHandle,
    model: Option<RuntimeViewModelInstanceHandle>,
    factory: PersistentFactory<SerializingFactory>,
    renderer: Option<SerializingRenderer>,
}
impl Fixture {
    fn new(bytes: &[u8], board: Option<&str>, binding: Binding) -> anyhow::Result<Self> {
        let mut factory = PersistentFactory::new(SerializingFactory::new());
        let retained =
            RuntimeFactoryHandle::from_factory(&mut factory).context("retained factory")?;
        let file = import_file(bytes, retained)?;
        let artboard = file
            .with_file(|file| match board {
                Some(name) => file.artboard_named(name),
                None => file.artboard_default(),
            })
            .context("selected artboard")?;
        let (width, height) = artboard.with_artboard(|a| (a.width(), a.height()));
        factory.borrow_mut().frame_size(width as u32, height as u32);
        let machine = artboard.state_machine_at(0).context("state machine 0")?;
        let model = match binding {
            Binding::None => None,
            binding => Some(
                file.with_file(|file| match binding {
                    Binding::Fresh => {
                        file.create_view_model_instance_for_artboard(artboard.core_handle())
                    }
                    Binding::Default => {
                        file.create_default_view_model_instance_for_artboard(artboard.core_handle())
                    }
                    Binding::Authored => {
                        let id = artboard.with_artboard(|a| a.base.view_model_id());
                        if id == u32::MAX {
                            file.create_view_model_instance_for_artboard(artboard.core_handle())
                        } else {
                            file.create_view_model_instance_at(id as usize, 0)
                        }
                    }
                    Binding::None => unreachable!(),
                })
                .context("producer view-model instance")?,
            ),
        };
        Ok(Self {
            _file: file,
            artboard,
            machine,
            model: model
                .map(ViewModelInstanceRuntime::new)
                .map(ViewModelInstanceRuntime::into_handle),
            factory,
            renderer: None,
        })
    }
    fn model(&self) -> anyhow::Result<&RuntimeViewModelInstanceHandle> {
        self.model.as_ref().context("bound model")
    }
    fn advance(&self, seconds: f32) {
        self.machine.advance_and_apply(seconds);
    }
    fn bind(&self) {
        if let Some(model) = &self.model {
            self.machine
                .with_instance_mut(|machine| machine.bind_view_model_instance(model.instance()));
        }
    }
    fn make_renderer(&mut self) {
        assert!(self.renderer.is_none(), "producer constructs one renderer");
        self.renderer = Some(self.factory.borrow().make_renderer());
    }
    fn draw(&mut self) {
        self.artboard
            .draw(self.renderer.as_mut().expect("producer renderer"));
    }
    fn frame(&self) {
        self.factory.borrow_mut().add_frame();
    }
    fn frames(&mut self, count: usize, seconds: f32) {
        for _ in 0..count {
            self.frame();
            self.advance(seconds);
            self.draw();
        }
    }
    fn changed(&self, expected: bool) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.artboard.with_artboard(|a| a.did_change()) == expected,
            "artboard.didChange must be {expected}"
        );
        Ok(())
    }
    fn click(&self, y: f32) {
        self.machine
            .with_instance_mut(|m| m.pointer_down(Vec2D::new(450.0, y), 0, PointerButton::Primary));
        self.machine
            .with_instance_mut(|m| m.pointer_up(Vec2D::new(450.0, y), 0, PointerButton::Primary));
    }
}

pub(crate) fn run(case: &Case, runtime: &Path) -> anyhow::Result<Vec<u8>> {
    let (source, board, binding) = match case.id.as_str() {
        "scripted_artboard_render" => ("coin.riv", None, Binding::None),
        "script_artboards" => ("script_artboard_test.riv", Some("Artboard"), Binding::None),
        "script_artboards_origin" => (
            "script_artboard_origin_test.riv",
            Some("Artboard"),
            Binding::None,
        ),
        "script_artboards_opacity" => (
            "script_artboard_opacity_test.riv",
            Some("Artboard"),
            Binding::None,
        ),
        "script_paths" => ("script_paths_test.riv", Some("PathsScript"), Binding::None),
        "script_path_effects" => (
            "script_path_effects_test.riv",
            Some("PathEffects"),
            Binding::None,
        ),
        "script_path_opacity" => (
            "script_paths_opacity_test.riv",
            Some("Artboard"),
            Binding::None,
        ),
        "scripted_as_path" => ("scripted_as_path.riv", None, Binding::Default),
        "reuse_path_in_effect" => ("reuse_path_in_effect.riv", None, Binding::Default),
        "scripted_path_effect_clip" => ("scripted_path_effect_clip.riv", None, Binding::Default),
        "script_affects_has_changed" => (
            "script_affects_has_changed.riv",
            Some("Main"),
            Binding::Fresh,
        ),
        "scripting_linear_animation" => (
            "scripting_linear_animation.riv",
            Some("Main"),
            Binding::Authored,
        ),
        "scripted_viewmodel_cache" => ("scripted_viewmodel_cache.riv", None, Binding::Default),
        "script_layout_grid" => (
            "script_layout_test.riv",
            Some("LayoutScript"),
            Binding::Authored,
        ),
        _ => anyhow::bail!("unsupported scripted artboard producer {}", case.id),
    };
    anyhow::ensure!(
        case.source == source && case.artboard == board.unwrap_or("default"),
        "producer source/artboard mismatch"
    );
    let bytes = std::fs::read(runtime.join("tests/unit_tests/assets").join(source))?;
    if case.id == "scripted_artboard_render" {
        let bytecode = crate::scripted_source::compile_source(
            "function render(artboard, renderer)\n artboard:advance(0.1)\n artboard:draw(renderer)\n artboard.data.Vertical.value += 5\nend\n",
        )?;
        // SAFETY: bytecode was freshly compiled from the literal trusted source above.
        return unsafe {
            nuxie_scripting::vm::upstream_test_scripted_artboard_render(&bytes, &bytecode)
        }
        .map_err(|error| anyhow::anyhow!(error.to_string()));
    }
    let mut f = Fixture::new(&bytes, board, binding)?;
    if case.id != "scripted_viewmodel_cache" {
        f.bind();
    }
    match case.id.as_str() {
        "script_artboards"
        | "script_artboards_origin"
        | "script_artboards_opacity"
        | "script_paths"
        | "script_path_effects"
        | "script_path_opacity" => {
            f.advance(0.1);
            f.make_renderer();
            f.draw();
            f.frames(60, 0.016);
        }
        "scripted_as_path" | "reuse_path_in_effect" => {
            f.make_renderer();
            f.advance(0.016);
            f.draw();
            if case.id == "reuse_path_in_effect" {
                f.frame();
            }
        }
        "scripted_path_effect_clip" => {
            f.make_renderer();
            f.advance(0.0);
            f.draw();
            f.frames(60, 1.0f32 / 60.0f32);
        }
        "script_affects_has_changed" => {
            let property = f.model()?.property_boolean("toLeft").context("toLeft")?;
            f.advance(0.1);
            f.make_renderer();
            f.changed(true)?;
            f.draw();
            f.frame();
            f.advance(1.0);
            f.changed(false)?;
            f.draw();
            property.set_value(true);
            f.frame();
            f.advance(0.1);
            f.changed(true)?;
            f.draw();
            f.frame();
            f.advance(0.1);
            f.changed(false)?;
            f.draw();
        }
        "scripting_linear_animation" => {
            f.advance(0.1);
            let time = f.model()?.property_number("time").context("time")?;
            let mode = f.model()?.property_string("mode").context("mode")?;
            f.make_renderer();
            f.draw();
            f.frames(60, 0.064);
            for value in [0.55, -1.0, 3.8, 40.0] {
                time.set_value(value);
                f.advance(0.016);
                f.advance(0.016);
                f.frame();
                f.draw();
                for value in ["frames", "percentage"] {
                    mode.set_value(value);
                    f.advance(0.016);
                    f.advance(0.016);
                    f.frame();
                    f.draw();
                }
            }
        }
        "scripted_viewmodel_cache" => {
            let trigger = f
                .model()?
                .property_trigger("createInstance")
                .context("createInstance")?;
            f.make_renderer();
            f.bind();
            f.advance(0.016);
            f.draw();
            f.frame();
            f.click(50.0);
            f.advance(0.016);
            trigger.trigger();
            f.advance(0.016);
            f.draw();
            f.frame();
            f.click(150.0);
            f.advance(0.016);
            f.draw();
            f.frame();
            trigger.trigger();
            f.advance(0.016);
            f.draw();
        }
        "script_layout_grid" => {
            let rows = f.model()?.property_number("Rows").context("Rows")?;
            let cols = f.model()?.property_number("Columns").context("Columns")?;
            anyhow::ensure!(
                rows.value() == 5.0 && cols.value() == 5.0,
                "initial rows/columns must be5"
            );
            f.advance(0.1);
            f.make_renderer();
            f.draw();
            f.frames(20, 0.016);
            rows.set_value(8.0);
            anyhow::ensure!(rows.value() == 8.0, "updated rows must be8");
            cols.set_value(7.0);
            anyhow::ensure!(cols.value() == 7.0, "updated columns must be7");
            f.frames(20, 0.016);
        }
        _ => unreachable!(),
    }
    drop(f.renderer);
    let output = f.factory.borrow().bytes().to_vec();
    Ok(output)
}
