#![allow(dead_code)]
use nuxie_render_api::{NullFactory, PersistentFactory};
pub use nuxie_runtime::source::{
    generated::{
        core_registry::CoreRegistry,
        layout::layout_component_style_base::LayoutComponentStyleBase as Style,
        layout_component_base::LayoutComponentBase,
    },
    layout::layout_enums::{LayoutAnimationStyle, LayoutDirection, LayoutStyleInterpolation},
    layout_component::LayoutComponent,
    math::{aabb::Aabb, raw_path::RawPath},
    shapes::path::Path,
};
pub use nuxie_runtime::{
    Artboard, CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
    RuntimeFileHandle,
};
pub struct Fixture {
    pub artboard: RuntimeArtboardInstanceHandle,
    _file: RuntimeFileHandle,
}
impl Fixture {
    pub fn new(asset: &str) -> Self {
        let file = read_file(asset);
        let source = file.with_file(File::artboard).expect("default artboard");
        let artboard = Artboard::instance_from_handle(&source).expect("artboard instance");
        Self {
            artboard,
            _file: file,
        }
    }
    pub fn layout(&self, style: LayoutAnimationStyle) -> CoreHandle {
        self.artboard
            .with_artboard(|a| a.find_all_handles::<LayoutComponent>())
            .into_iter()
            .find(|h| {
                !h.is_type_of(
                    nuxie_runtime::source::generated::artboard_base::ArtboardBase::TYPE_KEY,
                ) && h
                    .with(|o| {
                        o.as_layout_component().is_some_and(|l| {
                            l.style_handle().is_some() && l.animation_style() == style
                        })
                    })
                    .unwrap_or(false)
            })
            .expect("layout with requested animation style")
    }
    pub fn advance(&self, seconds: f32) {
        self.artboard.advance_default(seconds);
    }
}
pub fn read_file(asset: &str) -> RuntimeFileHandle {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let bytes = std::fs::read(
        std::path::PathBuf::from(root)
            .join("tests/unit_tests/assets")
            .join(asset),
    )
    .expect("pinned layout fixture");
    let mut factory = PersistentFactory::new(NullFactory::default());
    File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .expect("import layout")
}
pub fn number(h: &CoreHandle, key: u16, value: f32) {
    assert!(CoreRegistry::set_double_handle(h, key.into(), value));
}
pub fn uint(h: &CoreHandle, key: u16, value: u32) {
    assert!(CoreRegistry::set_uint_handle(h, key.into(), value));
}
pub fn boolean(h: &CoreHandle, key: u16, value: bool) {
    assert!(CoreRegistry::set_bool_handle(h, key.into(), value));
}
pub fn style(h: &CoreHandle) -> CoreHandle {
    h.with(|o| o.as_layout_component().unwrap().style_handle())
        .flatten()
        .unwrap()
}
pub fn width(h: &CoreHandle) -> f32 {
    h.with(|o| o.as_layout_component().unwrap().layout_width())
        .unwrap()
}
pub fn linear_tween(container: &CoreHandle) {
    let s = style(container);
    uint(
        &s,
        Style::INTERPOLATION_TYPE_PROPERTY_KEY,
        LayoutStyleInterpolation::Linear as u32,
    );
    number(&s, Style::INTERPOLATION_TIME_PROPERTY_KEY, 1.0);
}
pub fn local(h: &CoreHandle) -> RawPath {
    h.with_mut(|o| {
        o.as_layout_component_mut()
            .unwrap()
            .local_path()
            .unwrap()
            .raw_path()
            .clone()
    })
    .unwrap()
}
pub fn world_bounds(h: &CoreHandle) -> Aabb {
    h.with_mut(|o| {
        o.as_layout_component_mut()
            .unwrap()
            .world_path()
            .unwrap()
            .raw_path()
            .bounds()
    })
    .unwrap()
}
pub fn has_render_path(h: &CoreHandle) -> bool {
    h.with_mut(|o| {
        o.as_layout_component_mut()
            .unwrap()
            .local_path()
            .unwrap()
            .has_render_path()
    })
    .unwrap()
}
pub fn rounded_rect(h: &CoreHandle, radii: [f32; 4]) -> RawPath {
    let bounds = h
        .with(|o| {
            let l = o.as_layout_component().unwrap();
            Aabb::from_ltwh(0.0, 0.0, l.layout_width(), l.layout_height())
        })
        .unwrap();
    let mut p = RawPath::default();
    Path::add_rounded_rect(&mut p, bounds, radii);
    p
}
