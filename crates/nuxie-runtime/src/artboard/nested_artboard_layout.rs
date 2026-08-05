use super::*;

impl ArtboardInstance {
    pub(crate) fn mounted_layout_host_is_fenced(&self, host_local_id: usize) -> bool {
        self.consumed_mounted_layout_hosts.contains(&host_local_id)
    }

    pub(in crate::artboard) fn runtime_nested_artboard_layout_bounds_frame(
        &mut self,
    ) -> RuntimeNestedLayoutBoundsFrame {
        let key = RuntimeNestedLayoutBoundsCacheKey {
            graph_global_id: self.graph_global_id,
            layout_revision: self.layout_revision,
        };
        if self
            .nested_layout_bounds
            .as_ref()
            .is_none_or(|frame| frame.key != key)
        {
            self.nested_layout_bounds = Some(RuntimeNestedLayoutBoundsFrame {
                key,
                bounds: Arc::new(self.compute_runtime_nested_artboard_layout_bounds()),
            });
        }

        self.nested_layout_bounds
            .as_ref()
            .expect("nested layout bounds frame was just populated")
            .clone()
    }

    pub(in crate::artboard) fn compute_runtime_nested_artboard_layout_bounds(
        &self,
    ) -> Option<BTreeMap<usize, RuntimeLayoutBounds>> {
        if !self
            .nested_artboard_locals
            .iter()
            .any(|local_id| is_nested_artboard_layout(self.component(*local_id)))
        {
            return None;
        }
        let context = self.build_context.as_ref()?;
        let runtime = context.file.clone();
        let graph = context
            .artboards
            .iter()
            .find(|graph| graph.global_id == self.graph_global_id)?
            .clone();
        self.runtime_taffy_layout_bounds(&graph, Some(runtime.as_ref()))
    }

    pub(in crate::artboard) fn capture_initial_nested_artboard_layout_paint_frame(
        &mut self,
        host_local_id: usize,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
        mut paint_evaluation: ArtboardInstance,
    ) {
        if !is_nested_artboard_layout(self.component(host_local_id)) {
            return;
        }
        let Some(bounds) = layout_bounds.and_then(|bounds| bounds.get(&host_local_id).copied())
        else {
            return;
        };
        // C++ configures paints on this one mounted occurrence before
        // NestedArtboardLayout transfers its constraint space. Evaluate that
        // source-side shader state only on a script-free temporary occurrence.
        paint_evaluation.detach_initial_nested_layout_paint_binding_contexts();
        paint_evaluation.set_artboard_dimensions(bounds.width, bounds.height);
        if let Some(width_key) = property_key_for_name("LayoutComponent", "width") {
            paint_evaluation.set_double_property(0, width_key, bounds.width);
        }
        if let Some(height_key) = property_key_for_name("LayoutComponent", "height") {
            paint_evaluation.set_double_property(0, height_key, bounds.height);
        }
        paint_evaluation.update_components();
        let before_bind = paint_evaluation.capture_initial_nested_layout_paint_frame();
        paint_evaluation.advance_artboard_data_binds();
        paint_evaluation.update_components();
        let frame = paint_evaluation.capture_initial_nested_layout_paint_frame();
        if !frame.changed_from(&before_bind) {
            return;
        }
        if let Some(nested) = self.nested_artboards.get_mut(&host_local_id)
            && !nested.layout_data_transferred
            && nested.initial_layout_paint_frame.borrow().is_none()
        {
            nested.initial_layout_paint_frame.replace(Some(frame));
        }
    }

    pub(in crate::artboard) fn apply_nested_artboard_layout_bounds_after_parent_solve(
        &mut self,
    ) -> bool {
        if !self
            .nested_artboard_locals
            .iter()
            .any(|host_local_id| is_nested_artboard_layout(self.component(*host_local_id)))
        {
            return false;
        }
        let layout_frame = self.runtime_nested_artboard_layout_bounds_frame();
        let mut changed = false;
        for index in 0..self.nested_artboard_locals.len() {
            let host_local_id = self.nested_artboard_locals[index];
            if self
                .component(host_local_id)
                .is_some_and(RuntimeComponent::is_collapsed)
            {
                continue;
            }
            changed |= self.apply_nested_artboard_layout_bounds(
                host_local_id,
                layout_frame.bounds.as_ref().as_ref(),
                layout_frame.key,
            );
        }
        changed
    }

