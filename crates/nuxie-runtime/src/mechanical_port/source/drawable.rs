use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    core::CoreHandle,
    core_context::CoreContext,
    draw_rules::DrawRules,
    drawable_flag::DrawableFlag,
    generated::drawable_base::{DrawableBase, DrawableBaseCallbacks},
    hit_info::HitInfo,
    layout_component::LayoutComponent,
    math::{mat2d::Mat2D, vec2d::Vec2D},
    renderer::Renderer,
    shapes::{clipping_shape::ClippingShape, paint::blend_mode::BlendMode},
    status_code::StatusCode,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundsFidelity {
    None,
    Approximate,
    Exact,
}

pub struct Drawable {
    pub base: DrawableBase,
    clipping_shapes: Vec<CoreHandle>,
    layer_masks: Vec<CoreHandle>,
    source_of_layer_masks: Vec<CoreHandle>,
    pub(crate) flattened_draw_rules: Option<CoreHandle>,
    pub(crate) prev: Option<RuntimeDrawableWeakOccurrence>,
    pub(crate) next: Option<RuntimeDrawableWeakOccurrence>,
    runtime_flags: u8,
}

impl Default for Drawable {
    fn default() -> Self {
        Self {
            base: DrawableBase::default(),
            clipping_shapes: Vec::new(),
            layer_masks: Vec::new(),
            source_of_layer_masks: Vec::new(),
            flattened_draw_rules: None,
            prev: None,
            next: None,
            runtime_flags: 1,
        }
    }
}

impl crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
    for Drawable
{
    fn notify_property_changed(&mut self, property_key: u16) {
        crate::mechanical_port::source::core::Core::notify_property_changed(
            &mut self.base,
            property_key,
        );
    }
}

impl crate::mechanical_port::source::generated::world_transform_component_base::WorldTransformComponentBaseCallbacks for Drawable {
    fn notify_property_changed(&mut self, property_key: u16) {
        crate::mechanical_port::source::core::Core::notify_property_changed(&mut self.base, property_key);
    }
    fn opacity_changed(&mut self) {
        self.base.base.base.base.base.base.opacity_changed();
    }
}

impl crate::mechanical_port::source::generated::transform_component_base::TransformComponentBaseCallbacks for Drawable {
    fn notify_property_changed(&mut self, property_key: u16) {
        crate::mechanical_port::source::core::Core::notify_property_changed(&mut self.base, property_key);
    }
    fn rotation_changed(&mut self) {
        self.base.base.base.base.rotation_changed();
    }
    fn scale_x_changed(&mut self) {
        self.base.base.base.base.scale_x_changed();
    }
    fn scale_y_changed(&mut self) {
        self.base.base.base.base.scale_y_changed();
    }
}

impl crate::mechanical_port::source::generated::node_base::NodeBaseCallbacks for Drawable {
    fn notify_property_changed(&mut self, property_key: u16) {
        crate::mechanical_port::source::core::Core::notify_property_changed(
            &mut self.base,
            property_key,
        );
    }
    fn x_changed(&mut self) {
        self.base.base.x_changed();
    }
    fn y_changed(&mut self) {
        self.base.base.y_changed();
    }
    fn set_computed_local_x(&mut self, value: f32) {
        self.base.base.set_computed_local_x(value);
    }
    fn computed_local_x(&mut self) -> f32 {
        self.base.base.computed_local_x()
    }
    fn set_computed_local_y(&mut self, value: f32) {
        self.base.base.set_computed_local_y(value);
    }
    fn computed_local_y(&mut self) -> f32 {
        self.base.base.computed_local_y()
    }
    fn set_computed_world_x(&mut self, value: f32) {
        self.base.base.set_computed_world_x(value);
    }
    fn computed_world_x(&mut self) -> f32 {
        self.base.base.computed_world_x()
    }
    fn set_computed_world_y(&mut self, value: f32) {
        self.base.base.set_computed_world_y(value);
    }
    fn computed_world_y(&mut self) -> f32 {
        self.base.base.computed_world_y()
    }
    fn set_computed_root_x(&mut self, value: f32) {
        self.base.base.set_computed_root_x(value);
    }
    fn computed_root_x(&mut self) -> f32 {
        self.base.base.computed_root_x()
    }
    fn set_computed_root_y(&mut self, value: f32) {
        self.base.base.set_computed_root_y(value);
    }
    fn computed_root_y(&mut self) -> f32 {
        self.base.base.computed_root_y()
    }
    fn set_computed_width(&mut self, value: f32) {
        self.base.base.set_computed_width(value);
    }
    fn computed_width(&mut self) -> f32 {
        self.base.base.computed_width()
    }
    fn set_computed_height(&mut self, value: f32) {
        self.base.base.set_computed_height(value);
    }
    fn computed_height(&mut self) -> f32 {
        self.base.base.computed_height()
    }
}

