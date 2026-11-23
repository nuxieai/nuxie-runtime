//! A fresh instance of a hosted artboard runs its scripts from the start
//! (UNIV-6661). A driver gives a nested artboard host a fresh instance of its
//! artboard, or removes a keyed list row and mounts one with the same key,
//! and the scripted drawable inside the artboard runs its setup again, starts
//! its own state over, and the script object of the instance that left is
//! disposed in the round that replaced it.
//!
//! Oracle: the script itself. It numbers each instance it sets up (`init`)
//! and prints, on every advance, that number and the time the instance has
//! advanced, so a fresh instance shows as a new number counting from 0. The
//! VM's registry of live script objects says which objects are still set up.
use std::cell::RefCell;
use std::rc::Rc;

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::page_player::{
    DriverFault, DriverRound, HostDriver, PagePlayer, Playback, SceneObject, SceneWriter, Step,
};
use nuxie_runtime::source::scripted::scripted_drawable::ScriptedDrawable;
use nuxie_runtime::{
    ArtboardInstance, CoreHandle, File, ImportResult, RuntimeArtboardInstanceHandle,
    RuntimeFactoryHandle, RuntimeFileHandle, RuntimeScriptingVmHandle, keyed_rows_fixture,
    occurrence_fixture,
};
use nuxie_scripting::vm::ScriptVm;

const FRAME: f32 = 1.0 / 60.0;

/// Each instance takes the next number at `init` and starts its time at 0;
/// each advance prints the instance's number and its time in milliseconds.
const TICKER: &str = r#"
local made = 0
function init(self, context)
    made += 1
    self.serial = made
    self.elapsed = 0
    print("init " .. self.serial)
    return true
end
function advance(self, seconds)
    self.elapsed += seconds
    print("advance " .. self.serial .. " " .. math.floor(self.elapsed * 1000 + 0.5))
    return true
end
return function() return { init = init, advance = advance } end
"#;

/// `TICKER` as a script asset's contents: an unsigned content header, then
/// the bytecode.
fn ticker_payload() -> Vec<u8> {
    luaur_common::set_all_flags(true);
    let mut len = 0;
    let pointer = luaur_compiler::functions::luau_compile::luau_compile(
        TICKER.as_ptr().cast(),
        TICKER.len(),
        std::ptr::null_mut(),
        &mut len,
    );
    assert!(!pointer.is_null() && len > 0);
    let bytecode = unsafe { std::slice::from_raw_parts(pointer.cast::<u8>(), len) }.to_vec();
    unsafe extern "C" {
        fn free(p: *mut std::ffi::c_void);
    }
    unsafe { free(pointer.cast()) };
    assert_ne!(
        bytecode[0],
        0,
        "{}",
        String::from_utf8_lossy(&bytecode[1..])
    );
    [vec![0], bytecode].concat()
}

/// What the script printed, in order.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Line {
    Init(u32),
    Advance(u32, u32),
}

/// A driver from a closure, which sees every round.
struct FnDriver<D>(D);

impl<D: FnMut(&DriverRound<'_>, &mut SceneWriter<'_>)> HostDriver for FnDriver<D> {
    fn drive(
        &mut self,
        round: DriverRound<'_>,
        scene: &mut SceneWriter<'_>,
    ) -> Result<(), DriverFault> {
        (self.0)(&round, scene);
        Ok(())
    }
}

/// A file imported with a script VM whose console the test reads, played by
/// a page player on its state machine.
struct Page {
    _file: RuntimeFileHandle,
    _factory: PersistentFactory<RecordingFactory>,
    board: ArtboardInstance,
    playback: Playback,
    player: PagePlayer,
    vm: RuntimeScriptingVmHandle,
    lines: Rc<RefCell<Vec<Line>>>,
}

