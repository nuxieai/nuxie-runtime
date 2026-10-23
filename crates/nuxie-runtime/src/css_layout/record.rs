//! The five CSS layout record types as ordinary components of the Rive
//! record stream: `LayoutCssStyle`, `BoxBorder`, `BoxGradient`, `BoxShadow`
//! and `BoxPaint` (docs/CSS_LAYOUT.md, section 1). Each stores its values in
//! one slot per property key, outside the mechanical upstream registry.
use super::{
    keys::{self, Field},
    style::{CssRole, CssValues},
};
use crate::source::{
    component::Component,
    component_dirt::ComponentDirt,
    core::{
        Core, CoreHandle, CoreObject, CoreType, PropertySetterCompletion,
        binary_reader::BinaryReader,
        field_types::{core_callback_type::CallbackData, core_uint_type::CoreUintType},
    },
    core_context::CoreContext,
    generated::{
        component_base::{ComponentBase, ComponentBaseCallbacks},
        core_registry::{CoreCapabilities, CoreField, CoreRegistryObject},
        layout_component_base::LayoutComponentBase,
        transform_component_base::TransformComponentBase,
    },
    importers::import_stack::ImportStack,
    layout::{
        layout_node_provider,
        layout_style_applier::{LayoutStyleApplier, LayoutSyncContext},
    },
    layout_component::LayoutComponent,
    status_code::StatusCode,
};
use std::any::Any;
use taffy::prelude::Style;

/// One CSS record type; `TYPE` is its type key.
pub struct CssRecord<const TYPE: u16> {
    component: Component,
    values: Box<[f32]>,
}

pub type LayoutCssStyle = CssRecord<{ keys::LAYOUT_CSS_STYLE }>;
pub type BoxBorder = CssRecord<{ keys::BOX_BORDER }>;
pub type BoxGradient = CssRecord<{ keys::BOX_GRADIENT }>;
pub type BoxShadow = CssRecord<{ keys::BOX_SHADOW }>;
pub type BoxPaint = CssRecord<{ keys::BOX_PAINT }>;

struct NoCallbacks;
impl ComponentBaseCallbacks for NoCallbacks {
    fn notify_property_changed(&mut self, _property_key: u16) {}
}

fn subtype<const TYPE: u16>(key: u16) -> bool {
    key == TYPE || ComponentBase::is_type_of(key)
}

impl<const TYPE: u16> Default for CssRecord<TYPE> {
    fn default() -> Self {
        let (first, count) = keys::range(TYPE).expect("a CSS record type key");
        Self {
            component: Component::default(),
            values: (first..first + count as u16)
                .map(keys::default_value)
                .collect(),
        }
    }
}

impl<const TYPE: u16> CssRecord<TYPE> {
    pub const TYPE_KEY: u16 = TYPE;