impl DrawableBaseCallbacks for Drawable {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .base
            .base
            .base
            .base
            .base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }
}

impl Drawable {
    pub fn blend_mode(&self) -> BlendMode {
        match self.base.blend_mode_value() as u8 {
            3 => BlendMode::SrcOver,
            12 => BlendMode::Additive,
            14 => BlendMode::Screen,
            15 => BlendMode::Overlay,
            16 => BlendMode::Darken,
            17 => BlendMode::Lighten,
            18 => BlendMode::ColorDodge,
            19 => BlendMode::ColorBurn,
            20 => BlendMode::HardLight,
            21 => BlendMode::SoftLight,
            22 => BlendMode::Difference,
            23 => BlendMode::Exclusion,
            24 => BlendMode::Multiply,
            25 => BlendMode::Hue,
            26 => BlendMode::Saturation,
            27 => BlendMode::Color,
            28 => BlendMode::Luminosity,
            value => panic!("invalid blend mode {value}"),
        }
    }

    pub fn additiveness(&self) -> f32 {
        crate::mechanical_port::source::shapes::paint::blend_mode::additiveness_for(
            self.blend_mode(),
            self.base.additive_amount(),
        )
    }
    pub fn draw(&mut self, _renderer: &mut Renderer) {
        panic!("abstract Drawable::draw");
    }

    pub fn hit_test(&mut self, _info: &mut HitInfo, _transform: &Mat2D) -> Option<CoreHandle> {
        panic!("abstract Drawable::hit_test");
    }

    pub fn hit_test_point(
        &mut self,
        position: &Vec2D,
        skip_on_unclipped: bool,
        is_primary_hit: bool,
    ) -> bool {
        if self.is_hidden() {
            return false;
        }
        let this = self.base.base.base.base.base.handle();
        if let Some(hittable) = self.hittable_component()
            && this.as_ref() != Some(&hittable)
        {
            return hittable
                .with_mut(|hittable| {
                    hittable
                        .component_hit_test_point(position, skip_on_unclipped, is_primary_hit)
                        .unwrap_or(false)
                })
                .unwrap_or(false);
        }
        self.base
            .base
            .base
            .base
            .base
            .hit_test_point(position, skip_on_unclipped, is_primary_hit)
    }

    pub fn add_clipping_shape(&mut self, shape: CoreHandle) {
        self.clipping_shapes.push(shape);
    }

    pub fn clipping_shapes(&self) -> &[CoreHandle] {
        &self.clipping_shapes
    }