    pub(crate) fn acknowledge_consumed_mounted_layout_generation(
        &mut self,
        host_local_ids: &BTreeSet<usize>,
    ) {
        if host_local_ids.is_empty() {
            return;
        }
        let parent_layout = RuntimeNestedLayoutBoundsCacheKey {
            graph_global_id: self.graph_global_id,
            layout_revision: self.layout_revision,
        };
        let retained_transfers = host_local_ids
            .iter()
            .filter_map(|host_local_id| {
                self.previous_nested_layout_transfers
                    .remove(host_local_id)
                    .or_else(|| {
                        self.nested_artboards.get(host_local_id).and_then(|nested| {
                            nested
                                .layout_data_transfer_key
                                .map(|key| (key, nested.child.layout_constraint_bounds.clone()))
                        })
                    })
                    .map(|(key, child_bounds)| (*host_local_id, key, child_bounds))
            })
            .collect::<Vec<_>>();
        // The same-layer comparator has consumed this authored global write.
        // Preserve the already-transferred Yoga frame while advancing only
        // its generation fence; a later independent layout write will still
        // produce a new key and solve normally.
        if let Some(frame) = self.nested_layout_bounds.as_mut() {
            frame.key = parent_layout;
            if let Some(bounds) = Arc::make_mut(&mut frame.bounds).as_mut() {
                for (host_local_id, key, _) in &retained_transfers {
                    bounds.insert(*host_local_id, key.assigned_bounds);
                }
            }
        }
        for (host_local_id, mut key, child_bounds) in retained_transfers {
            let retained_child_bounds = child_bounds.clone();
            let hosted_bounds = self
                .component_parent_local(host_local_id)
                .and_then(|parent_local| {
                    self.nested_layout_bounds
                        .as_ref()
                        .and_then(|frame| frame.bounds.as_ref().as_ref())
                        .and_then(|bounds| bounds.get(&parent_local).copied())
                })
                .map_or(key.assigned_bounds, |parent| RuntimeLayoutBounds {
                    x: key.assigned_bounds.x - parent.x,
                    y: key.assigned_bounds.y - parent.y,
                    width: key.assigned_bounds.width,
                    height: key.assigned_bounds.height,
                });
            let Some(nested) = self.nested_artboards.get_mut(&host_local_id) else {
                continue;
            };
            nested.child.suppress_mounted_component_list_layout_updates = true;
            nested.child.layout_constraint_bounds = child_bounds.clone();
            nested.child.solved_layout_bounds = child_bounds;
            nested.child.added_to_host();
            nested
                .child
                .retain_runtime_layout_component_bounds(0, hosted_bounds, None);
            key.parent_layout = parent_layout;
            key.child_layout_revision = nested.child.layout_revision();
            nested.layout_data_transfer_key = Some(key);
            self.previous_nested_layout_transfers
                .insert(host_local_id, (key, retained_child_bounds));
        }
        self.consumed_mounted_layout_hosts
            .extend(host_local_ids.iter().copied());
    }

