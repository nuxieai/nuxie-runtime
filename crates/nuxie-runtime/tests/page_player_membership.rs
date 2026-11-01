//! A Screen added to or removed from the page's `screens` list between frames
//! is set up before the frame plays (UNIV-5924). Every test drives a page
//! player with the page model subscribed, as the SDKs do.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::live_move::{records::LiveSlot, screens};
use nuxie_runtime::live_move_fixture as fixture;
use nuxie_runtime::page_player::{self, Output, Playback, Step};
use nuxie_runtime::{
    source::{
        generated::core_registry::CoreRegistry, nested_artboard::NestedArtboard, node::Node,
        viewmodel::viewmodel_instance::ViewModelInstance,
    },
    *,
};
use std::collections::{HashMap, HashSet};

struct Page {
    file: RuntimeFileHandle,
    factory: PersistentFactory<RecordingFactory>,
    board: ArtboardInstance,
    playback: Playback,
    vm: RuntimeOwnedViewModelHandle,
    player: page_player::PagePlayer,
    /// Screen B's model, out of the list until a test adds it between frames.
    incoming: RuntimeOwnedViewModelHandle,
}
impl Page {
    /// The page at rest on Screen A alone. B's model is taken out of the
    /// list, and A is created, with its spare copies put away, by one row
    /// wake before the first frame.
    fn resting_on_a(bytes: &[u8]) -> Self {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            bytes,
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .expect("fixture imports");
        let mut board = ArtboardInstance::from_native(file.clone(), 0).unwrap();
        let root = board.native_handle();
        let model = file
            .with_file(|f| f.create_default_view_model_instance_for_artboard(root.core_handle()))
            .unwrap();
        let vm = RuntimeOwnedViewModelHandle::from_native(file.clone(), model).unwrap();
        board.bind_owned_view_model_handle(vm.clone());
        let mut machine = board.default_state_machine_instance().unwrap();
        machine.bind_owned_view_model_handle(vm.clone());
        let incoming = vm.list_item_by_property_name_path("screens", 1).unwrap();
        assert!(vm.remove_list_item_by_property_name_path("screens", 1));
        let mut page = Self {
            file,
            factory,
            board,
            playback: Playback::StateMachine(Box::new(machine)),
            vm,
            player: page_player::PagePlayer::default(),
            incoming,
        };
        page.wake(0);
        ok(page.step(0.0), "the first frame on A");
        assert!(!screens(&page.root()).unwrap().contains_key("B"));
        page
    }
    fn root(&self) -> RuntimeArtboardInstanceHandle {
        self.board.native_handle()
    }
    fn row(&self, name: &str) -> RuntimeArtboardInstanceHandle {
        screens(&self.root()).unwrap()[name].clone()
    }
    /// Add Screen B back to the page model's list. Nothing else runs.
    fn insert_b(&self) {
        assert!(
            self.vm
                .insert_list_item_by_property_name_path("screens", 1, &self.incoming)
        );
    }
    fn step(&mut self, elapsed_seconds: f32) -> Result<Output, page_player::Error> {
        self.player.step(
            &mut self.board,
            &mut self.playback,
            Step {
                elapsed_seconds,
                ..Default::default()
            },
            std::slice::from_ref(&self.vm),
            None,
        )
    }
    /// The SDKs' row wake, which flushes the list first (iOS's `.expand`).
    fn wake(&mut self, row: i32) {
        self.player
            .set_row_collapsed(
                &mut self.board,
                "screens",
                row,
                false,
                std::slice::from_ref(&self.vm),
                None,
            )
            .unwrap_or_else(|e| panic!("waking row {row}: {:?}: {}", e.kind, e.message));
    }
    fn begin(&mut self, incoming: &str, name: &str) {
        self.player
            .step_begin(
                &mut self.board,
                incoming,
                Some(name),
                std::slice::from_ref(&self.vm),
                None,
            )
            .unwrap_or_else(|e| panic!("begin {name}: {:?}: {}", e.kind, e.message));
    }
    fn end(&mut self, incoming: &str, name: &str) {
        self.player
            .step_end(
                &mut self.board,
                incoming,
                Some(name),
                std::slice::from_ref(&self.vm),
                None,
            )
            .unwrap_or_else(|e| panic!("end {name}: {:?}: {}", e.kind, e.message));
    }
    fn screen_model(&self, index: usize) -> RuntimeOwnedViewModelHandle {
        self.vm
            .list_item_by_property_name_path("screens", index)
            .unwrap()
    }
    /// The entry-event fixture's `entered` number in a Screen's model.
    fn entered(&self, index: usize) -> f32 {
        let model = self.screen_model(index);
        let value = model.borrow().number_value_by_property_name("entered");
        value.unwrap()
    }
    fn drawing(&mut self) -> String {
        self.factory.borrow_mut().clear();
        let mut renderer = self.factory.borrow().make_renderer();
        self.root().draw(&mut renderer);
        self.factory
            .borrow()
            .canonical_recording()
            .stream()
            .to_string()
    }
    /// Every spare slot in Screen B is empty, and every reported change
    /// belongs to a model the scene still holds. A spare's own models leave
    /// the scene with the spare.
    fn assert_spares_away(&self, output: &Output) {
        let slots = spare_slots(&self.row("B"));
        assert!(!slots.is_empty(), "Screen B has spare slots");
        for slot in &slots {
            assert!(
                child(slot).is_none(),
                "a spare slot in Screen B still holds a copy"
            );
        }
        let mut scene: HashSet<u64> = hosted_scene::HostedIndex::default()
            .active_owned_models(&self.root())
            .into_iter()
            .filter_map(|model| RuntimeOwnedViewModelHandle::from_native(self.file.clone(), model))
            .map(|model| model.instance_identity())
            .collect();
        scene.extend(
            self.vm
                .reachable_change_owner_snapshot()
                .unwrap()
                .iter()
                .map(RuntimeOwnedViewModelHandle::instance_identity),
        );
        for owner in &output.changed_owners {
            assert!(
                scene.contains(&owner.instance_identity()),
                "a change belongs to a model of view model {} that left the scene",
                owner.borrow().view_model_index()
            );
        }
        for change in &output.changes {
            assert!(
                scene.contains(&change.owner_instance_identity),
                "a reported change belongs to a model that left the scene"
            );
        }
    }
}
fn ok(result: Result<Output, page_player::Error>, what: &str) -> Output {
    result.unwrap_or_else(|e| panic!("{what}: {:?}: {}", e.kind, e.message))
}
fn child(host: &CoreHandle) -> Option<RuntimeArtboardInstanceHandle> {
    host.with(|o| o.as_nested_artboard().unwrap().artboard_instance_default())
        .flatten()
}
/// A Screen's flight slots: the hosts of its role-1 slot records.
fn spare_slots(row: &RuntimeArtboardInstanceHandle) -> Vec<CoreHandle> {
    row.with_artboard(|b| b.find_all_handles::<LiveSlot>())
        .into_iter()
        .filter_map(|record| {
            record
                .with_downcast::<LiveSlot, _>(|r| {
                    if r.values.role == 1 { r.parent() } else { None }
                })
                .flatten()
        })
        .collect()
}
fn vm(board: &RuntimeArtboardInstanceHandle) -> CoreHandle {
    board
        .data_context()
        .unwrap()
        .with_context(|c| c.main_view_model_instance())
        .unwrap()
}
fn number(vm: &CoreHandle, name: &str) -> f32 {
    let property = vm
        .with_downcast::<ViewModelInstance, _>(|vm| vm.property_value_named(name))
        .flatten()
        .unwrap_or_else(|| panic!("missing {name}"));
    CoreRegistry::get_double_handle(&property, 575).unwrap()
}
fn close(actual: f32, expected: f32, what: &str) {
    assert!(
        (actual - expected).abs() < 0.002,
        "{what}: expected {expected}, got {actual}"
    );
}
/// A step's reported changes and events, with each owner named by its place
/// in the page model, so two hosts' steps compare.
fn reported(page: &Page, output: &Output) -> (Vec<String>, Vec<String>) {
    let mut names = HashMap::new();
    names.insert(page.vm.instance_identity(), "page".to_owned());
    let rows = page
        .vm
        .list_item_count_by_property_name_path("screens")
        .unwrap();
    for index in 0..rows {
        names.insert(
            page.screen_model(index).instance_identity(),
            format!("screen {index}"),
        );
    }
    for owner in &output.changed_owners {
        names
            .entry(owner.instance_identity())
            .or_insert_with(|| format!("a model of view model {}", owner.borrow().view_model_index()));
    }
    let name = |id: u64| {
        names
            .get(&id)
            .cloned()
            .unwrap_or_else(|| "an unknown model".to_owned())
    };
    let changes = output
        .changes
        .iter()
        .map(|change| {
            let value = match &change.value {
                RuntimeViewModelChangeValue::List(items) => format!(
                    "List({:?})",
                    items.iter().map(|id| name(*id)).collect::<Vec<_>>()
                ),
                RuntimeViewModelChangeValue::ViewModel(item) => {
                    format!("ViewModel({:?})", item.map(&name))
                }
                value => format!("{value:?}"),
            };
            format!(
                "{} #{} = {value}",
                name(change.owner_instance_identity),
                change.property_index
            )
        })
        .collect();
    let events = output
        .events
        .iter()
        .map(|event| {
            format!(
                "{:?} in artboard {:?} at {}",
                event.name.as_deref().map(String::from_utf8_lossy),
                event.source_artboard_global_id,
                event.event_local_index
            )
        })
        .collect();
    (changes, events)
}

