use super::ArtboardInstance;

impl ArtboardInstance {
    pub(crate) fn bone_length(&self, local_id: usize) -> Option<f32> {
        self.component(local_id)?.concrete.bone.as_ref()?;
        self.objects
            .double_property_by_name(local_id, "length")
            .or(Some(0.0))
    }
}
