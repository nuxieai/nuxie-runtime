use crate::mechanical_port::source::{
    component::{ComponentDirt, has_dirt},
    core::CoreHandle,
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
        Self {
            base,
            details: NSlicerDetailsState::default(),
            slice_mesh: Box::new(SliceMesh::empty()),
        }
    }
    pub fn image_handle(&self) -> Option<CoreHandle> {
        self.base
            .parent_handle()
            .filter(|parent| parent.with_downcast::<Image, _>(|_| ()).is_some())
    }
    pub fn slice_mesh(&mut self) -> &mut SliceMesh {
        &mut self.slice_mesh
    }
    pub fn draw_mesh(
        &mut self,
        renderer: &mut dyn nuxie_render_api::Renderer,
        image: &dyn nuxie_render_api::RenderImage,
        sampler: nuxie_render_api::ImageSampler,
        blend: nuxie_render_api::BlendMode,
        opacity: f32,
    ) {
        let mesh = std::mem::take(&mut self.slice_mesh);
        mesh.draw(self, renderer, image, sampler, blend, opacity);
        self.slice_mesh = mesh;
    }
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(parent) = self.image_handle() else {
            return StatusCode::MissingObject;
        };
        let Some(this) = self.base.handle() else {
            return StatusCode::MissingObject;
        };
        self.slice_mesh = Box::new(SliceMesh::new(this.clone()));
        parent.with_downcast_mut::<Image, _>(|image| image.set_mesh(Some(this)));
        StatusCode::Ok
    }
    pub fn build_dependencies(&mut self) {
        self.base.build_dependencies();
        if let (Some(parent), Some(this)) = (self.base.parent_handle(), self.base.handle()) {
            parent.with_mut(|parent| parent.component_add_dependent(this));
        }
    }
    pub fn update(&mut self, value: ComponentDirt) {
        if has_dirt(value, ComponentDirt::N_SLICER)
            || has_dirt(value, ComponentDirt::WORLD_TRANSFORM)
        {
            let mut slice_mesh = std::mem::take(&mut self.slice_mesh);
            slice_mesh.update(self);
            self.slice_mesh = slice_mesh;
        }
        self.base.update(value);
    }
}

impl Default for NSlicer {
    fn default() -> Self {
        Self::new(NSlicerBase::default())
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
