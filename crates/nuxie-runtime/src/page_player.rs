//! One transactional page operation for every embedding host.
use crate::source::input::focusable::{Key, KeyModifiers};
use crate::*;
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, HashSet},
    rc::Rc,
};
const MAX_PLAYER_STEP_STATE_CHANGES: usize = 4096;
const MAX_PLAYER_STEP_RESULT_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    InvalidArgument,
    NotFound,
    LimitExceeded,
    ReentrantCall,
    RuntimeError,
}
#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}
fn failure<T>(kind: ErrorKind, message: impl Into<String>) -> Result<T, Error> {
    Err(Error {
        kind,
        message: message.into(),
    })
}
fn policy_error(error: RuntimeValuePolicyError) -> ErrorKind {
    match error {
        RuntimeValuePolicyError::NotFound => ErrorKind::NotFound,
        RuntimeValuePolicyError::LimitExceeded => ErrorKind::LimitExceeded,
        RuntimeValuePolicyError::BorrowConflict => ErrorKind::ReentrantCall,
        _ => ErrorKind::InvalidArgument,
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum HostValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    List(Vec<HostValue>),
    Object(BTreeMap<String, HostValue>),
}
#[derive(Debug, Clone, PartialEq)]
pub struct HostCommand {
    pub name: String,
    pub payload: HostValue,
}
/// VM-specific integration. The runtime owns the timing and the returned commands.
pub trait Effects {
    fn begin(&self) -> Box<dyn EffectTransaction>;
}
pub trait EffectTransaction {
    fn finish(self: Box<Self>) -> Result<Vec<HostCommand>, ScriptError>;
}
#[derive(Default)]
pub(crate) struct Boundary {
    active: Cell<bool>,
    poisoned: Cell<bool>,
}
struct Guard {
    boundary: Rc<Boundary>,
    committed: bool,
}
impl Drop for Guard {
    fn drop(&mut self) {
        self.boundary.active.set(false);
        if !self.committed || std::thread::panicking() {
            self.boundary.poisoned.set(true);
        }
    }
}
#[derive(Default)]
pub struct PagePlayer {
    pending_commands: Rc<RefCell<Vec<HostCommand>>>,
}
#[derive(Debug)]
pub struct StateChange {
    pub layer_index: usize,
    pub state_core_type: u32,
    pub state_global_id: u32,
}
pub struct Output {
    pub keep_going: bool,
    pub dirty: bool,
    pub settled: bool,
    pub pointers: Vec<RuntimeHitResult>,
    pub focus: Vec<u32>,
    pub states: Vec<StateChange>,
    pub events: Vec<OwnedPlayerEvent>,
    pub commands: Vec<HostCommand>,
    pub changes: Vec<RuntimeViewModelChange>,
    pub changed_owners: Vec<RuntimeOwnedViewModelHandle>,
    pub rule_reports: Vec<RuntimeValueRuleReport>,
}
/// Publication is part of the transaction. Dropping this value before commit
/// rolls VM values back and poisons the occurrence after runtime mutation.
pub struct PendingStep<'a, G = ()> {
    _host_scope: G,
    pub output: Option<Output>,
    transaction: Option<RuntimeOwnedViewModelGraphTransaction>,
    _scope: Option<live_move::FrameScope>,
    rules: Option<Rc<RefCell<RuntimeValuePolicyOperation>>>,
    policy: Option<&'a RuntimeValuePolicy>,
    commands: Rc<RefCell<Vec<HostCommand>>>,
    deferred: Vec<HostCommand>,
    guard: Guard,
}
impl<G> PendingStep<'_, G> {
    pub fn commit_values(&mut self) {
        if let Some(transaction) = self.transaction.take() {
            transaction.commit();
        }
    }
    pub fn commit(mut self) {
        self.commit_values();
        if let (Some(policy), Some(rules)) = (self.policy, &self.rules) {
            rules.borrow_mut().commit_groups(policy);
        }
        *self.commands.borrow_mut() = std::mem::take(&mut self.deferred);
        self.guard.committed = true;
    }
}
#[derive(Debug)]
pub enum Operation {
    Begin(String, Option<String>),
    End(String, Option<String>),
    Row(String, i32, bool),
}
impl Operation {
    fn apply(self, root: &RuntimeArtboardInstanceHandle) -> Result<(), ErrorKind> {
        let mut moves = live_move::LiveMoves::default();
        let result = match self {
            Self::Begin(incoming, name) => moves.step_begin(root, &incoming, name.as_deref()),
            Self::End(incoming, name) => moves.step_end(root, &incoming, name.as_deref()),
            Self::Row(name, index, collapsed) => {
                moves
                    .prepare(root)
                    .map_err(|_| ErrorKind::InvalidArgument)?;
                let list = root
                    .with_artboard(|b| {
                        b.find_handle::<source::artboard_component_list::ArtboardComponentList>(
                            &name,
                        )
                    })
                    .ok_or(ErrorKind::InvalidArgument)?;
                live_move::set_row_collapsed(&list, index, collapsed)
                    .map_err(|_| ErrorKind::InvalidArgument)?;
                moves.prepare(root)
            }
        };
        result.map_err(|_| ErrorKind::InvalidArgument)
    }
}
pub enum Playback {
    StaticArtboard,
    StateMachine(Box<StateMachineInstance>),
    LinearAnimation(Box<LinearAnimationInstance>),
}

#[derive(Debug)]
pub enum Input {
    Bool { index: usize, value: bool },
    Number { index: usize, value: f32 },
    Trigger { index: usize },
}

#[derive(Debug, Clone, Copy)]
pub enum PointerKind {
    Down,
    Move,
    Up,
    Exit,
}