/// R1. A host that steps right after the insert, as Android and iOS's first
/// Screen do. The new row's spare copies write on their first advance.
#[test]
fn a_screen_added_between_frames_puts_its_spare_copies_away_before_they_play() {
    let mut page = Page::resting_on_a(&fixture::spare_writes_fixture(false));
    page.insert_b();
    let output = ok(page.step(0.0), "the frame after the insert");
    page.assert_spares_away(&output);
}

/// R1's first control: the same order with a row wake between the insert
/// and the step.
#[test]
fn a_screen_woken_before_its_first_frame_puts_its_spare_copies_away() {
    let mut page = Page::resting_on_a(&fixture::spare_writes_fixture(false));
    page.insert_b();
    page.wake(1);
    let output = ok(page.step(0.0), "the frame after the insert and wake");
    page.assert_spares_away(&output);
}

/// R1's second control: spare copies that write nothing, added the same way.
#[test]
fn a_screen_whose_spares_write_nothing_opens_when_added_between_frames() {
    let mut page = Page::resting_on_a(&fixture::fixture(2));
    page.insert_b();
    let output = ok(page.step(0.0), "the frame after the insert");
    page.assert_spares_away(&output);
}

/// R2. Host F wakes the new row before its first frame; host U steps right
/// after the insert. Screen B's entry state fires an event whose listener
/// sets B's `entered` from 0 to 1.
#[test]
fn a_screen_added_between_frames_plays_its_first_frame_like_a_flushed_one() {
    let bytes = fixture::entry_event_fixture();
    let mut flushed = Page::resting_on_a(&bytes);
    flushed.insert_b();
    flushed.wake(1);
    let flushed_output = ok(flushed.step(0.0), "host F's first frame with B");
    let mut unflushed = Page::resting_on_a(&bytes);
    unflushed.insert_b();
    let unflushed_output = ok(unflushed.step(0.0), "host U's first frame with B");
    assert_eq!(
        flushed.entered(1),
        1.0,
        "host F: B's entry event reaches its listener in B's first frame"
    );
    assert_eq!(
        unflushed.entered(1),
        1.0,
        "host U: B's entry event reaches its listener in B's first frame"
    );
    assert_eq!(
        unflushed.drawing(),
        flushed.drawing(),
        "both hosts draw the same frame"
    );
    assert_eq!(
        reported(&unflushed, &unflushed_output),
        reported(&flushed, &flushed_output),
        "both hosts report the same changes and events"
    );
}