impl Page {
    fn open(bytes: &[u8]) -> Self {
        let lines = Rc::new(RefCell::new(Vec::new()));
        let sink = lines.clone();
        let vm = RuntimeScriptingVmHandle::new(Box::new(ScriptVm::new_with_log_sink(
            move |_, bytes| {
                let text = String::from_utf8_lossy(bytes);
                let words: Vec<_> = text.split_whitespace().collect();
                let number = |at: usize| words[at].parse::<u32>().expect("a number");
                let line = match words.first().copied() {
                    Some("init") => Line::Init(number(1)),
                    Some("advance") => Line::Advance(number(1), number(2)),
                    _ => panic!("the script said {text:?}"),
                };
                sink.borrow_mut().push(line);
            },
        )));
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let mut result = ImportResult::Malformed;
        let file = File::import(
            bytes,
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            Some(&mut result),
            None,
            Some(vm.clone()),
        )
        .unwrap_or_else(|| panic!("the fixture imports: {result:?}"));
        assert_eq!(result, ImportResult::Success);
        let mut board = ArtboardInstance::from_native(file.clone(), 0).unwrap();
        let machine = board.state_machine_instance(0).expect("the state machine");
        let mut page = Self {
            _file: file,
            _factory: factory,
            board,
            playback: Playback::StateMachine(Box::new(machine)),
            player: PagePlayer::default(),
            vm,
            lines,
        };
        page.step(0.0);
        page
    }

    fn root(&self) -> RuntimeArtboardInstanceHandle {
        self.board.native_handle()
    }

    fn step(&mut self, elapsed_seconds: f32) {
        self.player
            .step(
                &mut self.board,
                &mut self.playback,
                Step {
                    elapsed_seconds,
                    ..Step::default()
                },
                &[],
                None,
            )
            .expect("the step");
    }

    fn frames(&mut self, count: usize) {
        for _ in 0..count {
            self.step(FRAME);
        }
    }

    fn drive(&self, driver: impl FnMut(&DriverRound<'_>, &mut SceneWriter<'_>) + 'static) {
        self.player
            .set_driver(Some(Box::new(FnDriver(driver))))
            .expect("no step is running");
    }