#[derive(Debug, Clone, Copy)]
pub struct Pointer {
    pub kind: PointerKind,
    pub x: f32,
    pub y: f32,
    pub pointer_id: i32,
    pub timestamp_seconds: f32,
}

/// Product-neutral validated operation boundary. UNIV-1822 can append its
/// detached VM mutation candidate here after extending the caller-sized step
/// prefix, without changing the transaction/commit path below.
#[derive(Default)]
pub struct Step {
    pub inputs: Vec<Input>,
    pub pointers: Vec<Pointer>,
    pub focus_inputs: Vec<FocusInput>,
    pub elapsed_seconds: f32,
}

pub enum FocusInput {
    Next,
    Previous,
    Clear,
    Key {
        key: Key,
        modifiers: KeyModifiers,
        pressed: bool,
        repeat: bool,
    },
    Text(String),
}

fn apply_focus_input(machine: &mut StateMachineInstance, input: &FocusInput) -> u32 {
    u32::from(match input {
        FocusInput::Next => machine.focus_next(),
        FocusInput::Previous => machine.focus_previous(),
        FocusInput::Clear => {
            machine.clear_focus();
            false
        }
        FocusInput::Key {
            key,
            modifiers,
            pressed,
            repeat,
        } => machine.key_input(*key, *modifiers, *pressed, *repeat),
        FocusInput::Text(text) => machine.text_input(text),
    })
}

fn apply_player_pointer(machine: &mut StateMachineInstance, pointer: Pointer) -> RuntimeHitResult {
    let result = match pointer.kind {
        PointerKind::Down => machine.pointer_down(pointer.x, pointer.y, pointer.pointer_id),
        PointerKind::Move => machine.pointer_move(
            pointer.x,
            pointer.y,
            pointer.timestamp_seconds,
            pointer.pointer_id,
        ),
        PointerKind::Up => machine.pointer_up(pointer.x, pointer.y, pointer.pointer_id),
        PointerKind::Exit => machine.pointer_exit(pointer.x, pointer.y, pointer.pointer_id),
    };
    result
}

