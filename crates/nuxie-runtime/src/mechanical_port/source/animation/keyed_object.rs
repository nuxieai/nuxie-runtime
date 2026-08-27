use crate::mechanical_port::source::{
    animation::{keyed_callback_reporter::KeyedCallbackReporter, keyed_property::KeyedProperty},
    generated::animation::{
        keyed_object_base::KeyedObjectBase, linear_animation_base::LinearAnimationBase,
    },
    importers::{import_stack::ImportStack, linear_animation_importer::LinearAnimationImporter},
    status_code::StatusCode,
};
pub trait KeyedObjectContext {
    fn resolves_object(&self, id: u32) -> bool;
    fn resolve_object(&mut self, id: u32) -> Option<*mut ()>;
    fn object_supports_property(&self, id: u32, key: u32) -> bool;
    fn overrides_keyed_interpolation(&self, object: *mut (), key: u32) -> bool;
}
#[derive(Default)]
pub struct KeyedObject {
    pub base: KeyedObjectBase,
    keyed_properties: Vec<Box<KeyedProperty>>,
}
impl KeyedObject {
    pub fn add_keyed_property(&mut self, value: Box<KeyedProperty>) {
        self.keyed_properties.push(value);
    }
    pub fn get_property(&self, index: usize) -> Option<&KeyedProperty> {
        self.keyed_properties.get(index).map(Box::as_ref)
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
            if !context.object_supports_property(
                self.base.object_id(),
                self.keyed_properties[index].base.property_key(),
            ) {
                self.keyed_properties.remove(index);
                continue;
            }
            let code = self.keyed_properties[index].on_added_dirty(context);
            if code != StatusCode::Ok {
                return code;
            }
            index += 1;
        }
        StatusCode::Ok
    }
    pub fn on_added_clean(&mut self, context: &mut dyn KeyedObjectContext) -> StatusCode {
        for property in &mut self.keyed_properties {
            property.on_added_clean(context);
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
            if property.is_callback() {
                property.report_keyed_callbacks(
                    reporter,
                    self.base.object_id(),
                    from,
                    to,
                    at_start,
                );
            }
        }
    }
    pub fn apply(
        &mut self,
        artboard: &mut dyn KeyedObjectContext,
        time: f32,
        mix: f32,
        context: *const (),
    ) {
        let Some(object) = artboard.resolve_object(self.base.object_id()) else {
            return;
        };
        for property in &mut self.keyed_properties {
            if !property.is_callback() {
                let override_mix =
                    artboard.overrides_keyed_interpolation(object, property.base.property_key());
                property.apply(object, time, mix, context, override_mix);
            }
        }
    }
    pub fn import(self: Box<Self>, stack: &mut ImportStack) -> StatusCode {
        let raw = Box::into_raw(self);
        let Some(importer) = stack.latest::<LinearAnimationImporter>(LinearAnimationBase::TYPE_KEY)
        else {
            unsafe { drop(Box::from_raw(raw)) };
            return StatusCode::MissingObject;
        };
        importer.add_keyed_object(unsafe { Box::from_raw(raw) });
        unsafe { (*raw).base.base.import(stack) }
    }
}
