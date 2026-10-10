use crate::source::{
    core::CoreHandle,
    math::{mat2d::Mat2D, vec2d::Vec2D},
    nested_artboard::NestedArtboard,
    shapes::image::Image,
};

/// Identity-bearing projection of the two exact upstream MeshHost core types.
#[derive(Clone)]
pub struct MeshHost(CoreHandle);
impl MeshHost {
    pub fn from_handle(component: Option<CoreHandle>) -> Option<Self> {
        let component = component?;
        match component.core_type()? {
            Image::TYPE_KEY => {}
            NestedArtboard::TYPE_KEY => {
                component.with_downcast_mut::<NestedArtboard, _>(|n| {
                    n.mesh_host();
                })?;
            }
            _ => return None,
        }
        Some(Self(component))
    }
    pub fn set_mesh(&self, mesh: Option<CoreHandle>) {
        if self.0.core_type() == Some(Image::TYPE_KEY) {
            self.0.with_downcast_mut::<Image, _>(|i| i.set_mesh(mesh));
        } else {
            let state = self
                .0
                .with_downcast::<NestedArtboard, _>(|n| n.mesh_host_if_any().unwrap())
                .unwrap();
            state.borrow_mut().set_mesh(mesh);
        }
    }
    pub fn hosted_mesh(&self) -> Option<CoreHandle> {
        self.0
            .with_downcast::<Image, _>(Image::mesh)
            .unwrap_or_else(|| {
                self.0
                    .with_downcast::<NestedArtboard, _>(NestedArtboard::hosted_mesh)
                    .flatten()
            })
    }
    fn image_or<T>(
        &self,
        image: impl FnOnce(&Image) -> T,
        nested: impl FnOnce(&crate::source::nested_artboard_mesh_host::NestedArtboardMeshHost) -> T,
    ) -> T {
        if self.0.core_type() == Some(Image::TYPE_KEY) {
            self.0.with_downcast(image).unwrap()
        } else {
            let state = self
                .0
                .with_downcast::<NestedArtboard, _>(|n| n.mesh_host_if_any().unwrap())
                .unwrap();
            let value = nested(&state.borrow());
            value
        }
    }
    pub fn width(&self) -> f32 {
        self.image_or(Image::width, |n| n.width())
    }
    pub fn height(&self) -> f32 {
        self.image_or(Image::height, |n| n.height())
    }
    pub fn origin(&self) -> Vec2D {
        self.image_or(|i| Vec2D::new(i.origin_x(), i.origin_y()), |n| n.origin())
    }
    pub fn scale_x(&self) -> f32 {
        self.image_or(Image::render_scale_x, |n| n.scale_x())
    }
    pub fn scale_y(&self) -> f32 {
        self.image_or(Image::render_scale_y, |n| n.scale_y())
    }
    pub fn world_transform(&self) -> Mat2D {
        self.image_or(|i| *i.world_transform(), |n| n.world_transform())
    }
    pub fn ready(&self) -> bool {
        self.image_or(|i| i.image_asset().is_some(), |n| n.ready())
    }
    pub fn uv_transform(&self) -> nuxie_render_api::Mat2D {
        self.image_or(Image::mesh_host_uv_transform, |n| n.uv_transform())
    }
}