impl PagePlayer {
    pub fn operation<'a, G>(
        &self,
        artboard: &mut ArtboardInstance,
        playback: &mut Playback,
        prepared: Step,
        live_operation: Option<Operation>,
        subscribed_roots: &[RuntimeOwnedViewModelHandle],
        binding: Option<&RuntimeOwnedViewModelHandle>,
        policy: Option<&'a RuntimeValuePolicy>,
        can_skip_quiet_graphs: bool,
        effects: Option<&dyn Effects>,
        enter_rules: impl FnOnce(Option<Rc<RefCell<RuntimeValuePolicyOperation>>>) -> G,
    ) -> Result<PendingStep<'a, G>, Error> {
        let boundary = artboard
            .native_handle()
            .with_artboard_mut(|b| b.page_boundary.get_or_insert_with(Default::default).clone());
        if boundary.poisoned.get() {
            return failure(ErrorKind::RuntimeError, "page occurrence is poisoned");
        }
        if boundary.active.replace(true) {
            return failure(ErrorKind::ReentrantCall, "page occurrence is active");
        }
        let mut guard = Guard {
            boundary: boundary.clone(),
            committed: true,
        };
        if !prepared.elapsed_seconds.is_finite() || prepared.elapsed_seconds < 0.0 {
            return failure(ErrorKind::InvalidArgument, "invalid elapsed time");
        }
        validate_step(artboard, playback, &prepared)?;
        let advances = live_operation.is_none();
        let has_value_policy =
            policy.is_some_and(|p| p.has_markers() || p.has_rules() || p.has_groups());
        #[cfg(feature = "test-support")]
        let mut live_timer = hosted_scene::StepTimer::new("frame_scope");
        let live_frame_scope = match crate::live_move::LiveMoves::default()
            .frame_scope(&artboard.native_handle(), &subscribed_roots)
        {
            Ok(scope) => scope,
            Err(error) => {
                return failure(
                    ErrorKind::InvalidArgument,
                    format!("live frame scope: {error:#}"),
                );
            }
        };
        #[cfg(feature = "test-support")]
        live_timer.mark("owner_scope");
        let retained_subscribed_owners = if has_value_policy {
            match retain_scope(&subscribed_roots) {
                Ok(owners) => owners,
                Err(status) => {
                    return failure(status, "value policy subscription scope exceeds the limit");
                }
            }
        } else {
            Vec::new()
        };
        let has_subscriptions = !subscribed_roots.is_empty() || has_value_policy;
        let scene_view_model_roots = match has_subscriptions {
            true => {
                match scene_roots(&artboard, &subscribed_roots, MAX_PLAYER_STEP_STATE_CHANGES) {
                    Ok(roots) => roots,
                    Err(status) => {
                        return failure(
                            status,
                            "player occurrence view-model scope exceeds its bound or is invalid",
                        );
                    }
                }
            }
            false => Vec::new(),
        };
        let retained_policy_scope = if has_value_policy {
            match retain_scope(&scene_view_model_roots) {
                Ok(roots) => roots,
                Err(status) => {
                    return failure(status, "value policy scope exceeds the limit");
                }
            }
        } else {
            Vec::new()
        };
        #[cfg(feature = "test-support")]
        live_timer.mark("checkpoint");
        let view_model_transaction = match has_subscriptions {
            true => match if live_frame_scope.is_some() {
                RuntimeOwnedViewModelGraphTransaction::begin_live(
                    &scene_view_model_roots,
                    MAX_PLAYER_STEP_STATE_CHANGES,
                )
            } else {
                RuntimeOwnedViewModelGraphTransaction::begin(
                    &scene_view_model_roots,
                    MAX_PLAYER_STEP_STATE_CHANGES,
                )
            } {
                Ok(transaction) => Some(transaction),
                Err(RuntimeViewModelGraphTransactionError::LimitExceeded) => {
                    return failure(
                        ErrorKind::LimitExceeded,
                        "bound view-model graph exceeds the operation checkpoint bound",
                    );
                }
                Err(
                    RuntimeViewModelGraphTransactionError::Reentrant
                    | RuntimeViewModelGraphTransactionError::BorrowConflict,
                ) => {
                    return failure(
                        ErrorKind::ReentrantCall,
                        "bound view-model graph is already active",
                    );
                }
            },
            false => None,
        };
        #[cfg(feature = "test-support")]
        live_timer.mark("input_setup");
        let change_capture = if has_subscriptions {
            match RuntimeViewModelChangeCapture::begin_bounded(
                MAX_PLAYER_STEP_STATE_CHANGES,
                MAX_PLAYER_STEP_RESULT_BYTES,
            ) {
                Some(capture) => Some(capture),
                None => {
                    return failure(
                        ErrorKind::ReentrantCall,
                        "a view-model change capture is already active",
                    );
                }
            }
        } else {
            None
        };
        let rule_operation = match policy
            .filter(|policy| policy.has_rules() || policy.has_groups())
            .map(|policy| policy.begin_rules(&scene_view_model_roots))
            .transpose()
        {
            Ok(operation) => operation.map(|operation| Rc::new(RefCell::new(operation))),
            Err(error) => {
                return failure(policy_error(error), "value rule checkpoint failed");
            }
        };
        let _host_scope = enter_rules(rule_operation.clone());
        let mut rule_reports = Vec::new();
        let mut pointer_results = Vec::with_capacity(prepared.pointers.len());
        let mut focus_input_results = Vec::with_capacity(prepared.focus_inputs.len());
        let mut state_changes = Vec::new();
        let mut reported_events = Vec::new();
        let script_transaction = effects.map(Effects::begin);
        if let (Some(policy), Some(capture)) = (policy, change_capture.as_ref()) {
            policy.prepare_capture(capture);
        }
        guard.committed = false;
        let initial_groups_changed = match (policy, rule_operation.as_ref()) {
            (Some(policy), Some(operation)) => match operation.borrow_mut().apply_initial_groups(
                policy,
                &scene_view_model_roots,
                |_, _| Ok(()),
            ) {
                Ok(changed) => changed,
                Err(error) => {
                    boundary.poisoned.set(true);
                    return failure(policy_error(error), "initial rule group pass failed");
                }
            },
            _ => false,
        };
        #[cfg(feature = "test-support")]
        live_timer.mark("advance");
        let (mut keep_going, mut runtime_dirty, mut runtime_settled) =
            if let Some(operation) = live_operation {
                let root = artboard.native_handle();
                let membership = crate::hosted_scene::membership_revision(&root.core_handle());
                let serial = root.with_artboard(|b| b.change_serial());
                let applied = operation.apply(&root);
                if let Err(status) = applied {
                    // Caller preflight refusals leave the occurrence usable. A failure
                    // after ownership, component or VM mutation uses the step's poison
                    // policy; dropping the checkpoint alone cannot undo those effects.
                    if membership != crate::hosted_scene::membership_revision(&root.core_handle())
                        || serial != root.with_artboard(|b| b.change_serial())
                        || change_capture
                            .as_ref()
                            .is_some_and(|c| c.write_count().map_or(true, |n| n != 0))
                    {
                        boundary.poisoned.set(true);
                    }
                    guard.committed = !boundary.poisoned.get();
                    return failure(status, "live move operation refused");
                }
                (false, true, true)
            } else {
                match playback {
                    Playback::StateMachine(machine) => {
                        // Upstream command_server.cpp separates bindViewModelInstance from
                        // advanceStateMachine. Binding reports pending triggers, so it must
                        // not be repeated merely because the player advances another frame.
                        if let Some(view_model) = binding {
                            machine.bind_owned_view_model_handle(view_model.clone());
                        }
                        let mut input_dirty = false;
                        for input in prepared.inputs {
                            match input {
                                Input::Bool { index, value } => {
                                    input_dirty |= machine.set_bool(index, value);
                                }
                                Input::Number { index, value } => {
                                    input_dirty |= machine.set_number(index, value);
                                }
                                Input::Trigger { index } => {
                                    input_dirty |= machine.fire_trigger(index);
                                }
                            }
                        }
                        for pointer in prepared.pointers.iter().copied() {
                            pointer_results.push(apply_player_pointer(machine, pointer));
                        }
                        for input in &prepared.focus_inputs {
                            focus_input_results.push(apply_focus_input(machine, input));
                        }
                        // Pointer and focus callbacks can report events immediately. Drain
                        // those before advancement, then append events authored by the
                        // advance itself so the result preserves C++ production order.
                        reported_events = machine.take_reported_events();
                        let advance = match artboard.advance_state_machine_instances(
                            std::slice::from_mut(&mut **machine),
                            prepared.elapsed_seconds,
                            true,
                        ) {
                            Ok(advance) => advance,
                            Err(error) => {
                                boundary.poisoned.set(true);
                                return failure(
                                    ErrorKind::RuntimeError,
                                    format!("player advance failed: {error:#}"),
                                );
                            }
                        };
                        let state_change_count = machine.changed_state_count();
                        if state_change_count > MAX_PLAYER_STEP_STATE_CHANGES {
                            boundary.poisoned.set(true);
                            return failure(
                                ErrorKind::LimitExceeded,
                                "runtime state-change count exceeds step bound",
                            );
                        }
                        for index in 0..state_change_count {
                            let Some(state) = machine.changed_state(index) else {
                                boundary.poisoned.set(true);
                                return failure(
                                    ErrorKind::RuntimeError,
                                    "runtime state-change projection is inconsistent",
                                );
                            };
                            state_changes.push(StateChange {
                                // The exact source projection orders changed states by
                                // authored layer. Until the C ABI grows an optional-id
                                // representation, the compressed index and sentinel are
                                // the only lossless C representation available.
                                layer_index: index,
                                state_core_type: u32::from(state.core_type),
                                state_global_id: u32::MAX,
                            });
                        }
                        reported_events.extend(machine.take_reported_events());
                        (
                            advance.keep_going,
                            input_dirty || advance.changed,
                            !advance.keep_going,
                        )
                    }
                    Playback::LinearAnimation(animation) => {
                        pointer_results.resize(prepared.pointers.len(), RuntimeHitResult::None);
                        let advance = match artboard
                            .advance_linear_animation_instance(animation, prepared.elapsed_seconds)
                        {
                            Ok(advance) => advance,
                            Err(error) => {
                                boundary.poisoned.set(true);
                                return failure(
                                    ErrorKind::RuntimeError,
                                    format!("player advance failed: {error:#}"),
                                );
                            }
                        };
                        reported_events.extend(advance.reported_events);
                        (advance.keep_going, advance.changed, !advance.keep_going)
                    }
                    Playback::StaticArtboard => {
                        pointer_results.resize(prepared.pointers.len(), RuntimeHitResult::None);
                        let dirty = match artboard.advance(0.0) {
                            Ok(dirty) => dirty,
                            Err(error) => {
                                boundary.poisoned.set(true);
                                return failure(
                                    ErrorKind::RuntimeError,
                                    format!("player advance failed: {error:#}"),
                                );
                            }
                        };
                        // StaticScene's C++ keepGoing can report true for the initial
                        // zero-time settlement, but it does not represent continuing
                        // scheduled work for the host presentation loop.
                        (dirty, dirty, true)
                    }
                }
            };

        #[cfg(feature = "test-support")]
        live_timer.mark("follow");
        if advances {
            match crate::live_move::LiveMoves::default().after_advance(&artboard.native_handle()) {
                Ok(changed) => runtime_dirty |= changed,
                Err(error) => {
                    boundary.poisoned.set(true);
                    return failure(
                        ErrorKind::RuntimeError,
                        format!("live move update failed: {error:#}"),
                    );
                }
            }
        }

        #[cfg(feature = "test-support")]
        live_timer.mark("events");
        if view_model_transaction
            .as_ref()
            .is_some_and(|t| !t.checkpoint_within_limit())
        {
            // VM rollback cannot undo dispatched input, playback or flight ownership.
            boundary.poisoned.set(true);
            return failure(
                ErrorKind::LimitExceeded,
                "live frame checkpoint exceeds its bound",
            );
        }
        let events = own_reported_events(
            reported_events,
            pointer_results.len(),
            focus_input_results.len(),
            state_changes.len(),
        )
        .map_err(|(kind, message)| Error {
            kind,
            message: message.into(),
        })?;
        let commands = match script_transaction {
            Some(transaction) => transaction.finish().map_err(|error| Error {
                kind: ErrorKind::RuntimeError,
                message: format!("player step cannot project host effects: {error}"),
            })?,
            None => Vec::new(),
        };
        let commands = self
            .pending_commands
            .borrow()
            .iter()
            .cloned()
            .chain(commands)
            .collect::<Vec<_>>();
        let deferred_commands = (!advances).then(|| commands.clone());
        if initial_groups_changed {
            runtime_dirty = true;
            keep_going = true;
            runtime_settled = false;
        }
        #[cfg(feature = "test-support")]
        live_timer.mark("value_resolution");
        let resolved_view_model_changes = match change_capture {
            // A script refusal may need group output even after its capture rows
            // were consumed. Skip graph resolution only when both are quiet.
            Some(capture)
                if can_skip_quiet_graphs
                    && capture.write_count().is_ok_and(|count| count == 0)
                    && match (rule_operation.as_ref(), policy) {
                        (Some(operation), Some(policy)) => operation
                            .borrow()
                            .needs_policy_pass(policy, &capture, 0)
                            .is_ok_and(|pending| !pending),
                        _ => true,
                    } =>
            {
                if let Some(operation) = rule_operation.as_ref() {
                    rule_reports = operation.borrow().reports().to_vec();
                }
                drop(capture);
                Vec::new()
            }
            Some(capture) => {
                let mut roots = match scene_roots(
                    &artboard,
                    &subscribed_roots,
                    MAX_PLAYER_STEP_STATE_CHANGES,
                ) {
                    Ok(roots) => roots,
                    Err(status) => {
                        boundary.poisoned.set(true);
                        return failure(
                            status,
                            "player occurrence view-model scope exceeds its bound or is invalid",
                        );
                    }
                };
                roots.extend(retained_policy_scope);
                if let Some(policy) = policy {
                    let mut policy_cursor = 0;
                    loop {
                        let pending = match rule_operation.as_ref() {
                            Some(operation) => operation.borrow().needs_policy_pass(
                                policy,
                                &capture,
                                policy_cursor,
                            ),
                            None => capture
                                .write_count()
                                .map(|count| count != policy_cursor)
                                .map_err(|_| crate::RuntimeValuePolicyError::LimitExceeded),
                        };
                        match pending {
                            Ok(false) => break,
                            Ok(true) => {}
                            Err(error) => {
                                boundary.poisoned.set(true);
                                return failure(
                                    policy_error(error),
                                    "value policy capture exceeds its bound",
                                );
                            }
                        }
                        let rules_changed = match rule_operation
                            .as_ref()
                            .map(|operation| operation.borrow_mut().apply(policy, &capture, &roots))
                            .transpose()
                        {
                            Ok(changed) => changed.unwrap_or(false),
                            Err(error) => {
                                boundary.poisoned.set(true);
                                return failure(policy_error(error), "value rule pass failed");
                            }
                        };
                        policy_cursor = match capture.write_count() {
                            Ok(count) => count,
                            Err(_) => {
                                boundary.poisoned.set(true);
                                return failure(
                                    ErrorKind::LimitExceeded,
                                    "value policy capture exceeds its bound",
                                );
                            }
                        };
                        if policy.has_groups()
                            && let Some(operation) = rule_operation.as_ref()
                        {
                            let mut known = roots
                                .iter()
                                .map(RuntimeOwnedViewModelHandle::instance_identity)
                                .collect::<std::collections::BTreeSet<_>>();
                            roots.extend(
                                operation
                                    .borrow()
                                    .retained_roots()
                                    .into_iter()
                                    .filter(|owner| known.insert(owner.instance_identity())),
                            );
                        }
                        let marker_result =
                            policy.apply_markers(&capture, &roots, |owner, index, value| {
                                let mut owner = owner
                                    .try_borrow_mut()
                                    .map_err(|_| crate::RuntimeValuePolicyError::BorrowConflict)?;
                                Ok(owner.set_boolean_by_property_index(index, value))
                            });
                        let group_result = marker_result.and_then(|markers_changed| {
                            let groups_changed = rule_operation
                                .as_ref()
                                .map(|operation| {
                                    operation
                                        .borrow_mut()
                                        .apply_groups(policy, &roots, |_, _| Ok(()))
                                })
                                .transpose()?
                                .unwrap_or(false);
                            Ok(markers_changed || rules_changed || groups_changed)
                        });
                        match group_result {
                            Ok(false) => break,
                            Ok(true) => {
                                runtime_dirty = true;
                                keep_going = true;
                                runtime_settled = false;
                                policy.flush_marker_bindings(&artboard);
                                match scene_roots(
                                    &artboard,
                                    &subscribed_roots,
                                    MAX_PLAYER_STEP_STATE_CHANGES,
                                ) {
                                    Ok(current) => {
                                        let mut known = roots
                                            .iter()
                                            .map(RuntimeOwnedViewModelHandle::instance_identity)
                                            .collect::<std::collections::BTreeSet<_>>();
                                        roots.extend(
                                            current.into_iter().filter(|root| {
                                                known.insert(root.instance_identity())
                                            }),
                                        );
                                    }
                                    Err(status) => {
                                        boundary.poisoned.set(true);
                                        return failure(
                                            status,
                                            "value policy binding scope is invalid",
                                        );
                                    }
                                }
                            }
                            Err(error) => {
                                boundary.poisoned.set(true);
                                return failure(policy_error(error), "value policy pass failed");
                            }
                        }
                    }
                }
                if let Some(operation) = rule_operation.as_ref() {
                    let operation = operation.borrow();
                    roots.extend(operation.retained_roots());
                    rule_reports = operation.reports().to_vec();
                }
                // The advance can detach an occurrence after it wrote a bound
                // value, as a removed component-list row does. Resolve against the
                // frame's checkpointed owners as well as the live graph.
                let Some(transaction) = view_model_transaction.as_ref() else {
                    boundary.poisoned.set(true);
                    return failure(
                        ErrorKind::RuntimeError,
                        "runtime view-model change capture has no frame checkpoint",
                    );
                };
                match transaction.resolve_change_capture_with_owners(&roots, capture) {
                    Some(changes) => changes,
                    None => {
                        boundary.poisoned.set(true);
                        return failure(
                            ErrorKind::LimitExceeded,
                            "runtime view-model change journal exceeded its bound or left the player occurrence graph",
                        );
                    }
                }
            }
            None => Vec::new(),
        };
        #[cfg(feature = "test-support")]
        live_timer.mark("publication");

        let changed_owners = resolved_view_model_changes
            .iter()
            .map(|(o, _)| o.clone())
            .collect::<Vec<_>>();
        let mut subscribed_owners = retained_subscribed_owners;
        if !can_skip_quiet_graphs || !resolved_view_model_changes.is_empty() {
            for root in subscribed_roots {
                let Some(owners) = root.reachable_change_owner_snapshot() else {
                    return failure(ErrorKind::RuntimeError, "invalid subscription graph");
                };
                subscribed_owners.extend(owners);
            }
        }
        let changes = resolved_view_model_changes
            .into_iter()
            .filter(|(owner, _)| subscribed_owners.iter().any(|root| root.ptr_eq(owner)))
            .map(|(_, change)| change)
            .collect();
        runtime_dirty |= !changed_owners.is_empty();
        Ok(PendingStep {
            _host_scope,
            output: Some(Output {
                keep_going,
                dirty: runtime_dirty,
                settled: runtime_settled,
                pointers: pointer_results,
                focus: focus_input_results,
                states: state_changes,
                events,
                commands,
                changes,
                changed_owners,
                rule_reports,
            }),
            transaction: view_model_transaction,
            _scope: live_frame_scope,
            rules: rule_operation,
            policy,
            commands: self.pending_commands.clone(),
            deferred: deferred_commands.unwrap_or_default(),
            guard,
        })
    }
}
fn scene_roots(
    artboard: &ArtboardInstance,
    subscribed: &[RuntimeOwnedViewModelHandle],
    limit: usize,
) -> Result<Vec<RuntimeOwnedViewModelHandle>, ErrorKind> {
    collect_roots(
        artboard,
        subscribed,
        limit,
        &hosted_scene::HostedIndex::default().active(&artboard.native_handle()),
    )
}
fn collect_roots(
    artboard: &ArtboardInstance,
    subscribed: &[RuntimeOwnedViewModelHandle],
    limit: usize,
    boards: &[crate::CoreHandle],
) -> Result<Vec<RuntimeOwnedViewModelHandle>, ErrorKind> {
    if boards.len() > limit {
        return Err(ErrorKind::LimitExceeded);
    }
    let file = artboard.native_file();
    let mut roots = Vec::new();
    let mut seen = HashSet::new();
    let mut contexts = Vec::new();
    for root in subscribed {
        if seen.insert(root.native_handle()) {
            roots.push(root.clone());
        }
    }
    // Authored inline models are part of the scene too, including a flight
    // slot's model after its fresh child is retired during initial settlement.
    // This shared boundary supplies both eager and lazy checkpoints with the
    // same real scene roots, never with arbitrary owners from the write journal.
    for instance in
        crate::hosted_scene::HostedIndex::default().active_owned_models(&artboard.native_handle())
    {
        if seen.insert(instance.clone()) {
            roots.push(
                RuntimeOwnedViewModelHandle::from_native(file.clone(), instance)
                    .ok_or(ErrorKind::RuntimeError)?,
            );
        }
    }
    for board in boards {
        #[cfg(feature = "test-support")]
        if *board != artboard.native_handle().core_handle() {
            crate::hosted_scene::record_walk(crate::hosted_scene::Walk::OwnerCapture);
        }
        let mut context = board
            .with_downcast::<crate::Artboard, _>(|b| b.data_context())
            .flatten();
        while let Some(current) = context {
            if contexts.iter().any(|c| current.ptr_eq(c)) {
                break;
            }
            if contexts.len() >= limit {
                return Err(ErrorKind::LimitExceeded);
            }
            let (instances, parent) =
                current.with_context(|c| (c.view_model_instances().to_vec(), c.parent()));
            contexts.push(current);
            for instance in instances.into_iter().flatten() {
                if seen.insert(instance.clone()) {
                    roots.push(
                        RuntimeOwnedViewModelHandle::from_native(file.clone(), instance)
                            .ok_or(ErrorKind::RuntimeError)?,
                    );
                }
            }
            context = parent;
        }
    }
    if roots.len() > limit {
        return Err(ErrorKind::LimitExceeded);
    }
    Ok(roots)
}
fn retain_scope(
    roots: &[RuntimeOwnedViewModelHandle],
) -> Result<Vec<RuntimeOwnedViewModelHandle>, ErrorKind> {
    let mut owners = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for root in roots {
        for owner in root
            .reachable_change_owner_snapshot()
            .ok_or(ErrorKind::ReentrantCall)?
        {
            if seen.insert(owner.instance_identity()) {
                if owners.len() >= MAX_PLAYER_STEP_STATE_CHANGES {
                    return Err(ErrorKind::LimitExceeded);
                }
                owners.push(owner);
            }
        }
    }
    Ok(owners)
}

