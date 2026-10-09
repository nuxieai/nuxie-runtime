use crate::mechanical_port::source::{
    factory::RuntimeFactoryHandle,
    math::{aabb::Aabb, mat2d::Mat2D, path_types::PathDirection, raw_path::RawPath},
};
use nuxie_render_api::{FillRule, RenderPath};
pub struct ShapePaintPath {
    render_path_dirty: bool,
    render_path: Option<Box<dyn RenderPath>>,
    raw_path: RawPath,
    is_local: bool,
    fill_rule: FillRule,
}
impl Default for ShapePaintPath {
    fn default() -> Self {
        Self::new(true)
    }
}
impl ShapePaintPath {
    pub fn new(is_local: bool) -> Self {
        Self {
            render_path_dirty: true,
            render_path: None,
            raw_path: RawPath::default(),
            is_local,
            fill_rule: FillRule::Clockwise,
        }
    }
    pub fn with_fill_rule(is_local: bool, fill_rule: FillRule) -> Self {
        Self {
            fill_rule,
            ..Self::new(is_local)
        }
    }
    pub fn raw_path(&self) -> &RawPath {
        &self.raw_path
    }
    pub fn mutable_raw_path(&mut self) -> &mut RawPath {
        &mut self.raw_path
    }
    pub fn is_local(&self) -> bool {
        self.is_local
    }
    pub fn fill_rule(&self) -> FillRule {
        self.fill_rule
    }
    pub fn empty(&self) -> bool {
        self.raw_path.empty()
    }
    pub fn rewind(&mut self) {
        self.raw_path.rewind();
        self.render_path_dirty = true;
    }
    pub fn rewind_as(&mut self, is_local: bool, fill_rule: FillRule) {
        self.is_local = is_local;
        self.fill_rule = fill_rule;
        self.rewind();
    }
    pub fn rewind_local(&mut self, is_local: bool) {
        self.is_local = is_local;
        self.rewind();
    }
    pub fn add_path(&mut self, raw_path: &RawPath, transform: Option<&Mat2D>) {
        let iterator = self.raw_path.add_path(raw_path, transform);
        self.raw_path.prune_empty_segments_from(iterator);
        self.render_path_dirty = true;
    }
    pub fn add_shape_paint_path(&mut self, path: &ShapePaintPath, transform: Option<&Mat2D>) {
        self.add_path(path.raw_path(), transform);
    }
    pub fn add_path_backwards(&mut self, raw_path: &RawPath, transform: Option<&Mat2D>) {
        let iterator = self.raw_path.add_path_backwards(raw_path, transform);
        self.raw_path.prune_empty_segments_from(iterator);
        self.render_path_dirty = true;
    }
    pub fn add_shape_paint_path_backwards(
        &mut self,
        path: &ShapePaintPath,
        transform: Option<&Mat2D>,
    ) {
        self.add_path_backwards(path.raw_path(), transform);
    }
    pub fn add_path_clockwise(&mut self, raw_path: &RawPath, transform: Option<&Mat2D>) {
        let mut area = raw_path.compute_coarse_area();
        if let Some(transform) = transform {
            area *= transform.determinant();
        }
        if area < 0.0 {
            self.add_path_backwards(raw_path, transform);
        } else {
            self.add_path(raw_path, transform);
        }
    }
    pub fn add_rect(&mut self, aabb: Aabb, direction: PathDirection) {
        self.raw_path.add_rect(aabb, direction);
    }
    pub fn has_render_path(&self) -> bool {
        self.render_path.is_some() && !self.render_path_dirty
    }
    pub fn render_path(&mut self, factory: &RuntimeFactoryHandle) -> &mut dyn RenderPath {
        if self.render_path.is_none() {
            self.render_path =
                Some(factory.with_factory_mut(|factory| factory.make_empty_render_path()));
            let path = self.render_path.as_mut().unwrap();
            path.add_raw_path(self.raw_path.as_render_path_ref());
            path.fill_rule(self.fill_rule);
            self.render_path_dirty = false;
        }
        self.update_existing_render_path()
    }

    /// The source factory pointer is used only to create a RenderPath. Its
    /// nonvirtual Component/Artboard getters have no effect on an existing
    /// resource, so defer the Rust owner resolution until creation needs it.
    pub(crate) fn render_path_with_deferred_factory(
        &mut self,
        factory: impl FnOnce() -> RuntimeFactoryHandle,
    ) -> &mut dyn RenderPath {
        if self.render_path.is_none() {
            let factory = factory();
            return self.render_path(&factory);
        }
        self.update_existing_render_path()
    }

    fn update_existing_render_path(&mut self) -> &mut dyn RenderPath {
        if self.render_path_dirty {
            let path = self.render_path.as_mut().unwrap();
            path.rewind();
            path.add_raw_path(self.raw_path.as_render_path_ref());
            self.render_path_dirty = false;
        }
        self.render_path.as_deref_mut().unwrap()
    }

    /// Reborrow the resource selected by render_path without repeating its
    /// factory lookup or dirty-path callbacks.
    pub(crate) fn existing_render_path(&mut self) -> &mut dyn RenderPath {
        self.render_path
            .as_deref_mut()
            .expect("resolved RenderPath")
    }
}

