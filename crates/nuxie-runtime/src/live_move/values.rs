//! Occurrence values belong to a flight, never to a player or a retained row.
use crate::{
    CoreHandle, RuntimeArtboardInstanceHandle,
    source::{
        data_bind::{
            data_bind::{BINDINGS, DataBind},
            data_bind_context::DataBindContext,
        },
        generated::{
            core_registry::CoreRegistry,
            text::text_input_base::TextInputBase,
            viewmodel::{
                viewmodel_instance_color_base::ViewModelInstanceColorBase,
                viewmodel_instance_number_base::ViewModelInstanceNumberBase,
                viewmodel_instance_string_base::ViewModelInstanceStringBase,
            },
        },
        shapes::paint::color::{
            ColorInt, color_alpha, color_argb, color_blue, color_green, color_lerp, color_red,
        },
        text::text_input::TextInput,
        viewmodel::viewmodel_instance::ViewModelInstance,
    },
};
use std::{cell::Cell, collections::HashMap, rc::Rc};

#[derive(Clone, Copy)]
enum Start {
    Number(f32),
    Color(i32),
}

/// The binding has only a weak link here. Ordinary bind updates are held;
/// only the follower samples and projects its converted source-to-target value.
pub(crate) struct ValueBlend {
    start: Start,
    progress: Cell<f32>,
    sampling: Cell<bool>,
}
impl ValueBlend {
    pub(crate) fn sampling(&self) -> bool {
        self.sampling.get()
    }
    pub(crate) fn number(&self, end: f32) -> f32 {
        let Start::Number(start) = self.start else {
            return end;
        };
        let t = self.progress.get();
        if t == 0.0 {
            start
        } else if t == 1.0 {
            end
        } else {
            start + (end - start) * t
        }
    }
    pub(crate) fn color(&self, end: i32) -> i32 {
        let Start::Color(start) = self.start else {
            return end;
        };
        blend_color(start as u32, end as u32, self.progress.get()) as i32
    }
}

/// CSS blends colors premultiplied (CSS Color 4, 13.4): each end counts by its
/// alpha, so a clear end adds no color and the other end keeps its own.
fn blend_color(from: ColorInt, to: ColorInt, t: f32) -> ColorInt {
    let (a, b) = (color_alpha(from) as f32, color_alpha(to) as f32);
    if a == b {
        return color_lerp(from, to, t);
    }
    if t == 0.0 {
        return from;
    }
    if t == 1.0 {
        return to;
    }
    // Alpha and the channels share these weights. Subtracting from `a` instead
    // can cancel near a clear end, and a channel would round past 255 and wrap.
    let (a, b) = (a * (1.0 - t), b * t);
    let alpha = a + b;
    let mix = |x: u32, y: u32| ((x as f32 * a + y as f32 * b) / alpha).round() as i32;
    color_argb(
        alpha.round() as i32,
        mix(color_red(from), color_red(to)),
        mix(color_green(from), color_green(to)),
        mix(color_blue(from), color_blue(to)),
    )
}

