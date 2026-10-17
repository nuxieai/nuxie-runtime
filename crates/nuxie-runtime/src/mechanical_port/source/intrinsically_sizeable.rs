use crate::mechanical_port::source::component::Component;
use crate::mechanical_port::source::joystick::Joystick;
use crate::mechanical_port::source::layout::layout_enums::{LayoutDirection, LayoutScaleType};
use crate::mechanical_port::source::layout::layout_measure_mode::LayoutMeasureMode;
use crate::mechanical_port::source::math::vec2d::Vec2D;
use crate::mechanical_port::source::transform_component::TransformComponent;

/// Authored dimensions survive controlSize writing a solved size into the
/// host. Every assignment outside layout control is authored, including one
/// equal to the size layout last wrote, so the hosts' generated value setters
/// record it before their equality check and report it as a change.
#[derive(Default, Clone, Copy)]
pub(crate) struct LayoutNaturalSize {
    width: Option<f32>,
    height: Option<f32>,
    controlling: bool,
}

impl LayoutNaturalSize {
    pub(crate) fn get(&self, current: Vec2D) -> Vec2D {
        Vec2D::new(
            self.width.unwrap_or(current.x),
            self.height.unwrap_or(current.y),
        )
    }
    pub(crate) fn begin_control(&mut self, current: Vec2D) {
        self.width.get_or_insert(current.x);
        self.height.get_or_insert(current.y);
        self.controlling = true;
    }
    pub(crate) fn end_control(&mut self) {
        self.controlling = false;
    }
    /// Whether layout is writing the solved size now.
    pub(crate) fn controlling(&self) -> bool {
        self.controlling
    }
    /// Records an authored width; true when that changes the natural width
    /// the host had with `current` as its value.
    pub(crate) fn assign_width(&mut self, width: f32, current: f32) -> bool {
        if self.controlling {
            return false;
        }
        let before = self.width.unwrap_or(current);
        self.width = Some(width);
        before != width
    }
    /// Records an authored height; true when that changes the natural height.
    pub(crate) fn assign_height(&mut self, height: f32, current: f32) -> bool {
        if self.controlling {
            return false;
        }
        let before = self.height.unwrap_or(current);
        self.height = Some(height);
        before != height
    }
    /// A copy carries the authored size but never an open control.
    pub(crate) fn copied(&self) -> Self {
        Self {
            controlling: false,
            ..*self
        }
    }
}

/// A visible authored AutoHeight or Fixed text box keeps its declared width
/// as its min-content width when no participant owns that width, like a CSS
/// box with a specified width. Width-owning text instead reflows in its slot.
/// Heights still come from Rive's own measurement at the chosen width.
pub(crate) fn authored_text_box_width(text: &crate::source::text::text::Text) -> Option<f32> {
    use crate::source::text_engine::{TextOverflow, TextSizing};
    if text.layout_owns_width() || text.overflow() != TextOverflow::Visible {
        return None;
    }
    match text.effective_sizing() {
        TextSizing::AutoHeight | TextSizing::Fixed => {
            Some(text.base.width()).filter(|width| width.is_finite() && *width >= 0.0)
        }
        _ => None,
    }
}

/// Authored bounds for non-reflowing drawing content. A drawing group is not
/// a CSS formatting context, so its selected content supplies the same floor
/// as a graphic directly hosted by layout. Unknown measured hosts stay unknown.
pub(crate) fn natural_size_for_layout_host(
    host: &crate::source::core::CoreHandle,
) -> Option<Vec2D> {
    natural_bounds_for_layout_host(host, None).map(|bounds| {
        bounds.map_or(Vec2D::default(), |bounds| {
            Vec2D::new(bounds.width(), bounds.height())
        })
    })
}