    pub fn add_layer_mask(&mut self, mask: CoreHandle) {
        self.layer_masks.push(mask);
    }
    pub fn add_source_of_layer_mask(&mut self, mask: CoreHandle) {
        self.source_of_layer_masks.push(mask);
    }
    pub fn layer_masks(&self) -> &[CoreHandle] {
        &self.layer_masks
    }
    pub fn source_of_layer_masks(&self) -> &[CoreHandle] {
        &self.source_of_layer_masks
    }
    pub fn painted_bounds_from_local(
        local: &crate::mechanical_port::source::math::aabb::Aabb,
        world: &Mat2D,
        paints: Option<
            &crate::mechanical_port::source::shapes::shape_paint_container::ShapePaintContainer,
        >,
        out: &mut crate::mechanical_port::source::math::aabb::Aabb,
    ) -> BoundsFidelity {
        use crate::mechanical_port::source::math::aabb::Aabb;
        if local.is_empty_or_nan() {
            return BoundsFidelity::None;
        }
        let mut bounds = world.map_bounding_box(*local);
        if bounds.is_empty_or_nan() {
            *out = Aabb::default();
            return BoundsFidelity::Exact;
        }
        let mut trustworthy = true;
        if paints.is_some() {
            let reach = crate::mechanical_port::source::shapes::paint::paint_outset::shape_paints_world_reach(paints, world);
            if reach.world_outset > 0.0 {
                bounds = bounds.outset(reach.world_outset, reach.world_outset);
            }
            trustworthy = reach.trustworthy;
        }
        *out = bounds;
        if trustworthy {
            BoundsFidelity::Exact
        } else {
            BoundsFidelity::Approximate
        }
    }

    pub fn is_selectable(&self) -> bool {
        self.base.drawable_flags() as u16 & DrawableFlag::SELECTABLE.0 != 0
    }
    pub fn is_hidden(&self) -> bool {
        self.base.drawable_flags() as u16 & DrawableFlag::HIDDEN.0 == DrawableFlag::HIDDEN.0
            || self
                .base
                .base
                .base
                .base
                .base
                .has_dirt(ComponentDirt::COLLAPSED)
    }

    pub fn is_target_opaque(&self) -> bool {
        self.base.drawable_flags() as u16 & DrawableFlag::OPAQUE.0 == DrawableFlag::OPAQUE.0
    }

    pub fn is_proxy(&self) -> bool {
        false
    }
    pub fn is_clip_start(&self) -> bool {
        false
    }
    pub fn is_clip_end(&self) -> bool {
        false
    }
    pub fn will_clip(&self) -> bool {
        false
    }
    pub fn will_draw(&self) -> bool {
        !self.is_hidden()
    }
    pub fn set_needs_save_operation(&mut self, value: bool) {
        self.runtime_flags = if value {
            self.runtime_flags | 1
        } else {
            self.runtime_flags & !1
        };
    }
    pub fn needs_save_operation(&self) -> bool {
        self.runtime_flags & 1 != 0
    }

    pub fn has_custom_properties(&self) -> bool {
        self.runtime_flags & 2 != 0
    }
    pub fn mark_has_custom_properties(&mut self) {
        self.runtime_flags |= 2;
    }

    pub fn custom_property(&self, name_id: u32) -> Option<CoreHandle> {
        use crate::mechanical_port::source::{
            custom_property::CustomProperty,
            generated::{core_registry::CoreRegistry, custom_property_base::CustomPropertyBase},
        };
        self.children().iter().find_map(|child| {
            let property = CustomProperty::tagging(child)?;
            (CoreRegistry::get_uint_handle(
                &property,
                CustomPropertyBase::NAME_ID_PROPERTY_KEY.into(),
            ) == Some(name_id))
            .then_some(property)
        })
    }

    pub fn custom_property_handle(owner: &CoreHandle, name_id: u32) -> Option<CoreHandle> {
        owner
            .with(|o| o.as_drawable().and_then(|d| d.custom_property(name_id)))
            .flatten()
    }

    #[cfg(feature = "tools")]
    pub fn tagging_properties_handle(owner: &CoreHandle) -> Vec<CoreHandle> {
        owner.with(|o| o.as_drawable().map(|d| d.children().iter().filter_map(crate::mechanical_port::source::custom_property::CustomProperty::tagging).collect())).flatten().unwrap_or_default()
    }

    pub fn draw_handle(owner: &CoreHandle, renderer: &mut Renderer) {
        crate::mechanical_port::source::generated::core_registry::drawable_draw_handle(
            owner, renderer,
        );
    }

