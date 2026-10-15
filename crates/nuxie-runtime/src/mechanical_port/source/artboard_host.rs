use crate::mechanical_port::source::{
    artboard::{RuntimeArtboardInstanceHandle, RuntimeArtboardInstanceWeakHandle},
    core::CoreHandle,
    data_bind::data_context::RuntimeDataContextHandle,
    data_bind_path_referencer::DataBindPathReferencer,
    file::RuntimeFileWeakHandle,
    math::{mat2d::Mat2D, vec2d::Vec2D},
};

// Every mark is the change sequence current when it was made, so no mark
// passes the highest one made on this thread.
thread_local! {
    static HIGHEST_INACTIVE_MARK: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// The latest change sequence a host saw while its artboards were inactive,
/// shared with the artboards it mounts.
#[derive(Clone, Default)]
pub struct InactiveChangeMark(std::rc::Rc<std::cell::Cell<u64>>);

impl InactiveChangeMark {
    pub fn get(&self) -> u64 {
        self.0.get()
    }

    /// Mark the change sequence current now.
    pub fn mark_latest(&self) {
        let sequence = crate::source::viewmodel::viewmodel_instance_value::ViewModelInstanceValue::latest_change_sequence();
        self.0.set(sequence);
        HIGHEST_INACTIVE_MARK.with(|highest| highest.set(highest.get().max(sequence)));
    }

    /// No mark on this thread is above this.
    pub fn highest() -> u64 {
        HIGHEST_INACTIVE_MARK.with(std::cell::Cell::get)
    }
}

pub trait ArtboardHost {
    /// The host's actual inactive sequence storage is shared with mounted
    /// artboards so a child can read it while its host is advancing it.
    fn inactive_change_sequence_state(&self) -> InactiveChangeMark;
    fn inactive_change_sequence(&self) -> u64 {
        self.inactive_change_sequence_state().get()
    }
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
    fn hosted_row_woke(&mut self, _artboard: &CoreHandle, _row: u32) {}
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
    /// The file the hosted artboard was bound in from, when it isn't file().
    fn foreign_file(&self) -> Option<RuntimeFileWeakHandle> {
        None
    }
    fn host_component(&self) -> Option<CoreHandle>;
    // Upstream ArtboardHost provides a no-op default; only hosts that relink
    // their own context override it (include/rive/artboard_host.hpp).
    fn relink_data_context(&mut self, _view_model_instance: Option<CoreHandle>) {}
    fn type_(&self) -> i32;
}