/// Terminal failure is shared by every player of the native occurrence.
pub fn is_poisoned(root: &RuntimeArtboardInstanceHandle) -> bool {
    root.with_artboard(|b| b.page_boundary.as_ref().is_some_and(|s| s.poisoned.get()))
}

fn validate_step(
    artboard: &ArtboardInstance,
    playback: &Playback,
    step: &Step,
) -> Result<(), Error> {
    if step.inputs.len() > 4096 || step.pointers.len() > 4096 || step.focus_inputs.len() > 4096 {
        return failure(
            ErrorKind::LimitExceeded,
            "player step batch exceeds fixed item bounds",
        );
    }
    let board = match playback {
        Playback::StateMachine(m) => Some(m.native_artboard()),
        Playback::LinearAnimation(a) => Some(a.native_artboard()),
        Playback::StaticArtboard => None,
    };
    if board.is_some_and(|b| b.core_handle() != artboard.native_handle().core_handle()) {
        return failure(
            ErrorKind::InvalidArgument,
            "playback belongs to another artboard",
        );
    }
    match playback {
        Playback::StateMachine(machine) => {
            for input in &step.inputs {
                let (index, kind) = match input {
                    Input::Bool { index, .. } => (*index, StateMachineInputKind::Bool),
                    Input::Number { index, value } => {
                        if !value.is_finite() {
                            return failure(ErrorKind::InvalidArgument, "nonfinite input");
                        }
                        (*index, StateMachineInputKind::Number)
                    }
                    Input::Trigger { index } => (*index, StateMachineInputKind::Trigger),
                };
                if !machine.input(index).is_some_and(|i| i.kind() == kind) {
                    return failure(
                        ErrorKind::InvalidArgument,
                        "input index or type does not match",
                    );
                }
            }
        }
        _ if !step.inputs.is_empty() || !step.focus_inputs.is_empty() => {
            return failure(ErrorKind::NotFound, "inputs require a state-machine player");
        }
        _ => {}
    }
    for p in &step.pointers {
        if !p.x.is_finite()
            || !p.y.is_finite()
            || !p.timestamp_seconds.is_finite()
            || p.timestamp_seconds < 0.0
        {
            return failure(ErrorKind::InvalidArgument, "invalid pointer input");
        }
    }
    let mut text_bytes = 0usize;
    for input in &step.focus_inputs {
        if let FocusInput::Text(text) = input {
            text_bytes = text_bytes.saturating_add(text.len());
            if text.len() > 1024 * 1024 || text_bytes > 4 * 1024 * 1024 {
                return failure(ErrorKind::LimitExceeded, "focus text exceeds step bound");
            }
        }
    }
    Ok(())
}
impl PagePlayer {
    /// Apply one ordinary frame and commit its owned result.
    pub fn step(
        &self,
        artboard: &mut ArtboardInstance,
        playback: &mut Playback,
        step: Step,
        subscriptions: &[RuntimeOwnedViewModelHandle],
        effects: Option<&dyn Effects>,
    ) -> Result<Output, Error> {
        self.run(artboard, playback, step, None, subscriptions, effects)
    }
    pub fn step_begin(
        &self,
        artboard: &mut ArtboardInstance,
        incoming: &str,
        name: Option<&str>,
        subscriptions: &[RuntimeOwnedViewModelHandle],
        effects: Option<&dyn Effects>,
    ) -> Result<(), Error> {
        self.run(
            artboard,
            &mut Playback::StaticArtboard,
            Step::default(),
            Some(Operation::Begin(incoming.into(), name.map(str::to_owned))),
            subscriptions,
            effects,
        )
        .map(|_| ())
    }
    pub fn step_end(
        &self,
        artboard: &mut ArtboardInstance,
        incoming: &str,
        name: Option<&str>,
        subscriptions: &[RuntimeOwnedViewModelHandle],
        effects: Option<&dyn Effects>,
    ) -> Result<(), Error> {
        self.run(
            artboard,
            &mut Playback::StaticArtboard,
            Step::default(),
            Some(Operation::End(incoming.into(), name.map(str::to_owned))),
            subscriptions,
            effects,
        )
        .map(|_| ())
    }
    pub fn set_row_collapsed(
        &self,
        artboard: &mut ArtboardInstance,
        list: &str,
        index: i32,
        collapsed: bool,
        subscriptions: &[RuntimeOwnedViewModelHandle],
        effects: Option<&dyn Effects>,
    ) -> Result<(), Error> {
        self.run(
            artboard,
            &mut Playback::StaticArtboard,
            Step::default(),
            Some(Operation::Row(list.into(), index, collapsed)),
            subscriptions,
            effects,
        )
        .map(|_| ())
    }
    fn run(
        &self,
        artboard: &mut ArtboardInstance,
        playback: &mut Playback,
        step: Step,
        operation: Option<Operation>,
        subscriptions: &[RuntimeOwnedViewModelHandle],
        effects: Option<&dyn Effects>,
    ) -> Result<Output, Error> {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut pending = self.operation(
                artboard,
                playback,
                step,
                operation,
                subscriptions,
                None,
                None,
                false,
                effects,
                |_| (),
            )?;
            let output = pending.output.take().expect("pending output");
            pending.commit();
            Ok(output)
        }))
        .unwrap_or_else(|_| failure(ErrorKind::RuntimeError, "page operation panicked"))
    }
}

