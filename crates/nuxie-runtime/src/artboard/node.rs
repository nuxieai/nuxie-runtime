use super::ArtboardInstance;
use crate::components::Mat2D;

impl ArtboardInstance {
    pub(crate) fn runtime_node_computed_local_transform(&self, local_id: usize) -> Option<Mat2D> {
        let handle = self.component_handle(local_id)?;
        let component = self.objects.component(handle)?;
        let node = component.concrete.node.as_ref()?;
        let parent_world = component
            .parent_transform
            .and_then(|parent| self.objects.component(parent))
            .map(|parent| parent.transform.world_transform);
        Some(node.computed_local_transform(parent_world, component.transform.world_transform))
    }
}
