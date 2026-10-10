use crate::source::{
    artboard::Artboard,
    bitmap_cache::BitmapCache,
    core::CoreHandle,
    math::{mat2d::Mat2D, vec2d::Vec2D},
    nested_artboard::NestedArtboard,
    offscreen_raster as offscreen,
};
use std::{cell::RefCell, rc::Rc};

/// Allocated only when a mesh or N-slice attaches to a Node-mode instance.
pub struct NestedArtboardMeshHost {
    owner: CoreHandle,
    mesh: Option<CoreHandle>,
    raster: Rc<RefCell<offscreen::CachedRaster>>,
    raster_source: Option<CoreHandle>,
    uv_transform: nuxie_render_api::Mat2D,
    uv_valid: bool,
}
impl NestedArtboardMeshHost {
    pub fn new(owner: CoreHandle) -> Self {
        Self {
            owner,
            mesh: None,
            raster: Default::default(),
            raster_source: None,
            uv_transform: nuxie_render_api::Mat2D::IDENTITY,
            uv_valid: false,
        }
    }
    fn instance(&self) -> Option<CoreHandle> {
        self.owner
            .with_downcast::<NestedArtboard, _>(|n| {
                n.artboard_instance_default().map(|a| a.core_handle())
            })
            .flatten()
    }
    fn size_source(&self) -> Option<CoreHandle> {
        self.instance().or_else(|| {
            self.owner
                .with_downcast::<NestedArtboard, _>(NestedArtboard::source_artboard)
                .flatten()
        })
    }
    pub fn reset_raster(&mut self) {
        self.raster.borrow_mut().release();
        self.raster_source = None;
        self.uv_valid = false;
    }
    pub fn set_mesh(&mut self, mesh: Option<CoreHandle>) {
        if self.mesh == mesh {
            return;
        }
        self.mesh = mesh;
        self.uv_valid = false;
        if self.mesh.is_none() {
            self.raster.borrow_mut().release();
            self.raster_source = None;
        }
    }
    pub fn hosted_mesh(&self) -> Option<CoreHandle> {
        self.mesh.clone()
    }
    pub fn raster_held(&self) -> bool {
        self.raster.borrow().canvas.is_some()
    }
    pub fn width(&self) -> f32 {
        self.size_source()
            .and_then(|a| a.with_downcast::<Artboard, _>(|a| a.bounds().width()))
            .unwrap_or(0.0)
    }
    pub fn height(&self) -> f32 {
        self.size_source()
            .and_then(|a| a.with_downcast::<Artboard, _>(|a| a.bounds().height()))
            .unwrap_or(0.0)
    }
    pub fn origin(&self) -> Vec2D {
        self.instance()
            .and_then(|a| {
                a.with_downcast::<Artboard, _>(|a| {
                    let b = a.bounds();
                    Vec2D::new(
                        if b.width() != 0.0 {
                            -b.left() / b.width()
                        } else {
                            0.0
                        },
                        if b.height() != 0.0 {
                            -b.top() / b.height()
                        } else {
                            0.0
                        },
                    )
                })
            })
            .unwrap_or_default()
    }
    pub fn world_transform(&self) -> Mat2D {
        let world = self
            .owner
            .with_downcast::<NestedArtboard, _>(|n| *n.world_transform())
            .unwrap();
        self.instance()
            .and_then(|a| {
                a.with_downcast::<Artboard, _>(|a| {
                    if a.has_self_transform() {
                        world * a.self_transform()
                    } else {
                        world
                    }
                })
            })
            .unwrap_or(world)
    }
    pub fn scale_x(&self) -> f32 {
        self.owner
            .with_downcast::<NestedArtboard, _>(|n| n.scale_x())
            .unwrap()
    }
    pub fn scale_y(&self) -> f32 {
        self.owner
            .with_downcast::<NestedArtboard, _>(|n| n.scale_y())
            .unwrap()
    }
    pub fn ready(&self) -> bool {
        self.instance().is_some()
    }
    pub fn mesh_raster_resolution(&self) -> f32 {
        self.instance()
            .and_then(|a| a.with_downcast::<Artboard, _>(Artboard::bitmap_cache))
            .flatten()
            .and_then(|c| c.with_downcast::<BitmapCache, _>(|c| c.resolution()))
            .unwrap_or(1.0)
    }
    pub fn uv_transform(&self) -> nuxie_render_api::Mat2D {
        let Some(bounds) = self
            .instance()
            .and_then(|a| a.with_downcast::<Artboard, _>(Artboard::bounds))
        else {
            return nuxie_render_api::Mat2D::IDENTITY;
        };
        let mut plan = offscreen::RasterPlan::default();
        if !offscreen::plan_fixed_raster_size(&bounds, self.mesh_raster_resolution(), &mut plan) {
            return nuxie_render_api::Mat2D::IDENTITY;
        }
        nuxie_render_api::Mat2D([
            plan.bounds.width() * plan.raster_scale / plan.width_px as f32,
            0.0,
            0.0,
            plan.bounds.height() * plan.raster_scale / plan.height_px as f32,
            0.0,
            0.0,
        ])
    }
    fn under_clip(&self) -> bool {
        let mut current = Some(self.owner.clone());
        while let Some(c) = current {
            let (clipped, parent, host) = c
                .with(|o| {
                    let clipped = o
                        .as_drawable()
                        .is_some_and(|d| !d.clipping_shapes().is_empty())
                        || (c != self.owner && o.as_layout_component().is_some_and(|l| l.clip()));
                    let parent = o.as_component().and_then(|c| c.parent_handle());
                    let host = o
                        .as_any()
                        .downcast_ref::<Artboard>()
                        .and_then(Artboard::host);
                    (clipped, parent, host)
                })
                .unwrap();
            if clipped {
                return true;
            }
            current = parent.or_else(|| {
                host.and_then(|h| {
                    h.with(|h| h.as_artboard_host().and_then(|h| h.host_component()))
                        .flatten()
                })
            });
        }
        false
    }
    pub fn composite_drops_clip(&self) -> bool {
        let Some(instance) = self.instance() else {
            return false;
        };
        if !self.under_clip() {
            return false;
        }
        instance
            .with_downcast::<Artboard, _>(Artboard::factory)
            .flatten()
            .and_then(|f| f.with_factory_mut(|f| f.canvas_content_host()))
            .is_some_and(|h| h.borrow().composite_drops_clips())
    }
    /// Raster callbacks may reenter the owner. Retain identity, not a core borrow.
    pub fn draw(state: &Rc<RefCell<Self>>, renderer: &mut dyn nuxie_render_api::Renderer) -> bool {
        let Some(instance) = state.borrow().instance() else {
            return true;
        };
        {
            let mut state = state.borrow_mut();
            if state.raster_source.as_ref() != Some(&instance) {
                state.raster.borrow_mut().release();
                state.raster_source = Some(instance.clone());
                state.uv_valid = false;
            }
        }
        if instance.with_downcast::<Artboard, _>(Artboard::consume_invisible_change) == Some(true) {
            return true;
        }
        let resolution = state.borrow().mesh_raster_resolution();
        let raster = state.borrow().raster.clone();
        let result = Artboard::raster_image_handle(&instance, renderer, &raster, resolution, true);
        let Some((image, plan, host)) = result else {
            return false;
        };
        let uv = state.borrow().uv_transform();
        let (mesh, owner, changed) = {
            let mut s = state.borrow_mut();
            let changed = !s.uv_valid || s.uv_transform != uv;
            s.uv_transform = uv;
            s.uv_valid = true;
            (s.mesh.clone().unwrap(), s.owner.clone(), changed)
        };
        if changed {
            mesh.with_mut(|m| m.mesh_drawable_uv_transform_changed(uv));
        }
        let blend = owner
            .with_downcast::<NestedArtboard, _>(|n| n.blend_mode())
            .unwrap();
        let mut placement = offscreen::begin_placement(renderer, Some(&host), &plan);
        let opacity = placement.opacity;
        mesh.with_mut(|m| {
            m.mesh_drawable_draw(
                placement.renderer(),
                image.as_ref(),
                nuxie_render_api::ImageSampler::LINEAR_CLAMP,
                blend.into(),
                opacity,
                0.0,
            )
        });
        offscreen::end_composite(placement);
        true
    }
}
