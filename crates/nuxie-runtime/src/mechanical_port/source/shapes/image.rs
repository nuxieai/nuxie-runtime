use crate::mechanical_port::source::{
    assets::{file_asset_referencer::FileAssetReferencer, image_asset::ImageAsset},
    core::{Core, CoreHandle, StatusCode},
    generated::shapes::image_base::ImageBase,
    hit_info::HitInfo,
    importers::import_stack::ImportStack,
    layout::{
        LayoutDirection, LayoutMeasureMode, LayoutScaleType, alignment::Alignment,
        layout_participant::LayoutParticipant,
    },
    math::{aabb::Aabb, hit_test::HitTester, mat2d::Mat2D, vec2d::Vec2D},
    renderer::{BlendMode, ImageSampler, Renderer},
};

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ImageFit {
    Resize,
    Contain,
    Cover,
    FitWidth,
    FitHeight,
    None,
    ScaleDown,
    Fill,
}

impl From<u32> for ImageFit {
    fn from(value: u32) -> Self {
        match value as u8 {
            0 => Self::Resize,
            1 => Self::Contain,
            2 => Self::Cover,
            3 => Self::FitWidth,
            4 => Self::FitHeight,
            5 => Self::None,
            6 => Self::ScaleDown,
            // Upstream's switch shares its default with fill, not resize.
            _ => Self::Fill,
        }
    }
}

pub struct Image {
    pub base: ImageBase,
    pub file_asset_referencer: FileAssetReferencer,
    mesh: Option<CoreHandle>,
    layout_width: f32,
    layout_height: f32,
    layout_offset_x: f32,
    layout_offset_y: f32,
    layout_scale_x: f32,
    layout_scale_y: f32,
    layout_scale_separate: bool,
}

impl Default for Image {
    fn default() -> Self {
        Self {
            base: ImageBase::default(),
            file_asset_referencer: FileAssetReferencer::default(),
            mesh: None,
            layout_width: f32::NAN,
            layout_height: f32::NAN,
            layout_offset_x: 0.0,
            layout_offset_y: 0.0,
            layout_scale_x: 1.0,
            layout_scale_y: 1.0,
            layout_scale_separate: true,
        }
    }
}

impl Image {
    pub fn draw_occurrence(owner: &CoreHandle, renderer: &mut Renderer) {
        let Some((image, mesh, save, world, origin_x, origin_y, blend_mode, opacity)) = owner
            .with_downcast::<Self, _>(|owner| {
                Some((
                    owner.render_image()?,
                    owner.mesh.clone(),
                    owner.base.needs_save_operation(),
                    *owner.base.world_transform(),
                    owner.base.origin_x(),
                    owner.base.origin_y(),
                    owner.base.blend_mode(),
                    owner.base.render_opacity(),
                ))
            })
            .flatten()
        else {
            return;
        };
        if save {
            renderer.save();
        }
        if let Some(mesh) = mesh {
            mesh.with_mut(|mesh| {
                mesh.mesh_drawable_draw(
                    renderer,
                    image.as_ref(),
                    ImageSampler::LINEAR_CLAMP,
                    blend_mode.into(),
                    opacity,
                )
            });
        } else {
            renderer.transform(nuxie_render_api::Mat2D(*world.values()));
            renderer.translate(
                -(image.width() as f32) * origin_x,
                -(image.height() as f32) * origin_y,
            );
            renderer.draw_image(
                Some(image.as_ref()),
                ImageSampler::LINEAR_CLAMP,
                blend_mode.into(),
                opacity,
            );
        }
        if save {
            renderer.restore();
        }
    }

    pub fn set_asset_occurrence(owner: &CoreHandle, asset: Option<CoreHandle>) {
        let Some(asset) =
            asset.filter(|asset| asset.with_downcast::<ImageAsset, _>(|_| ()).is_some())
        else {
            return;
        };
        let (mesh, image, is_instance) = owner
            .with_downcast_mut::<Self, _>(|image| {
                image
                    .file_asset_referencer
                    .set_asset(owner.clone(), Some(asset));
                let is_instance = image.mesh.is_some()
                    && image
                        .base
                        .artboard_handle()
                        .and_then(|artboard| {
                            artboard.with(|artboard| {
                                artboard
                                    .as_artboard()
                                    .map(|artboard| artboard.is_instance())
                            })
                        })
                        .flatten()
                        .expect("Image artboard");
                (image.mesh.clone(), image.render_image(), is_instance)
            })
            .expect("retained Image");
        if !is_instance && let (Some(mesh), Some(image)) = (mesh, image) {
            mesh.with_mut(|mesh| mesh.mesh_drawable_on_asset_loaded(image.as_ref()));
        }
        owner.with_downcast_mut::<Self, _>(Self::update_image_scale);
    }

