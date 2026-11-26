//! Nuxie-owned spring easings: `spring(p)` and `bouncy(p)`.
//!
//! Rive's keyframe interpolators are a cubic ease, a cubic value curve and an
//! elastic ring whose swing halves every tenth of the segment whatever it is
//! asked. A spring that settles without passing its key, or a bounce whose
//! damping and frequency follow one strength, is neither. `SpringInterpolator`
//! (type 60004) is a keyframe interpolator beside them: a key's
//! `interpolatorId` names it as it names an ElasticInterpolator, and anything
//! else that takes an interpolator takes it too. Its `kind` (60027) picks the
//! curve and its `strength` (60028) is the strength p from 0 to 1.
//!
//! Both curves are worked in 64-bit floats from the strength and time as
//! 32-bit floats, start exactly at 0 and end exactly at 1.
use crate::source::{
    animation::keyframe_interpolator::KeyFrameInterpolator,
    core::{
        Core, CoreObject, CoreType, PropertySetterCompletion,
        binary_reader::BinaryReader,
        field_types::{
            core_callback_type::CallbackData, core_double_type::CoreDoubleType,
            core_uint_type::CoreUintType,
        },
    },
    core_context::CoreContext,
    generated::{
        animation::keyframe_interpolator_base::KeyFrameInterpolatorBase,
        core_registry::{CoreCapabilities, CoreField, CoreRegistryObject},
    },
    importers::import_stack::ImportStack,
    status_code::StatusCode,
};
use std::any::Any;

/// `kind`: a critically damped step on smoothstepped time.
pub const SPRING: u32 = 0;
/// `kind`: an underdamped step that passes the next key and settles on it.
pub const BOUNCY: u32 = 1;
/// The `kind` property.
pub const KIND_PROPERTY_KEY: u16 = 60027;
/// The `strength` property.
pub const STRENGTH_PROPERTY_KEY: u16 = 60028;
/// The strength a record that writes none has, and that not a number reads as.
pub const DEFAULT_STRENGTH: f32 = 0.6;

/// The share of the way from a key's value to the next at `time`, the share
/// of the segment's time gone, for a curve `kind` at `strength`. Time is
/// clamped to 0 to 1 (not a number is 0), strength to 0 to 1 (not a number
/// is [`DEFAULT_STRENGTH`]); an unknown kind is linear.
pub fn ease(kind: u32, strength: f32, time: f32) -> f32 {
    let t = if time.is_nan() {
        0.0
    } else {
        time.clamp(0.0, 1.0)
    };
    if t >= 1.0 {
        return 1.0;
    }
    if t <= 0.0 {
        return 0.0;
    }
    let p = if strength.is_nan() {
        DEFAULT_STRENGTH
    } else {
        strength.clamp(0.0, 1.0)
    } as f64;
    let t = t as f64;
    match kind {
        SPRING => {
            let k = 8.0 * p + 3.5;
            let rise = |u: f64| 1.0 - (-k * u).exp() * (1.0 + k * u);
            let u = t * t * (3.0 - 2.0 * t);
            (rise(u) / rise(1.0)) as f32
        }
        BOUNCY => {
            let omega = 8.0 * p + 10.0;
            let zeta = (0.55 - 0.35 * p).clamp(0.05, 0.9);
            let root = (1.0 - zeta * zeta).sqrt();
            let damped = omega * root;
            let osc = |t: f64| {
                1.0 - (-zeta * omega * t).exp()
                    * ((damped * t).cos() + zeta / root * (damped * t).sin())
            };
            (osc(t) / osc(1.0)) as f32
        }
        _ => t as f32,
    }
}

pub struct SpringInterpolator {
    interpolator: KeyFrameInterpolator,
    kind: u32,
    strength: f32,
}

impl Default for SpringInterpolator {
    fn default() -> Self {
        Self {
            interpolator: KeyFrameInterpolator::default(),
            kind: SPRING,
            strength: DEFAULT_STRENGTH,
        }
    }
}

impl SpringInterpolator {
    pub const TYPE_KEY: u16 = 60004;

    fn subtype(key: u16) -> bool {
        key == Self::TYPE_KEY || KeyFrameInterpolatorBase::is_type_of(key)
    }

    pub fn kind(&self) -> u32 {
        self.kind
    }

    pub fn strength(&self) -> f32 {
        self.strength
    }

    /// Returns whether the value changed.
    pub fn set_kind(&mut self, kind: u32) -> bool {
        let changed = self.kind != kind;
        self.kind = kind;
        changed
    }

    /// Returns whether the value changed.
    pub fn set_strength(&mut self, strength: f32) -> bool {
        let changed = self.strength.to_bits() != strength.to_bits();
        self.strength = strength;
        changed
    }