    pub fn values(&self) -> CssValues<'_> {
        CssValues {
            first: keys::range(TYPE).expect("a CSS record type key").0,
            values: &self.values,
        }
    }
    pub fn component(&self) -> &Component {
        &self.component
    }
    fn slot(&self, key: u16) -> Option<usize> {
        let (first, count) = keys::range(TYPE)?;
        let index = usize::from(key.checked_sub(first)?);
        // A slot inside the span that another type owns is not this one's.
        (index < count && keys::owner(key).is_some_and(|(owner, _)| owner == TYPE)).then_some(index)
    }
    pub fn number(&self, key: u16) -> Option<f32> {
        self.slot(key).map(|slot| self.values[slot])
    }
    /// Why this record cannot load, if it cannot.
    pub fn refusal(&self) -> Option<String> {
        super::style::refusal(self.values(), TYPE)
    }
    /// A write from a setter, an animation, a binding or a script. Values
    /// are stored as written; an enum outside its range reads as its
    /// initial value until it is set back in range.
    pub(crate) fn set_number(
        &mut self,
        key: u16,
        value: f32,
        completion: &mut PropertySetterCompletion,
    ) {
        let Some(slot) = self.slot(key) else {
            return;
        };
        let value = match keys::field(key) {
            Some(Field::Enum { .. }) => value.round(),
            _ => value,
        };
        if self.values[slot] == value || (self.values[slot].is_nan() && value.is_nan()) {
            return;
        }
        self.values[slot] = value;
        completion.record(CoreObject::core(self), key);
        self.changed(key);
    }
    fn changed(&mut self, key: u16) {
        let Some(parent) = self.component.parent_handle() else {
            return;
        };
        match TYPE {
            keys::LAYOUT_CSS_STYLE => {
                if parent.is_type_of(LayoutComponentBase::TYPE_KEY) {
                    if key == keys::DISPLAY {
                        let hidden = self.values().display_none();
                        LayoutComponent::css_display_changed_occurrence(&parent, hidden);
                    }
                    // Boxes that inherit the direction follow it.
                    if key == keys::DIRECTION {
                        LayoutComponent::direction_changed_occurrence(&parent);
                    }
                    if key == keys::DISPLAY
                        || key == keys::FLEX_DIRECTION
                        || key == keys::JUSTIFY_ITEMS
                    {
                        LayoutComponent::css_flow_changed_occurrence(&parent);
                    }
                    // What only the box's paint and clip read.
                    if (keys::RADIUS_TOP_LEFT_PERCENT..keys::RADIUS_TOP_LEFT_PERCENT + 4)
                        .contains(&key)
                        || (keys::BORDER_TOP..keys::BORDER_TOP + 4).contains(&key)
                        || key == keys::OVERFLOW_X
                        || key == keys::OVERFLOW_Y
                    {
                        LayoutComponent::css_paint_changed_occurrence(&parent);
                    }
                }
                // CSS painting order reads these, for a box or a
                // participant (docs/CSS_LAYOUT.md, 4.4).
                if matches!(
                    key,
                    keys::DISPLAY
                        | keys::POSITION
                        | keys::Z_INDEX_KIND
                        | keys::Z_INDEX
                        | keys::OVERFLOW_X
                        | keys::OVERFLOW_Y
                ) {
                    mark_draw_order_dirty(&parent);
                }
                mark_owner_dirty(&parent);
            }
            // A Stroke's BoxBorder and a Feather's BoxShadow shape the paths
            // their box draws them with.
            keys::BOX_BORDER => css_paint_changed(Some(parent)),
            keys::BOX_SHADOW => css_paint_changed(
                parent
                    .with(|feather| feather.component_parent_handle())
                    .flatten(),
            ),
            // A BoxGradient places its gradient, which rebuilds its shader.
            keys::BOX_GRADIENT => {
                parent
                    .with_mut(|gradient| gradient.component_add_dirt(ComponentDirt::PAINT, false));
            }
            // A BoxPaint sizes only its box's paint: nothing reflows.
            keys::BOX_PAINT => LayoutComponent::css_paint_changed_occurrence(&parent),
            _ => {}
        }
    }
    /// Parent kinds each record type accepts (docs/CSS_LAYOUT.md, 1.1).
    fn parent_supported(parent: &CoreHandle) -> bool {
        use crate::source::generated::shapes::paint::{
            feather_base::FeatherBase, linear_gradient_base::LinearGradientBase,
            stroke_base::StrokeBase,
        };
        match TYPE {
            keys::LAYOUT_CSS_STYLE => {
                parent.is_type_of(LayoutComponentBase::TYPE_KEY)
                    || parent.is_type_of(TransformComponentBase::TYPE_KEY)
            }
            // A box or an artboard; it paints only in CSS mode.
            keys::BOX_PAINT => parent.is_type_of(LayoutComponentBase::TYPE_KEY),
            keys::BOX_BORDER => parent.is_type_of(StrokeBase::TYPE_KEY),
            // RadialGradient extends LinearGradient.
            keys::BOX_GRADIENT => parent.is_type_of(LinearGradientBase::TYPE_KEY),
            keys::BOX_SHADOW => parent.is_type_of(FeatherBase::TYPE_KEY),
            _ => false,
        }
    }
    fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.component.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        // Refused forms fail the import (docs/CSS_LAYOUT.md, 1.4).
        if self.refusal().is_some()
            || !self
                .component
                .parent_handle()
                .is_some_and(|parent| Self::parent_supported(&parent))
        {
            return StatusCode::InvalidObject;
        }
        StatusCode::Ok
    }
    fn on_added_clean(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        let (Some(this), Some(parent)) = (self.component.handle(), self.component.parent_handle())
        else {
            return StatusCode::Ok;
        };
        if TYPE != keys::LAYOUT_CSS_STYLE && TYPE != keys::BOX_PAINT {
            return StatusCode::Ok;
        }
        // At most one record of each of these types per parent.
        let first = parent
            .with(|parent| {
                parent.as_container_component().and_then(|container| {
                    container
                        .children()
                        .iter()
                        .find(|child| child.is_type_of(TYPE))
                        .cloned()
                })
            })
            .flatten();
        if first.as_ref() != Some(&this) {
            return StatusCode::InvalidObject;
        }
        if TYPE == keys::BOX_PAINT {
            LayoutComponent::css_paint_changed_occurrence(&parent);
            return StatusCode::Ok;
        }
        if parent.is_type_of(LayoutComponentBase::TYPE_KEY) {
            parent.with_mut(|object| {
                if let Some(layout) = object.as_layout_component_mut() {
                    layout.add_layout_style_applier(this.clone());
                }
            });
            let hidden = self.values().display_none();
            LayoutComponent::css_display_changed_occurrence(&parent, hidden);
        } else {
            // A participant also finds this record among its host's children
            // when its layout data is created after this point.
            layout_node_provider::with_mut(&parent, |provider| {
                provider.add_layout_style_applier(this.clone())
            });
        }
        mark_owner_dirty(&parent);
        StatusCode::Ok
    }
}