/// `transform` maps the host's local space into the measured box's space;
/// None is the measured host itself. Shapes bound their actual geometry, so a
/// rotated curve keeps its precise extent rather than a rotated box.
fn natural_bounds_for_layout_host(
    host: &crate::source::core::CoreHandle,
    transform: Option<crate::source::math::mat2d::Mat2D>,
) -> Option<Option<crate::source::math::aabb::Aabb>> {
    use crate::source::{
        math::{aabb::Aabb, mat2d::Mat2D},
        shapes::{image::Image, shape::Shape},
        solo::Solo,
    };
    enum Content {
        Placed(Aabb),
        Rect(Aabb),
        Children(Vec<crate::source::core::CoreHandle>),
        Unknown,
    }
    let content = host.with_mut(|object| {
        if object
            .as_component()
            .is_some_and(|component| component.is_collapsed())
        {
            return Content::Children(Vec::new());
        }
        if let Some(shape) = object.as_any().downcast_ref::<Shape>() {
            return Content::Placed(shape.natural_bounds_for_layout(transform));
        }
        // A host's rectangle sits about its origin, as it draws, so a group
        // places it where it is.
        if let Some(image) = object.as_any().downcast_ref::<Image>() {
            let (width, height) = (image.width(), image.height());
            return Content::Rect(Aabb::from_ltwh(
                -width * image.base.origin_x(),
                -height * image.base.origin_y(),
                width,
                height,
            ));
        }
        if let Some(bounds) = object
            .as_intrinsically_sizeable_mut()
            .and_then(|host| host.natural_bounds_for_layout())
        {
            return Content::Rect(bounds);
        }
        // Ordinary drawing nodes inherit TransformComponent's zero measurer.
        // Concrete measured hosts need their own contract instead of a walk.
        if object.as_text_mut().is_some()
            || object.as_text_input_mut().is_some()
            || object.as_scripted_layout().is_some()
            || object.as_nested_artboard().is_some()
            || object.as_layout_component().is_some()
        {
            return Content::Unknown;
        }
        if let Some(solo) = object.as_any().downcast_ref::<Solo>() {
            return Content::Children(solo.active_component().into_iter().collect());
        }
        Content::Children(
            object
                .as_container_component()
                .map(|container| container.children().to_vec())
                .or_else(|| object.as_node().map(|node| node.children().to_vec()))
                .unwrap_or_default(),
        )
    })?;
    match content {
        Content::Placed(bounds) => Some(Some(bounds)),
        // A transformed rectangle's bounds are its mapped corners exactly.
        Content::Rect(bounds) => {
            Some(Some(transform.map_or(bounds, |transform| {
                transform.map_bounding_box(bounds)
            })))
        }
        Content::Unknown => None,
        Content::Children(children) => {
            let outer = transform.unwrap_or_else(Mat2D::identity);
            let mut result: Option<Aabb> = None;
            for child in children {
                let local = child.with(|object| {
                    object
                        .as_transform_component()
                        .map_or_else(Mat2D::identity, |transform| transform.transform())
                })?;
                if let Some(bounds) = natural_bounds_for_layout_host(&child, Some(outer * local))? {
                    if let Some(result) = &mut result {
                        result.expand(bounds);
                    } else {
                        result = Some(bounds);
                    }
                }
            }
            Some(result)
        }
    }
}

pub trait IntrinsicallySizeable {
    /// CSS min-content of a non-reflowing host, independent of its solved size.
    fn natural_size_for_layout(&self) -> Option<Vec2D> {
        None
    }
    /// That natural size placed in the host's local space, as it draws.
    fn natural_bounds_for_layout(&self) -> Option<crate::source::math::aabb::Aabb> {
        self.natural_size_for_layout()
            .map(|size| crate::source::math::aabb::Aabb::from_ltwh(0.0, 0.0, size.x, size.y))
    }
    fn measure_layout(
        &mut self,
        _width: f32,
        _width_mode: LayoutMeasureMode,
        _height: f32,
        _height_mode: LayoutMeasureMode,
    ) -> Vec2D {
        Vec2D::default()
    }

    fn control_size(
        &mut self,
        _size: Vec2D,
        _width_scale_type: LayoutScaleType,
        _height_scale_type: LayoutScaleType,
        _direction: LayoutDirection,
    ) {
    }

    fn should_propagate_size_to_children(&self) -> bool {
        true
    }
}

pub fn from(
    component: &mut dyn crate::mechanical_port::source::core::CoreObject,
) -> Option<&mut dyn IntrinsicallySizeable> {
    component.as_intrinsically_sizeable_mut()
}