#[derive(Default)]
pub(super) struct FlightValues(Vec<(CoreHandle, Rc<ValueBlend>)>);
impl Drop for FlightValues {
    fn drop(&mut self) {
        // Cancellation/reconciliation can retain the mounted binding instead
        // of replacing it. Queue its source-only handoff at the next ordinary
        // settlement; never write values from destruction or visit other rows.
        for (bind, blend) in std::mem::take(&mut self.0) {
            drop(blend);
            if bind.is_alive() {
                DataBind::add_dirt_handle(&bind, BINDINGS, false);
            }
        }
    }
}
impl Clone for FlightValues {
    // Flight clones are public pose/control snapshots. They must not keep the
    // live flight's binding overrides alive after landing or reconciliation.
    fn clone(&self) -> Self {
        Self::default()
    }
}
impl FlightValues {
    pub(super) fn attach(&mut self, bind: &CoreHandle, target: &CoreHandle) {
        let Some(key) = bind
            .with(|o| {
                o.as_data_bind()
                    .filter(|b| b.to_target())
                    .map(|b| b.property_key())
            })
            .flatten()
        else {
            return;
        };
        let start = match key as u16 {
            ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY => {
                CoreRegistry::get_double_handle(target, key as i32).map(Start::Number)
            }
            ViewModelInstanceColorBase::PROPERTY_VALUE_PROPERTY_KEY => {
                CoreRegistry::get_color_handle(target, key as i32).map(Start::Color)
            }
            _ => None,
        };
        let Some(start) = start else {
            return;
        };
        let blend = Rc::new(ValueBlend {
            start,
            progress: Cell::new(0.0),
            sampling: Cell::new(false),
        });
        bind.with_mut(|o| {
            let bind = o.as_data_bind_mut().unwrap();
            bind.require_source_adoption();
            bind.live_move_blend = Rc::downgrade(&blend);
        });
        self.0.push((bind.clone(), blend));
    }
    pub(super) fn follow(&self, progress: f32) {
        for (bind, blend) in &self.0 {
            if !bind.is_alive() {
                continue;
            }
            blend.progress.set(progress);
            blend.sampling.set(true);
            // Sample even a clean/once binding: its destination is live for
            // the entire flight. Converters still use the ordinary bind path.
            DataBind::update_handle(bind, BINDINGS);
            blend.sampling.set(false);
        }
    }
}

/// Only edited fields whose local string property had no outgoing occurrence
/// binding can override an incoming prop. An internal field <-> unit-property
/// bind is local state, not a Screen occurrence binding.
#[derive(Default)]
pub(super) struct TypedText(HashMap<u32, String>);
impl TypedText {
    pub(super) fn capture(instance: &RuntimeArtboardInstanceHandle, bound: &[u32]) -> Self {
        let Some(vm) = instance
            .data_context()
            .and_then(|c| c.with_context(|c| c.main_view_model_instance()))
        else {
            return Self::default();
        };
        let values = vm
            .with_downcast::<ViewModelInstance, _>(|v| v.property_values().to_vec())
            .unwrap_or_default();
        let binds = instance.with_artboard(|b| b.data_bind_handles());
        let mut bound = bound.to_vec();
        // After take-off the mounted props are the landing props, whereas the
        // emptied placement's inline model still describes its placeholder.
        for bind in &binds {
            if let Some(target) = bind
                .with(|o| {
                    let b = o.as_data_bind()?;
                    b.is_instance_value_bind().then(|| b.target()).flatten()
                })
                .flatten()
                .filter(|t| values.contains(t))
            {
                if let Some(id) = target
                    .with(|o| {
                        o.as_view_model_instance_value()
                            .map(|v| v.view_model_property_id())
                    })
                    .flatten()
                {
                    bound.push(id);
                }
            }
        }
        let mut text = HashMap::new();
        for field in instance.with_artboard(|b| b.find_all_handles::<TextInput>()) {
            let Some(typed) = field
                .with_downcast::<TextInput, _>(|f| f.live_move_typed_text())
                .flatten()
            else {
                continue;
            };
            for bind in &binds {
                let source = bind
                    .with(|o| {
                        let b = o.as_data_bind()?;
                        (b.target().as_ref() == Some(&field)
                            && b.property_key() == u32::from(TextInputBase::TEXT_PROPERTY_KEY))
                        .then(|| b.source())
                        .flatten()
                    })
                    .flatten()
                    .filter(|s| values.contains(s));
                if let Some(source) = source {
                    let id = source
                        .with(|o| {
                            o.as_view_model_instance_value()
                                .map(|v| v.view_model_property_id())
                        })
                        .flatten();
                    if let Some(id) = id.filter(|id| !bound.contains(id)) {
                        text.insert(id, typed.clone());
                    }
                }
            }
        }
        Self(text)
    }
    pub(super) fn write_back(
        &self,
        id: u32,
        bind: &CoreHandle,
        target: &CoreHandle,
        instance: &RuntimeArtboardInstanceHandle,
    ) {
        let Some(text) = self.0.get(&id) else {
            return;
        };
        let key = i32::from(ViewModelInstanceStringBase::PROPERTY_VALUE_PROPERTY_KEY);
        // Occurrence props in this runtime target unit VM properties. The
        // TextInput itself retains its internal binding and edited buffer.
        if CoreRegistry::get_string_handle(target, key).is_none() {
            return;
        }
        CoreRegistry::set_string_handle(target, key, text.clone());
        DataBindContext::bind_from_context_handle(bind, instance.data_context());
        let Some((source, to_source)) = bind
            .with(|o| {
                let b = o.as_data_bind()?;
                Some((b.source()?, b.to_source()))
            })
            .flatten()
        else {
            return;
        };
        if to_source {
            // Use the binding's reverse converter and normal source setter.
            DataBind::update_source_binding_handle(bind, true);
        } else if CoreRegistry::get_string_handle(&source, key).is_some() {
            // A one-way string prop has no reverse conversion contract.
            CoreRegistry::set_string_handle(&source, key, text.clone());
        }
    }
}

