#[derive(Debug, Clone)]
pub struct RuntimeKeyedObject {
    pub global_id: u32,
    pub object_id: usize,
    pub target_local_id: usize,
    pub keyed_properties: Vec<RuntimeKeyedProperty>,
}
