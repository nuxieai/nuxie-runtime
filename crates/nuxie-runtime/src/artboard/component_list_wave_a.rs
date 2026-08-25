    // Executable translations of the Wave A component-list cases rejected as
    // metadata-only or narrower-than-upstream by `wave-a-review.md`.

    struct UpstreamComponentListFixture {
        file: RuntimeFile,
        graphs: GraphFile,
        artboard_index: usize,
        artboard: ArtboardInstance,
        context: RuntimeOwnedViewModelContext,
        list_local: usize,
    }

    impl UpstreamComponentListFixture {
        fn load(asset: &str, artboard_name: &str) -> Self {
            let root = std::env::var_os("RIVE_RUNTIME_DIR")
                .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
            let fixture = PathBuf::from(root).join("tests/unit_tests/assets").join(asset);
            let file = read_runtime_file(
                &std::fs::read(&fixture)
                    .unwrap_or_else(|error| panic!("read {}: {error}", fixture.display())),
            )
            .unwrap_or_else(|error| panic!("import {}: {error:#}", fixture.display()));
            let graphs = GraphFile::from_runtime_file(&file)
                .unwrap_or_else(|error| panic!("graph {}: {error:#}", fixture.display()));
            let artboard_index = graphs
                .artboards
                .iter()
                .position(|graph| graph.name.as_deref() == Some(artboard_name))
                .unwrap_or_else(|| panic!("missing artboard {artboard_name} in {asset}"));
            let graph = &graphs.artboards[artboard_index];
            let mut artboard = ArtboardInstance::from_graph_with_artboards(
                &file,
                graph,
                &graphs.artboards,
            )
            .unwrap_or_else(|error| panic!("instantiate {artboard_name} in {asset}: {error:#}"));
            let view_model_index = file
                .object(graph.global_id as usize)
                .and_then(|object| object.uint_property("viewModelId"))
                .and_then(|value| usize::try_from(value).ok())
                .expect("fixture artboard has a default view model");
            let main = RuntimeOwnedViewModelInstance::from_instance(&file, view_model_index, 0)
                .or_else(|| RuntimeOwnedViewModelInstance::new(&file, view_model_index))
                .expect("createDefaultViewModelInstance");
            let mut context = RuntimeOwnedViewModelContext::from_main(main);
            context.complete_for_artboard(&file, artboard_index);
            assert!(artboard.bind_owned_view_model_artboard_contexts(&file, &context));
            let list_local = graph
                .component_lists
                .iter()
                .find(|list| list.name.as_deref() == Some("List"))
                .or_else(|| graph.component_lists.first())
                .expect("fixture ArtboardComponentList")
                .local_id;
            Self {
                file,
                graphs,
                artboard_index,
                artboard,
                context,
                list_local,
            }
        }

        fn advance(&mut self, elapsed: f32) -> UpdateComponentsReport {
            self.artboard
                .advance(elapsed)
                .expect("Artboard::advance succeeds");
            self.artboard.update_components()
        }

        fn list(&self) -> &RuntimeConstrainableListState {
            self.artboard
                .component_list_state(self.list_local)
                .expect("live ArtboardComponentList occurrence")
        }

        fn rows(&self) -> &[RuntimeComponentListItemInstance] {
            &self.list().items
        }
    }

    fn assert_component_list_rows_and_state_machines(
        fixture: &UpstreamComponentListFixture,
        expected_name: &str,
    ) {
        assert!(!fixture.list().logical_items.is_empty());
        assert_eq!(fixture.rows().len(), fixture.list().logical_items.len());
        for row in fixture.rows() {
            assert_eq!(row.child.profile_name, expected_name);
            assert!(!row.state_machines.is_empty());
            assert!(
                crate::state_machine::state_machine_instance::component_list_wave_a_state_machine_belongs_to_artboard(
                    &row.state_machines[0],
                    &row.child,
                ),
                "row state machine retains the mounted row Artboard definition owner",
            );
            assert_eq!(row.child.graph_global_id, fixture.graphs.artboards.iter()
                .find(|graph| graph.name.as_deref() == Some(expected_name))
                .expect("row source artboard")
                .global_id);
        }
    }

    #[test]
    fn wave_a_component_list_case_01_artboard_count() {
        let mut fixture = UpstreamComponentListFixture::load("component_list_1.riv", "Main");
        let report = fixture.advance(0.0);
        assert!(report.did_layout, "ArtboardComponentList::syncStyleChanges");
        assert_eq!(fixture.list().logical_items.len(), 8);
        assert!(fixture.list().logical_items.get(9).is_none());
        assert!(fixture.rows().iter().find(|row| row.logical_index == 9).is_none());
        assert!(fixture.rows().get(9).is_none());
    }

    #[test]
    fn wave_a_component_list_case_02_artboards_and_state_machines() {
        let mut fixture = UpstreamComponentListFixture::load("component_list_1.riv", "Main");
        fixture.advance(0.0);
        assert_component_list_rows_and_state_machines(&fixture, "Item");
    }

    #[test]
    fn wave_a_component_list_case_03_layout_nodes() {
        let mut fixture = UpstreamComponentListFixture::load("component_list_1.riv", "Main");
        fixture.advance(0.0);
        assert_eq!(fixture.rows().len(), fixture.list().logical_items.len());
        for row in fixture.rows() {
            assert!(row.settled_layout_size.get().is_some());
            assert!(row.child.layout_node_owned_by_host);
        }
    }

    #[test]
    fn wave_a_component_list_case_04_layout_bounds() {
        let mut fixture = UpstreamComponentListFixture::load("component_list_1.riv", "Main");
        fixture.advance(0.0);
        for (index, row) in fixture.rows().iter().enumerate() {
            let bounds_origin = row.child.component(0)
                .and_then(|component| component.concrete.layout.as_ref())
                .map(|layout| layout.position())
                .expect("mounted child root layoutBounds");
            assert_eq!(bounds_origin.1, index as f32 * 60.0);
        }
    }

    #[test]
    fn wave_a_component_list_case_05_data_context() {
        let mut fixture = UpstreamComponentListFixture::load("component_list_1.riv", "Main");
        fixture.advance(0.0);
        let expected = ["ONE", "TWO", "THREE", "THREE", "THREE", "THREE", "TWO", "ONE"];
        assert_eq!(fixture.rows().len(), expected.len());
        for (row, expected) in fixture.rows().iter().zip(expected) {
            let actual = row
                .context
                .borrow()
                .string_value_by_property_name("Label")
                .expect("row dataContext Label string");
            assert_eq!(actual.as_ref(), expected.as_bytes());
            assert!(row.child.owned_view_model_context().is_some());
        }
    }

    #[test]
    fn wave_a_component_list_case_06_labels() {
        let mut fixture = UpstreamComponentListFixture::load("component_list_1.riv", "Main");
        fixture.advance(0.0);
        let expected = ["ONE", "TWO", "THREE", "THREE", "THREE", "THREE", "TWO", "ONE"];
        for (row, expected) in fixture.rows().iter().zip(expected) {
            let text = row.child.root_text_value_run("TextLabel")
                .or_else(|| row.child.slots().iter()
                    .find(|slot| slot.type_name == Some("TextValueRun"))
                    .and_then(|slot| property_key_for_name("TextValueRun", "text")
                        .and_then(|key| row.child.string_property(slot.local_id, key))))
                .expect("Text::runs()[0] text");
            assert_eq!(text, expected.as_bytes());
        }
    }

    #[test]
    #[ignore = "expected-red: row three Hover is already true after the pointer moves over row one instead of the pinned false state"]
    fn wave_a_component_list_case_07_state_machine_listener() {
        let mut fixture = UpstreamComponentListFixture::load("component_list_1.riv", "Main");
        let mut root_machine = fixture.artboard.state_machine_instance(0)
            .expect("State Machine 1 instance");
        root_machine.bind_owned_view_model_contexts(&fixture.context);
        root_machine.advance_data_context();
        fixture.advance(0.0);
        assert_eq!(fixture.rows()[0].state_machines[0].get_bool("Hover").and_then(|input| input.bool_value()), Some(false));
        assert_eq!(
            crate::state_machine::state_machine_instance::component_list_wave_a_hit_components_count(
                &fixture.rows()[0].state_machines[0],
            ),
            1,
        );
        root_machine.pointer_move(&mut fixture.artboard, 100.0, 30.0, 0.0, 0);
        fixture.artboard.advance(0.0).expect("nested hover advance");
        root_machine.advance_and_apply(&mut fixture.artboard, 0.0).expect("root hover apply");
        assert_eq!(fixture.rows()[0].state_machines[0].get_bool("Hover").and_then(|input| input.bool_value()), Some(true));
        assert_eq!(fixture.rows()[2].state_machines[0].get_bool("Hover").and_then(|input| input.bool_value()), Some(false));
        assert_eq!(
            crate::state_machine::state_machine_instance::component_list_wave_a_hit_components_count(
                &fixture.rows()[2].state_machines[0],
            ),
            1,
        );
        root_machine.pointer_move(&mut fixture.artboard, 100.0, 150.0, 0.0, 0);
        fixture.artboard.advance(0.0).expect("third-row hover advance");
        root_machine.advance_and_apply(&mut fixture.artboard, 0.0).expect("third-row hover apply");
        assert_eq!(fixture.rows()[2].state_machines[0].get_bool("Hover").and_then(|input| input.bool_value()), Some(true));
    }

    #[test]
    fn wave_a_component_list_case_08_number_to_list_count() {
        let mut fixture = UpstreamComponentListFixture::load("component_list_2.riv", "Main");
        let report = fixture.advance(0.0);
        assert!(report.did_layout, "ArtboardComponentList::syncStyleChanges");
        assert_eq!(fixture.list().logical_items.len(), 12);
        assert!(fixture.list().logical_items.get(13).is_none());
        assert!(fixture.rows().get(13).is_none());
        assert!(fixture.list().logical_items.iter().all(|item| item.mapped_artboard_global.is_some()));
        let root = fixture.context.main_handle().expect("default VMI").clone();
        assert!(root.borrow_mut().set_number_by_property_name("ItemCount", 6.0));
        fixture.advance(0.0);
        assert_eq!(fixture.list().logical_items.len(), 6);
    }

    #[test]
    fn wave_a_component_list_case_09_number_to_list_artboards_and_state_machines() {
        let mut fixture = UpstreamComponentListFixture::load("component_list_2.riv", "Main");
        fixture.advance(0.0);
        assert_component_list_rows_and_state_machines(&fixture, "Item");
    }

    #[test]
    fn wave_a_component_list_case_10_number_to_list_layout_nodes() {
        let mut fixture = UpstreamComponentListFixture::load("component_list_2.riv", "Main");
        fixture.advance(0.0);
        assert_eq!(fixture.rows().len(), fixture.list().logical_items.len());
        assert!(fixture.rows().iter().all(|row| row.settled_layout_size.get().is_some() && row.child.layout_node_owned_by_host));
    }

    #[test]
    fn wave_a_component_list_case_11_number_to_list_data_context() {
        let mut fixture = UpstreamComponentListFixture::load("component_list_2.riv", "Main");
        fixture.advance(0.0);
        for (index, row) in fixture.rows().iter().enumerate() {
            assert_eq!(row.context.borrow().component_list_item_index(), Some(index as u64));
            assert!(row.child.owned_view_model_context().is_some());
        }
    }

    #[test]
    fn wave_a_component_list_case_12_number_to_list_labels() {
        let mut fixture = UpstreamComponentListFixture::load("component_list_2.riv", "Main");
        fixture.advance(0.0);
        for (index, row) in fixture.rows().iter().enumerate() {
            let text = row.child.root_text_value_run("ItemLabel")
                .or_else(|| row.child.slots().iter()
                    .find(|slot| slot.type_name == Some("TextValueRun"))
                    .and_then(|slot| property_key_for_name("TextValueRun", "text")
                        .and_then(|key| row.child.string_property(slot.local_id, key))))
                .expect("Text::runs()[0] text");
            assert_eq!(text, index.to_string().as_bytes());
        }
    }

    #[test]
    fn wave_a_component_list_case_13_virtualized_artboards_and_state_machines() {
        let mut fixture = UpstreamComponentListFixture::load("component_list_virtualized.riv", "Main");
        fixture.advance(0.0);
        assert_eq!(fixture.list().logical_items.len(), 20);
        assert_eq!(fixture.rows().len(), 5);
        for index in 0..20 {
            let row = fixture.rows().iter().find(|row| row.logical_index == index);
            if index < 5 {
                let row = row.expect("first five Artboard instances are mounted");
                assert_eq!(row.child.profile_name, "ItemArtboard");
                assert!(!row.state_machines.is_empty());
                assert!(
                    crate::state_machine::state_machine_instance::component_list_wave_a_state_machine_belongs_to_artboard(
                        &row.state_machines[0],
                        &row.child,
                    ),
                );
            } else {
                assert!(row.is_none());
            }
        }
    }

    #[test]
    fn wave_a_component_list_case_14_virtualized_layout_bounds() {
        let mut fixture = UpstreamComponentListFixture::load("component_list_virtualized.riv", "Main");
        assert!(component_list_virtualization(&fixture.artboard, fixture.list_local).is_some());
        fixture.advance(0.0);
        assert_eq!(fixture.list().logical_items.len(), 20);
        for index in 0..20 {
            let row = fixture.rows().iter().find(|row| row.logical_index == index);
            if index < 5 {
                let row = row.expect("first five Artboard instances are mounted");
                assert_eq!(row.transform.0[4], index as f32 * 110.0);
            } else {
                assert!(row.is_none());
            }
        }
    }

    #[test]
    fn wave_a_component_list_case_15_virtualized_scroll() {
        let mut fixture = UpstreamComponentListFixture::load("component_list_virtualized.riv", "Main");
        let initial_scrolls = fixture.artboard.scroll_constraint_occurrences();
        assert_eq!(initial_scrolls.len(), 1);
        let scroll_local = initial_scrolls[0].constraint_local_id;
        let offset_key = property_key_for_name("ScrollConstraint", "scrollOffsetX").unwrap();
        let infinite_key = property_key_for_name("ScrollConstraint", "infinite").unwrap();
        let index_key = property_key_for_name("ScrollConstraint", "scrollIndex").unwrap();
        assert_eq!(fixture.artboard.double_property(scroll_local, offset_key), Some(0.0));
        fixture.advance(0.0);
        assert!(fixture.artboard.set_double_property(scroll_local, index_key, 2.0));
        fixture.artboard.update_pass();
        let snapshot = fixture.artboard.scroll_constraint_occurrences()[0];
        assert_eq!(fixture.artboard.bool_property(scroll_local, infinite_key), Some(true));
        assert_eq!(fixture.list().logical_items.len(), 20);
        assert_eq!(snapshot.offset, (-220.0, 0.0));
        assert_eq!(snapshot.clamped_offset, (-220.0, 0.0));
        assert_eq!(snapshot.lower_bound, (f32::NEG_INFINITY, 0.0));
        assert_eq!(snapshot.upper_bound, (f32::INFINITY, 0.0));
        assert_eq!(fixture.artboard.double_property(scroll_local, index_key), Some(2.0));
        let content_width_key = property_key_for_name(
            "ScrollConstraint",
            "computedContentWidth",
        )
        .unwrap();
        assert_eq!(
            fixture.artboard.double_property(scroll_local, content_width_key),
            Some(2200.0),
        );
        let content = fixture.artboard.component_handle(snapshot.content_local_id)
            .and_then(|handle| fixture.artboard.objects.component(handle))
            .expect("scroll content component");
        let viewport_local = content.parent
            .and_then(|handle| fixture.artboard.objects.component_local_id(handle))
            .expect("scroll viewport parent");
        let viewport = fixture.artboard.layout_bounds(viewport_local)
            .expect("scroll viewport layout");
        assert_eq!(viewport.width, 500.0);
    }

    #[test]
    fn wave_a_component_list_case_16_manual_scroll_direct_assertions() {
        let mut fixture = UpstreamComponentListFixture::load("component_list_virtualized.riv", "Main");
        let mut machine = fixture.artboard.state_machine_instance(0).expect("state machine 0");
        machine.bind_owned_view_model_contexts(&fixture.context);
        machine.advance_data_context();
        machine.advance_and_apply(&mut fixture.artboard, 0.1).expect("initial advance");
        let scroll_local = fixture.artboard.scroll_constraint_occurrences()[0].constraint_local_id;
        let offset_x_key = property_key_for_name("ScrollConstraint", "scrollOffsetX").unwrap();
        let offset_y_key = property_key_for_name("ScrollConstraint", "scrollOffsetY").unwrap();
        let percent_y_key = property_key_for_name("ScrollConstraint", "scrollPercentY").unwrap();
        let index_key = property_key_for_name("ScrollConstraint", "scrollIndex").unwrap();
        assert_eq!(fixture.artboard.double_property(scroll_local, percent_y_key), Some(0.0));
        assert_eq!(fixture.artboard.double_property(scroll_local, offset_y_key), Some(0.0));
        assert_eq!(fixture.artboard.double_property(scroll_local, index_key), Some(0.0));
        assert!(!fixture.artboard.scroll_constraint_occurrences()[0].physics_running);
        machine.pointer_move(&mut fixture.artboard, 250.0, 50.0, 0.0, 0);
        machine.pointer_down(&mut fixture.artboard, 250.0, 50.0, 0);
        machine.advance_and_apply(&mut fixture.artboard, 0.1).expect("drag start");
        machine.pointer_move(&mut fixture.artboard, 50.0, 50.0, 0.1, 0);
        machine.advance_and_apply(&mut fixture.artboard, 0.1).expect("drag move");
        assert_eq!(fixture.artboard.double_property(scroll_local, offset_x_key), Some(-200.0));
        let index = fixture.artboard.double_property(scroll_local, index_key).unwrap();
        assert!((index - 1.818182).abs() < 0.00001);
        machine.pointer_up(&mut fixture.artboard, 50.0, 50.0, 0);
        assert!(fixture.artboard.scroll_constraint_occurrences()[0].physics_running);
    }

    #[test]
    fn wave_a_component_list_case_20_non_layout_initial_positions() {
        let mut fixture = UpstreamComponentListFixture::load("component_list_grouped.riv", "MainArtboard");
        fixture.advance(0.1);
        assert_eq!(fixture.rows().len(), 3);
        for (row, expected) in fixture.rows().iter().zip([(0.0, 0.0), (50.0, 0.0), (100.0, 0.0)]) {
            assert_eq!(row.context.borrow().number_value_by_property_name("x"), Some(expected.0));
            assert_eq!(row.context.borrow().number_value_by_property_name("y"), Some(expected.1));
        }
    }

    #[test]
    fn wave_a_component_list_case_21_follow_path_initial_item_count() {
        let mut fixture = UpstreamComponentListFixture::load("component_list_follow_path.riv", "Main");
        fixture.advance(0.1);
        let root = fixture.context.main_handle().expect("default VMI");
        assert_eq!(root.borrow().number_value_by_property_name("ItemCount"), Some(10.0));
        assert_eq!(fixture.list().logical_items.len(), 10);
        assert!(root.borrow_mut().set_number_by_property_name("ItemCount", 5.0));
        for _ in 0..30 { fixture.advance(0.016); }
        assert_eq!(fixture.list().logical_items.len(), 5);
    }

    #[test]
    fn wave_a_component_list_case_28_default_order() {
        let mut fixture = UpstreamComponentListFixture::load("component_list_1.riv", "Main");
        fixture.advance(0.0);
        let order = crate::artboard_component_list_order::runtime_component_list_order(
            &fixture.file,
            fixture.list(),
        );
        let count = fixture.list().logical_items.len();
        assert!(count > 0);
        assert_eq!(order.indices.len(), count);
        assert_eq!(order.indices, (0..count).collect::<Vec<_>>());
        assert_eq!(order.indices.iter().rev().copied().collect::<Vec<_>>(), (0..count).rev().collect::<Vec<_>>());
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct WaveAClipBounds {
        min_x: f32,
        min_y: f32,
        max_x: f32,
        max_y: f32,
    }

    impl WaveAClipBounds {
        fn intersect(self, other: Self) -> Self {
            Self {
                min_x: self.min_x.max(other.min_x),
                min_y: self.min_y.max(other.min_y),
                max_x: self.max_x.min(other.max_x),
                max_y: self.max_y.min(other.max_y),
            }
        }
    }

    #[derive(Clone, Copy)]
    struct WaveAClipState {
        transform: Mat2D,
        clip: Option<WaveAClipBounds>,
    }

    #[derive(Debug)]
    struct WaveAClipDraw {
        bounds: WaveAClipBounds,
        clip: Option<WaveAClipBounds>,
    }

    fn wave_a_recording_values(line: &str, prefix: &str, suffix: &str) -> Vec<f32> {
        let start = line.find(prefix).expect("recording prefix") + prefix.len();
        let tail = &line[start..];
        let end = tail.find(suffix).expect("recording suffix");
        tail[..end]
            .split(',')
            .map(|value| value.parse::<f32>().expect("recording float"))
            .collect()
    }

    fn wave_a_recording_path_bounds(line: &str, transform: Mat2D) -> WaveAClipBounds {
        let start = line.find("points=[").expect("recorded path points") + "points=[".len();
        let tail = &line[start..];
        let end = tail.find("]}").expect("recorded path point terminator");
        let mut points = Vec::new();
        for pair in tail[..end].split("),(") {
            let pair = pair.trim_matches(['(', ')']);
            let mut values = pair.split(',').map(|value| {
                value.parse::<f32>().expect("recorded path coordinate")
            });
            let (x, y) = transform.transform_point(
                values.next().expect("path x"),
                values.next().expect("path y"),
            );
            points.push((x, y));
        }
        WaveAClipBounds {
            min_x: points.iter().map(|point| point.0).fold(f32::INFINITY, f32::min),
            min_y: points.iter().map(|point| point.1).fold(f32::INFINITY, f32::min),
            max_x: points.iter().map(|point| point.0).fold(f32::NEG_INFINITY, f32::max),
            max_y: points.iter().map(|point| point.1).fold(f32::NEG_INFINITY, f32::max),
        }
    }

    fn wave_a_clip_probe(stream: &str) -> Vec<WaveAClipDraw> {
        let mut stack = vec![WaveAClipState {
            transform: Mat2D::IDENTITY,
            clip: None,
        }];
        let mut draws = Vec::new();
        for line in stream.lines() {
            match line {
                "save" => stack.push(*stack.last().expect("clip state")),
                "restore" => {
                    assert!(stack.len() > 1, "balanced recording restore");
                    stack.pop();
                }
                _ if line.starts_with("transform matrix=[") => {
                    let values = wave_a_recording_values(line, "transform matrix=[", "]");
                    assert_eq!(values.len(), 6);
                    let next = Mat2D(values.try_into().expect("six matrix values"));
                    let state = stack.last_mut().expect("clip state");
                    state.transform = state.transform.multiply(next);
                }
                _ if line.starts_with("clipPath ") => {
                    let state = stack.last_mut().expect("clip state");
                    let bounds = wave_a_recording_path_bounds(line, state.transform);
                    state.clip = Some(state.clip.map_or(bounds, |clip| clip.intersect(bounds)));
                }
                _ if line.starts_with("drawPath ") => {
                    let state = *stack.last().expect("clip state");
                    draws.push(WaveAClipDraw {
                        bounds: wave_a_recording_path_bounds(line, state.transform),
                        clip: state.clip,
                    });
                }
                _ => {}
            }
        }
        assert_eq!(stack.len(), 1, "balanced recording saves");
        draws
    }

    #[test]
    fn wave_a_component_list_case_30_clipped_viewport() {
        let mut fixture = UpstreamComponentListFixture::load("component_list_clipped_viewport.riv", "Main");
        fixture.advance(0.0);
        assert_eq!(fixture.list().logical_items.len(), 6);
        assert!(component_list_virtualization(&fixture.artboard, fixture.list_local).is_some());
        assert_eq!(fixture.rows().len(), 4);
        let graph = fixture.graphs.artboards[fixture.artboard_index].clone();
        let mut factory = RecordingFactory::new();
        let mut renderer = factory.make_renderer();
        fixture.artboard.draw_artboard(
            &fixture.file,
            &graph,
            &fixture.graphs.artboards,
            &mut factory,
            &mut renderer,
            &Default::default(),
            None,
            true,
        ).expect("draw clipped viewport fixture");
        let recording = factory.canonical_recording();
        let draws = wave_a_clip_probe(recording.stream());
        let viewport = WaveAClipBounds {
            min_x: 100.0,
            min_y: 100.0,
            max_x: 300.0,
            max_y: 300.0,
        };
        assert_eq!(draws.len(), 6);
        for item in &draws[1..=4] {
            assert_eq!(item.clip, Some(viewport));
        }
        assert!(draws[4].bounds.max_y > viewport.max_y);
        assert_eq!(draws[5].bounds.min_y, 360.0);
        if let Some(overlay_clip) = draws[5].clip {
            assert!(overlay_clip.max_y > viewport.max_y);
        }

        let scroll_local = fixture.artboard.scroll_constraint_occurrences()[0].constraint_local_id;
        let index_key = property_key_for_name("ScrollConstraint", "scrollIndex").unwrap();
        assert!(fixture.artboard.set_double_property(scroll_local, index_key, 2.0));
        fixture.advance(0.0);
        assert!(fixture.rows().iter().all(|row| row.logical_index != 0));
        assert!(fixture.rows().iter().any(|row| row.logical_index == 5));

        let mut scrolled_factory = RecordingFactory::new();
        let mut scrolled_renderer = scrolled_factory.make_renderer();
        fixture.artboard.draw_artboard(
            &fixture.file,
            &graph,
            &fixture.graphs.artboards,
            &mut scrolled_factory,
            &mut scrolled_renderer,
            &Default::default(),
            None,
            true,
        ).expect("draw scrolled clipped viewport fixture");
        let scrolled_recording = scrolled_factory.canonical_recording();
        let scrolled_draws = wave_a_clip_probe(scrolled_recording.stream());
        assert_eq!(scrolled_draws.len(), 6);
        for item in &scrolled_draws[1..=4] {
            assert_eq!(item.clip.map(|clip| (clip.min_y, clip.max_y)), Some((100.0, 300.0)));
        }
        assert_eq!(scrolled_draws[1].bounds.min_y, viewport.min_y);
    }