/// R2b. The new row's first frame plays the frame's own time: Screen B's
/// "entrance" keys its header's x from 200 to 300 over one second.
#[test]
fn a_screen_added_between_frames_plays_the_frames_time() {
    for (what, bytes) in [
        ("entry event", fixture::entry_event_fixture()),
        ("spare writes", fixture::spare_writes_fixture(false)),
    ] {
        let mut page = Page::resting_on_a(&bytes);
        page.insert_b();
        ok(
            page.step(0.25),
            &format!("{what}: the 0.25 s frame after the insert"),
        );
        let header = page
            .row("B")
            .with_artboard(|b| b.find_handle::<Node>("header"))
            .unwrap();
        close(
            CoreRegistry::get_double_handle(&header, 13).unwrap(),
            225.0,
            &format!("{what}: B's header x after its first 0.25 s frame"),
        );
    }
}

/// R3. The flush runs only for a frame with list changes waiting.
#[test]
fn a_frame_with_nothing_pending_runs_no_flush() {
    let mut page = Page::resting_on_a(&fixture::fixture(2));
    hosted_scene::measure_walks(true);
    for _ in 0..3 {
        ok(page.step(1.0 / 60.0), "a quiet frame");
    }
    assert_eq!(
        hosted_scene::measure_walks(true).membership_flush,
        0,
        "quiet frames run no flush"
    );
    page.insert_b();
    ok(page.step(0.0), "the frame after the insert");
    assert_eq!(
        hosted_scene::measure_walks(true).membership_flush,
        1,
        "the frame after an insert runs one flush"
    );
    ok(page.step(1.0 / 60.0), "the frame after that");
    assert_eq!(
        hosted_scene::measure_walks(false).membership_flush,
        0,
        "the next quiet frame runs no flush"
    );
}

