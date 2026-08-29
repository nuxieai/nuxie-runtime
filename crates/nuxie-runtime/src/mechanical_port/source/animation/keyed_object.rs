use crate::mechanical_port::source::{
    animation::{
        interpolating_keyframe::KeyFrameValueContext,
        keyed_callback_reporter::KeyedCallbackReporter, keyed_property::KeyedProperty,
    },
    core::CoreHandle,
    core_context::CoreContext,
    generated::animation::{
        keyed_object_base::KeyedObjectBase, linear_animation_base::LinearAnimationBase,
    },
    importers::{import_stack::ImportStack, linear_animation_importer::LinearAnimationImporter},
    status_code::StatusCode,
};
pub trait KeyedObjectContext: CoreContext {
    fn resolves_object(&self, id: u32) -> bool;
    fn resolve_object(&mut self, id: u32) -> Option<CoreHandle>;
    fn object_supports_property(&self, id: u32, key: u32) -> bool;
    fn overrides_keyed_interpolation(&self, object: &CoreHandle, key: u32) -> bool;
}
#[derive(Default)]
pub struct KeyedObject {
    pub base: KeyedObjectBase,
    keyed_properties: Vec<CoreHandle>,
}
impl KeyedObject {
    pub fn add_keyed_property(&mut self, value: CoreHandle) {
        self.keyed_properties.push(value);
    }
    pub fn keyed_properties(&self) -> &[CoreHandle] {
        &self.keyed_properties
    }
    pub fn get_property(&self, index: usize) -> Option<CoreHandle> {
        self.keyed_properties.get(index).cloned()
    }
    pub fn num_keyed_properties(&self) -> usize {
        self.keyed_properties.len()
    }
    pub fn on_added_dirty(&mut self, context: &mut dyn KeyedObjectContext) -> StatusCode {
        if !context.resolves_object(self.base.object_id()) {
            return StatusCode::MissingObject;
        }
        let mut index = 0;
        while index < self.keyed_properties.len() {
            let property_key = self.keyed_properties[index]
                .with_downcast::<KeyedProperty, _>(|property| property.base.property_key());
            let Some(property_key) = property_key else {
                return StatusCode::MissingObject;
            };
            if !context.object_supports_property(self.base.object_id(), property_key) {
                self.keyed_properties.remove(index);
                continue;
            }
            let code = self.keyed_properties[index]
                .with_downcast_mut::<KeyedProperty, _>(|property| property.on_added_dirty(context))
                .unwrap_or(StatusCode::MissingObject);
            if code != StatusCode::Ok {
                return code;
            }
            index += 1;
        }
        StatusCode::Ok
    }
    pub fn on_added_clean(&mut self, context: &mut dyn KeyedObjectContext) -> StatusCode {
        for property in &self.keyed_properties {
            let code = property
                .with_downcast_mut::<KeyedProperty, _>(|property| property.on_added_clean(context))
                .unwrap_or(StatusCode::MissingObject);
            if code != StatusCode::Ok {
                return code;
            }
        }
        StatusCode::Ok
    }
    pub fn report_keyed_callbacks(
        &self,
        reporter: &mut dyn KeyedCallbackReporter,
        from: f32,
        to: f32,
        at_start: bool,
    ) {
        for property in &self.keyed_properties {
            property.with_downcast::<KeyedProperty, _>(|property| {
                if property.is_callback() {
                    property.report_keyed_callbacks(
                        reporter,
                        self.base.object_id(),
                        from,
                        to,
                        at_start,
                    );
                }
            });
        }
    }
    pub fn apply(
        &mut self,
        artboard: &mut dyn KeyedObjectContext,
        time: f32,
        mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
    ) {
        let Some(object) = artboard.resolve_object(self.base.object_id()) else {
            return;
        };
        for property in &self.keyed_properties {
            property.with_downcast::<KeyedProperty, _>(|property| {
                if !property.is_callback() {
                    let override_mix = artboard
                        .overrides_keyed_interpolation(&object, property.base.property_key());
                    property.apply(object.clone(), time, mix, context, override_mix);
                }
            });
        }
    }
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = stack.latest::<LinearAnimationImporter>(LinearAnimationBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        let Some(this) = self.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        importer.add_keyed_object(this);
        self.base.base.import(stack)
    }
}

impl std::ops::Deref for KeyedObject {
    type Target = KeyedObjectBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for KeyedObject {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