    pub fn is_child_of_layout(&self, layout: &CoreHandle) -> bool {
        let mut current = self.base.base.base.base.base.handle();
        while let Some(component) = current {
            if &component == layout {
                return true;
            }
            current = component
                .with(|current| current.component_parent_handle())
                .flatten();
        }
        false
    }

    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.base.base.base.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        // C++ casts to the unsigned-char enum before checking its supported
        // cases. Validate that byte before constructing a Rust enum, so a
        // malformed drawable follows InvalidObject instead of panicking.
        match self.base.blend_mode_value() as u8 {
            3 | 12 | 14..=28 => StatusCode::Ok,
            _ => StatusCode::InvalidObject,
        }
    }

    pub fn hittable_component(&self) -> Option<CoreHandle> {
        self.base.base.base.base.base.handle()
    }

    pub fn empty_clip_count(&self) -> i32 {
        0
    }

    pub fn next_drawable(&self) -> Option<RuntimeDrawableOccurrence> {
        self.next
            .as_ref()
            .and_then(RuntimeDrawableWeakOccurrence::upgrade)
    }
    pub fn prev_drawable(&self) -> Option<RuntimeDrawableOccurrence> {
        self.prev
            .as_ref()
            .and_then(RuntimeDrawableWeakOccurrence::upgrade)
    }

    /// Read the fresh range link; the next range iteration validates authored
    /// identity before invoking any receiver operation. Other callers still
    /// use prev_drawable when they need an immediately live occurrence.
    pub(crate) fn prev_drawable_for_draw_range(&self) -> Option<RuntimeDrawableOccurrence> {
        match self.prev.as_ref()? {
            RuntimeDrawableWeakOccurrence::Authored(handle) => {
                Some(RuntimeDrawableOccurrence::Authored(handle.clone()))
            }
            previous @ RuntimeDrawableWeakOccurrence::RuntimeProxy(_) => previous.upgrade(),
        }
    }
}

pub trait ProxyDrawing {
    fn layer_mask_marker(
        &self,
    ) -> Option<Rc<RefCell<crate::mechanical_port::source::layer_mask::LayerMaskProxyDrawable>>>
    {
        None
    }
    fn draw_proxy(&mut self, renderer: &mut Renderer, needs_save_operation: bool);
    fn is_proxy_hidden(&self) -> bool;
    fn owner_handle(&self) -> CoreHandle;
    fn hittable_component(&self) -> Option<CoreHandle> {
        Some(self.owner_handle())
    }
    fn empty_clip_count(&mut self) -> i32 {
        0
    }
    fn is_clip_start(&self) -> bool {
        false
    }
    fn is_clip_end(&self) -> bool {
        false
    }
    fn will_clip(&self) -> bool {
        false
    }
}

#[derive(Clone)]
pub enum RuntimeDrawableOccurrence {
    Authored(CoreHandle),
    RuntimeProxy(Rc<RefCell<DrawableProxy>>),
}

#[derive(Clone)]
pub enum RuntimeDrawableWeakOccurrence {
    Authored(CoreHandle),
    RuntimeProxy(Weak<RefCell<DrawableProxy>>),
}

pub(crate) enum DrawRangePrelude {
    Skip,
    ClipStart,
    ClipEnd,
    Draw,
}