impl PagePlayer {
    /// Intrinsic height in points of the top expanded Screen at the page width.
    pub fn content_height(&self, artboard: &ArtboardInstance) -> Result<f32, Error> {
        use source::{
            artboard_component_list::ArtboardComponentList, layout_component::LayoutComponent,
        };
        let root = artboard.native_handle();
        if is_poisoned(&root) {
            return failure(ErrorKind::RuntimeError, "page occurrence is poisoned");
        }
        if root.with_artboard(|b| b.page_boundary.as_ref().is_some_and(|s| s.active.get())) {
            return failure(ErrorKind::ReentrantCall, "page occurrence is active");
        }
        if !hosted_scene::enabled(&root) {
            return failure(ErrorKind::InvalidArgument, "root is not a page");
        }
        let list = root
            .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
            .ok_or_else(|| Error {
                kind: ErrorKind::InvalidArgument,
                message: "page has no screens".into(),
            })?;
        let row = list
            .with_downcast_mut::<ArtboardComponentList, _>(|l| {
                let index = l.ordered_list_indices().last().copied()?;
                l.artboard_instance(index)
            })
            .flatten()
            .ok_or_else(|| Error {
                kind: ErrorKind::NotFound,
                message: "page has no expanded Screen".into(),
            })?;
        let width = artboard.artboard_dimensions().0;
        let height = LayoutComponent::intrinsic_height_occurrence(&row.core_handle(), width)
            .ok_or_else(|| Error {
                kind: ErrorKind::RuntimeError,
                message: "Screen intrinsic layout is not settled".into(),
            })?;
        Ok(height)
    }
}

