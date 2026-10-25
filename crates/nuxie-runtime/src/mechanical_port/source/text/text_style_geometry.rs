use crate::RuntimeFactoryHandle;
use crate::mechanical_port::source::{
    math::raw_path::RawPath, shapes::paint::shape_paint_path::ShapePaintPath,
};
use nuxie_render_api::RenderPaint;
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Opacity(pub(super) f32);
impl Eq for Opacity {}
impl PartialOrd for Opacity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Opacity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

/// The glyph paths of each opacity, ordered by opacity as the source's
/// std::map orders them. A style holds a handful at most, so a sorted vector
/// keeps the map's order and lookups without its tree walks.
#[derive(Default)]
pub(super) struct OpacityPaths(Vec<(Opacity, ShapePaintPath)>);

impl OpacityPaths {
    pub(super) fn clear(&mut self) {
        self.0.clear();
    }

    pub(super) fn len(&self) -> usize {
        self.0.len()
    }

    /// The path of `opacity`, inserted empty in order when absent.
    pub(super) fn entry_or_default(&mut self, opacity: Opacity) -> &mut ShapePaintPath {
        let index = match self.0.binary_search_by(|(key, _)| key.cmp(&opacity)) {
            Ok(index) => index,
            Err(index) => {
                self.0.insert(index, (opacity, ShapePaintPath::default()));
                index
            }
        };
        &mut self.0[index].1
    }

    pub(super) fn get_mut(&mut self, opacity: Opacity) -> Option<&mut ShapePaintPath> {
        self.0
            .binary_search_by(|(key, _)| key.cmp(&opacity))
            .ok()
            .map(|index| &mut self.0[index].1)
    }

    pub(super) fn all_opaque(&self) -> bool {
        self.0.iter().all(|(opacity, _)| opacity.0 == 1.0)
    }

    pub(super) fn iter_mut(&mut self) -> impl Iterator<Item = (Opacity, &mut ShapePaintPath)> {
        self.0.iter_mut().map(|(opacity, path)| (*opacity, path))
    }
}
#[derive(Default)]
pub struct TextStyleGeometry {
    pub(super) opacity_paths: OpacityPaths,
    pub(super) paint_pool: Vec<Box<dyn RenderPaint>>,
    pub(super) path: ShapePaintPath,
    has_contents: bool,
}
impl TextStyleGeometry {
    pub fn rewind(&mut self) {
        self.path.rewind();
        self.has_contents = false;
        self.opacity_paths.clear();
    }
    pub fn has_contents(&self) -> bool {
        self.has_contents
    }
    pub fn path(&mut self) -> &mut ShapePaintPath {
        &mut self.path
    }
    pub fn add_path(&mut self, raw: &RawPath, opacity: f32) -> bool {
        let had = self.has_contents;
        self.has_contents = true;
        if opacity > 0.0 {
            self.path.add_path_clockwise(raw, None);
            self.opacity_paths
                .entry_or_default(Opacity(opacity))
                .add_path_clockwise(raw, None);
        }
        !had
    }
    pub fn reserve_paints(&mut self, factory: &RuntimeFactoryHandle) {
        while self.paint_pool.len() < self.opacity_paths.len() {
            self.paint_pool
                .push(factory.with_factory_mut(|factory| factory.make_render_paint()));
        }
    }
}
