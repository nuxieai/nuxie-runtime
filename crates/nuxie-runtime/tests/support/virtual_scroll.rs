//! Shared fixture plumbing for dc75beed's literal virtualized-scroll tests.
#![allow(dead_code, unused_imports)]
#[path = "layout_955.rs"]
mod layout;
pub use layout::{LayoutComponentBase, Style, boolean, number, read_file, style, uint};
pub use nuxie_runtime::source::{
    artboard_component_list::ArtboardComponentList,
    constraints::scrolling::scroll_constraint::ScrollConstraint,
    core::CoreType,
    generated::{
        constraints::draggable_constraint_base::DraggableConstraintBase,
        constraints::scrolling::scroll_constraint_base::ScrollConstraintBase as Scroll,
        layout::layout_sizing_style_base::LayoutSizingStyleBase as Sizing,
    },
    layout_component::LayoutComponent,
    math::{aabb::Aabb, vec2d::Vec2D},
    viewmodel::{
        viewmodel_instance::ViewModelInstance, viewmodel_instance_list::ViewModelInstanceList,
    },
};
pub use nuxie_runtime::{
    Artboard, CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFileHandle,
};
pub fn read<T: std::any::Any, R>(h: &CoreHandle, f: impl FnOnce(&T) -> R) -> R {
    h.with_downcast(f).expect("live source owner")
}
pub fn write<T: std::any::Any, R>(h: &CoreHandle, f: impl FnOnce(&mut T) -> R) -> R {
    h.with_downcast_mut(f).expect("live source owner")
}
pub fn approx(actual: f32, expected: f32) {
    // Catch Approx's default float epsilon is 100 * FLT_EPSILON.
    assert!(
        (actual - expected).abs() <= 100.0 * f32::EPSILON * expected.abs(),
        "{actual} != {expected}"
    );
}
pub struct ScrollFixture {
    pub artboard: RuntimeArtboardInstanceHandle,
    pub instance: CoreHandle,
    pub list: CoreHandle,
    pub scroll: CoreHandle,
    _file: RuntimeFileHandle,
}
impl ScrollFixture {
    pub fn new(asset: &str) -> Self {
        let file = read_file(asset);
        let artboard = file
            .with_file(|file| file.artboard_named("Main"))
            .expect("Main instance");
        let instance = file
            .with_file_mut(|file| {
                file.create_default_view_model_instance_for_artboard(artboard.core_handle())
            })
            .expect("default view model");
        artboard.bind_view_model_instance(Some(instance.clone()));
        let list = artboard
            .with_artboard(|a| a.find_handle::<ArtboardComponentList>("List"))
            .unwrap();
        let scroll =
            artboard.with_artboard(|a| a.find_all_handles::<ScrollConstraint>()[0].clone());
        Self {
            artboard,
            instance,
            list,
            scroll,
            _file: file,
        }
    }
    pub fn wrap(width_scale: u32) -> Self {
        let f = Self::new("component_list_virtualized.riv");
        uint(
            &f.content_style(),
            Sizing::LAYOUT_WIDTH_SCALE_TYPE_PROPERTY_KEY,
            width_scale,
        );
        uint(&f.content_style(), Style::FLEX_WRAP_VALUE_PROPERTY_KEY, 1);
        f.direction(1);
        boolean(&f.scroll, Scroll::INFINITE_PROPERTY_KEY, false);
        f.settle();
        f
    }
    pub fn grid() -> Self {
        let f = Self::new("layout/layout_scroll_grid_virtualized.riv");
        f.settle();
        f
    }
    pub fn content(&self) -> CoreHandle {
        self.artboard
            .with_artboard(|a| a.find_handle::<LayoutComponent>("Content"))
            .unwrap()
    }
    pub fn content_style(&self) -> CoreHandle {
        style(&self.content())
    }
    pub fn direction(&self, direction: u32) {
        uint(
            &self.scroll,
            DraggableConstraintBase::DIRECTION_VALUE_PROPERTY_KEY,
            direction,
        );
    }
    pub fn settle(&self) {
        for _ in 0..3 {
            self.artboard.advance_default(0.0);
        }
    }
    pub fn realized(&self) -> Vec<i32> {
        read::<ArtboardComponentList, _>(&self.list, |list| {
            (0..list.artboard_count() as i32)
                .filter(|&i| list.artboard_instance(i).is_some())
                .collect()
        })
    }
    pub fn bounds(&self, i: usize) -> Aabb {
        read::<ArtboardComponentList, _>(&self.list, |list| list.layout_bounds_for_node(i))
    }
    pub fn drawn_at(&self, i: i32) -> Vec2D {
        read::<ArtboardComponentList, _>(&self.list, |list| {
            let item = list.artboard_instance(i).expect("realized item");
            let t = list.world_transform_for_artboard(&item);
            Vec2D::new(t[4], t[5])
        })
    }
    pub fn offset_x(&self, x: f32) {
        write::<ScrollConstraint, _>(&self.scroll, |s| s.set_offset_x(x));
    }
    pub fn offset_y(&self, y: f32) {
        write::<ScrollConstraint, _>(&self.scroll, |s| s.set_offset_y(y));
    }
    pub fn scroll_x(&self, x: f32) {
        number(&self.scroll, Scroll::SCROLL_OFFSET_X_PROPERTY_KEY, x);
    }
    pub fn scroll_y(&self, y: f32) {
        number(&self.scroll, Scroll::SCROLL_OFFSET_Y_PROPERTY_KEY, y);
    }
    pub fn fixed_wrap_width(&self) {
        uint(
            &self.content_style(),
            Sizing::LAYOUT_WIDTH_SCALE_TYPE_PROPERTY_KEY,
            0,
        );
        number(
            &self.content(),
            LayoutComponentBase::WIDTH_PROPERTY_KEY,
            800.0,
        );
    }
    pub fn narrow_grid(&self) {
        number(
            &self.artboard.core_handle(),
            LayoutComponentBase::WIDTH_PROPERTY_KEY,
            250.0,
        );
        uint(
            &self.content_style(),
            Sizing::LAYOUT_WIDTH_SCALE_TYPE_PROPERTY_KEY,
            2,
        );
    }
}