    pub(in crate::artboard) fn apply_nested_artboard_layout_bounds(
        &mut self,
        host_local_id: usize,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
        parent_layout: RuntimeNestedLayoutBoundsCacheKey,
    ) -> bool {
        if !is_nested_artboard_layout(self.component(host_local_id)) {
            return false;
        }
        let Some(bounds) = layout_bounds.and_then(|bounds| bounds.get(&host_local_id).copied())
        else {
            return false;
        };
        let hosted_bounds = self
            .component_parent_local(host_local_id)
            .and_then(|parent_local| layout_bounds.and_then(|all| all.get(&parent_local).copied()))
            .map_or(bounds, |parent| RuntimeLayoutBounds {
                x: bounds.x - parent.x,
                y: bounds.y - parent.y,
                width: bounds.width,
                height: bounds.height,
            });
        let transferred_intrinsic_size = self
            .nested_artboards
            .get(&host_local_id)
            .filter(|nested| !nested.layout_data_transferred)
            .and_then(|nested| {
                nested
                    .child
                    .runtime_file()
                    .zip(nested.child.runtime_graph())
                    .and_then(|(runtime, graph)| {
                        nested
                            .child
                            .runtime_taffy_layout_bounds(graph, Some(runtime))
                    })
                    .and_then(|bounds| bounds.get(&0).copied())
            })
            .map(|bounds| (Some(bounds.width), Some(bounds.height)));
        let hug_axis_changed = |property_name: &str, intrinsic: Option<f32>, assigned: f32| {
            property_key_for_name("NestedArtboardLayout", property_name)
                .and_then(|key| self.uint_property(host_local_id, key))
                == Some(2)
                && intrinsic.is_some_and(|intrinsic| (intrinsic - assigned).abs() > 1.0e-4)
        };
        if transferred_intrinsic_size.is_some_and(|(width, height)| {
            hug_axis_changed("instanceWidthScaleType", width, bounds.width)
                || hug_axis_changed("instanceHeightScaleType", height, bounds.height)
        }) {
            // A nested child can settle its own transferred descendants after
            // the parent's first detached measurement. C++ still has one
            // shared Yoga tree, so that newer intrinsic participates before
            // the host receives its first layout result. Publish the host
            // again and defer the transfer one pass; otherwise an animating
            // root would visibly interpolate from the stale preliminary size.
            if let Some(nested) = self.nested_artboards.get(&host_local_id) {
                nested
                    .transferred_hug_size
                    .set(transferred_intrinsic_size.unwrap());
            }
            return crate::layout_node_provider::mark_layout_node_dirty(self, host_local_id);
        }
        let Some(nested) = self.nested_artboards.get_mut(&host_local_id) else {
            return false;
        };

        if self.consumed_mounted_layout_hosts.contains(&host_local_id) {
            if let Some(key) = nested.layout_data_transfer_key.as_mut() {
                let consumed_generation_arrived =
                    key.child_layout_revision != nested.child.layout_revision();
                key.parent_layout = parent_layout;
                if consumed_generation_arrived {
                    if let Some((_, retained_bounds)) =
                        self.previous_nested_layout_transfers.remove(&host_local_id)
                    {
                        nested.child.layout_constraint_bounds = retained_bounds.clone();
                        nested.child.solved_layout_bounds = retained_bounds;
                        // The child draw cache may already have memoized the
                        // consumed detached solve. Publish the restored
                        // parent-owned snapshot as a new local generation.
                        nested.child.mark_layout_changed();
                    }
                    self.consumed_mounted_layout_hosts.remove(&host_local_id);
                    nested.child.suppress_mounted_component_list_layout_updates = false;
                }
                key.child_layout_revision = nested.child.layout_revision();
            }
            return false;
        }

        let first_transfer = !nested.layout_data_transferred;
        if let Some(intrinsic_size) = transferred_intrinsic_size {
            nested.transferred_hug_size.set(intrinsic_size);
        }
        let refresh_constraint_bounds = nested.layout_data_transfer_key.is_none_or(|key| {
            key.parent_layout != parent_layout
                || key.assigned_bounds != bounds
                || key.child_layout_revision != nested.child.layout_revision
        });
        if !first_transfer && refresh_constraint_bounds {
            if let Some(key) = nested.layout_data_transfer_key {
                self.previous_nested_layout_transfers.insert(
                    host_local_id,
                    (key, nested.child.layout_constraint_bounds.clone()),
                );
            }
        }
        let mut changed = nested
            .child
            .set_artboard_dimensions(bounds.width, bounds.height);
        if first_transfer {
            // The recursive host bind above has applied the rounded initial
            // values but has not yet consumed their component dirt. Settle
            // that unconstrained component state before taking the one Yoga
            // layout snapshot owned by the parent.
            changed |= nested.child.update_components().did_update;
            // The standalone settle above must not consume the mounted
            // Artboard's first-host placement. C++ keeps m_justAddedToHost
            // armed until the transferred Yoga node receives its first
            // parent-owned result (`artboard.cpp:1061-1073`;
            // `layout_component.cpp:1117-1137`).
            nested.child.added_to_host();
            nested.child.layout_node_owned_by_host = true;
            // The standalone pre-transfer settle measured list rows in the
            // child's old root space. Re-arm exactly one hosted list tail so
            // the first parent-owned Yoga snapshot establishes their mounted
            // transforms; later child updates do not run an independent
            // layout solve while `takeLayoutData()` ownership is active.
            let list_locals = nested.child.component_list_locals().collect::<Vec<_>>();
            for list_local in list_locals {
                if let Some(items) = nested.child.component_list_items_mut(list_local) {
                    for item in items {
                        item.settled_layout_size.set(None);
                    }
                }
            }
        }

        // Match NestedArtboardLayout's mounted ordering: the constraint space
        // exists before its root LayoutComponent width/height dirt is raised.
        // Reversing these two operations changes the first layout solve.
        if refresh_constraint_bounds {
            nested.child.refresh_layout_constraint_bounds();
            // C++ calculates the transferred child node and its descendants
            // inside the parent's Yoga tree, then publishes every provider's
            // new bounds in the same `updateLayoutBounds` traversal. Rust's
            // decomposed child graph has already consumed its dirty-layout
            // membership, so publish the host-owned solve here directly.
            nested.child.retain_host_owned_layout_constraint_bounds();
            changed = true;
        } else {
            changed |= !nested.child.layout_constraint_bounds_enabled;
            nested.child.enable_layout_constraint_bounds();
        }
        if let Some(width_key) = property_key_for_name("LayoutComponent", "width") {
            changed |= nested.child.set_double_property(0, width_key, bounds.width);
        }
        if let Some(height_key) = property_key_for_name("LayoutComponent", "height") {
            changed |= nested
                .child
                .set_double_property(0, height_key, bounds.height);
        }
        // `NestedArtboardLayout::layoutNode` transfers the mounted Artboard's
        // root Yoga node into the parent tree. Retain that node's parent-local
        // left/top on the child occurrence; C++ draw later reads
        // Artboard::layoutX/layoutY rather than resolving the parent Yoga tree
        // a second time. The transfer consumes the root node's first Yoga
        // result; later child-local constraint refreshes must not republish a
        // fresh Taffy root position into that mounted owner
        // (`nested_artboard_layout.cpp:24-42,53-78`;
        // `artboard.cpp:1245-1253,1332-1341`).
        nested
            .child
            .retain_runtime_layout_component_bounds(0, hosted_bounds, None);
        // Release the detached-update suppression only after the parent Yoga
        // assignment arrives, so the same-pass child update settles mounted
        // component-list rows in the accepted constraint frame.
        nested.child.suppress_mounted_component_list_layout_updates = false;
        nested.layout_data_transferred = true;
        if changed {
            nested.child.update_pass();
        }
        nested
            .transferred_hug_layout_generation
            .set(nested.child.runtime_transferred_layout_generation());
        // Record after assigned-root writes and their child update pass. Those
        // writes dirty the transferred root node themselves; only a later
        // child layout generation should emulate C++ `markHostingLayoutDirty`
        // and request another parent-owned constraint refresh.
        let transfer_key = RuntimeNestedLayoutDataTransferKey {
            parent_layout,
            assigned_bounds: bounds,
            child_layout_revision: nested.child.layout_revision,
        };
        nested.layout_data_transfer_key = Some(transfer_key);
        if first_transfer {
            self.previous_nested_layout_transfers.insert(
                host_local_id,
                (transfer_key, nested.child.layout_constraint_bounds.clone()),
            );
        }
        if changed {
            crate::layout_node_provider::mark_layout_node_dirty(self, host_local_id);
        }
        changed
    }
}
