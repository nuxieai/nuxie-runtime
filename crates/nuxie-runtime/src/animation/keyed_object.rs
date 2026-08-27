// Mechanical owner for pinned `KeyedObject` runtime behavior.
//
// The approved AF-7 own-by-value adaptation performs `import`,
// `onAddedDirty`, and `onAddedClean` while `RuntimeFile` is flattened:
// `keyed_object_importer` validates the retained `LinearAnimation` importer,
// and `build_linear_animations` resolves `objectId`, removes unsupported
// properties, and propagates property/keyframe validation failures before it
// publishes this immutable owner. The live callback and apply traversals stay
// here in their pinned source order.

#[derive(Debug, Clone)]
pub struct RuntimeKeyedObject {
    pub global_id: u32,
    pub object_id: usize,
    pub target_local_id: usize,
    pub keyed_properties: Vec<RuntimeKeyedProperty>,
}

impl RuntimeKeyedObject {
    pub(crate) fn new(global_id: u32, object_id: usize, target_local_id: usize) -> Self {
        Self {
            global_id,
            object_id,
            target_local_id,
            keyed_properties: Vec::new(),
        }
    }

    pub(crate) fn add_keyed_property(&mut self, property: RuntimeKeyedProperty) {
        self.keyed_properties.push(property);
    }

    fn report_keyed_callbacks(
        &self,
        seconds_from: f32,
        seconds_to: f32,
        is_at_start_frame: bool,
        callback_sink: &mut dyn FnMut(RuntimeKeyedCallback, Option<StateMachineReportedEvent>),
    ) {
        for property in &self.keyed_properties {
            // Mirrors CoreRegistry::isCallback(property->propertyKey()).
            if !is_callback_property_key(property.property_key) {
                continue;
            }
            property.report_keyed_callbacks(
                self.target_local_id,
                seconds_from,
                seconds_to,
                is_at_start_frame,
                callback_sink,
            );
        }
    }

    fn apply(
        &self,
        instance: &mut ArtboardInstance,
        seconds: f32,
        mix: f32,
        key_frame_values: RuntimeKeyFrameValueContext<'_>,
        animation_instance: Option<&LinearAnimationInstance>,
    ) -> bool {
        // C++ resolves objectId on every apply and returns for a missing Core.
        // Rust resolves that id to target_local_id before publication, erases
        // invalid owners, and retains immutable artboard slots for the owner's
        // lifetime. The typed setters still return false defensively.
        let mut changed = false;
        for property in &self.keyed_properties {
            if is_callback_property_key(property.property_key) {
                continue;
            }
            changed |= property.apply(
                instance,
                self.target_local_id,
                seconds,
                mix,
                key_frame_values,
                animation_instance,
            );
        }
        changed
    }

    pub fn get_property(&self, index: usize) -> Option<&RuntimeKeyedProperty> {
        self.keyed_properties.get(index)
    }

    pub fn num_keyed_properties(&self) -> usize {
        self.keyed_properties.len()
    }
}