impl RuntimeDrawableOccurrence {
    /// The callback-free shared-receiver part of Artboard::drawDrawableRange.
    /// Keep emptyClipCount's mutable access separate, and end this access before
    /// pending clips or the drawable call the renderer. The virtual predicates
    /// retain their source order and short circuits.
    pub(crate) fn draw_range_prelude(
        &self,
        suppressed_by_empty_clip: bool,
        has_pending_clips: bool,
    ) -> DrawRangePrelude {
        match self {
            Self::Authored(handle) => handle
                .with(|object| {
                    if !object.drawable_will_draw() || suppressed_by_empty_clip {
                        DrawRangePrelude::Skip
                    } else if object.drawable_is_clip_start() {
                        DrawRangePrelude::ClipStart
                    } else if has_pending_clips && object.drawable_is_clip_end() {
                        DrawRangePrelude::ClipEnd
                    } else {
                        DrawRangePrelude::Draw
                    }
                })
                .unwrap_or(DrawRangePrelude::Skip),
            // ProxyDrawing is an open host boundary. Preserve its individual
            // virtual getter borrows rather than widening the proxy's loan.
            Self::RuntimeProxy(_) => {
                if !self.will_draw() || suppressed_by_empty_clip {
                    DrawRangePrelude::Skip
                } else if self.is_clip_start() {
                    DrawRangePrelude::ClipStart
                } else if has_pending_clips && self.is_clip_end() {
                    DrawRangePrelude::ClipEnd
                } else {
                    DrawRangePrelude::Draw
                }
            }
        }
    }

    pub fn layer_mask_marker(
        &self,
    ) -> Option<Rc<RefCell<crate::mechanical_port::source::layer_mask::LayerMaskProxyDrawable>>>
    {
        self.with_proxy(|proxy| proxy.proxy_drawing.layer_mask_marker())
            .flatten()
    }
    pub fn is_mask_start(&self) -> bool {
        self.layer_mask_marker().is_some_and(|m| {
            m.borrow().op == crate::mechanical_port::source::layer_mask::LayerMaskOp::MaskStart
        })
    }
    pub fn is_mask_end(&self) -> bool {
        self.layer_mask_marker().is_some_and(|m| {
            m.borrow().op == crate::mechanical_port::source::layer_mask::LayerMaskOp::MaskEnd
        })
    }
    pub fn layer_masks(&self) -> Vec<CoreHandle> {
        if self.layer_mask_marker().is_some() {
            return Vec::new();
        }
        match self {
            Self::Authored(h) => h
                .with(|o| o.as_drawable().map(|d| d.layer_masks().to_vec()))
                .flatten()
                .unwrap_or_default(),
            Self::RuntimeProxy(p) => p
                .borrow()
                .hittable_component()
                .and_then(|h| h.with(|o| o.as_drawable().map(|d| d.layer_masks().to_vec())))
                .flatten()
                .unwrap_or_default(),
        }
    }
    pub fn source_of_layer_masks(&self) -> Vec<CoreHandle> {
        if self.layer_mask_marker().is_some() {
            return Vec::new();
        }
        match self {
            Self::Authored(h) => h
                .with(|o| o.as_drawable().map(|d| d.source_of_layer_masks().to_vec()))
                .flatten()
                .unwrap_or_default(),
            Self::RuntimeProxy(p) => p
                .borrow()
                .hittable_component()
                .and_then(|h| {
                    h.with(|o| o.as_drawable().map(|d| d.source_of_layer_masks().to_vec()))
                })
                .flatten()
                .unwrap_or_default(),
        }
    }
    pub fn painted_world_bounds(
        &self,
        out: &mut crate::mechanical_port::source::math::aabb::Aabb,
    ) -> BoundsFidelity {
        if self.layer_mask_marker().is_some() {
            return BoundsFidelity::None;
        }
        let target = match self {
            Self::Authored(h) => Some(h.clone()),
            Self::RuntimeProxy(p) => p.borrow().hittable_component(),
        };
        target
            .and_then(|h| h.with_mut(|o| o.painted_world_bounds(out)))
            .unwrap_or(BoundsFidelity::None)
    }
    pub fn with_component<R>(
        &self,
        use_component: impl FnOnce(&crate::mechanical_port::source::component::Component) -> R,
    ) -> Option<R> {
        match self {
            Self::Authored(handle) => {
                handle.with(|object| object.as_component().map(use_component))?
            }
            Self::RuntimeProxy(proxy) => Some(use_component(&proxy.borrow().base)),
        }
    }

