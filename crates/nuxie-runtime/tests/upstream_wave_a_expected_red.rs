//! Executable native translations of the formerly retained Wave A component-list cases.
use nuxie_render_api as render;
use nuxie_runtime::source::{
    animation::state_machine_instance::{RuntimeStateMachineInstanceHandle, StateMachineInstance},
    artboard::{Artboard, RuntimeArtboardInstanceHandle},
    artboard_component_list::ArtboardComponentList,
    constraints::scrolling::scroll_constraint::ScrollConstraint,
    core::{CoreHandle, CoreType},
    factory::RuntimeFactoryHandle,
    file::{File, RuntimeFileHandle},
    math::{aabb::Aabb, mat2d::Mat2D, vec2d::Vec2D},
    text::{
        text::{Text, TextValueRunHandle},
        text_value_run::TextValueRun,
    },
    viewmodel::{
        symbol_type::SymbolType, viewmodel_instance::ViewModelInstance,
        viewmodel_instance_number::ViewModelInstanceNumber,
        viewmodel_instance_string::ViewModelInstanceString,
        viewmodel_instance_symbol_list_index::ViewModelInstanceSymbolListIndex,
    },
};
use std::{any::Any, path::PathBuf};

fn factory_handle(factory: impl render::Factory + 'static) -> RuntimeFactoryHandle {
    let mut factory = render::PersistentFactory::new(factory);
    RuntimeFactoryHandle::from_factory(&mut factory).expect("retained native factory")
}
fn import(name: &str, factory: RuntimeFactoryHandle) -> RuntimeFileHandle {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
    let bytes =
        std::fs::read(root.join("tests/unit_tests/assets").join(name)).expect("pinned fixture");
    File::import(&bytes, factory, None, None, None).expect("native File import")
}
fn find<T: CoreType>(artboard: &RuntimeArtboardInstanceHandle, name: &str) -> CoreHandle {
    artboard
        .with_artboard(|artboard| artboard.find_handle::<T>(name))
        .expect(name)
}
fn read<T: std::any::Any, R>(handle: &CoreHandle, action: impl FnOnce(&T) -> R) -> R {
    handle.with_downcast(action).expect("live native owner")
}
fn write<T: std::any::Any, R>(handle: &CoreHandle, action: impl FnOnce(&mut T) -> R) -> R {
    handle.with_downcast_mut(action).expect("live native owner")
}
fn property(instance: &CoreHandle, name: &str) -> CoreHandle {
    read::<ViewModelInstance, _>(instance, |instance| instance.property_value_named(name))
        .expect(name)
}
struct Fixture {
    _file: RuntimeFileHandle,
    artboard: RuntimeArtboardInstanceHandle,
    instance: CoreHandle,
    list: CoreHandle,
}
impl Fixture {
    fn new(name: &str) -> Self {
        Self::with_factory(name, factory_handle(render::RecordingFactory::default()))
    }
    fn with_factory(name: &str, factory: RuntimeFactoryHandle) -> Self {
        let file = import(name, factory);
        let artboard = file
            .with_file(|file| file.artboard_named("Main"))
            .expect("Main");
        let instance = file
            .with_file_mut(|file| {
                file.create_default_view_model_instance_for_artboard(artboard.core_handle())
            })
            .expect("default VMI");
        artboard.bind_view_model_instance(Some(instance.clone()));
        let list = find::<ArtboardComponentList>(&artboard, "List");
        Self {
            _file: file,
            artboard,
            instance,
            list,
        }
    }
    fn advance(&self) {
        self.artboard.advance_default(0.0);
    }
    fn count(&self) -> usize {
        read::<ArtboardComponentList, _>(&self.list, ArtboardComponentList::artboard_count)
    }
    fn item_artboard(&self, i: usize) -> Option<RuntimeArtboardInstanceHandle> {
        read::<ArtboardComponentList, _>(&self.list, |list| list.artboard_instance(i as i32))
    }
    fn machine(&self, i: usize) -> Option<RuntimeStateMachineInstanceHandle> {
        read::<ArtboardComponentList, _>(&self.list, |list| list.state_machine_instance(i as i32))
    }
}