/// A value as one of a view model value's kinds holds it, read through its
/// field type.
#[derive(Clone, Debug, PartialEq)]
enum Written {
    Uint(u32),
    Double(f32),
    Str(String),
    Bool(bool),
    Color(i32),
}
impl Written {
    fn read(value: &CoreHandle, key: u16) -> Option<Self> {
        use crate::source::core::field_types::{
            core_bool_type::CoreBoolType, core_color_type::CoreColorType,
            core_double_type::CoreDoubleType, core_string_type::CoreStringType,
            core_uint_type::CoreUintType,
        };
        let key = i32::from(key);
        match CoreRegistry::property_field_id(key) {
            CoreUintType::ID => CoreRegistry::get_uint_handle(value, key).map(Self::Uint),
            CoreDoubleType::ID => CoreRegistry::get_double_handle(value, key).map(Self::Double),
            CoreStringType::ID => CoreRegistry::get_string_handle(value, key).map(Self::Str),
            CoreBoolType::ID => CoreRegistry::get_bool_handle(value, key).map(Self::Bool),
            CoreColorType::ID => CoreRegistry::get_color_handle(value, key).map(Self::Color),
            _ => None,
        }
    }
    /// Writes through the property's setter, so its kind's change hooks run
    /// as for any other write.
    fn write(&self, value: &CoreHandle, key: u16) {
        let key = i32::from(key);
        match self {
            Self::Uint(v) => CoreRegistry::set_uint_handle(value, key, *v),
            Self::Double(v) => CoreRegistry::set_double_handle(value, key, *v),
            Self::Str(v) => CoreRegistry::set_string_handle(value, key, v.clone()),
            Self::Bool(v) => CoreRegistry::set_bool_handle(value, key, *v),
            Self::Color(v) => CoreRegistry::set_color_handle(value, key, *v),
        };
    }
    /// Whether this kind blends over a flight: a number or a color.
    fn blends(&self) -> bool {
        matches!(self, Self::Double(_) | Self::Color(_))
    }
    /// At `t` from `self` to `to`: a number or a color blends as a bound one
    /// does, anything else is already `to`.
    fn at(&self, to: &Self, t: f32) -> Self {
        match (self, to) {
            (Self::Double(a), Self::Double(b)) if t < 1.0 => Self::Double(a + (b - a) * t),
            (Self::Color(a), Self::Color(b)) if t < 1.0 => {
                Self::Color(blend_color(*a as u32, *b as u32, t) as i32)
            }
            _ => to.clone(),
        }
    }
}

/// The key that holds a view model value's written value, for each kind a
/// carried instance takes from the placement it arrives at: the kinds that
/// hold nothing in place of what is written, by the one property each
/// declares for it. `UNWRITTEN_KINDS` names the rest.
const WRITTEN_KEYS: [u16; 6] = {
    use crate::source::generated::viewmodel::{
        viewmodel_instance_boolean_base::ViewModelInstanceBooleanBase,
        viewmodel_instance_enum_base::ViewModelInstanceEnumBase,
        viewmodel_instance_symbol_list_index_base::ViewModelInstanceSymbolListIndexBase,
    };
    [
        ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY,
        ViewModelInstanceColorBase::PROPERTY_VALUE_PROPERTY_KEY,
        ViewModelInstanceStringBase::PROPERTY_VALUE_PROPERTY_KEY,
        ViewModelInstanceBooleanBase::PROPERTY_VALUE_PROPERTY_KEY,
        ViewModelInstanceEnumBase::PROPERTY_VALUE_PROPERTY_KEY,
        ViewModelInstanceSymbolListIndexBase::PROPERTY_VALUE_PROPERTY_KEY,
    ]
};

