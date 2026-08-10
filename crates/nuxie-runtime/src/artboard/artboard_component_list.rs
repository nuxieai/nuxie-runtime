use super::*;

impl ArtboardInstance {
    /// Ported from C++ `src/artboard_component_list.cpp::updateList`,
    /// `findArtboard`, and `createArtboardAt`.
    pub(crate) fn sync_component_list_items(
        &mut self,
        file: &RuntimeFile,
        list_local_id: usize,
        contexts: Vec<RuntimeOwnedViewModelHandle>,
    ) -> bool {
        let Some(build_context) = self.build_context.clone() else {
            return false;
        };
        let Some(parent_graph) = build_context
            .artboards
            .iter()
            .find(|graph| graph.global_id == self.graph_global_id)
        else {
            return false;
        };
        let Some(component_list) = parent_graph
            .component_lists
            .iter()
            .find(|list| list.local_id == list_local_id)
        else {
            return false;
        };

        let entries = self
            .component_list_state(list_local_id)
            .and_then(|list| list.source.as_ref())
            .map(|source| source.item_entries_with_logical_indices(file))
            .unwrap_or_else(|| {
                contexts
                    .into_iter()
                    .enumerate()
                    .map(|(index, instance)| {
                        set_component_list_item_index(file, &mut instance.borrow_mut(), index);
                        let occurrence_identity = instance.borrow().instance_identity();
                        RuntimeOwnedViewModelListItemEntry {
                            // NumberToList owns stable, unique generated VMIs.
                            occurrence_identity,
                            instance,
                        }
                    })
                    .collect()
            });
        let resolve_child_graph = |context: &RuntimeOwnedViewModelInstance| {
            let view_model_index = context.view_model_index();
            let mapped_index =
                artboard_list_map_rule_for_view_model(&component_list.map_rules, view_model_index)
                    .and_then(|rule| usize::try_from(rule.artboard_id).ok());
            mapped_index
                .and_then(|index| build_context.artboards.get(index))
                .or_else(|| {
                    build_context.artboards.iter().find(|graph| {
                        file.object(graph.global_id as usize)
                            .and_then(|artboard| artboard.uint_property("viewModelId"))
                            .and_then(|value| usize::try_from(value).ok())
                            == Some(view_model_index)
                    })
                })
        };

        let previous_logical = self
            .component_list_state_mut(list_local_id)
            .map(|list| std::mem::take(&mut list.logical_items))
            .unwrap_or_default();
        let mut logical_items = Vec::with_capacity(entries.len());
        for entry in entries {
            let mapped_artboard_global =
                resolve_child_graph(&entry.instance.borrow()).map(|graph| graph.global_id);
            let previous = previous_logical.iter().find(|item| {
                item.occurrence_identity == entry.occurrence_identity
                    && item.mapped_artboard_global == mapped_artboard_global
            });
            let settled_size = self
                .component_list_items(list_local_id)
                .and_then(|items| {
                    items.iter().find(|item| {
                        item.occurrence_identity == entry.occurrence_identity
                            && Some(item.child.graph_global_id) == mapped_artboard_global
                    })
                })
                .and_then(|item| item.settled_layout_size.get());
            let size = settled_size
                .or_else(|| previous.map(|item| item.size))
                .unwrap_or_else(|| {
                    mapped_artboard_global
                        .and_then(|global_id| file.object(global_id as usize))
                        .map(|artboard| {
                            (
                                artboard.double_property("width").unwrap_or(0.0),
                                artboard.double_property("height").unwrap_or(0.0),
                            )
                        })
                        .unwrap_or((0.0, 0.0))
                });
            logical_items.push(RuntimeComponentListLogicalItem {
                occurrence_identity: entry.occurrence_identity,
                context: entry.instance,
                size,
                mapped_artboard_global,
            });
        }
        let logical_changed = previous_logical.len() != logical_items.len()
            || previous_logical
                .iter()
                .zip(&logical_items)
                .any(|(before, after)| {
                    before.occurrence_identity != after.occurrence_identity
                        || before.mapped_artboard_global != after.mapped_artboard_global
                        || before.size != after.size
                });
        let desired = if component_list_virtualization(self, list_local_id).is_some() {
            // A virtualized list's mounted occurrences are the retained
            // interface state. ScrollVirtualizer adds/removes them directly;
            // list synchronization only preserves still-live identities.
            self.component_list_items(list_local_id)
                .map(|items| {
                    items
                        .iter()
                        .map(|item| item.logical_index)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        } else {
            (0..logical_items.len()).collect()
        };
        let desired = desired
            .into_iter()
            .filter(|index| {
                logical_items
                    .get(*index)
                    .is_some_and(|item| item.mapped_artboard_global.is_some())
            })
            .collect::<Vec<_>>();
        let existing_matches = self
            .component_list_items(list_local_id)
            .is_some_and(|existing| {
                existing.len() == desired.len()
                    && existing.iter().zip(&desired).all(|(item, index)| {
                        let logical = &logical_items[*index];
                        item.logical_index == *index
                            && item.occurrence_identity == logical.occurrence_identity
                            && item.context_is_current(&logical.context)
                    })
            });
        if existing_matches {
            if let Some(list) = self.component_list_state_mut(list_local_id) {
                list.logical_items = logical_items;
            }
            if logical_changed {
                if let Some(list) = self.component_list_state(list_local_id) {
                    *list.order_cache.borrow_mut() = Default::default();
                }
                crate::layout_node_provider::mark_layout_node_dirty(self, list_local_id);
                self.mark_prepared_changed();
            }
            return logical_changed;
        }

        // C++ keys mounted artboards/state machines by the list-item wrapper,
        // not by its VMI. Preserve overlapping wrapper occurrences across
        // reorder and virtual-window changes.
        let previous_items = self
            .component_list_state_mut(list_local_id)
            .map(|list| std::mem::take(&mut list.items))
            .unwrap_or_default();
        let mut reusable_items = previous_items.into_iter().map(Some).collect::<Vec<_>>();
        let parent_data_context = self.artboard_owned_data_context.clone().unwrap_or_default();
        let mut item_context_changed = false;
        let mut items = Vec::with_capacity(desired.len());
        for logical_index in desired {
            let logical = logical_items[logical_index].clone();
            let context = logical.context.clone();
            if let Some(existing_index) = reusable_items.iter().position(|candidate| {
                candidate.as_ref().is_some_and(|item| {
                    item.occurrence_identity == logical.occurrence_identity
                        && Some(item.child.graph_global_id) == logical.mapped_artboard_global
                })
            }) {
                let mut item = reusable_items[existing_index]
                    .take()
                    .expect("component-list identity match must retain an item");
                if !item.context_is_current(&context) {
                    item.context = context.clone();
                    item.context_rebind_sink = crate::view_model_cell::RuntimeCellDirtSink::new();
                    context.add_rebind_dependent(&item.context_rebind_sink);
                    item.draw_index_sink = component_list_draw_index_sink(file, &context);
                    let child_data_context = RuntimeOwnedDataContext::with_local_handles(
                        [context.clone()],
                        Some(&parent_data_context),
                    );
                    item.child.bind_owned_view_model_artboard_data_context(
                        file,
                        &child_data_context,
                        true,
                        true,
                    );
                    // The row-first DataContext bind clears the
                    // public facade while doing so. Restore the facade after
                    // the bind so scripting observes the same row main that
                    // C++ installs in `ArtboardComponentList::bindArtboard`
                    // (`artboard_component_list.cpp:1530-1543`).
                    item.child.artboard_owned_view_model_context = Some(
                        RuntimeOwnedViewModelContext::from_main_handle(context.clone()),
                    );
                    for state_machine in &mut item.state_machines {
                        if state_machine.bind_owned_view_model_data_context(&child_data_context) {
                            state_machine.advance_data_context();
                        }
                    }
                    item.child.advance_artboard_data_binds_with_elapsed(0.0);
                    item.child.update_pass();
                    item.consume_context_rebind_dirt();
                    item_context_changed = true;
                }
                let old_profile_path = item.child.profile_path.clone();
                let new_profile_path = component_list_profile_path(
                    &self.profile_path,
                    &item.child.profile_name,
                    component_list.name.as_deref().unwrap_or_default(),
                    logical_index,
                );
                item.child
                    .replace_profile_path_prefix(&old_profile_path, &new_profile_path);
                item.logical_index = logical_index;
                items.push(item);
                continue;
            }
            if let Some(item) = self.create_component_list_item_instance(
                file,
                list_local_id,
                logical_index,
                logical,
            ) {
                items.push(item);
            }
        }
        if let Some(list) = self.component_list_state_mut(list_local_id) {
            list.item_transforms = items.iter().map(|item| item.transform).collect();
            list.logical_items = logical_items;
            list.items = items;
        }
        if let Some(list) = self.component_list_state(list_local_id) {
            *list.order_cache.borrow_mut() = Default::default();
        }
        let changed = !existing_matches || logical_changed || item_context_changed;
        if changed {
            self.mark_nested_structure_changed();
            crate::layout_node_provider::mark_layout_node_dirty(self, list_local_id);
            self.mark_prepared_changed();
        }
        changed
    }

    /// Mount one retained `VirtualizingComponent` item.
    ///
    /// The FL-A adapter deliberately calls this one index at a time, matching
    /// `ScrollVirtualizer::virtualize` → `VirtualizingComponent::addVirtualizable`
    /// (`scroll_virtualizer.cpp:244-293`). The direct
    /// `src/artboard_component_list.cpp` owner now supplies the FL-D pool and
    /// fresh-state restoration used by this mount path.
    pub(in crate::artboard) fn create_component_list_item_instance(
        &mut self,
        file: &RuntimeFile,
        list_local_id: usize,
        logical_index: usize,
        logical: RuntimeComponentListLogicalItem,
    ) -> Option<RuntimeComponentListItemInstance> {
        let build_context = self.build_context.clone()?;
        let parent_graph = build_context
            .artboards
            .iter()
            .find(|graph| graph.global_id == self.graph_global_id)?;
        let component_list = parent_graph
            .component_lists
            .iter()
            .find(|list| list.local_id == list_local_id)?;
        let child_graph = logical.mapped_artboard_global.and_then(|global_id| {
            build_context
                .artboards
                .iter()
                .find(|graph| graph.global_id == global_id)
        })?;
        let context = logical.context;
        let mut visiting = BTreeSet::from([self.graph_global_id]);
        let profile_path = component_list_profile_path(
            &self.profile_path,
            child_graph.name.as_deref().unwrap_or_default(),
            component_list.name.as_deref().unwrap_or_default(),
            logical_index,
        );
        let mut child = ArtboardInstance::from_graph_inner(
            file,
            child_graph,
            &build_context.artboards,
            &mut visiting,
            Some(build_context.clone()),
            Arc::clone(&self.semantic_geometry_authority),
            false,
            profile_path,
        )
        .ok()?;
        // `Artboard::onAddedClean` clears the authored canvas placement from
        // every mounted instance before list bindings and state machines run.
        // Later animation/data-bind writes remain live and are consumed by
        // `Artboard::worldBounds()` (`artboard.cpp:1080-1092,1807-1814`).
        for axis in ["x", "y"] {
            if let Some(property_key) = property_key_for_name("Node", axis) {
                child
                    .objects
                    .set_generated_double_property(0, property_key, 0.0);
            }
        }
        child.inherit_audio_configuration_from(&self.audio_event_playback);
        child.set_frame_origin(false);
        let parent_data_context = self.artboard_owned_data_context.clone().unwrap_or_default();
        let child_data_context = RuntimeOwnedDataContext::with_local_handles(
            [context.clone()],
            Some(&parent_data_context),
        );
        child.bind_owned_view_model_artboard_data_context(file, &child_data_context, true, true);
        child.artboard_owned_view_model_context = Some(
            RuntimeOwnedViewModelContext::from_main_handle(context.clone()),
        );
        let selected_machine_indices = artboard_list_map_rule_for_view_model(
            &component_list.map_rules,
            context.borrow().view_model_index(),
        )
        .filter(|rule| !rule.state_machine_ids.is_empty())
        .map(|rule| rule.state_machine_ids.clone())
        .unwrap_or_else(|| {
            let default_state_machine_index = file
                .object(child_graph.global_id as usize)
                .and_then(|artboard| artboard.uint_property("defaultStateMachineId"));
            vec![component_list_default_state_machine_index(
                default_state_machine_index,
                child.state_machines.len(),
            )]
        });
        let mut state_machines = Vec::with_capacity(selected_machine_indices.len());
        for state_machine_index in selected_machine_indices {
            let Some(mut state_machine) = child.state_machine_instance(state_machine_index) else {
                continue;
            };
            state_machine.bind_owned_view_model_data_context(&child_data_context);
            // C++ `ArtboardComponentList::linkStateMachineToArtboard` installs
            // the row DataContext and immediately runs `updateDataBinds(false)`
            // before the first state advance
            // (`artboard_component_list.cpp:1492-1543`).
            state_machine.advance_data_context();
            state_machines.push(state_machine);
        }
        if let Some(parent_focus) = self.external_focus_domain.as_ref() {
            let child_identity = child.instance_identity();
            for state_machine in &mut state_machines {
                // C++ `ArtboardComponentList::linkStateMachineToArtboard`
                // installs the parent Artboard's FocusManager immediately
                // after the row machine is created and data-bound
                // (`artboard_component_list.cpp:641-678`).
                state_machine.install_external_focus(parent_focus, child_identity);
            }
            child.install_external_focus_domain(parent_focus);
        }
        child.advance_artboard_data_binds_with_elapsed(0.0);
        child.update_pass();
        let context_rebind_sink = crate::view_model_cell::RuntimeCellDirtSink::new();
        context.add_rebind_dependent(&context_rebind_sink);
        let draw_index_sink = component_list_draw_index_sink(file, &context);
        Some(RuntimeComponentListItemInstance {
            child: Box::new(child),
            render_resources: RefCell::new(crate::draw::RuntimeOccurrenceRenderResources::default()),
            state_machines,
            context_rebind_sink,
            draw_index_sink,
            context,
            occurrence_identity: logical.occurrence_identity,
            logical_index,
            settled_layout_size: Cell::new(None),
            transform: Mat2D::IDENTITY,
            // Render caches outlive list topology changes. Seed each row with
            // stable wrapper identity so a same-length replacement cannot
            // reuse the prior occupant's paint cache.
            render_cache_revision: logical.occurrence_identity,
        })
    }

    /// Literal owner-level `ArtboardComponentList::updateLayoutBounds` tail:
    /// update mounted row bounds once, write those sizes into the full logical
    /// vector, compute `m_layoutSize`, then force the retained virtualizer
    /// (`artboard_component_list.cpp:245-260,1758-1788`).
    pub(in crate::artboard) fn update_component_list_layout_bounds(
        &mut self,
        root_transform: Mat2D,
    ) -> bool {
        // C++ deliberately reaches `computeLayoutBounds()` even when the
        // mounted-row loop is empty. That owner tail recomputes the logical
        // list size and force-settles its ScrollVirtualizer; returning early
        // would leave an empty-mounted virtualized list one settlement behind
        // (`artboard_component_list.cpp:245-260,1758-1788`).
        let mut assigned_bounds = self.runtime_component_list_assigned_layout_bounds();
        for list_local in self.component_list_locals() {
            let Some(items) = self.component_list_items(list_local) else {
                continue;
            };
            assigned_bounds.entry(list_local).or_insert_with(|| {
                items
                    .iter()
                    .map(|item| {
                        let (width, height) = runtime_component_list_item_layout_size(item);
                        RuntimeLayoutBounds {
                            x: 0.0,
                            y: 0.0,
                            width,
                            height,
                        }
                    })
                    .collect()
            });
        }
        let mut changed = false;
        for list_index in 0..self.component_list_count() {
            let Some(list_local) = self.component_list_local_at(list_index) else {
                continue;
            };
            let bounds = assigned_bounds.remove(&list_local).unwrap_or_default();
            let mut measured_sizes = Vec::new();
            let virtualized =
                crate::constraints::scrolling::scroll_virtualizer::component_list_virtualization(
                    self, list_local,
                )
                .is_some();
            if let Some(items) = self.component_list_items_mut(list_local) {
                for (item, bounds) in items.iter_mut().zip(bounds) {
                    let previous_size = runtime_component_list_item_layout_size(item);
                    changed |=
                        runtime_apply_component_list_item_layout_bounds(&mut item.child, bounds);
                    if item.settled_layout_size.get().is_none() {
                        // C++ marks the row as newly hosted before its first
                        // parent-owned `updateLayoutBounds`; the later
                        // component traversal performs `updatePass(false)`.
                        item.child.added_to_host();
                    }
                    // The mounted root Yoga node remains owned by the hosting
                    // layout. Retain its parent-local location through the
                    // LayoutComponent owner so a size delta publishes the
                    // same Path-before-World dirt as C++ before the later
                    // child traversal (`artboard_component_list.cpp:220-229`;
                    // `layout_component.cpp:1153-1178`).
                    item.child
                        .retain_runtime_layout_component_bounds(0, bounds, None);
                    // The transferred root Yoga node is the size owner after
                    // the parent solve. Do not immediately run a standalone
                    // child solve and overwrite its parent-local location.
                    let measured = (bounds.width, bounds.height);
                    if virtualized && measured != previous_size {
                        // `setVirtualizablePosition` stores `position -
                        // artboard->origin()`. A same-frame layout-width
                        // change therefore moves the retained translation by
                        // exactly the origin delta before draw
                        // (`artboard_component_list.cpp:1728-1740`;
                        // `artboard.cpp:1729-1734`).
                        item.transform.0[4] += (measured.0 - previous_size.0) * item.child.origin_x;
                        item.transform.0[5] += (measured.1 - previous_size.1) * item.child.origin_y;
                    }
                    item.settled_layout_size.set(Some(measured));
                    measured_sizes.push((item.logical_index, measured));
                }
            }
            let transforms = runtime_component_list_item_base_transforms(self, list_local);
            if let Some(list) = self.component_list_state_mut(list_local)
                && list.item_transforms != transforms
            {
                list.item_transforms = transforms;
                changed = true;
            }
            if let Some(list_component) = self.component_handle(list_local) {
                // This Rust tail performs the parent-owned Yoga solve after
                // the dependency walk. Reapply the same list constraints that
                // C++ runs immediately after
                // `updateArtboardsWorldTransform`; otherwise the refreshed
                // hosted-layout bases overwrite FollowPath's retained result
                // (`artboard_component_list.cpp:1300-1358`).
                changed |= crate::constraints::constrainable_list::apply_list_constraints(
                    self,
                    list_component,
                );
            }

            let style_local = self.layout_component_style_local(list_local);
            let is_row = style_local
                .and_then(|local| {
                    property_key_for_name("LayoutComponentStyle", "flexDirectionValue")
                        .and_then(|key| self.uint_property(local, key))
                })
                .map(|direction| matches!(direction, 2 | 3))
                .unwrap_or(true);
            let gap_property = if is_row {
                "gapHorizontal"
            } else {
                "gapVertical"
            };
            let gap = style_local
                .and_then(|local| {
                    property_key_for_name("LayoutComponentStyle", gap_property)
                        .and_then(|key| self.double_property(local, key))
                })
                .unwrap_or(0.0);
            let scroll_constraint = self.component_list_state(list_local).and_then(|list| {
                list.layout_constraints
                    .iter()
                    .copied()
                    .find(|constraint| self.component_at(*constraint).concrete.scroll.is_some())
            });
            let mut layout_size_changed = false;
            if let Some(list) = self.component_list_state_mut(list_local) {
                for (logical_index, size) in measured_sizes {
                    if let Some(logical) = list.logical_items.get_mut(logical_index) {
                        layout_size_changed |= logical.size != size;
                        logical.size = size;
                    }
                }
                let mut width: f32 = 0.0;
                let mut height: f32 = 0.0;
                let item_count = list.logical_items.len();
                for (index, item) in list.logical_items.iter().enumerate() {
                    let real_gap = if index + 1 == item_count { 0.0 } else { gap };
                    if is_row {
                        width += item.size.0 + real_gap;
                        height = height.max(item.size.1);
                    } else {
                        width = width.max(item.size.0);
                        height += item.size.1 + real_gap;
                    }
                }
                layout_size_changed |= list.layout_size != (width, height);
                list.layout_size = (width, height);
            }
            if layout_size_changed {
                changed = true;
                crate::layout_node_provider::mark_layout_node_dirty(self, list_local);
            }
            if let Some(constraint) = scroll_constraint {
                changed |=
                    crate::constraints::scrolling::scroll_virtualizer::constrain_scroll_virtualizer(
                        self, constraint, true,
                    );
            }
            let roots = self
                .runtime_component_list_child_root_transforms(root_transform)
                .remove(&list_local)
                .unwrap_or_default();
            if let Some(items) = self.component_list_items_mut(list_local) {
                for (item_index, item) in items.iter_mut().enumerate() {
                    let child_root_transform =
                        roots.get(item_index).copied().unwrap_or(root_transform);
                    let mut script_mode = RuntimeScriptUpdateMode::HostOnly;
                    changed |= item
                        .child
                        .update_pass_with_script_mode(&mut script_mode, child_root_transform);
                }
            }
        }
        changed
    }

    pub(in crate::artboard) fn advance_component_list_entry(
        &mut self,
        list_local: usize,
        elapsed_seconds: f32,
        new_frame: bool,
        script_mode: &mut RuntimeScriptAdvanceMode<'_>,
    ) -> Result<bool, ScriptError> {
        if self
            .component(list_local)
            .is_none_or(RuntimeComponent::is_collapsed)
            || self
                .component_list_items(list_local)
                .is_none_or(|items| items.is_empty())
        {
            return Ok(false);
        }
        let parent_data_context = self.artboard_owned_data_context.clone().unwrap_or_default();
        let mut source_changed = false;
        let mut child_dirty = false;
        let mut keep_going = false;
        let mut first_script_error = None;
        let Some(items) = self.component_list_items_mut(list_local) else {
            return Ok(false);
        };
        for item in items {
            let mut row_changed = false;
            if !item.context_is_current(&item.context)
                && let Some(file) = item.child.runtime_file_arc()
            {
                let child_data_context = RuntimeOwnedDataContext::with_local_handles(
                    [item.context.clone()],
                    Some(&parent_data_context),
                );
                row_changed |= item.child.bind_owned_view_model_artboard_data_context(
                    &file,
                    &child_data_context,
                    true,
                    true,
                );
                item.child.artboard_owned_view_model_context = Some(
                    RuntimeOwnedViewModelContext::from_main_handle(item.context.clone()),
                );
                for state_machine in &mut item.state_machines {
                    if state_machine.bind_owned_view_model_data_context(&child_data_context) {
                        row_changed = true;
                        row_changed |= state_machine.advance_data_context();
                    }
                }
                item.consume_context_rebind_dirt();
                source_changed = true;
            }
            for state_machine in &mut item.state_machines {
                if new_frame {
                    row_changed |= item
                        .child
                        .advance_state_machine_instance(state_machine, elapsed_seconds);
                } else if item
                    .child
                    .try_change_state_machine_instance_unconditionally(state_machine)
                {
                    // Component-list rows use the literal unguarded
                    // `tryChangeState()` branch when `NewFrame` is absent.
                    // A row mounted during the preceding update therefore
                    // enters its initial state in this same outer pass
                    // (`artboard_component_list.cpp:827-854`).
                    // The successful probe is not itself a keep-going term:
                    // C++ composes only the following nested `advance`
                    // return. The transition's dirt still drives settlement.
                    row_changed |= item.child.advance_state_machine_instance_after_state_probe(
                        state_machine,
                        elapsed_seconds,
                    );
                }
            }
            let child_result = if new_frame {
                item.child
                    .advance_retained_components_collect_events_with_scripts(
                        elapsed_seconds,
                        true,
                        script_mode,
                        None,
                        None,
                    )
            } else {
                item.child
                    .advance_retained_components_collect_events_with_scripts(
                        elapsed_seconds,
                        false,
                        script_mode,
                        None,
                        None,
                    )
            };
            match child_result {
                Ok(changed) => row_changed |= changed,
                Err(error) => {
                    first_script_error.get_or_insert(error);
                }
            }
            row_changed |= item
                .child
                .advance_artboard_data_binds_with_elapsed(elapsed_seconds);
            child_dirty |= item.child.has_dirt(ComponentDirt::COMPONENTS);
            if item
                .context_rebind_sink
                .peek_dirt()
                .contains(crate::view_model_cell::RuntimeCellDirt::BINDINGS)
            {
                source_changed = true;
            }
            // The row Artboard's root Yoga node belongs to the hosting list.
            // Nested animation/data-bind dirt does not detach that node or
            // discard its parent-assigned size; only topology/pool mounting
            // creates an occurrence without a hosted result.
            keep_going |= row_changed;
        }
        if source_changed {
            self.mark_component_list_source_changed(list_local);
        }
        if child_dirty {
            self.add_dirt(list_local, ComponentDirt::COMPONENTS, false);
        }
        if let Some(error) = first_script_error {
            return Err(error);
        }
        Ok(keep_going)
    }
}
