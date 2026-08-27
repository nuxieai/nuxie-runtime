use crate::mechanical_port::source::{
    component::{ComponentDirt, has_dirt},
    core_context::{CoreContext, StatusCode},
    generated::layout::n_slicer_base::NSlicerBase,
    layout::n_slicer_details::{NSlicerDetails, NSlicerDetailsState},
    shapes::{image::Image, slice_mesh::SliceMesh},
};

pub struct NSlicer {
    pub base: NSlicerBase,
    details: NSlicerDetailsState,
    slice_mesh: Box<SliceMesh>,
}

impl NSlicer {
    pub const TYPE_KEY: u16 = NSlicerBase::TYPE_KEY;
    pub fn new(base: NSlicerBase) -> Self {
        let mut value = Self {
            base,
            details: NSlicerDetailsState::default(),
            slice_mesh: Box::new(SliceMesh::empty()),
        };
        value.slice_mesh = Box::new(SliceMesh::new(&mut value));
        value
    }
    pub fn image(&mut self) -> Option<&mut Image> {
        self.base
            .parent_mut()
            .and_then(|parent| parent.as_mut::<Image>())
    }
    pub fn slice_mesh(&mut self) -> &mut SliceMesh {
        &mut self.slice_mesh
    }
    pub fn on_added_dirty(&mut self, context: &mut CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        if !self.base.parent().is::<Image>() {
            return StatusCode::MissingObject;
        }
        self.base
            .parent_mut()
            .as_mut::<Image>()
            .unwrap()
            .set_mesh(&mut *self.slice_mesh);
        StatusCode::Ok
    }
    pub fn build_dependencies(&mut self) {
        self.base.build_dependencies();
        self.base
            .parent_mut()
            .add_dependent(self.base.as_component_mut_ptr());
    }
    pub fn update(&mut self, value: ComponentDirt) {
        if has_dirt(value, ComponentDirt::N_SLICER)
            || has_dirt(value, ComponentDirt::WORLD_TRANSFORM)
        {
            self.slice_mesh.update();
        }
        self.base.update(value);
    }
}

impl NSlicerDetails for NSlicer {
    fn details_state(&mut self) -> &mut NSlicerDetailsState {
        &mut self.details
    }
    fn axis_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::N_SLICER);
    }
}
