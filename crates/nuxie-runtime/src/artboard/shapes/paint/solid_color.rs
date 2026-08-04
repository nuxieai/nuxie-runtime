use std::sync::Arc;

use super::ArtboardInstance;

impl ArtboardInstance {
    pub(crate) fn solid_color_value(&self, local_id: usize) -> Option<u32> {
        self.objects.solid_color_value(local_id)
    }

    pub(in crate::artboard) fn settle_runtime_solid_color_callback(
        &self,
        local_id: usize,
        value: u32,
    ) {
        let Some(context) = self.build_context.as_ref() else {
            return;
        };
        let Ok(graph_global_id) = usize::try_from(self.graph_global_id) else {
            return;
        };
        let Some(graph_index) = context
            .artboard_index_by_global
            .get(graph_global_id)
            .copied()
            .flatten()
        else {
            return;
        };
        let graphs = Arc::clone(&context.artboards);
        let Some(graph) = graphs.get(graph_index) else {
            return;
        };
        self.settle_runtime_solid_color_callback_with_graph(local_id, value, graph);
    }
}
