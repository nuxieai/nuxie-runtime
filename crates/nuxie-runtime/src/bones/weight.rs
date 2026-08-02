fn deform_point_from_skin(
    point: (f32, f32),
    indices: u32,
    weights: u32,
    skin_world: Mat2D,
    bone_transforms: &[Mat2D],
) -> Option<(f32, f32)> {
    let mut blended = [0.0; 6];
    for index in 0..4 {
        let shift = index * 8;
        let weight = ((weights >> shift) & 0xff) as u8;
        if weight == 0 {
            continue;
        }
        let bone_index = ((indices >> shift) & 0xff) as usize;
        let bone_transform = bone_transforms.get(bone_index)?;
        let normalized_weight = f32::from(weight) / 255.0;
        for (target, value) in blended.iter_mut().zip(bone_transform.0) {
            // Clang contracts each C++ `accumulator += value * weight` in
            // `Weight::deform`; Rust does not contract implicitly. Preserve
            // that per-site rounding exactly (`src/bones/weight.cpp:35-48`;
            // `docs/PORTING.md` §4.2).
            *target = value.mul_add(normalized_weight, *target);
        }
    }
    let skinned = skin_world.transform_point(point.0, point.1);
    Some(Mat2D(blended).transform_point(skinned.0, skinned.1))
}

impl ArtboardInstance {
    pub(crate) fn runtime_vertex_weight_state(
        &self,
        vertex_local: usize,
    ) -> Option<crate::components::RuntimeWeightState> {
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
    /// buffer. The caller supplies live Vertex points; packed indices/values
    /// are always read from this occurrence's generated storage.
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
            .map(|weight| weight.is_cubic)
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
        let cubic_translations = if is_cubic_weight {
            let (in_point, out_point) = cubic_points.unwrap_or((point, point));
            let in_values = property_key_for_name("CubicWeight", "inValues")
                .and_then(|key| self.uint_property(weight_local, key))
                .and_then(|value| u32::try_from(value).ok())
                .unwrap_or(255);
            let in_indices = property_key_for_name("CubicWeight", "inIndices")
                .and_then(|key| self.uint_property(weight_local, key))
                .and_then(|value| u32::try_from(value).ok())
                .unwrap_or(1);
            let out_values = property_key_for_name("CubicWeight", "outValues")
                .and_then(|key| self.uint_property(weight_local, key))
                .and_then(|value| u32::try_from(value).ok())
                .unwrap_or(255);
            let out_indices = property_key_for_name("CubicWeight", "outIndices")
                .and_then(|key| self.uint_property(weight_local, key))
                .and_then(|value| u32::try_from(value).ok())
                .unwrap_or(1);
            let Some(in_translation) = deform_point_from_skin(
                in_point,
                in_indices,
                in_values,
                skin_state.world_transform,
                &skin_state.bone_transforms,
            ) else {
                return false;
            };
            let Some(out_translation) = deform_point_from_skin(
                out_point,
                out_indices,
                out_values,
                skin_state.world_transform,
                &skin_state.bone_transforms,
            ) else {
                return false;
            };
            Some((in_translation, out_translation))
        } else {
            None
        };

        let state = self
            .objects
            .component_mut(weight)
            .and_then(|component| component.concrete.weight.as_mut())
            .expect("validated Weight handle owns Weight state");
        state.translation = translation;
        if let Some((in_translation, out_translation)) = cubic_translations {
            state.in_translation = in_translation;
            state.out_translation = out_translation;
        }
        true
    }
}