    pub fn with_component_mut<R>(
        &self,
        use_component: impl FnOnce(&mut crate::mechanical_port::source::component::Component) -> R,
    ) -> Option<R> {
        match self {
            Self::Authored(handle) => {
                handle.with_mut(|object| object.as_component_mut().map(use_component))?
            }
            Self::RuntimeProxy(proxy) => Some(use_component(&mut proxy.borrow_mut().base)),
        }
    }

    pub fn hit_test_point(
        &self,
        position: &Vec2D,
        skip_on_unclipped: bool,
        is_primary_hit: bool,
    ) -> bool {
        match self {
            Self::Authored(handle) => handle
                .with_mut(|object| {
                    object.component_hit_test_point(position, skip_on_unclipped, is_primary_hit)
                })
                .flatten()
                .unwrap_or(false),
            Self::RuntimeProxy(proxy) => {
                if proxy.borrow().is_hidden() {
                    return false;
                }
                let owner = proxy.borrow().hittable_component();
                if let Some(owner) = owner {
                    owner
                        .with_mut(|owner| {
                            owner.component_hit_test_point(
                                position,
                                skip_on_unclipped,
                                is_primary_hit,
                            )
                        })
                        .flatten()
                        .unwrap_or(false)
                } else {
                    // Clip and mask markers have no hittable target. As in
                    // Drawable::hitTestPoint, fall through to their own Component.
                    self.with_component(|component| {
                        component.hit_test_point(position, skip_on_unclipped, is_primary_hit)
                    })
                    .unwrap_or(false)
                }
            }
        }
    }

    pub fn is_target_opaque(&self) -> bool {
        if self.layer_mask_marker().is_some() {
            return false;
        }
        match self {
            Self::Authored(handle) => handle
                .with(|object| object.as_drawable().map(Drawable::is_target_opaque))
                .flatten()
                .unwrap_or(false),
            Self::RuntimeProxy(proxy) => proxy.borrow_mut().is_target_opaque(),
        }
    }

    pub fn authored(handle: CoreHandle) -> Self {
        Self::Authored(handle)
    }

    pub fn runtime_proxy(proxy: Rc<RefCell<DrawableProxy>>) -> Self {
        Self::RuntimeProxy(proxy)
    }

    pub fn authored_handle(&self) -> Option<CoreHandle> {
        match self {
            Self::Authored(handle) => Some(handle.clone()),
            Self::RuntimeProxy(_) => None,
        }
    }

    pub fn is_hidden(&self) -> bool {
        match self {
            Self::Authored(handle) => handle
                .with(|object| object.drawable_is_hidden())
                .unwrap_or(true),
            Self::RuntimeProxy(proxy) => proxy.borrow().is_hidden(),
        }
    }

    pub fn will_draw(&self) -> bool {
        match self {
            Self::Authored(handle) => handle
                .with(|object| object.drawable_will_draw())
                .unwrap_or(false),
            Self::RuntimeProxy(proxy) => !proxy.borrow().is_hidden(),
        }
    }

    pub fn draw(&self, renderer: &mut Renderer) -> bool {
        match self {
            Self::Authored(handle) => {
                crate::mechanical_port::source::generated::core_registry::drawable_draw_handle(
                    handle, renderer,
                )
            }
            Self::RuntimeProxy(proxy) => {
                proxy.borrow_mut().draw(renderer);
                true
            }
        }
    }

    pub fn add_to_render_path(
        &self,
        path: &mut crate::mechanical_port::source::renderer::RenderPath,
        transform: &Mat2D,
    ) -> bool {
        match self {
            Self::Authored(handle) => handle
                .with_mut(|object| object.drawable_add_to_render_path(path, transform))
                .unwrap_or(false),
            Self::RuntimeProxy(_) => false,
        }
    }

    pub fn add_to_raw_path(
        &self,
        path: &mut crate::mechanical_port::source::math::raw_path::RawPath,
        transform: Option<&Mat2D>,
    ) -> bool {
        match self {
            Self::Authored(handle) => handle
                .with_mut(|object| object.drawable_add_to_raw_path(path, transform))
                .unwrap_or(false),
            Self::RuntimeProxy(_) => false,
        }
    }

