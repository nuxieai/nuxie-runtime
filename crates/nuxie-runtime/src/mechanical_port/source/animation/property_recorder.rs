#[derive(Clone, Debug)]
pub enum RecordedValue {
    Float(f32),
    Int(i32),
    Uint(u32),
    String(String),
    Bool(bool),
}
#[derive(Default)]
pub struct CoreObjectData {
    pub object_id: u32,
    property_keys: Vec<u16>,
}
impl CoreObjectData {
    pub fn new(id: u32) -> Self {
        Self {
            object_id: id,
            property_keys: Vec::new(),
        }
    }
    pub fn add_property_key(&mut self, key: u16) {
        if !self.property_keys.contains(&key) {
            self.property_keys.push(key)
        }
    }
    pub fn property_keys(&mut self) -> &mut Vec<u16> {
        &mut self.property_keys
    }
}
pub trait PropertyRecorderSource {
    fn collect_artboard(&self, recorder: &mut PropertyRecorder);
    fn collect_state_machine(&self, recorder: &mut PropertyRecorder);
    fn property_value(&self, object: u32, key: u16) -> Option<RecordedValue>;
    fn apply_property(&mut self, object: u32, key: u16, value: &RecordedValue);
    fn apply_input(&mut self, name: &str, value: &RecordedValue);
}
#[derive(Default)]
pub struct PropertyRecorder {
    artboard_records: Vec<(u32, u16, RecordedValue)>,
    state_machine_records: Vec<(String, RecordedValue)>,
    core_objects: Vec<CoreObjectData>,
}
impl PropertyRecorder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn clear(&mut self) {
        self.artboard_records.clear()
    }
    pub fn record_artboard(&mut self, source: &dyn PropertyRecorderSource) {
        self.artboard_records.clear();
        self.core_objects.clear();
        source.collect_artboard(self);
        let keys: Vec<_> = self
            .core_objects
            .iter()
            .flat_map(|o| o.property_keys.iter().map(move |k| (o.object_id, *k)))
            .collect();
        for (object, key) in keys {
            if let Some(value) = source.property_value(object, key) {
                self.artboard_records.push((object, key, value));
            }
        }
    }
    pub fn record_state_machine_inputs(&mut self, source: &dyn PropertyRecorderSource) {
        self.state_machine_records.clear();
        source.collect_state_machine(self)
    }
    pub fn add_property_key(&mut self, object: u32, key: u16) {
        if let Some(data) = self.core_objects.iter_mut().find(|v| v.object_id == object) {
            data.add_property_key(key)
        } else {
            let mut data = CoreObjectData::new(object);
            data.add_property_key(key);
            self.core_objects.push(data)
        }
    }
    pub fn record_input(&mut self, name: String, value: RecordedValue) {
        self.state_machine_records.push((name, value))
    }
    pub fn apply_artboard(&self, target: &mut dyn PropertyRecorderSource) {
        for (object, key, value) in &self.artboard_records {
            target.apply_property(*object, *key, value)
        }
    }
    pub fn apply_state_machine(&self, target: &mut dyn PropertyRecorderSource) {
        for (name, value) in &self.state_machine_records {
            target.apply_input(name, value)
        }
    }
    pub fn record_data_binds(&mut self, source: &dyn PropertyRecorderSource) {
        source.collect_artboard(self)
    }
    pub fn record_state_machine(&mut self, source: &dyn PropertyRecorderSource) {
        source.collect_state_machine(self)
    }
    pub fn record_state_machine_input(&mut self, name: String, value: RecordedValue) {
        self.record_input(name, value)
    }
    pub fn record_state_machine_layer(&mut self, source: &dyn PropertyRecorderSource) {
        source.collect_state_machine(self)
    }
    pub fn record_state_machine_layer_state(&mut self, source: &dyn PropertyRecorderSource) {
        source.collect_state_machine(self)
    }
    pub fn record_linear_animation(&mut self, source: &dyn PropertyRecorderSource) {
        source.collect_artboard(self)
    }
    pub fn record_keyed_object(&mut self, source: &dyn PropertyRecorderSource) {
        source.collect_artboard(self)
    }
}
