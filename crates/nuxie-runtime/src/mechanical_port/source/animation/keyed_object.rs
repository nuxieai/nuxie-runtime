//! Runtime KeyedObject owner translated against pinned Rive 160085c654874d35.

use crate::mechanical_port::source::{
    animation::{
        interpolating_keyframe::KeyFrameValueContext,
        keyed_callback_reporter::KeyedCallbackReporter, keyed_property::KeyedProperty,
        keyframe_cells::{KeyedObjectCells, KeyedPropertyCells, PropertyEntry},
        keyframe_interpolator::KeyFrameInterpolator,
    },
    core::CoreHandle,
    core_context::CoreContext,
    generated::animation::{
        keyed_object_base::KeyedObjectBase, linear_animation_base::LinearAnimationBase,
    },
    importers::{import_stack::ImportStack, linear_animation_importer::LinearAnimationImporter},
    lazy_vector::LazyVector,
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
    keyed_properties: LazyVector<CoreHandle>,
}
impl KeyedObject {
    pub fn add_keyed_property(&mut self, value: CoreHandle) {
        let cells = value
            .with_downcast::<KeyedProperty, _>(|property| std::rc::Rc::clone(property.base.cells()))
            .expect("KeyedObject retains KeyedProperty occurrences");
        self.add_keyed_property_with_cells(value, cells);
    }

    /// Add a keyed property the caller already borrows, with its shared cells.
    pub(crate) fn add_keyed_property_with_cells(
        &mut self,
        value: CoreHandle,
        cells: std::rc::Rc<KeyedPropertyCells>,
    ) {
        self.base.cells().push_property(PropertyEntry {
            handle: value.clone(),
            cells,
        });
        self.keyed_properties.push_back(value);
    }
    pub fn keyed_properties(&self) -> &[CoreHandle] {
        self.keyed_properties.view()
    }
    pub fn get_property(&self, index: usize) -> Option<CoreHandle> {
        self.keyed_properties.view().get(index).cloned()
    }
    pub fn num_keyed_properties(&self) -> usize {
        self.keyed_properties.size()
    }
    pub fn on_added_dirty(&mut self, context: &mut dyn KeyedObjectContext) -> StatusCode {
        if !context.resolves_object(self.base.object_id()) {
            return StatusCode::MissingObject;
        }
        let core_object = context.resolve_object(self.base.object_id());
        let mut index = 0;
        while index < self.keyed_properties.size() {
            let property_key = self.keyed_properties.view()[index]
                .with_downcast::<KeyedProperty, _>(|property| property.base.property_key());
            let Some(property_key) = property_key else {
                return StatusCode::MissingObject;
            };
            if !context.object_supports_property(self.base.object_id(), property_key) {
                self.keyed_properties.remove(index);
                self.base.cells().remove_property(index);
                continue;
            }
            if property_key == u32::from(crate::source::generated::layout_component_base::LayoutComponentBase::CLIP_PROPERTY_KEY) {
                if let Some(object) = &core_object {
                    object.with_mut(|object| {
                        if let Some(layout) = object.as_layout_component_mut() {
                            layout.mark_clip_may_be_dynamic();
                        }
                    });
                }
            }
            let code = self.keyed_properties.view()[index]
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
        for property in self.keyed_properties.iter() {
            property
                .with_downcast_mut::<KeyedProperty, _>(|property| property.on_added_clean(context));
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
        for property in self.keyed_properties.iter() {
            let is_callback = property
                .with_downcast::<KeyedProperty, _>(KeyedProperty::is_callback)
                .expect("KeyedObject retains KeyedProperty occurrences");
            if is_callback {
                KeyedProperty::report_keyed_callbacks_occurrence(
                    property,
                    reporter,
                    self.base.object_id(),
                    from,
                    to,
                    at_start,
                );
            }
        }
    }
    /// Enter through an occurrence when callbacks can edit this keyed owner.
    /// objectId is read for each property, at the same point as the C++ call.
    pub(crate) fn report_keyed_callbacks_occurrence(
        owner: &CoreHandle,
        reporter: &mut dyn KeyedCallbackReporter,
        from: f32,
        to: f32,
        at_start: bool,
    ) {
        // The property snapshot, each property's key and the per-callback
        // object id read through the shared cells, as fresh as before.
        let cells = owner
            .with_downcast::<Self, _>(|object| std::rc::Rc::clone(object.base.cells()))
            .expect("callback traversal retains its KeyedObject");
        Self::report_keyed_callbacks_cells(&cells, reporter, from, to, at_start);
    }

    /// report_keyed_callbacks_occurrence for a keyed object entered through
    /// the shared cells its animation recorded at import.
    pub(crate) fn report_keyed_callbacks_cells(
        cells: &KeyedObjectCells,
        reporter: &mut dyn KeyedCallbackReporter,
        from: f32,
        to: f32,
        at_start: bool,
    ) {
        let properties = cells.properties();
        for property in properties.iter() {
            if !crate::mechanical_port::source::generated::core_registry::CoreRegistry::is_callback(
                property.cells.property_key(),
            ) {
                continue;
            }
            KeyedProperty::report_keyed_callbacks_occurrence(
                &property.handle,
                reporter,
                cells.object_id(),
                from,
                to,
                at_start,
            );
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
        let interpolator_host = KeyFrameInterpolator::host_from(object.clone());
        for property in self.keyed_properties.iter() {
            let is_callback = property
                .with_downcast::<KeyedProperty, _>(KeyedProperty::is_callback)
                .expect("KeyedObject retains KeyedProperty occurrences");
            if !is_callback {
                KeyedProperty::apply_occurrence_with_interpolator_host(
                    property,
                    object.clone(),
                    time,
                    mix,
                    context,
                    artboard,
                    interpolator_host.as_ref(),
                );
            }
        }
    }

    /// Match source application while letting host interpolation hooks edit
    /// this definition: only occurrence identities cross the hook boundary.
    pub(crate) fn apply_occurrence(
        owner: &CoreHandle,
        artboard: &mut dyn KeyedObjectContext,
        time: f32,
        mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
    ) {
        // The id, the target resolution and then the property snapshot are
        // read in that order through the shared cells; the open resolution
        // may edit this keyed owner.
        let cells = owner
            .with_downcast::<Self, _>(|object| std::rc::Rc::clone(object.base.cells()))
            .expect("animation application retains its KeyedObject");
        Self::apply_cells(&cells, artboard, time, mix, context);
    }

    /// apply_occurrence for a keyed object entered through the shared cells
    /// its animation recorded at import.
    pub(crate) fn apply_cells(
        cells: &KeyedObjectCells,
        artboard: &mut dyn KeyedObjectContext,
        time: f32,
        mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
    ) {
        let Some(object) = artboard.resolve_object(cells.object_id()) else {
            return;
        };
        // InterpolatorHost::from, once for every property of this target.
        let interpolator_host = KeyFrameInterpolator::host_of(&object);
        let properties = cells.properties();
        for property in properties.iter() {
            if !crate::mechanical_port::source::generated::core_registry::CoreRegistry::is_callback(
                property.cells.property_key(),
            ) {
                KeyedProperty::apply_cells(
                    &property.cells,
                    &object,
                    time,
                    mix,
                    context,
                    artboard,
                    interpolator_host.as_ref(),
                );
            }
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
        importer.add_keyed_object_with_cells(this, std::rc::Rc::clone(self.base.cells()));
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