// Intrinsic probes may reuse authored script measurements, never invoke script.
// Native scripted measure has no constraint arguments: the last raw natural
// size is valid at a different width/height offer until the next layout solve.
thread_local! { static INTRINSIC_PROBE: Cell<Option<bool>> = const { Cell::new(None) }; }
pub(crate) struct IntrinsicProbe(Option<bool>);
impl IntrinsicProbe {
    pub(crate) fn enter() -> Self {
        Self(INTRINSIC_PROBE.with(|s| s.replace(Some(false))))
    }
    pub(crate) fn failed(&self) -> bool {
        INTRINSIC_PROBE.with(|s| s.get() == Some(true))
    }
    pub(crate) fn active() -> bool {
        INTRINSIC_PROBE.with(|s| s.get().is_some())
    }
    pub(crate) fn unavailable() {
        INTRINSIC_PROBE.with(|s| s.set(Some(true)));
    }
}
impl Drop for IntrinsicProbe {
    fn drop(&mut self) {
        INTRINSIC_PROBE.with(|s| s.set(self.0));
    }
}

const MAX_PLAYER_STEP_EVENTS: usize = 4096;
const MAX_PLAYER_EVENT_PROPERTIES: usize = 256;
const MAX_PLAYER_STEP_EVENT_PROPERTIES_TOTAL: usize = 4096;
#[derive(Debug)]
pub enum OwnedPlayerEventPropertyValue {
    Number(f32),
    Bool(bool),
    String(Box<[u8]>),
    Color(u32),
    Enum(u64),
    Trigger(u64),
}