    pub fn draw(&mut self, renderer: &mut Renderer) {
        let Some(render_image) = self.render_image() else {
            return;
        };
        if self.base.needs_save_operation() {
            renderer.save();
        }
        let width = render_image.width() as f32;
        let height = render_image.height() as f32;
        if let Some(mesh) = self.mesh.clone() {
            mesh.with_mut(|mesh| {
                mesh.mesh_drawable_draw(
                    renderer,
                    render_image.as_ref(),
                    ImageSampler::LINEAR_CLAMP,
                    self.base.blend_mode().into(),
                    self.base.render_opacity(),
                );
            });
        } else {
            renderer.transform(nuxie_render_api::Mat2D(
                *self.base.world_transform().values(),
            ));
            renderer.translate(
                -width * self.base.origin_x(),
                -height * self.base.origin_y(),
            );
            renderer.draw_image(
                Some(render_image.as_ref()),
                ImageSampler::LINEAR_CLAMP,
                self.base.blend_mode().into(),
                self.base.render_opacity(),
            );
        }
        if self.base.needs_save_operation() {
            renderer.restore();
        }
    }

    pub fn will_draw(&self) -> bool {
        self.base.will_draw() && self.base.render_opacity() != 0.0 && self.image_asset().is_some()
    }

    pub fn hit_test<'a>(&'a self, hinfo: &HitInfo, xform: Mat2D) -> Option<&'a Core> {
        let render_image = self.render_image()?;
        let width = render_image.width() as f32;
        let height = render_image.height() as f32;
        if self.mesh.is_some() {
            println!("Missing mesh");
        } else {
            let matrix = xform
                * self.base.world_transform()
                * Mat2D::from_translate(
                    -width * self.base.origin_x(),
                    -height * self.base.origin_y(),
                );
            let mut tester = HitTester::new(hinfo.area());
            tester.add_rect(Aabb::new(0.0, 0.0, width, height), matrix);
            if tester.test() {
                return Some(self.base.as_core());
            }
        }
        None
    }

    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(this) = self.base.handle() else {
            return StatusCode::MissingObject;
        };
        let result = self.file_asset_referencer.register_referencer(this, stack);
        if result != StatusCode::Ok {
            return result;
        }
        let major = stack.major_version();
        let minor = stack.minor_version();
        self.layout_scale_separate = major > 7 || (major == 7 && minor >= 2);
        self.base.import(stack)
    }

    pub fn asset_id(&self) -> u32 {
        self.base.asset_id()
    }

    pub fn set_asset(&mut self, asset: Option<CoreHandle>) {
        if let Some(asset) =
            asset.filter(|asset| asset.with_downcast::<ImageAsset, _>(|_| ()).is_some())
        {
            let this = self.base.handle().expect("live Image owner");
            self.file_asset_referencer.set_asset(this, Some(asset));
            if let Some(mesh) = self.mesh.clone() {
                let is_instance = self
                    .base
                    .artboard_handle()
                    .and_then(|artboard| {
                        artboard.with(|artboard| {
                            artboard
                                .as_artboard()
                                .map(|artboard| artboard.is_instance())
                        })
                    })
                    .flatten()
                    .expect("Image artboard");
                if !is_instance && let Some(render_image) = self.render_image() {
                    mesh.with_mut(|mesh| {
                        mesh.mesh_drawable_on_asset_loaded(render_image.as_ref());
                    });
                }
            }
            self.update_image_scale();
        }
    }

    pub fn asset_updated(&mut self) {
        self.update_image_scale();
        self.base.mark_world_transform_dirty();
    }

    pub fn clone_definition(&self) -> Self {
        let mut twin = Self::default();
        let mut base = std::mem::take(&mut twin.base);
        base.copy(&self.base, &mut twin);
        twin.base = base;
        twin.layout_scale_separate = self.layout_scale_separate;
        if let Some(asset) = self.file_asset_referencer.asset() {
            twin.file_asset_referencer.set_asset_unattached(Some(asset));
            twin.update_image_scale();
        }
        twin
    }

    pub fn set_mesh(&mut self, mesh: Option<CoreHandle>) {
        if self.mesh == mesh {
            return;
        }
        self.mesh = mesh;
        self.update_image_scale();
    }

    pub fn width(&self) -> f32 {
        self.image_asset()
            .and_then(|asset| {
                asset.with_downcast::<ImageAsset, _>(|asset| {
                    asset
                        .render_image()
                        .map(|image| image.width() as f32)
                        .unwrap_or_else(|| asset.base.width())
                })
            })
            .unwrap_or(0.0)
    }

    pub fn height(&self) -> f32 {
        self.image_asset()
            .and_then(|asset| {
                asset.with_downcast::<ImageAsset, _>(|asset| {
                    asset
                        .render_image()
                        .map(|image| image.height() as f32)
                        .unwrap_or_else(|| asset.base.height())
                })
            })
            .unwrap_or(0.0)
    }

    pub fn measure_layout(
        &self,
        width: f32,
        width_mode: LayoutMeasureMode,
        height: f32,
        height_mode: LayoutMeasureMode,
    ) -> Vec2D {
        Vec2D::new(
            if width_mode == LayoutMeasureMode::Exactly {
                width
            } else {
                self.width()
            },
            if height_mode == LayoutMeasureMode::Exactly {
                height
            } else {
                self.height()
            },
        )
    }

    pub fn control_size(
        &mut self,
        size: Vec2D,
        _width: LayoutScaleType,
        _height: LayoutScaleType,
        _direction: LayoutDirection,
    ) {
        if self.layout_width != size.x || self.layout_height != size.y {
            self.layout_width = size.x;
            self.layout_height = size.y;
            self.update_image_scale();
        }
    }

    pub(crate) fn try_compose_world_transform_override(&mut self) -> bool {
        let participant = self.base.children().iter().find_map(|child| {
            child
                .with(|child| {
                    child.as_any().downcast_ref::<crate::mechanical_port::source::layout::layout_participant::LayoutParticipant>().map(|participant| {
                        (participant.resolved_left(), participant.resolved_top())
                    })
                })
                .flatten()
        });
        let parent_world = self.base.parent_transform_component().and_then(|parent| {
            parent
                .with(|parent| {
                    parent
                        .as_world_transform_component()
                        .map(|parent| *parent.world_transform())
                })
                .flatten()
        });
        if let (Some((left, top)), Some(parent_world)) = (participant, parent_world) {
            let base = Mat2D::from_translation(Vec2D::new(left, top));
            self.base
                .set_world_transform(parent_world * base * *self.base.transform());
            return true;
        }
        false
    }

    pub fn layout_participant(&self) -> Option<CoreHandle> {
        self.base
            .children()
            .iter()
            .find(|child| {
                child
                    .with_downcast::<LayoutParticipant, _>(|_| ())
                    .is_some()
            })
            .cloned()
    }

    pub fn is_participating_in_layout(&self) -> bool {
        self.layout_participant().is_some()
    }

    pub fn render_scale_x(&self) -> f32 {
        self.base.scale_x() * self.layout_scale_x
    }
    pub fn render_scale_y(&self) -> f32 {
        self.base.scale_y() * self.layout_scale_y
    }
    pub fn computed_width(&self) -> f32 {
        self.width() * self.render_scale_x()
    }
    pub fn computed_height(&self) -> f32 {
        self.height() * self.render_scale_y()
    }

    pub(crate) fn update_transform_after_super(&mut self) {
        self.base
            .transform_mut()
            .scale_by_values(self.layout_scale_x, self.layout_scale_y);
        self.base.transform_mut()[4] += self.layout_offset_x;
        self.base.transform_mut()[5] += self.layout_offset_y;
    }

    fn update_image_scale(&mut self) {
        if self.image_asset().is_none() {
            if self.layout_offset_x != 0.0 || self.layout_offset_y != 0.0 {
                self.layout_offset_x = 0.0;
                self.layout_offset_y = 0.0;
                self.base.mark_transform_dirty();
            }
            return;
        }
        let mut new_offset_x = 0.0;
        let mut new_offset_y = 0.0;
        if let Some(render_image) = self
            .render_image()
            .filter(|_| !self.layout_width.is_nan() && !self.layout_height.is_nan())
        {
            let image_width = render_image.width() as f32;
            let image_height = render_image.height() as f32;
            let fit = ImageFit::from(self.base.fit());
            let (new_scale_x, new_scale_y) = match fit {
                ImageFit::Contain => {
                    let scale =
                        (self.layout_width / image_width).min(self.layout_height / image_height);
                    (scale, scale)
                }
                ImageFit::Cover => {
                    let scale =
                        (self.layout_width / image_width).max(self.layout_height / image_height);
                    (scale, scale)
                }
                ImageFit::FitWidth => {
                    let scale = self.layout_width / image_width;
                    (scale, scale)
                }
                ImageFit::FitHeight => {
                    let scale = self.layout_height / image_height;
                    (scale, scale)
                }
                ImageFit::None => (1.0, 1.0),
                ImageFit::ScaleDown => {
                    let scale = (self.layout_width / image_width)
                        .min(self.layout_height / image_height)
                        .min(1.0);
                    (scale, scale)
                }
                ImageFit::Fill | ImageFit::Resize => (
                    self.layout_width / image_width,
                    self.layout_height / image_height,
                ),
            };
            if fit != ImageFit::Resize || self.is_participating_in_layout() {
                let mut bounds_left = -image_width * self.base.origin_x();
                let mut bounds_top = -image_height * self.base.origin_y();
                if self.mesh.as_ref().is_some_and(|mesh| {
                    mesh.core_type() == Some(crate::mechanical_port::source::generated::shapes::mesh_base::MeshBase::TYPE_KEY)
                }) {
                    bounds_left = -image_width * 0.5;
                    bounds_top = -image_height * 0.5;
                }
                let alignment = Alignment::new(self.base.alignment_x(), self.base.alignment_y());
                let x_align = (alignment.x() + 1.0) * 0.5;
                let y_align = (alignment.y() + 1.0) * 0.5;
                new_offset_x = -(bounds_left * new_scale_x)
                    + (self.layout_width - image_width * new_scale_x) * x_align;
                new_offset_y = -(bounds_top * new_scale_y)
                    + (self.layout_height - image_height * new_scale_y) * y_align;
            }
            if self.layout_scale_separate {
                if new_scale_x != self.layout_scale_x || new_scale_y != self.layout_scale_y {
                    self.layout_scale_x = new_scale_x;
                    self.layout_scale_y = new_scale_y;
                    self.base.mark_transform_dirty();
                }
            } else if new_scale_x != self.base.scale_x() || new_scale_y != self.base.scale_y() {
                self.base.set_scale_x(new_scale_x);
                self.base.set_scale_y(new_scale_y);
            }
        }
        if new_offset_x != self.layout_offset_x || new_offset_y != self.layout_offset_y {
            self.layout_offset_x = new_offset_x;
            self.layout_offset_y = new_offset_y;
            self.base.mark_transform_dirty();
        }
    }

    pub fn local_bounds(&self) -> Aabb {
        if self.image_asset().is_none() {
            return Aabb::default();
        }
        Aabb::from_ltwh(
            -self.width() * self.base.origin_x(),
            -self.height() * self.base.origin_y(),
            self.width(),
            self.height(),
        )
    }

    pub fn image_asset(&self) -> Option<CoreHandle> {
        self.file_asset_referencer
            .asset()
            .filter(|asset| asset.with_downcast::<ImageAsset, _>(|_| ()).is_some())
    }
    pub fn render_image(&self) -> Option<crate::mechanical_port::source::renderer::RenderImageRef> {
        self.image_asset()?
            .with_downcast::<ImageAsset, _>(|asset| asset.render_image().cloned())
            .flatten()
    }
    pub fn mesh(&self) -> Option<CoreHandle> {
        self.mesh.clone()
    }
}