/// The record a component's children hold, if any.
pub fn find_in(children: &[CoreHandle]) -> Option<CoreHandle> {
    children
        .iter()
        .find(|child| child.is_type_of(keys::LAYOUT_CSS_STYLE))
        .cloned()
}

/// `owner`'s artboard sorts its draw order again.
pub(crate) fn mark_draw_order_dirty(owner: &CoreHandle) {
    let artboard =
        if owner.is_type_of(crate::source::generated::artboard_base::ArtboardBase::TYPE_KEY) {
            Some(owner.clone())
        } else {
            owner
                .with(|object| object.component_artboard_handle())
                .flatten()
        };
    if let Some(artboard) = artboard {
        artboard.with_mut(|object| object.component_add_dirt(ComponentDirt::DRAW_ORDER, false));
    }
}

/// `paint`'s box redraws its CSS paint paths, when it is a box.
pub(crate) fn css_paint_changed(paint: Option<CoreHandle>) {
    let owner = paint
        .and_then(|paint| {
            paint
                .with(|paint| paint.component_parent_handle())
                .flatten()
        })
        .filter(|owner| owner.is_type_of(LayoutComponentBase::TYPE_KEY));
    if let Some(owner) = owner {
        LayoutComponent::css_paint_changed_occurrence(&owner);
    }
}

fn mark_owner_dirty(parent: &CoreHandle) {
    if parent.is_type_of(LayoutComponentBase::TYPE_KEY) {
        LayoutComponent::mark_layout_node_dirty_occurrence(parent, false);
    } else {
        layout_node_provider::with_mut(parent, |provider| provider.mark_layout_node_dirty(false));
    }
}

impl LayoutStyleApplier for LayoutCssStyle {
    fn apply_css_style(&self, style: &mut Style, context: &LayoutSyncContext) {
        super::style::apply(self.values(), style, context.css_role, context.is_ltr);
        // Video inherits Image's type. A direct picture participant owns
        // replaced sizing; a box applies it to its filling picture separately.
        let picture = self.component.parent_handle().filter(|host| {
            context.css_role == CssRole::Participant
                && host.is_type_of(crate::source::generated::shapes::image_base::ImageBase::TYPE_KEY)
        });
        style.item_is_replaced = picture.is_some();
        if let Some(picture) = picture {
            crate::source::layout::layout_participant::apply_picture_natural_ratio(style, &picture);
        }
    }
}