    pub fn hit_test(&self, info: &mut HitInfo, transform: &Mat2D) -> Option<CoreHandle> {
        match self {
            Self::Authored(handle) => handle
                .with_mut(|object| object.drawable_hit_test(info, transform))
                .flatten(),
            Self::RuntimeProxy(_) => None,
        }
    }

    pub fn downgrade(&self) -> RuntimeDrawableWeakOccurrence {
        match self {
            Self::Authored(handle) => RuntimeDrawableWeakOccurrence::Authored(handle.clone()),
            Self::RuntimeProxy(proxy) => {
                RuntimeDrawableWeakOccurrence::RuntimeProxy(Rc::downgrade(proxy))
            }
        }
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Authored(a), Self::Authored(b)) => a == b,
            (Self::RuntimeProxy(a), Self::RuntimeProxy(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }

    pub fn with<R>(&self, use_drawable: impl FnOnce(&Drawable) -> R) -> Option<R> {
        match self {
            Self::Authored(handle) => {
                handle.with(|object| object.as_drawable().map(use_drawable))?
            }
            Self::RuntimeProxy(proxy) => Some(use_drawable(&proxy.borrow().base)),
        }
    }

    pub fn with_mut<R>(&self, use_drawable: impl FnOnce(&mut Drawable) -> R) -> Option<R> {
        match self {
            Self::Authored(handle) => {
                handle.with_mut(|object| object.as_drawable_mut().map(use_drawable))?
            }
            Self::RuntimeProxy(proxy) => Some(use_drawable(&mut proxy.borrow_mut().base)),
        }
    }

    pub fn with_proxy<R>(&self, use_proxy: impl FnOnce(&DrawableProxy) -> R) -> Option<R> {
        match self {
            Self::RuntimeProxy(proxy) => Some(use_proxy(&proxy.borrow())),
            Self::Authored(_) => None,
        }
    }

    pub fn with_proxy_mut<R>(&self, use_proxy: impl FnOnce(&mut DrawableProxy) -> R) -> Option<R> {
        match self {
            Self::RuntimeProxy(proxy) => Some(use_proxy(&mut proxy.borrow_mut())),
            Self::Authored(_) => None,
        }
    }

    pub fn is_clip_start(&self) -> bool {
        match self {
            Self::Authored(handle) => handle
                .with(|object| object.drawable_is_clip_start())
                .unwrap_or(false),
            Self::RuntimeProxy(proxy) => proxy.borrow().is_clip_start(),
        }
    }

    pub fn is_clip_end(&self) -> bool {
        match self {
            Self::Authored(handle) => handle
                .with(|object| object.drawable_is_clip_end())
                .unwrap_or(false),
            Self::RuntimeProxy(proxy) => proxy.borrow().is_clip_end(),
        }
    }

    pub fn will_clip(&self) -> bool {
        match self {
            Self::Authored(handle) => handle
                .with(|object| object.drawable_will_clip())
                .unwrap_or(false),
            Self::RuntimeProxy(proxy) => proxy.borrow().will_clip(),
        }
    }

    pub fn empty_clip_count(&self) -> i32 {
        match self {
            Self::Authored(handle) => handle
                .with_mut(|object| object.drawable_empty_clip_count())
                .unwrap_or_default(),
            Self::RuntimeProxy(proxy) => proxy.borrow_mut().empty_clip_count(),
        }
    }
}

impl PartialEq for RuntimeDrawableOccurrence {
    fn eq(&self, other: &Self) -> bool {
        self.ptr_eq(other)
    }
}
impl Eq for RuntimeDrawableOccurrence {}
impl std::hash::Hash for RuntimeDrawableOccurrence {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use std::hash::Hash;
        match self {
            Self::Authored(handle) => {
                0u8.hash(state);
                handle.hash(state);
            }
            Self::RuntimeProxy(proxy) => {
                1u8.hash(state);
                Rc::as_ptr(proxy).hash(state);
            }
        }
    }
}