/// The key of `value`'s written value, by its kind; None for a kind a
/// carried instance keeps as it left.
pub(super) fn written_key(value: &CoreHandle) -> Option<u16> {
    WRITTEN_KEYS
        .into_iter()
        .find(|&key| value.is_type_of(CoreRegistry::property_owner_type_key(u32::from(key))))
}

/// The kinds a carried instance keeps as it left, by type key. A test checks
/// every kind of view model value is here or has a key in `WRITTEN_KEYS`.
/// - A list (its rows), a trigger (a fire count) and a reference to another
///   model instance (what it refers to is an object, not the id it was
///   written with) hold no one written value.
/// - An image, a font, a blob and an artboard can hold one a host, a binding
///   or a script gave them in place of the file's, with their id at "none",
///   so writing the id alone would not replace it (UNIV-6481).
#[cfg(test)]
const UNWRITTEN_KINDS: [u16; 7] = {
    use crate::source::generated::viewmodel::{
        viewmodel_instance_artboard_base::ViewModelInstanceArtboardBase,
        viewmodel_instance_asset_blob_base::ViewModelInstanceAssetBlobBase,
        viewmodel_instance_asset_font_base::ViewModelInstanceAssetFontBase,
        viewmodel_instance_asset_image_base::ViewModelInstanceAssetImageBase,
        viewmodel_instance_list_base::ViewModelInstanceListBase,
        viewmodel_instance_trigger_base::ViewModelInstanceTriggerBase,
        viewmodel_instance_viewmodel_base::ViewModelInstanceViewModelBase,
    };
    [
        ViewModelInstanceListBase::TYPE_KEY,
        ViewModelInstanceTriggerBase::TYPE_KEY,
        ViewModelInstanceViewModelBase::TYPE_KEY,
        ViewModelInstanceAssetImageBase::TYPE_KEY,
        ViewModelInstanceAssetFontBase::TYPE_KEY,
        ViewModelInstanceAssetBlobBase::TYPE_KEY,
        ViewModelInstanceArtboardBase::TYPE_KEY,
    ]
};

/// One value of a carried instance that the step changes: the instance's own
/// value and its key, the value it shows at take-off, and what each
/// placement writes, as the file has them; None where that placement binds
/// it, so its binding gives the value instead.
struct Change {
    target: CoreHandle,
    key: u16,
    start: Written,
    from: Option<Written>,
    to: Option<Written>,
}