/// Dispatch controlSize without holding a host borrow across a child callback.
pub fn control_size_handle(
    owner: &crate::mechanical_port::source::core::CoreHandle,
    size: Vec2D,
    width: LayoutScaleType,
    height: LayoutScaleType,
    direction: LayoutDirection,
) -> bool {
    if owner.is_type_of(
        crate::mechanical_port::source::generated::shapes::shape_base::ShapeBase::TYPE_KEY,
    ) {
        crate::mechanical_port::source::shapes::shape::Shape::control_size_occurrence(
            owner, size, width, height, direction,
        );
        return true;
    }
    owner
        .with_mut(|owner| {
            let Some(sizeable) = owner.as_intrinsically_sizeable_mut() else {
                return false;
            };
            sizeable.control_size(size, width, height, direction);
            true
        })
        .unwrap_or(false)
}

macro_rules! intrinsic_owner {
    ($owner:path $(, $natural:ident)?) => {
        impl IntrinsicallySizeable for $owner {
            $(
            fn natural_size_for_layout(&self) -> Option<Vec2D> {
                Some(<$owner>::$natural(self))
            }
            )?
            fn measure_layout(
                &mut self,
                width: f32,
                width_mode: LayoutMeasureMode,
                height: f32,
                height_mode: LayoutMeasureMode,
            ) -> Vec2D {
                <$owner>::measure_layout(self, width, width_mode, height, height_mode)
            }
            fn control_size(
                &mut self,
                size: Vec2D,
                width: LayoutScaleType,
                height: LayoutScaleType,
                direction: LayoutDirection,
            ) {
                <$owner>::control_size(self, size, width, height, direction)
            }
        }
    };
}
intrinsic_owner!(
    crate::mechanical_port::source::shapes::shape::Shape,
    natural_size_for_layout
);
intrinsic_owner!(crate::mechanical_port::source::shapes::image::Image);
intrinsic_owner!(
    crate::mechanical_port::source::shapes::parametric_path::ParametricPath,
    natural_size_for_layout
);
intrinsic_owner!(crate::mechanical_port::source::text::text::Text);
intrinsic_owner!(crate::mechanical_port::source::text::text_input::TextInput);
impl IntrinsicallySizeable for crate::source::nested_artboard::NestedArtboard {
    fn natural_size_for_layout(&self) -> Option<Vec2D> {
        Self::natural_size_for_layout(self)
    }
    fn natural_bounds_for_layout(&self) -> Option<crate::source::math::aabb::Aabb> {
        Self::natural_bounds_for_layout(self)
    }
    fn measure_layout(
        &mut self,
        width: f32,
        width_mode: LayoutMeasureMode,
        height: f32,
        height_mode: LayoutMeasureMode,
    ) -> Vec2D {
        Self::measure_layout(self, width, width_mode, height, height_mode)
    }
}
intrinsic_owner!(crate::mechanical_port::source::scripted::scripted_layout::ScriptedLayout);
impl IntrinsicallySizeable for crate::mechanical_port::source::layout_component::LayoutComponent {
    fn measure_layout(
        &mut self,
        width: f32,
        width_mode: LayoutMeasureMode,
        height: f32,
        height_mode: LayoutMeasureMode,
    ) -> Vec2D {
        Self::measure_layout(self, width, width_mode, height, height_mode)
    }
}
impl IntrinsicallySizeable for crate::mechanical_port::source::layout::n_sliced_node::NSlicedNode {
    fn natural_size_for_layout(&self) -> Option<Vec2D> {
        Some(Self::natural_size_for_layout(self))
    }
    fn measure_layout(
        &mut self,
        width: f32,
        width_mode: LayoutMeasureMode,
        height: f32,
        height_mode: LayoutMeasureMode,
    ) -> Vec2D {
        Self::measure_layout(self, width, width_mode, height, height_mode)
    }
    fn control_size(
        &mut self,
        size: Vec2D,
        width: LayoutScaleType,
        height: LayoutScaleType,
        direction: LayoutDirection,
    ) {
        Self::control_size(self, size, width, height, direction);
    }
    fn should_propagate_size_to_children(&self) -> bool {
        false
    }
}
