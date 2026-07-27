//! Direct Rust owner for pinned C++ `include/rive/shapes/vertex.hpp` and
//! `src/shapes/vertex.cpp`.

use crate::ArtboardInstance;
use crate::bones::weight::{RuntimeWeightState, deform_point_from_skin};
use crate::properties::property_key_for_name;

impl ArtboardInstance {
    pub(crate) fn runtime_vertex_weight_state(
        &self,
        vertex_local: usize,
    ) -> Option<RuntimeWeightState> {
        let vertex = self.component_handle(vertex_local)?;
        let weight = self
            .objects
            .component(vertex)?
            .concrete
            .vertex
            .as_ref()?
            .weight?;
        self.objects.component(weight)?.concrete.weight
    }

    /// Settle one retained Weight/CubicWeight from the Skin-owned transform
    /// buffer. The base point follows `Vertex::deform`; the derived in/out
    /// points delegate to the focused CubicVertex owner.
    pub(crate) fn deform_runtime_vertex_weight(
        &mut self,
        vertex_local: usize,
        point: (f32, f32),
        cubic_points: Option<((f32, f32), (f32, f32))>,
    ) -> bool {
        let Some(vertex) = self.component_handle(vertex_local) else {
            return false;
        };
        let Some(weight) = self
            .objects
            .component(vertex)
            .and_then(|component| component.concrete.vertex.as_ref())
            .and_then(|vertex| vertex.weight)
        else {
            return false;
        };
        let Some(skinnable) = self
            .objects
            .component(vertex)
            .and_then(|component| component.parent)
        else {
            return false;
        };
        let Some(skin) = self
            .objects
            .component(skinnable)
            .and_then(|component| component.concrete.skinnable.as_ref())
            .and_then(|skinnable| skinnable.skin)
        else {
            return false;
        };
        let Some(weight_local) = self.objects.component_local_id(weight) else {
            return false;
        };
        let Some(is_cubic_weight) = self
            .objects
            .component(weight)
            .and_then(|component| component.concrete.weight.as_ref())
            .map(|weight| weight.cubic.is_some())
        else {
            return false;
        };
        let values = property_key_for_name("Weight", "values")
            .and_then(|key| self.uint_property(weight_local, key))
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(255);
        let indices = property_key_for_name("Weight", "indices")
            .and_then(|key| self.uint_property(weight_local, key))
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(1);

        let Some(skin_state) = self
            .objects
            .component(skin)
            .and_then(|component| component.concrete.skin.as_ref())
        else {
            return false;
        };
        let Some(translation) = deform_point_from_skin(
            point,
            indices,
            values,
            skin_state.world_transform,
            &skin_state.bone_transforms,
        ) else {
            return false;
        };
        let cubic_state = if is_cubic_weight {
            let (in_point, out_point) = cubic_points.unwrap_or((point, point));
            let Some(cubic_state) = super::cubic_vertex::deform_weight(
                self,
                weight_local,
                in_point,
                out_point,
                skin_state.world_transform,
                &skin_state.bone_transforms,
            ) else {
                return false;
            };
            Some(cubic_state)
        } else {
            None
        };

        let state = self
            .objects
            .component_mut(weight)
            .and_then(|component| component.concrete.weight.as_mut())
            .expect("validated Weight handle owns Weight state");
        state.translation = translation;
        if let Some(cubic_state) = cubic_state {
            state.cubic = Some(cubic_state);
        }
        true
    }
}
