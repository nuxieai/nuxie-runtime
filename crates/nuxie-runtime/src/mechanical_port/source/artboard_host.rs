use crate::mechanical_port::source::{
    artboard::{RuntimeArtboardInstanceHandle, RuntimeArtboardInstanceWeakHandle},
    core::CoreHandle,
    data_bind::data_context::RuntimeDataContextHandle,
    data_bind_path_referencer::DataBindPathReferencer,
    file::RuntimeFileWeakHandle,
    math::{mat2d::Mat2D, vec2d::Vec2D},
};

pub trait ArtboardHost {
    fn data_bind_path_referencer(&self) -> &DataBindPathReferencer;
    fn artboard_count(&self) -> usize;
    fn artboard_instance(&self, index: i32) -> Option<RuntimeArtboardInstanceHandle>;
    fn internal_data_context(&mut self, data_context: RuntimeDataContextHandle);
    fn bind_view_model_instance(
        &mut self,
        view_model_instance: CoreHandle,
        parent: RuntimeDataContextHandle,
    );
    fn clear_data_context(&mut self);
    fn unbind(&mut self);
    fn update_data_binds(&mut self);
    fn mark_hosting_layout_dirty(&mut self, artboard_instance: RuntimeArtboardInstanceWeakHandle);
    fn parent_artboard(&self) -> Option<CoreHandle>;
    fn hit_test_host(
        &mut self,
        position: &Vec2D,
        skip_on_unclipped: bool,
        artboard: RuntimeArtboardInstanceWeakHandle,
    ) -> bool;
    fn host_transform_point(
        &self,
        position: &Vec2D,
        artboard: RuntimeArtboardInstanceWeakHandle,
    ) -> Vec2D;
    fn world_transform_for_artboard(&self, artboard: RuntimeArtboardInstanceWeakHandle) -> Mat2D;
    fn mark_host_transform_dirty(&mut self);
    fn is_layout_provider(&self) -> bool {
        false
    }
    fn set_file(&mut self, value: Option<RuntimeFileWeakHandle>);
    fn file(&self) -> Option<RuntimeFileWeakHandle>;
    fn host_component(&self) -> Option<CoreHandle>;
    // Upstream ArtboardHost provides a no-op default; only hosts that relink
    // their own context override it (include/rive/artboard_host.hpp).
    fn relink_data_context(&mut self, _view_model_instance: Option<CoreHandle>) {}
    fn type_(&self) -> i32;
}