    pub fn transform(&self, factor: f32) -> f32 {
        ease(self.kind, self.strength, factor)
    }

    pub fn transform_value(&self, from: f32, to: f32, factor: f32) -> f32 {
        from + (to - from) * self.transform(factor)
    }

    /// The registry's by-key uint setter for this record's `kind`.
    pub(crate) fn set_uint_property(
        object: &mut dyn Any,
        property_key: i32,
        value: u32,
        completion: &mut PropertySetterCompletion,
    ) -> bool {
        if property_key != i32::from(KIND_PROPERTY_KEY) {
            return false;
        }
        if let Some(curve) = object.downcast_mut::<Self>()
            && curve.set_kind(value)
        {
            completion.record(CoreObject::core(curve), KIND_PROPERTY_KEY);
        }
        true
    }

    /// The registry's by-key double setter for this record's `strength`.
    pub(crate) fn set_double_property(
        object: &mut dyn Any,
        property_key: i32,
        value: f32,
        completion: &mut PropertySetterCompletion,
    ) -> bool {
        if property_key != i32::from(STRENGTH_PROPERTY_KEY) {
            return false;
        }
        if let Some(curve) = object.downcast_mut::<Self>()
            && curve.set_strength(value)
        {
            completion.record(CoreObject::core(curve), STRENGTH_PROPERTY_KEY);
        }
        true
    }

    /// This record's property, by key, as the registry's getters read it.
    pub(crate) fn property(object: &dyn Any, property_key: i32) -> Option<f32> {
        let curve = object.downcast_ref::<Self>();
        match property_key {
            key if key == i32::from(KIND_PROPERTY_KEY) => {
                Some(curve.map_or(0.0, |curve| curve.kind as f32))
            }
            key if key == i32::from(STRENGTH_PROPERTY_KEY) => {
                Some(curve.map_or(0.0, |curve| curve.strength))
            }
            _ => None,
        }
    }

    /// The field type of this record's property `property_key`.
    pub(crate) fn field_id(property_key: i32) -> Option<i32> {
        match property_key {
            key if key == i32::from(KIND_PROPERTY_KEY) => Some(CoreUintType::ID),
            key if key == i32::from(STRENGTH_PROPERTY_KEY) => Some(CoreDoubleType::ID),
            _ => None,
        }
    }
}

impl CoreType for SpringInterpolator {
    const TYPE_KEY: u16 = SpringInterpolator::TYPE_KEY;
}

impl CoreObject for SpringInterpolator {
    fn core(&self) -> &Core {
        &self.interpolator.base.base
    }
    fn core_mut(&mut self) -> &mut Core {
        &mut self.interpolator.base.base
    }
    fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    fn is_type_of(&self, key: u16) -> bool {
        Self::subtype(key)
    }
    fn type_predicate(&self) -> fn(u16) -> bool {
        Self::subtype
    }
    fn clone_boxed(&self) -> Option<Box<dyn CoreObject>> {
        Some(Box::new(Self {
            kind: self.kind,
            strength: self.strength,
            ..Self::default()
        }))
    }
    fn deserialize(&mut self, key: u16, reader: &mut BinaryReader<'_>) -> bool {
        match key {
            KIND_PROPERTY_KEY => {
                self.kind = CoreUintType::deserialize(reader);
                true
            }
            STRENGTH_PROPERTY_KEY => {
                self.strength = CoreDoubleType::deserialize(reader);
                true
            }
            _ => false,
        }
    }
}

