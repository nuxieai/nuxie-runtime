pub trait KeyedCallbackReporter {
    fn report_keyed_callback(&mut self, object_id: u32, property_key: u32, elapsed_seconds: f32);
}