/// Registry dispatch for the CSS record keys. Returns whether `object` is a
/// CSS record (whatever the key).
pub(crate) fn set_property(
    object: &mut dyn Any,
    key: u16,
    value: f32,
    completion: &mut PropertySetterCompletion,
) -> bool {
    macro_rules! try_set {
        ($($ty:ty),*) => {
            $(if let Some(record) = object.downcast_mut::<$ty>() {
                record.set_number(key, value, completion);
                return true;
            })*
        };
    }
    try_set!(LayoutCssStyle, BoxBorder, BoxGradient, BoxShadow, BoxPaint);
    false
}

pub(crate) fn get_property(object: &dyn Any, key: u16) -> Option<f32> {
    macro_rules! try_get {
        ($($ty:ty),*) => {
            $(if let Some(record) = object.downcast_ref::<$ty>() {
                return record.number(key);
            })*
        };
    }
    try_get!(LayoutCssStyle, BoxBorder, BoxGradient, BoxShadow, BoxPaint);
    None
}

impl<const TYPE: u16> CoreType for CssRecord<TYPE> {
    const TYPE_KEY: u16 = TYPE;
}

impl<const TYPE: u16> CoreRegistryObject for CssRecord<TYPE> {
    fn as_registry_any(&self) -> &dyn Any {
        self
    }
    fn as_registry_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn is_type_of(&self, key: u16) -> bool {
        subtype::<TYPE>(key)
    }
    fn set_uint_with_completion(
        &mut self,
        field: CoreField,
        value: u32,
        completion: &mut PropertySetterCompletion,
    ) {
        if field == CoreField::ComponentParentId && self.component.base.set_parent_id_value(value) {
            completion.record(
                CoreObject::core(self),
                ComponentBase::PARENT_ID_PROPERTY_KEY,
            );
        }
    }
    fn set_string_with_completion(
        &mut self,
        field: CoreField,
        value: String,
        completion: &mut PropertySetterCompletion,
    ) {
        if field == CoreField::ComponentName && self.component.base.set_name_value(value) {
            completion.record(CoreObject::core(self), ComponentBase::NAME_PROPERTY_KEY);
        }
    }
    fn set_color_with_completion(
        &mut self,
        _: CoreField,
        _: i32,
        _: &mut PropertySetterCompletion,
    ) {
    }
    fn set_bool_with_completion(
        &mut self,
        _: CoreField,
        _: bool,
        _: &mut PropertySetterCompletion,
    ) {
    }
    fn set_double_with_completion(
        &mut self,
        _: CoreField,
        _: f32,
        _: &mut PropertySetterCompletion,
    ) {
    }
    fn set_int_with_completion(&mut self, _: CoreField, _: i32, _: &mut PropertySetterCompletion) {}
    fn set_callback_with_completion(
        &mut self,
        _: CoreField,
        _: CallbackData<'_>,
        _: &mut PropertySetterCompletion,
    ) {
    }
    fn get_uint(&mut self, field: CoreField) -> u32 {
        if field == CoreField::ComponentParentId {
            self.component.base.parent_id()
        } else {
            0
        }
    }
    fn get_string(&mut self, field: CoreField) -> String {
        if field == CoreField::ComponentName {
            self.component.base.name().to_owned()
        } else {
            String::new()
        }
    }
    fn get_color(&mut self, _: CoreField) -> i32 {
        0
    }
    fn get_bool(&mut self, _: CoreField) -> bool {
        false
    }
    fn get_double(&mut self, _: CoreField) -> f32 {
        0.0
    }
    fn get_int(&mut self, _: CoreField) -> i32 {
        0
    }
}

