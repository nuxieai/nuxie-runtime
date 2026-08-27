use std::rc::Rc;

use crate::mechanical_port::source::{
    artboard::{Artboard, ArtboardInstance},
    component::Component,
    data_bind::data_context::DataContext,
    data_bind_path_referencer::DataBindPathReferencer,
    file::File,
    math::{mat2d::Mat2D, vec2d::Vec2D},
    viewmodel::viewmodel_instance::ViewModelInstance,
};

pub trait ArtboardHost: DataBindPathReferencer {
    fn artboard_count(&self) -> usize;
    fn artboard_instance(&mut self, index: i32) -> Option<&mut ArtboardInstance>;
    fn internal_data_context(&mut self, data_context: Rc<DataContext>);
    fn bind_view_model_instance(
        &mut self,
        view_model_instance: Rc<ViewModelInstance>,
        parent: Rc<DataContext>,
    );
    fn clear_data_context(&mut self);
    fn unbind(&mut self);
    fn update_data_binds(&mut self);
    fn mark_hosting_layout_dirty(&mut self, _artboard_instance: *mut ArtboardInstance) {}
    fn parent_artboard(&mut self) -> &mut Artboard;
    fn hit_test_host(
        &mut self,
        position: &Vec2D,
        skip_on_unclipped: bool,
        artboard: *mut ArtboardInstance,
    ) -> bool;
    fn host_transform_point(&self, position: &Vec2D, artboard: *mut ArtboardInstance) -> Vec2D;
    fn world_transform_for_artboard(&self, artboard: *mut ArtboardInstance) -> Mat2D;
    fn mark_host_transform_dirty(&mut self);
    fn is_layout_provider(&self) -> bool {
        false
    }
    fn set_file(&mut self, value: *mut File);
    fn file(&self) -> *mut File;
    fn host_component(&mut self) -> Option<&mut Component> {
        None
    }
    fn relink_data_context(&mut self, _view_model_instance: Rc<ViewModelInstance>) {}
    fn type_(&self) -> i32;
}
