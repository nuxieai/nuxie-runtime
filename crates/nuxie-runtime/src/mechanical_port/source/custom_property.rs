use crate::mechanical_port::source::generated::custom_property_base::CustomPropertyBase;
use crate::mechanical_port::source::{
    core::CoreHandle, core_context::CoreContext, generated::core_registry::CoreRegistry,
    status_code::StatusCode,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum CustomPropertyKind {
    Number,
    Boolean,
    String,
    Color,
    Enumeration,
    Trigger,
}

#[derive(Default)]
pub struct CustomProperty {
    pub base: CustomPropertyBase,
}

impl CustomProperty {
    pub const NO_NAME_ID: u32 = u32::MAX;

    pub fn tags_parent(&self) -> bool {
        self.name_id() != Self::NO_NAME_ID
    }

    pub fn tagging(child: &CoreHandle) -> Option<CoreHandle> {
        (child.is_type_of(CustomPropertyBase::TYPE_KEY)
            && CoreRegistry::get_uint_handle(
                child,
                CustomPropertyBase::NAME_ID_PROPERTY_KEY.into(),
            ) != Some(Self::NO_NAME_ID))
        .then(|| child.clone())
    }

    pub fn kind_handle(property: &CoreHandle) -> CustomPropertyKind {
        use crate::mechanical_port::source::generated::{
            custom_property_boolean_base::CustomPropertyBooleanBase,
            custom_property_color_base::CustomPropertyColorBase,
            custom_property_enum_base::CustomPropertyEnumBase,
            custom_property_string_base::CustomPropertyStringBase,
            custom_property_trigger_base::CustomPropertyTriggerBase,
        };
        match property.with(|p| p.core_type()).unwrap() {
            CustomPropertyBooleanBase::TYPE_KEY => CustomPropertyKind::Boolean,
            CustomPropertyStringBase::TYPE_KEY => CustomPropertyKind::String,
            CustomPropertyColorBase::TYPE_KEY => CustomPropertyKind::Color,
            CustomPropertyEnumBase::TYPE_KEY => CustomPropertyKind::Enumeration,
            CustomPropertyTriggerBase::TYPE_KEY => CustomPropertyKind::Trigger,
            _ => CustomPropertyKind::Number,
        }
    }

    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        if self.tags_parent() {
            if let Some(parent) = self.parent_handle() {
                parent.with_mut(|parent| {
                    if let Some(drawable) = parent.as_drawable_mut() {
                        drawable.mark_has_custom_properties();
                    }
                });
            }
        }
        self.base.base.on_added_clean(context)
    }
}

impl std::ops::Deref for CustomProperty {
    type Target = CustomPropertyBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for CustomProperty {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