#[cfg(test)]
mod deferred_factory_tests {
    use super::*;
    use nuxie_render_api::{Mat2D as RenderMat2D, RawPathRef};
    use std::{
        any::Any,
        cell::{Cell, RefCell},
        rc::Rc,
    };

    struct TracePath(Rc<RefCell<Vec<&'static str>>>, Rc<Cell<bool>>);
    impl RenderPath for TracePath {
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
        fn rewind(&mut self) {
            self.0.borrow_mut().push("rewind");
        }
        fn fill_rule(&mut self, _: FillRule) {
            self.0.borrow_mut().push("fill_rule");
        }
        fn add_raw_path(&mut self, _: RawPathRef<'_>) {
            self.0.borrow_mut().push("add_raw_path");
            assert!(!self.1.replace(false), "first path upload failed");
        }
        fn add_render_path(&mut self, _: &dyn RenderPath, _: RenderMat2D) {
            unreachable!();
        }
        fn add_render_path_self(&mut self, _: RenderMat2D) {
            unreachable!();
        }
        fn move_to(&mut self, _: f32, _: f32) {
            unreachable!();
        }
        fn line_to(&mut self, _: f32, _: f32) {
            unreachable!();
        }
        fn cubic_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {
            unreachable!();
        }
        fn close(&mut self) {
            unreachable!();
        }
    }

    #[test]
    fn existing_clean_and_dirty_paths_preserve_identity_and_callback_order_without_factory() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let mut path = ShapePaintPath::new(true);
        path.render_path = Some(Box::new(TracePath(
            calls.clone(),
            Rc::new(Cell::new(false)),
        )));
        path.render_path_dirty = false;
        let clean =
            path.render_path_with_deferred_factory(|| panic!("clean resource needs no factory"));
        assert!(Rc::ptr_eq(
            &clean.as_any().downcast_ref::<TracePath>().unwrap().0,
            &calls
        ));
        assert!(calls.borrow().is_empty());
        path.rewind();
        let dirty =
            path.render_path_with_deferred_factory(|| panic!("dirty resource needs no factory"));
        assert!(Rc::ptr_eq(
            &dirty.as_any().downcast_ref::<TracePath>().unwrap().0,
            &calls
        ));
        assert_eq!(&*calls.borrow(), &["rewind", "add_raw_path"]);
        assert!(path.has_render_path());
        path.render_path_with_deferred_factory(|| panic!("updated resource needs no factory"));
        assert_eq!(&*calls.borrow(), &["rewind", "add_raw_path"]);
    }

    struct TraceFactory {
        calls: Rc<RefCell<Vec<&'static str>>>,
        fail_first_upload: Rc<Cell<bool>>,
    }
    impl nuxie_render_api::Factory for TraceFactory {
        fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
            self.calls.borrow_mut().push("make_empty");
            Box::new(TracePath(
                self.calls.clone(),
                self.fail_first_upload.clone(),
            ))
        }
        fn make_render_buffer(
            &mut self,
            _: nuxie_render_api::RenderBufferType,
            _: nuxie_render_api::RenderBufferFlags,
            _: usize,
        ) -> Box<dyn nuxie_render_api::RenderBuffer> {
            unreachable!()
        }
        fn make_linear_gradient(
            &mut self,
            _: f32,
            _: f32,
            _: f32,
            _: f32,
            _: &[nuxie_render_api::ColorInt],
            _: &[f32],
        ) -> Box<dyn nuxie_render_api::RenderShader> {
            unreachable!()
        }
        fn make_radial_gradient(
            &mut self,
            _: f32,
            _: f32,
            _: f32,
            _: &[nuxie_render_api::ColorInt],
            _: &[f32],
        ) -> Box<dyn nuxie_render_api::RenderShader> {
            unreachable!()
        }
        fn make_render_path(
            &mut self,
            _: nuxie_render_api::RawPath,
            _: FillRule,
        ) -> Box<dyn RenderPath> {
            unreachable!()
        }
        fn make_render_paint(&mut self) -> Box<dyn nuxie_render_api::RenderPaint> {
            unreachable!()
        }
        fn decode_image(
            &mut self,
            _: &[u8],
        ) -> Result<Box<dyn nuxie_render_api::RenderImage>, nuxie_render_api::ImageDecodeError>
        {
            unreachable!()
        }
    }

    #[test]
    fn failed_first_upload_retains_the_published_resource_for_retry() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let mut factory = nuxie_render_api::PersistentFactory::new(TraceFactory {
            calls: calls.clone(),
            fail_first_upload: Rc::new(Cell::new(true)),
        });
        let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
        let mut path = ShapePaintPath::new(true);
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            path.render_path(&factory);
        }))
        .is_err());
        assert!(
            path.render_path.is_some(),
            "the resource is published before upload"
        );
        assert!(!path.has_render_path());
        path.render_path_with_deferred_factory(|| {
            panic!("retry must reuse the published resource")
        });
        assert!(path.has_render_path());
        assert_eq!(
            &*calls.borrow(),
            &["make_empty", "add_raw_path", "rewind", "add_raw_path"]
        );
    }
}