/// R4. A guard for the removal that runs before the advance: the landed unit
/// goes home to Screen A with A's bindings.
#[test]
fn a_screen_removed_between_frames_sends_its_loans_home_first() {
    let mut page = Page::resting_on_a(&fixture::fixture(2));
    page.insert_b();
    page.wake(1);
    ok(page.step(0.0), "B's first frame");
    let home = page
        .row("A")
        .with_artboard(|b| b.find_all_handles::<NestedArtboard>())[0]
        .clone();
    let unit = child(&home).unwrap();
    page.begin("B", "t1");
    for _ in 0..3 {
        ok(page.step(1.0 / 60.0), "a flight frame");
    }
    page.end("B", "t1");
    ok(page.step(1.0 / 60.0), "the first frame in B");
    assert!(child(&home).is_none(), "the unit has landed in B");
    assert!(page.vm.remove_list_item_by_property_name_path("screens", 1));
    ok(page.step(0.0), "the frame after B's removal");
    assert!(!screens(&page.root()).unwrap().contains_key("B"));
    assert_eq!(
        child(&home).map(|c| c.core_handle()),
        Some(unit.core_handle()),
        "the unit is back in A's slot"
    );
    assert!(
        page.screen_model(0)
            .borrow_mut()
            .set_number_by_property_name("offset", 77.0)
    );
    ok(page.step(1.0 / 60.0), "the frame after A's offset changes");
    close(
        number(&vm(&unit), "prop"),
        77.0,
        "the unit's prop follows A's offset",
    );
}