#[derive(Debug)]
pub struct OwnedPlayerEventProperty {
    pub name: Option<Box<[u8]>>,
    pub value: OwnedPlayerEventPropertyValue,
}

#[derive(Debug)]
pub struct OwnedPlayerEvent {
    pub event_local_index: usize,
    pub event_core_type: u32,
    pub name: Option<Box<[u8]>>,
    pub url: Option<Box<[u8]>>,
    pub target: Option<Box<[u8]>>,
    pub seconds_delay: f32,
    pub properties: Vec<OwnedPlayerEventProperty>,
    pub view_model_instance_id: Option<u64>,
}

fn push_owned_bytes(total: &mut usize, value: &[u8]) -> Result<Box<[u8]>, ErrorKind> {
    *total = total
        .checked_add(value.len())
        .filter(|total| *total <= MAX_PLAYER_STEP_RESULT_BYTES)
        .ok_or(ErrorKind::LimitExceeded)?;
    Ok(value.to_vec().into_boxed_slice())
}

fn push_optional_owned_bytes(
    total: &mut usize,
    value: Option<&str>,
) -> Result<Option<Box<[u8]>>, ErrorKind> {
    value
        .map(|value| push_owned_bytes(total, value.as_bytes()))
        .transpose()
}

pub fn own_reported_events(
    reported: Vec<StateMachineReportedEvent>,
    pointer_result_count: usize,
    focus_input_result_count: usize,
    state_change_count: usize,
) -> Result<Vec<OwnedPlayerEvent>, (ErrorKind, &'static str)> {
    if reported.len() > MAX_PLAYER_STEP_EVENTS {
        return Err((
            ErrorKind::LimitExceeded,
            "runtime event count exceeds step bound",
        ));
    }
    let total_properties = reported.iter().try_fold(0usize, |total, event| {
        if event.properties().len() > MAX_PLAYER_EVENT_PROPERTIES {
            return Err((
                ErrorKind::LimitExceeded,
                "runtime event property count exceeds per-event step bound",
            ));
        }
        total
            .checked_add(event.properties().len())
            .filter(|total| *total <= MAX_PLAYER_STEP_EVENT_PROPERTIES_TOTAL)
            .ok_or((
                ErrorKind::LimitExceeded,
                "runtime event properties exceed aggregate step bound",
            ))
    })?;
    let result_prefix_bytes = pointer_result_count
        .checked_mul(std::mem::size_of::<u32>())
        .and_then(|bytes| {
            focus_input_result_count
                .checked_mul(std::mem::size_of::<u32>())
                .and_then(|focus_bytes| bytes.checked_add(focus_bytes))
        })
        .and_then(|bytes| {
            state_change_count
                .checked_mul(std::mem::size_of::<StateChange>())
                .and_then(|state_bytes| bytes.checked_add(state_bytes))
        });
    let mut owned_bytes_total = reported
        .len()
        .checked_mul(std::mem::size_of::<OwnedPlayerEvent>())
        .and_then(|bytes| {
            total_properties
                .checked_mul(std::mem::size_of::<OwnedPlayerEventProperty>())
                .and_then(|property_bytes| bytes.checked_add(property_bytes))
        })
        .and_then(|bytes| result_prefix_bytes.and_then(|prefix| bytes.checked_add(prefix)))
        .filter(|bytes| *bytes <= MAX_PLAYER_STEP_RESULT_BYTES)
        .ok_or((
            ErrorKind::LimitExceeded,
            "runtime event projection exceeds structural byte bound",
        ))?;
    let mut events = Vec::with_capacity(reported.len());
    for event in reported {
        if !event.seconds_delay().is_finite() || event.seconds_delay() < 0.0 {
            return Err((ErrorKind::RuntimeError, "runtime event delay is invalid"));
        }
        let name = push_optional_owned_bytes(&mut owned_bytes_total, event.name())
            .map_err(|status| (status, "runtime event bytes exceed step bound"))?;
        let url = push_optional_owned_bytes(&mut owned_bytes_total, event.url())
            .map_err(|status| (status, "runtime event bytes exceed step bound"))?;
        let target = push_optional_owned_bytes(&mut owned_bytes_total, event.target())
            .map_err(|status| (status, "runtime event bytes exceed step bound"))?;
        let mut properties = Vec::with_capacity(event.properties().len());
        for property in event.properties() {
            let name = push_optional_owned_bytes(&mut owned_bytes_total, property.name.as_deref())
                .map_err(|status| (status, "runtime event bytes exceed step bound"))?;
            let value = match &property.value {
                RuntimeEventPropertyValue::Number(value) => {
                    OwnedPlayerEventPropertyValue::Number(*value)
                }
                RuntimeEventPropertyValue::Bool(value) => {
                    OwnedPlayerEventPropertyValue::Bool(*value)
                }
                RuntimeEventPropertyValue::String(value) => OwnedPlayerEventPropertyValue::String(
                    push_owned_bytes(&mut owned_bytes_total, value)
                        .map_err(|status| (status, "runtime event bytes exceed step bound"))?,
                ),
                RuntimeEventPropertyValue::Color(value) => {
                    OwnedPlayerEventPropertyValue::Color(*value)
                }
                RuntimeEventPropertyValue::Enum(value) => {
                    OwnedPlayerEventPropertyValue::Enum(*value)
                }
                RuntimeEventPropertyValue::Trigger(value) => {
                    OwnedPlayerEventPropertyValue::Trigger(*value)
                }
            };
            properties.push(OwnedPlayerEventProperty { name, value });
        }
        events.push(OwnedPlayerEvent {
            view_model_instance_id: event
                .context()
                .and_then(|context| context.view_model_instance_id()),
            event_local_index: event.event_local_index().unwrap_or(usize::MAX),
            event_core_type: event.event_core_type(),
            name,
            url,
            target,
            seconds_delay: event.seconds_delay(),
            properties,
        });
    }
    Ok(events)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Error {}