/// What a carried instance takes from the placement it arrives at, as the
/// written values of where it is, for the kinds `WRITTEN_KEYS` names: each
/// value the arriving placement writes, where the leaving one binds it or
/// writes it differently. A number or a
/// color blends from what the instance shows at take-off to the arriving
/// value over the flight, as a bound one does; anything else switches at
/// take-off and is then the instance's own. A value written the same on
/// both is left alone, so the instance keeps what it made of it. A cancel
/// that takes the instance home gives it back what the placement it left
/// writes, including a value the arriving placement binds.
#[derive(Default)]
pub(super) struct WrittenValues(Vec<Change>);
impl Clone for WrittenValues {
    // As a flight's values: a snapshot of a flight never writes its values.
    fn clone(&self) -> Self {
        Self::default()
    }
}
impl WrittenValues {
    /// Reads `from` and `to`, the two placements' own model instances as
    /// the file has them, against `model`, the instance's own.
    /// `bound_from` and `bound_to` name the properties each placement binds.
    pub(super) fn read(
        from: &CoreHandle,
        to: &CoreHandle,
        model: &CoreHandle,
        bound_from: &[u32],
        bound_to: &[u32],
    ) -> Self {
        let values = |model: &CoreHandle| {
            model
                .with_downcast::<ViewModelInstance, _>(|v| v.property_values().to_vec())
                .unwrap_or_default()
        };
        let id = |value: &CoreHandle| {
            value
                .with(|o| {
                    o.as_view_model_instance_value()
                        .map(|v| v.view_model_property_id())
                })
                .flatten()
        };
        let (leaving, arriving) = (values(from), values(to));
        let find = |values: &[CoreHandle], wanted: u32| {
            values.iter().find(|v| id(v) == Some(wanted)).cloned()
        };
        let mut changes = Vec::new();
        for target in values(model) {
            let Some(property) = id(&target) else {
                continue;
            };
            let Some(key) = written_key(&target) else {
                continue;
            };
            let written = |values: &[CoreHandle], bound: &[u32]| {
                if bound.contains(&property) {
                    Some(None)
                } else {
                    find(values, property).and_then(|v| Written::read(&v, key).map(Some))
                }
            };
            let (Some(from), Some(to), Some(start)) = (
                written(&leaving, bound_from),
                written(&arriving, bound_to),
                Written::read(&target, key),
            ) else {
                continue;
            };
            let changes_here = match (&from, &to) {
                (Some(from), Some(to)) => from != to,
                (None, None) => false,
                _ => true,
            };
            if changes_here {
                changes.push(Change {
                    target,
                    key,
                    start,
                    from,
                    to,
                });
            }
        }
        Self(changes)
    }

    /// At take-off: each arriving value that does not blend switches.
    pub(super) fn take_off(&self) {
        for change in &self.0 {
            if let Some(to) = change.to.as_ref().filter(|to| !to.blends()) {
                to.write(&change.target, change.key);
            }
        }
    }

    /// At `progress` of the flight: each arriving value that blends.
    pub(super) fn follow(&self, progress: f32) {
        for change in &self.0 {
            if let Some(to) = change.to.as_ref().filter(|to| to.blends()) {
                change
                    .start
                    .at(to, progress)
                    .write(&change.target, change.key);
            }
        }
    }

    /// Landed where it flew, or left there by a cancel: the arriving values.
    pub(super) fn arrive(&self) {
        self.follow(1.0);
    }

    /// Carried home by a cancel: what the placement it left writes.
    pub(super) fn home(&self) {
        for change in &self.0 {
            if let Some(from) = &change.from {
                from.write(&change.target, change.key);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::generated::{
        core_type_tree::{PARENT_TYPE_KEYS, has_ancestor},
        viewmodel::viewmodel_instance_value_base::ViewModelInstanceValueBase,
    };

    /// Every kind of view model value either has its written value carried
    /// or is named as one a carried instance keeps as it left, so a kind
    /// added to the format fails here until it is one or the other. Kinds
    /// that others extend (a symbol, an asset) are covered through the kinds
    /// a file holds.
    #[test]
    fn every_kind_of_value_is_carried_or_named() {
        let value = ViewModelInstanceValueBase::TYPE_KEY;
        let kinds = (0..PARENT_TYPE_KEYS.len() as u16)
            .filter(|&kind| has_ancestor(kind, value))
            .collect::<Vec<_>>();
        let owners = WRITTEN_KEYS.map(|key| CoreRegistry::property_owner_type_key(u32::from(key)));
        for owner in owners {
            assert!(kinds.contains(&owner), "{owner} is a kind of value");
        }
        for kind in &kinds {
            let extended = kinds
                .iter()
                .any(|&other| PARENT_TYPE_KEYS[usize::from(other)] == *kind);
            if extended {
                continue;
            }
            let carried = owners
                .iter()
                .any(|&owner| owner == *kind || has_ancestor(*kind, owner));
            assert!(
                carried != UNWRITTEN_KINDS.contains(kind),
                "kind {kind}: carried {carried}, named as kept {}",
                UNWRITTEN_KINDS.contains(kind)
            );
        }
    }
}