fn assert_items(f: &Fixture) {
    for i in 0..f.count() {
        let artboard = f.item_artboard(i).expect("item artboard");
        assert_eq!(
            artboard.with_artboard(|artboard| artboard.name().to_owned()),
            "Item"
        );
        let machine = f.machine(i).expect("item state machine");
        assert!(machine.with_instance(|machine| machine.artboard().ptr_eq(&artboard.downgrade())));
    }
}
fn assert_layout_nodes(f: &Fixture) {
    for i in 0..f.count() {
        assert!(
            read::<ArtboardComponentList, _>(&f.list, |list| list.layout_node(i as i32)).is_some()
        );
    }
}
fn text_of_first_run(artboard: &RuntimeArtboardInstanceHandle, name: &str) -> String {
    let text = find::<Text>(artboard, name);
    let run = read::<Text, _>(&text, |text| text.runs()[0].clone());
    match run {
        TextValueRunHandle::Core(run) => {
            read::<TextValueRun, _>(&run, |run| run.base.text().to_owned())
        }
        TextValueRunHandle::Runtime(run) => run.borrow().base.text().to_owned(),
    }
}
#[test]
fn component_list_case_01_direct_port_expected_red() {
    let f = Fixture::new("component_list_1.riv");
    f.advance();
    assert!(write::<ArtboardComponentList, _>(
        &f.list,
        ArtboardComponentList::sync_style_changes
    ));
    assert_eq!(f.count(), 8);
    assert!(read::<ArtboardComponentList, _>(&f.list, |list| list.layout_node(9)).is_none());
    assert!(f.item_artboard(9).is_none());
    assert!(f.machine(9).is_none());
}
#[test]
fn component_list_case_02_direct_port_expected_red() {
    let f = Fixture::new("component_list_1.riv");
    f.advance();
    assert_items(&f);
}
#[test]
fn component_list_case_03_direct_port_expected_red() {
    let f = Fixture::new("component_list_1.riv");
    f.advance();
    assert_layout_nodes(&f);
}
#[test]
fn component_list_case_04_direct_port_expected_red() {
    let f = Fixture::new("component_list_1.riv");
    f.advance();
    for i in 0..f.count() {
        let artboard = f.item_artboard(i).expect("item artboard");
        assert_eq!(
            artboard.with_artboard(|artboard| artboard.layout_bounds().top()),
            (i * 60) as f32
        );
    }
}
#[test]
fn component_list_case_05_direct_port_expected_red() {
    let f = Fixture::new("component_list_1.riv");
    f.advance();
    let labels = [
        "ONE", "TWO", "THREE", "THREE", "THREE", "THREE", "TWO", "ONE",
    ];
    for i in 0..f.count() {
        let artboard = f.item_artboard(i).expect("item artboard");
        let context = artboard.data_context().expect("data context");
        let instance = context
            .with_context(|context| context.main_view_model_instance())
            .expect("main VMI");
        assert_eq!(
            read::<ViewModelInstanceString, _>(
                &property(&instance, "Label"),
                ViewModelInstanceString::value
            ),
            labels[i]
        );
    }
}
#[test]
fn component_list_case_06_direct_port_expected_red() {
    let f = Fixture::new("component_list_1.riv");
    f.advance();
    let labels = [
        "ONE", "TWO", "THREE", "THREE", "THREE", "THREE", "TWO", "ONE",
    ];
    for i in 0..f.count() {
        assert_eq!(
            text_of_first_run(&f.item_artboard(i).expect("item artboard"), "TextLabel"),
            labels[i]
        );
    }
}
#[test]
fn component_list_case_07_direct_port_expected_red() {
    let file = import(
        "component_list_1.riv",
        factory_handle(render::RecordingFactory::default()),
    );
    let artboard = file
        .with_file(|file| file.artboard_named("Main"))
        .expect("Main");
    let definition = artboard
        .with_artboard(|artboard| artboard.base.state_machine_named("State Machine 1"))
        .expect("machine definition");
    let machine = StateMachineInstance::new(definition, artboard.downgrade());
    let instance = file
        .with_file_mut(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
        })
        .expect("default VMI");
    artboard.bind_view_model_instance(Some(instance.clone()));
    let list = find::<ArtboardComponentList>(&artboard, "List");
    let f = Fixture {
        _file: file,
        artboard,
        instance,
        list,
    };
    f.advance();
    for (index, y) in [(0, 30.0), (2, 150.0)] {
        let child = f.machine(index).expect("item machine");
        assert!(!child.with_instance(|machine| machine.get_bool("Hover").expect("Hover").value()));
        assert_eq!(
            child.with_instance(|machine| machine.hit_components_count()),
            1
        );
        machine.with_instance_mut(|machine| {
            machine.pointer_move(Vec2D::new(100.0, y), 0.0, 0);
        });
        f.advance();
        machine.advance_and_apply(0.0);
        assert!(child.with_instance(|machine| machine.get_bool("Hover").expect("Hover").value()));
    }
}
#[test]
fn component_list_case_08_direct_port_expected_red() {
    let f = Fixture::new("component_list_2.riv");
    f.advance();
    assert!(write::<ArtboardComponentList, _>(
        &f.list,
        ArtboardComponentList::sync_style_changes
    ));
    assert_eq!(f.count(), 12);
    assert!(read::<ArtboardComponentList, _>(&f.list, |list| list.layout_node(13)).is_none());
    assert!(f.item_artboard(13).is_none());
    assert!(f.machine(13).is_none());
    for i in 0..f.count() {
        assert!(
            read::<ArtboardComponentList, _>(&f.list, |list| list.list_item(i as i32)).is_some()
        );
    }
    write::<ViewModelInstanceNumber, _>(&property(&f.instance, "ItemCount"), |value| {
        value.set_value(6.0)
    });
    f.advance();
    assert_eq!(f.count(), 6);
}
#[test]
fn component_list_case_09_direct_port_expected_red() {
    let f = Fixture::new("component_list_2.riv");
    f.advance();
    assert_items(&f);
}
#[test]
fn component_list_case_10_direct_port_expected_red() {
    let f = Fixture::new("component_list_2.riv");
    f.advance();
    assert_layout_nodes(&f);
}
#[test]
fn component_list_case_11_direct_port_expected_red() {
    let f = Fixture::new("component_list_2.riv");
    f.advance();
    for i in 0..f.count() {
        f.item_artboard(i).expect("item artboard");
        let item = read::<ArtboardComponentList, _>(&f.list, |list| list.list_item(i as i32))
            .expect("list item");
        let instance = item
            .with(|item| {
                item.as_view_model_instance_list_item()
                    .unwrap()
                    .view_model_instance()
            })
            .flatten()
            .expect("item VMI");
        let symbol = read::<ViewModelInstance, _>(&instance, |instance| {
            instance.property_value_for_symbol(SymbolType::ItemIndex)
        })
        .expect("itemIndex");
        assert_eq!(
            read::<ViewModelInstanceSymbolListIndex, _>(&symbol, |symbol| symbol
                .base
                .property_value()),
            i as u32
        );
    }
}
#[test]
fn component_list_case_12_direct_port_expected_red() {
    let f = Fixture::new("component_list_2.riv");
    f.advance();
    let labels = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11"];
    for i in 0..f.count() {
        assert_eq!(
            text_of_first_run(&f.item_artboard(i).expect("item artboard"), "ItemLabel"),
            labels[i]
        );
    }
}
#[test]
fn component_list_case_13_direct_port_expected_red() {
    let f = Fixture::new("component_list_virtualized.riv");
    f.advance();
    assert_eq!(f.count(), 20);
    for i in 0..f.count() {
        if i < 5 {
            let artboard = f.item_artboard(i).expect("mounted artboard");
            assert_eq!(
                artboard.with_artboard(|artboard| artboard.name().to_owned()),
                "ItemArtboard"
            );
            assert!(
                f.machine(i)
                    .expect("mounted machine")
                    .with_instance(|machine| machine.artboard().ptr_eq(&artboard.downgrade()))
            );
        } else {
            assert!(f.item_artboard(i).is_none());
            assert!(f.machine(i).is_none());
        }
    }
}
#[test]
fn component_list_case_14_direct_port_expected_red() {
    let f = Fixture::new("component_list_virtualized.riv");
    assert!(read::<ArtboardComponentList, _>(
        &f.list,
        ArtboardComponentList::virtualization_enabled
    ));
    f.advance();
    for i in 0..f.count() {
        if i < 5 {
            f.item_artboard(i).expect("mounted artboard");
            assert_eq!(
                read::<ArtboardComponentList, _>(&f.list, |list| list
                    .layout_bounds_for_node(i)
                    .left()),
                i as f32 * 110.0
            );
        } else {
            assert!(f.item_artboard(i).is_none());
        }
    }
}
#[test]
fn component_list_case_15_direct_port_expected_red() {
    let f = Fixture::new("component_list_virtualized.riv");
    let scrolls = f
        .artboard
        .with_artboard(|artboard| artboard.find_all_handles::<ScrollConstraint>());
    assert_eq!(scrolls.len(), 1);
    let scroll = &scrolls[0];
    assert_eq!(
        read::<ScrollConstraint, _>(scroll, ScrollConstraint::offset_x),
        0.0
    );
    f.advance();
    write::<ScrollConstraint, _>(scroll, |scroll| scroll.set_scroll_index(2.0));
    read::<ScrollConstraint, _>(scroll, |scroll| {
        assert!(scroll.infinite());
        assert_eq!(scroll.scroll_item_count(), 20);
        assert_eq!(scroll.offset_x(), -220.0);
        assert_eq!(scroll.clamped_offset_x(), -220.0);
        assert_eq!(scroll.min_offset_x(), f32::INFINITY);
        assert_eq!(scroll.max_offset_x(), f32::NEG_INFINITY);
        assert_eq!(scroll.offset_y(), 0.0);
        assert_eq!(scroll.min_offset_y(), 0.0);
        assert_eq!(scroll.max_offset_y(), 0.0);
        assert_eq!(scroll.clamped_offset_y(), 0.0);
        assert_eq!(scroll.scroll_index(), 2.0);
        assert_eq!(scroll.content_width(), 2200.0);
        assert_eq!(scroll.viewport_width(), 500.0);
    });
}
#[test]
fn component_list_case_28_direct_port_expected_red() {
    let f = Fixture::new("component_list_1.riv");
    f.advance();
    let n = f.count();
    assert!(n > 0);
    let order =
        write::<ArtboardComponentList, _>(&f.list, |list| list.ordered_list_indices().to_vec());
    assert_eq!(order.len(), n);
    for i in 0..n {
        assert_eq!(order[i], i as i32);
    }
    for (hit_index, index) in order.iter().rev().enumerate() {
        assert_eq!(*index, (n - 1 - hit_index) as i32);
    }
}