impl RuntimeDrawableWeakOccurrence {
    pub fn upgrade(&self) -> Option<RuntimeDrawableOccurrence> {
        match self {
            Self::Authored(handle) => handle
                .is_alive()
                .then(|| RuntimeDrawableOccurrence::Authored(handle.clone())),
            Self::RuntimeProxy(proxy) => {
                proxy.upgrade().map(RuntimeDrawableOccurrence::RuntimeProxy)
            }
        }
    }
}

pub struct DrawableProxy {
    pub base: Drawable,
    proxy_drawing: Box<dyn ProxyDrawing>,
}

impl DrawableProxy {
    pub fn new(proxy_drawing: Box<dyn ProxyDrawing>) -> Self {
        Self {
            base: Drawable::default(),
            proxy_drawing,
        }
    }

    pub fn draw(&mut self, renderer: &mut Renderer) {
        self.proxy_drawing
            .draw_proxy(renderer, self.base.needs_save_operation());
    }
    pub fn is_hidden(&self) -> bool {
        self.proxy_drawing.is_proxy_hidden()
    }
    pub fn hittable_component(&self) -> Option<CoreHandle> {
        self.proxy_drawing.hittable_component()
    }
    pub fn is_target_opaque(&mut self) -> bool {
        self.hittable_component()
            .and_then(|h| {
                h.with(|hittable| {
                    hittable
                        .as_drawable()
                        .is_some_and(Drawable::is_target_opaque)
                })
            })
            .unwrap_or(false)
    }
    pub fn hit_test(&mut self, _info: &mut HitInfo, _transform: &Mat2D) -> Option<CoreHandle> {
        None
    }
    pub fn is_proxy(&self) -> bool {
        true
    }
    pub fn empty_clip_count(&mut self) -> i32 {
        self.proxy_drawing.empty_clip_count()
    }
    pub fn is_clip_start(&self) -> bool {
        self.proxy_drawing.is_clip_start()
    }
    pub fn is_clip_end(&self) -> bool {
        self.proxy_drawing.is_clip_end()
    }
    pub fn will_clip(&self) -> bool {
        self.proxy_drawing.will_clip()
    }
    pub fn proxy_drawing(&self) -> &dyn ProxyDrawing {
        &*self.proxy_drawing
    }
}

impl std::ops::Deref for Drawable {
    type Target = DrawableBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for Drawable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl std::ops::Deref for DrawableProxy {
    type Target = Drawable;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for DrawableProxy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
use std::{
    cell::{Ref, RefCell, RefMut},
    rc::{Rc, Weak},
};

#[cfg(test)]
mod source_proxy_tests {
    use super::*;
    use crate::source::{
        core::CoreArena,
        layer_mask::{LayerMask, LayerMaskOp},
        shapes::clipping_shape::ClippingShapeStart,
    };

    #[test]
    fn clip_and_mask_markers_use_their_own_component_hit_fallback() {
        let arena = CoreArena::default();
        let clipping = arena.insert(ClippingShape::default());
        let clip = clipping
            .with_downcast_mut::<ClippingShape, _>(|clip| {
                clip.create_proxy_drawable(Box::new(ClippingShapeStart::default()))
                    .unwrap()
            })
            .unwrap();
        let mask = arena.insert(LayerMask::default());
        let marker = mask
            .with_downcast_mut::<LayerMask, _>(|mask| {
                mask.create_proxy_drawable(LayerMaskOp::MaskStart)
            })
            .unwrap();
        for occurrence in [clip, marker] {
            assert!(
                occurrence
                    .with_proxy(|proxy| proxy.hittable_component().is_none())
                    .unwrap()
            );
            assert!(occurrence.hit_test_point(&Vec2D::new(1.0, 2.0), false, true));
            assert!(!occurrence.is_target_opaque());
        }
    }
}