impl<const TYPE: u16> CoreObject for CssRecord<TYPE> {
    fn core(&self) -> &Core {
        &self.component.base.base
    }
    fn core_mut(&mut self) -> &mut Core {
        &mut self.component.base.base
    }
    fn core_type(&self) -> u16 {
        TYPE
    }
    fn is_type_of(&self, key: u16) -> bool {
        subtype::<TYPE>(key)
    }
    fn type_predicate(&self) -> fn(u16) -> bool {
        subtype::<TYPE>
    }
    fn clone_boxed(&self) -> Option<Box<dyn CoreObject>> {
        let mut clone = Self::default();
        clone
            .component
            .base
            .copy(&self.component.base, &mut NoCallbacks);
        clone.values = self.values.clone();
        Some(Box::new(clone))
    }
    fn deserialize(&mut self, key: u16, reader: &mut BinaryReader<'_>) -> bool {
        if let Some(slot) = self.slot(key) {
            self.values[slot] = match keys::field(key) {
                Some(Field::Enum { .. }) => CoreUintType::deserialize(reader) as f32,
                _ => reader.read_float32(),
            };
            return true;
        }
        self.component
            .base
            .deserialize(key, reader, &mut NoCallbacks)
    }
}

impl<const TYPE: u16> CoreCapabilities for CssRecord<TYPE> {
    fn component_on_dirty(&mut self, _dirt: crate::source::component_dirt::ComponentDirt) -> bool {
        true
    }
    fn component_on_dirty_is_noop(&self) -> bool {
        true
    }
    fn as_layout_style_applier(&self) -> Option<&dyn LayoutStyleApplier> {
        (TYPE == keys::LAYOUT_CSS_STYLE)
            .then(|| self.as_registry_any().downcast_ref::<LayoutCssStyle>())
            .flatten()
            .map(|record| record as &dyn LayoutStyleApplier)
    }
    fn lifecycle_validate(&mut self, context: &mut dyn CoreContext) -> Option<bool> {
        Some(self.component.validate(context))
    }
    fn lifecycle_on_added_dirty(&mut self, context: &mut dyn CoreContext) -> Option<StatusCode> {
        Some(self.on_added_dirty(context))
    }
    fn lifecycle_on_added_clean(&mut self, context: &mut dyn CoreContext) -> Option<StatusCode> {
        Some(self.on_added_clean(context))
    }
    fn lifecycle_import(&mut self, stack: &mut ImportStack) -> Option<StatusCode> {
        Some(self.component.import(stack))
    }
    fn as_component(&self) -> Option<&Component> {
        Some(&self.component)
    }
    fn as_component_mut(&mut self) -> Option<&mut Component> {
        Some(&mut self.component)
    }
}

/// The record of a box, read for its CSS mode questions.
pub fn with_values<R>(record: &CoreHandle, f: impl FnOnce(CssValues<'_>) -> R) -> Option<R> {
    record.with_downcast::<LayoutCssStyle, _>(|record| f(record.values()))
}

/// The paint size a box's `BoxPaint` gives, if its children hold one
/// (docs/CSS_LAYOUT.md, 4.2): on each axis its points plus its percentage
/// of the border box's `size` on that axis, never below zero.
pub(crate) fn box_paint_size(children: &[CoreHandle], size: (f32, f32)) -> Option<(f32, f32)> {
    children
        .iter()
        .find(|child| child.is_type_of(keys::BOX_PAINT))?
        .with_downcast::<BoxPaint, _>(|record| {
            let axis = |points, percent, border: f32| {
                let value = record.number(points).unwrap_or(0.0)
                    + record.number(percent).unwrap_or(0.0) / 100.0 * border;
                Some(value)
                    .filter(|value| value.is_finite())
                    .map_or(0.0, |value| value.max(0.0))
            };
            (
                axis(keys::PAINT_WIDTH, keys::PAINT_WIDTH_PERCENT, size.0),
                axis(keys::PAINT_HEIGHT, keys::PAINT_HEIGHT_PERCENT, size.1),
            )
        })
}

/// The role a sync context gives a node's record.
pub fn role(is_artboard: bool, is_participant: bool) -> CssRole {
    if is_participant {
        CssRole::Participant
    } else if is_artboard {
        CssRole::Artboard
    } else {
        CssRole::Box
    }
}