// Renderer/factory observations are the pinned ClipProbe boundary, not a list/layout substitute.
#[derive(Default)]
struct ClipProbePath {
    raw: render::RawPath,
}
impl render::RenderPath for ClipProbePath {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn rewind(&mut self) {
        self.raw = render::RawPath::default();
    }
    fn fill_rule(&mut self, _: render::FillRule) {}
    fn add_render_path(&mut self, _: &dyn render::RenderPath, _: render::Mat2D) {}
    fn add_render_path_self(&mut self, _: render::Mat2D) {}
    fn add_render_path_backwards(&mut self, _: &dyn render::RenderPath, _: render::Mat2D) {}
    fn add_raw_path(&mut self, path: &render::RawPath) {
        self.raw.add_path(path, render::Mat2D::IDENTITY);
    }
    fn move_to(&mut self, _: f32, _: f32) {}
    fn line_to(&mut self, _: f32, _: f32) {}
    fn cubic_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {}
    fn close(&mut self) {}
}
#[derive(Default)]
struct ClipProbeFactory(render::RecordingFactory);
impl render::Factory for ClipProbeFactory {
    fn make_render_buffer(
        &mut self,
        t: render::RenderBufferType,
        f: render::RenderBufferFlags,
        n: usize,
    ) -> Box<dyn render::RenderBuffer> {
        render::Factory::make_render_buffer(&mut self.0, t, f, n)
    }
    fn make_linear_gradient(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        c: &[u32],
        s: &[f32],
    ) -> Box<dyn render::RenderShader> {
        render::Factory::make_linear_gradient(&mut self.0, sx, sy, ex, ey, c, s)
    }
    fn make_radial_gradient(
        &mut self,
        x: f32,
        y: f32,
        r: f32,
        c: &[u32],
        s: &[f32],
    ) -> Box<dyn render::RenderShader> {
        render::Factory::make_radial_gradient(&mut self.0, x, y, r, c, s)
    }
    fn make_render_path(
        &mut self,
        raw: render::RawPath,
        _: render::FillRule,
    ) -> Box<dyn render::RenderPath> {
        Box::new(ClipProbePath { raw })
    }
    fn make_empty_render_path(&mut self) -> Box<dyn render::RenderPath> {
        Box::new(ClipProbePath::default())
    }
    fn make_render_paint(&mut self) -> Box<dyn render::RenderPaint> {
        render::Factory::make_render_paint(&mut self.0)
    }
    fn decode_image(
        &mut self,
        data: &[u8],
    ) -> Result<Box<dyn render::RenderImage>, render::ImageDecodeError> {
        render::Factory::decode_image(&mut self.0, data)
    }
}
#[derive(Clone, Copy, Default)]
struct ProbeState {
    transform: Mat2D,
    clip: Option<Aabb>,
}
struct DrawEvent {
    bounds: Aabb,
    clip: Option<Aabb>,
}
struct ClipProbeRenderer {
    stack: Vec<ProbeState>,
    draws: Vec<DrawEvent>,
}
impl Default for ClipProbeRenderer {
    fn default() -> Self {
        Self {
            stack: vec![ProbeState::default()],
            draws: Vec::new(),
        }
    }
}
impl ClipProbeRenderer {
    fn world_bounds(&self, path: &dyn render::RenderPath) -> Aabb {
        let path = path
            .as_any()
            .downcast_ref::<ClipProbePath>()
            .expect("ClipProbePath");
        let points: Vec<_> = path
            .raw
            .points()
            .iter()
            .map(|point| Vec2D::new(point.x, point.y))
            .collect();
        self.stack
            .last()
            .unwrap()
            .transform
            .map_bounding_box_points(&points)
    }
}
impl render::Renderer for ClipProbeRenderer {
    fn save(&mut self) {
        self.stack.push(*self.stack.last().unwrap());
    }
    fn restore(&mut self) {
        self.stack.pop().unwrap();
    }
    fn transform(&mut self, transform: render::Mat2D) {
        let [a, b, c, d, e, f] = transform.0;
        self.stack.last_mut().unwrap().transform *= Mat2D::new(a, b, c, d, e, f);
    }
    fn clip_path(&mut self, path: &dyn render::RenderPath) {
        let world = self.world_bounds(path);
        let state = self.stack.last_mut().unwrap();
        state.clip = Some(match state.clip {
            None => world,
            Some(clip) => Aabb::new(
                clip.min_x.max(world.min_x),
                clip.min_y.max(world.min_y),
                clip.max_x.min(world.max_x),
                clip.max_y.min(world.max_y),
            ),
        });
    }
    fn draw_path(&mut self, path: &dyn render::RenderPath, _: &dyn render::RenderPaint) {
        self.draws.push(DrawEvent {
            bounds: self.world_bounds(path),
            clip: self.stack.last().unwrap().clip,
        });
    }
    fn modulate_opacity(&mut self, _: f32) {}
    fn draw_image(
        &mut self,
        _: Option<&dyn render::RenderImage>,
        _: render::ImageSampler,
        _: render::BlendMode,
        _: f32,
    ) {
    }
    fn draw_image_mesh(
        &mut self,
        _: Option<&dyn render::RenderImage>,
        _: render::ImageSampler,
        _: Option<&dyn render::RenderBuffer>,
        _: Option<&dyn render::RenderBuffer>,
        _: Option<&dyn render::RenderBuffer>,
        _: u32,
        _: u32,
        _: render::BlendMode,
        _: f32,
    ) {
    }
}
#[test]
fn component_list_case_30_direct_port_expected_red() {
    let viewport = Aabb::new(100.0, 100.0, 300.0, 300.0);
    let f = Fixture::with_factory(
        "component_list_clipped_viewport.riv",
        factory_handle(ClipProbeFactory::default()),
    );
    f.advance();
    assert_eq!(f.count(), 6);
    assert!(read::<ArtboardComponentList, _>(
        &f.list,
        ArtboardComponentList::virtualization_enabled
    ));
    assert_eq!(
        (0..f.count())
            .filter(|i| f.item_artboard(*i).is_some())
            .count(),
        4
    );
    let mut renderer = ClipProbeRenderer::default();
    f.artboard.draw(&mut renderer);
    assert_eq!(renderer.draws.len(), 6);
    for i in 1..=4 {
        let clip = renderer.draws[i].clip.expect("item clip");
        assert_eq!(clip.min_x, viewport.min_x);
        assert_eq!(clip.min_y, viewport.min_y);
        assert_eq!(clip.max_x, viewport.max_x);
        assert_eq!(clip.max_y, viewport.max_y);
    }
    assert!(renderer.draws[4].bounds.max_y > viewport.max_y);
    let overlay = &renderer.draws[5];
    assert_eq!(overlay.bounds.min_y, 360.0);
    if let Some(clip) = overlay.clip {
        assert!(clip.max_y > viewport.max_y);
    }
    let scrolls = f
        .artboard
        .with_artboard(|artboard| artboard.find_all_handles::<ScrollConstraint>());
    assert_eq!(scrolls.len(), 1);
    write::<ScrollConstraint, _>(&scrolls[0], |scroll| scroll.set_scroll_index(2.0));
    f.advance();
    assert!(f.item_artboard(0).is_none());
    assert!(f.item_artboard(5).is_some());
    let mut scrolled = ClipProbeRenderer::default();
    f.artboard.draw(&mut scrolled);
    assert_eq!(scrolled.draws.len(), 6);
    for i in 1..=4 {
        let clip = scrolled.draws[i].clip.expect("scrolled item clip");
        assert_eq!(clip.min_y, viewport.min_y);
        assert_eq!(clip.max_y, viewport.max_y);
    }
    assert_eq!(scrolled.draws[1].bounds.min_y, viewport.min_y);
}