    /// Run `write` in the first round of the next step, and nothing after.
    fn write_once(&mut self, write: impl FnOnce(&mut SceneWriter<'_>) + 'static) {
        let write = RefCell::new(Some(write));
        self.drive(move |round, scene| {
            if round.index == 0
                && let Some(write) = write.borrow_mut().take()
            {
                write(scene);
            }
        });
        self.step(FRAME);
        self.player.set_driver(None).expect("no step is running");
    }

    /// How many instances the script has set up.
    fn inits(&self) -> u32 {
        self.lines
            .borrow()
            .iter()
            .filter(|line| matches!(line, Line::Init(_)))
            .count() as u32
    }

    /// The last time instance `serial` printed, and how many lines it has
    /// printed since line `from`.
    fn last_time(&self, serial: u32, from: usize) -> Option<(u32, usize)> {
        let lines = self.lines.borrow();
        let times: Vec<_> = lines[from..]
            .iter()
            .filter_map(|line| match line {
                Line::Advance(s, time) if *s == serial => Some(*time),
                _ => None,
            })
            .collect();
        times.last().map(|time| (*time, times.len()))
    }

    /// The numbers of the instances that advanced since line `from`.
    fn advancing(&self, from: usize) -> Vec<u32> {
        let mut serials: Vec<_> = self.lines.borrow()[from..]
            .iter()
            .filter_map(|line| match line {
                Line::Advance(serial, _) => Some(*serial),
                _ => None,
            })
            .collect();
        serials.sort_unstable();
        serials.dedup();
        serials
    }
}

/// The scripted drawable of the artboard `instance`.
fn script_of(instance: &RuntimeArtboardInstanceHandle) -> CoreHandle {
    instance
        .with_artboard(|board| board.find_handle::<ScriptedDrawable>("Script"))
        .expect("the card's scripted drawable")
}

/// The instance the host at `local_id` of `root` holds.
fn hosted(root: &RuntimeArtboardInstanceHandle, local_id: usize) -> RuntimeArtboardInstanceHandle {
    root.with_artboard(|board| board.objects()[local_id].clone())
        .expect("the host")
        .with(|host| host.as_nested_artboard()?.artboard_instance_default())
        .flatten()
        .expect("the hosted card")
}

/// What the round that replaced an instance saw of the script object that
/// left: whether the VM still knew it and whether it still existed, and how
/// many script objects the VM knew.
#[derive(Debug, Default, PartialEq, Eq)]
struct AtTheSwap {
    known: bool,
    alive: bool,
    count: usize,
}

fn at_the_swap(vm: &RuntimeScriptingVmHandle, old: &CoreHandle) -> AtTheSwap {
    AtTheSwap {
        known: vm.knows_scripted_object(old),
        alive: old.with(|_| ()).is_some(),
        count: vm.scripted_object_count(),
    }
}

/// Proof B: a host given a fresh instance of its card runs the card's
/// script's setup once more, the new script starts from the start, and the
/// old script object is disposed in the round that replaced it. A second
/// replacement leaves the number of script objects the VM knows where it
/// was. The other use's script runs on untouched.
#[test]
fn a_replaced_instance_sets_its_script_up_again_and_disposes_the_old_one() {
    use occurrence_fixture::{PageMachine, page};
    let payload = ticker_payload();
    let mut page = Page::open(&occurrence_fixture::scripted_two_uses_fixture(
        PageMachine::OneLayer,
        &payload,
    ));
    page.frames(30);
    let root = page.root();
    let second_card = hosted(&root, page::SECOND);
    // Hold the first use's script object only: a handle on its instance
    // would keep the instance alive.
    let first_script = script_of(&hosted(&root, page::FIRST));
    let second_script = script_of(&second_card);
    assert!(page.vm.knows_scripted_object(&first_script));
    assert!(page.vm.knows_scripted_object(&second_script));
    let count = page.vm.scripted_object_count();
    let inits = page.inits();
    let before = page.lines.borrow().len();
    let serials = page.advancing(before.saturating_sub(4));
    assert_eq!(serials.len(), 2, "both uses' scripts advance: {serials:?}");
    for serial in &serials {
        let (time, _) = page.last_time(*serial, 0).unwrap();
        assert!(time >= 450, "instance {serial} has moved on: {time} ms");
    }

    for swap in 1..=2u32 {
        let old = script_of(&hosted(&page.root(), page::FIRST));
        let seen = Rc::new(RefCell::new(None));
        let (vm, record) = (page.vm.clone(), seen.clone());
        let old_in_round = old.clone();
        page.write_once(move |scene| {
            assert_eq!(
                scene.replace_instance(&SceneObject::root(page::FIRST)),
                Ok(true)
            );
            *record.borrow_mut() = Some(at_the_swap(&vm, &old_in_round));
        });
        let seen = seen.borrow_mut().take().expect("the driver ran");
        assert_eq!(
            seen,
            AtTheSwap {
                known: false,
                alive: false,
                count,
            },
            "swap {swap}: the old script object is gone in the round that replaced it"
        );
        assert_eq!(page.inits(), inits + swap, "swap {swap}: one more init");
        let new_script = script_of(&hosted(&page.root(), page::FIRST));
        assert_ne!(new_script, old);
        assert!(page.vm.knows_scripted_object(&new_script));
        assert_eq!(page.vm.scripted_object_count(), count, "swap {swap}");
        // Untouched: the other use keeps its instance and its script.
        assert_eq!(
            hosted(&page.root(), page::SECOND).core_handle(),
            second_card.core_handle()
        );
        assert!(page.vm.knows_scripted_object(&second_script));
    }

    // The newest instance counts from 0; the other use's runs on, and the
    // first use's first instance no longer advances.
    let serial = inits + 2;
    let from = page.lines.borrow().len();
    let start = page.last_time(serial, 0).map_or(0, |(time, _)| time);
    let advancing = page.advancing(from.saturating_sub(2));
    let survivor = *serials
        .iter()
        .find(|s| advancing.contains(s))
        .unwrap_or_else(|| panic!("the other use's script advances: {advancing:?}"));
    let (other_before, _) = page.last_time(survivor, 0).unwrap();
    page.frames(6);
    let expected = |frames: f32| (frames * FRAME * 1000.0).round() as u32;
    assert!(
        start <= expected(1.0),
        "it started from the start: {start} ms"
    );
    let (time, advances) = page
        .last_time(serial, from)
        .expect("the new script advances");
    assert_eq!(advances, 6);
    assert!(
        time.abs_diff(start + expected(6.0)) <= 1,
        "the new script's time is its own: {time} ms, {start} ms at the swap"
    );
    let (other, _) = page.last_time(survivor, from).unwrap();
    assert!(
        other.abs_diff(other_before + expected(6.0)) <= 1,
        "the other use's script ran on: {other} ms from {other_before} ms"
    );
    let mut live = vec![survivor, serial];
    live.sort_unstable();
    assert_eq!(
        page.advancing(from),
        live,
        "only the live instances advance"
    );
}

/// A keyed row removed and a row with the same key mounted again in one round
/// is a fresh instance: its script ran `init`, its time starts from the
/// start, and the removed row's script object is disposed when the round
/// ends, as the list takes the round's rows.
#[test]
fn a_keyed_row_mounted_again_sets_its_script_up_again() {
    use keyed_rows_fixture::page;
    let payload = ticker_payload();
    let mut page = Page::open(&keyed_rows_fixture::scripted_keyed_plans_fixture(&payload));
    let cards = SceneObject::root(page::CARDS);
    let list = cards.clone();
    page.write_once(move |scene| assert_eq!(scene.mount_row(&list, b"monthly"), Ok(true)));
    page.frames(30);
    let row = |page: &Page| {
        use nuxie_runtime::source::artboard_component_list::ArtboardComponentList;
        page.root()
            .with_artboard(|board| board.find_all_handles::<ArtboardComponentList>()[0].clone())
            .with_downcast::<ArtboardComponentList, _>(|list| list.artboard_instance(0))
            .flatten()
            .expect("the mounted row")
    };
    // Hold the script object only: a handle on the row's instance would
    // keep the instance alive.
    let old = script_of(&row(&page));
    assert!(page.vm.knows_scripted_object(&old));
    let count = page.vm.scripted_object_count();
    let inits = page.inits();
    let (moved, _) = page.last_time(inits, 0).expect("the row's script advances");
    assert!(moved >= 450, "the row's script has moved on: {moved} ms");

    // Round 0 removes the row and mounts the key again; round 1, which a
    // changed round always brings, sees what the round left.
    let seen = Rc::new(RefCell::new(Vec::new()));
    let (vm, record, old_in_round) = (page.vm.clone(), seen.clone(), old.clone());
    page.drive(move |round, scene| match round.index {
        0 => {
            assert_eq!(scene.remove_row(&cards, b"monthly"), Ok(true));
            assert_eq!(scene.mount_row(&cards, b"monthly"), Ok(true));
            record.borrow_mut().push(at_the_swap(&vm, &old_in_round));
        }
        1 => record.borrow_mut().push(at_the_swap(&vm, &old_in_round)),
        _ => {}
    });
    page.step(FRAME);
    page.player.set_driver(None).expect("no step is running");
    assert_eq!(
        *seen.borrow(),
        [
            // The list takes a round's order of rows once, after the round's
            // last operation (docs/page-player.md, Host driver), so the
            // removed row is still there while the round runs, beside the
            // new one.
            AtTheSwap {
                known: true,
                alive: true,
                count: count + 1,
            },
            AtTheSwap {
                known: false,
                alive: false,
                count,
            },
        ],
        "the removed row's script object is gone when the round that removed it ends"
    );
    assert_eq!(page.inits(), inits + 1, "the new row's script ran init");
    let new = script_of(&row(&page));
    assert_ne!(new, old);
    assert!(page.vm.knows_scripted_object(&new));
    assert_eq!(page.vm.scripted_object_count(), count);

    let serial = inits + 1;
    let from = page.lines.borrow().len();
    let start = page.last_time(serial, 0).map_or(0, |(time, _)| time);
    assert!(start <= 17, "it started from the start: {start} ms");
    page.frames(6);
    let (time, _) = page.last_time(serial, from).unwrap();
    assert!(
        time.abs_diff(start + 100) <= 1,
        "the new row's time is its own: {time} ms from {start} ms"
    );
    assert_eq!(page.advancing(from), vec![serial]);
}
