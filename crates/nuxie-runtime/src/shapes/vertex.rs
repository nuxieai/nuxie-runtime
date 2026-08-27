use crate::{
    ArtboardInstance,
    components::{ComponentHandle, RuntimeWeightState},
    properties::property_key_for_name,
};

/// Retained `Vertex::m_Weight`. The occurrence-local handle is Rust's arena
/// equivalent of the source pointer and defaults to null.
#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeVertexState {
    pub(crate) weight: Option<ComponentHandle>,
}

impl ArtboardInstance {
    fn runtime_vertex_weight_handle(&self, vertex_local: usize) -> Option<ComponentHandle> {
        let vertex = self.component_handle(vertex_local)?;
        self.objects
            .component(vertex)?
            .concrete
            .vertex
            .as_ref()?
            .weight
    }

    /// `Vertex::hasWeight` and the typed `weight<T>` accessor share this
    /// occurrence-owned relationship. Import-time `Weight::onAddedDirty`
    /// installs the handle after validating the concrete parent.
    pub(crate) fn runtime_vertex_has_weight(&self, vertex_local: usize) -> bool {
        self.runtime_vertex_weight_handle(vertex_local).is_some()
    }

    pub(crate) fn runtime_vertex_weight_state(
        &self,
        vertex_local: usize,
    ) -> Option<RuntimeWeightState> {
        let weight = self.runtime_vertex_weight_handle(vertex_local)?;
        self.objects.component(weight)?.concrete.weight
    }

    /// Direct `Vertex::deform`: settle the linked Weight translation from the
    /// authored point, world transform, packed indices/values, and the Skin's
    /// bone-transform buffer. Cubic control translations are dispatched by
    /// the same virtual call boundary in the retained Rust representation.
    pub(crate) fn deform_runtime_vertex_weight(
        &mut self,
        vertex_local: usize,
        point: (f32, f32),
        cubic_points: Option<((f32, f32), (f32, f32))>,
    ) -> bool {
        self.deform_linked_vertex_weight(vertex_local, point, cubic_points)
    }
}

/// Direct `Vertex::renderTranslation`: a linked Weight always owns the render
/// position; otherwise the generated `x`/`y` properties are returned.
pub(crate) fn render_translation(
    artboard: &ArtboardInstance,
    local_id: usize,
    x: f32,
    y: f32,
) -> Option<(f32, f32)> {
    if artboard.runtime_vertex_has_weight(local_id) {
        return Some(artboard.runtime_vertex_weight_state(local_id)?.translation);
    }
    Some((x, y))
}

/// Direct `Vertex::{xChanged,yChanged}`.
pub(crate) fn position_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    type_name: &str,
    property_key: u16,
) -> Option<bool> {
    if !["x", "y"]
        .into_iter()
        .any(|name| property_key_for_name(type_name, name) == Some(property_key))
    {
        return None;
    }
    Some(super::path_vertex::mark_geometry_dirty(artboard, local_id))
}
