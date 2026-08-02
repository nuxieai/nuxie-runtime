impl ArtboardInstance {
    fn replace_profile_path_prefix(
        &mut self,
        old_prefix: &[crate::ProfilePathSegment],
        new_prefix: &[crate::ProfilePathSegment],
    ) {
        if let Some(suffix) = self.profile_path.strip_prefix(old_prefix) {
            let mut path = Vec::with_capacity(new_prefix.len().saturating_add(suffix.len()));
            path.extend_from_slice(new_prefix);
            path.extend_from_slice(suffix);
            self.profile_path = path;
        }
        for nested in self.nested_artboards.values_mut() {
            nested
                .child
                .replace_profile_path_prefix(old_prefix, new_prefix);
        }
        let list_locals = self
            .component_lists
            .iter()
            .filter_map(|handle| self.component_local_id(*handle))
            .collect::<Vec<_>>();
        for list_local in list_locals {
            let Some(items) = self.component_list_items_mut(list_local) else {
                continue;
            };
            for item in items {
                item.child
                    .replace_profile_path_prefix(old_prefix, new_prefix);
            }
        }
    }
}