impl CoreRegistryObject for SpringInterpolator {
    fn as_registry_any(&self) -> &dyn Any {
        self
    }
    fn as_registry_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn is_type_of(&self, key: u16) -> bool {
        Self::subtype(key)
    }
    fn set_uint_with_completion(&mut self, _: CoreField, _: u32, _: &mut PropertySetterCompletion) {
    }
    fn set_string_with_completion(
        &mut self,
        _: CoreField,
        _: String,
        _: &mut PropertySetterCompletion,
    ) {
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
    fn get_uint(&mut self, _: CoreField) -> u32 {
        0
    }
    fn get_string(&mut self, _: CoreField) -> String {
        String::new()
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

impl CoreCapabilities for SpringInterpolator {
    fn lifecycle_on_added_dirty(&mut self, _: &mut dyn CoreContext) -> Option<StatusCode> {
        Some(StatusCode::Ok)
    }
    fn lifecycle_import(&mut self, stack: &mut ImportStack) -> Option<StatusCode> {
        Some(KeyFrameInterpolator::import(self, stack))
    }
    fn keyframe_interpolator_initialize(&mut self) -> bool {
        true
    }
    fn keyframe_interpolator_transform_value(
        &mut self,
        from: f32,
        to: f32,
        factor: f32,
    ) -> Option<f32> {
        Some(self.transform_value(from, to, factor))
    }
    fn keyframe_interpolator_transform(&mut self, factor: f32) -> Option<f32> {
        Some(self.transform(factor))
    }
    fn keyframe_interpolator_is_scripted(&self) -> Option<bool> {
        Some(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The curves written out by hand from the definition, at the strengths
    /// and times the recording tracks' own tests sample.
    fn spring(p: f64, t: f64) -> f64 {
        let k = 8.0 * p + 3.5;
        let u = t * t * (3.0 - 2.0 * t);
        (1.0 - (-k * u).exp() * (1.0 + k * u)) / (1.0 - (-k).exp() * (1.0 + k))
    }

    fn bouncy(p: f64, t: f64) -> f64 {
        let w = 8.0 * p + 10.0;
        let z = (0.55 - 0.35 * p).clamp(0.05, 0.9);
        let wd = w * (1.0 - z * z).sqrt();
        let osc = |t: f64| {
            1.0 - (-z * w * t).exp() * ((wd * t).cos() + z / (1.0 - z * z).sqrt() * (wd * t).sin())
        };
        osc(t) / osc(1.0)
    }

    #[test]
    fn both_curves_match_their_formulas() {
        for p in [0.0_f32, 0.3, 0.6, 1.0] {
            for t in [0.05_f32, 0.25, 0.5, 0.75, 0.95] {
                let (pd, td) = (f64::from(p), f64::from(t));
                assert_eq!(
                    ease(SPRING, p, t),
                    spring(pd, td) as f32,
                    "spring({p}) at {t}"
                );
                assert_eq!(
                    ease(BOUNCY, p, t),
                    bouncy(pd, td) as f32,
                    "bouncy({p}) at {t}"
                );
            }
        }
        // p = 0.6, t = 0.3: k = 8.3 on u = 0.216, worked by hand.
        assert!((ease(SPRING, 0.6, 0.3) - 0.536_2).abs() < 1e-4);
    }

    #[test]
    fn curves_end_on_their_keys_and_clamp() {
        for kind in [SPRING, BOUNCY] {
            for p in [0.0, 0.6, 1.0, f32::NAN, -3.0, 7.0] {
                assert_eq!(ease(kind, p, 0.0), 0.0);
                assert_eq!(ease(kind, p, 1.0), 1.0);
                assert_eq!(ease(kind, p, -0.5), 0.0);
                assert_eq!(ease(kind, p, 1.5), 1.0);
                assert_eq!(ease(kind, p, f32::NAN), 0.0);
            }
            assert_eq!(ease(kind, f32::NAN, 0.4), ease(kind, DEFAULT_STRENGTH, 0.4));
            assert_eq!(ease(kind, -3.0, 0.4), ease(kind, 0.0, 0.4));
            assert_eq!(ease(kind, 7.0, 0.4), ease(kind, 1.0, 0.4));
        }
        assert_eq!(ease(9, 0.6, 0.4), 0.4);
    }

    #[test]
    fn bouncy_passes_its_key_and_spring_does_not() {
        let peak = |kind| {
            (1..1000)
                .map(|i| ease(kind, 0.6, i as f32 / 1000.0))
                .fold(0.0_f32, f32::max)
        };
        assert!(peak(BOUNCY) > 1.05, "{}", peak(BOUNCY));
        assert!(peak(SPRING) <= 1.0, "{}", peak(SPRING));
    }

    #[test]
    fn record_reads_its_fields_and_eases_values() {
        let mut curve = SpringInterpolator::default();
        assert_eq!((curve.kind(), curve.strength()), (SPRING, DEFAULT_STRENGTH));
        let kind = [BOUNCY as u8];
        assert!(curve.deserialize(KIND_PROPERTY_KEY, &mut BinaryReader::new(&kind)));
        let strength = 0.3_f32.to_le_bytes();
        assert!(curve.deserialize(STRENGTH_PROPERTY_KEY, &mut BinaryReader::new(&strength)));
        assert!(!curve.deserialize(1, &mut BinaryReader::new(&[0])));
        assert_eq!((curve.kind(), curve.strength()), (BOUNCY, 0.3));
        assert!(CoreObject::is_type_of(
            &curve,
            KeyFrameInterpolatorBase::TYPE_KEY
        ));
        let value = curve.keyframe_interpolator_transform_value(10.0, 30.0, 0.5);
        assert_eq!(value, Some(10.0 + 20.0 * ease(BOUNCY, 0.3, 0.5)));
        assert_eq!(curve.keyframe_interpolator_is_scripted(), Some(false));
    }
}
