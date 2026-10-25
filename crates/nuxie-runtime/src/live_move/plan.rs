use super::{
    empty_slot,
    follow::{Flight, SizeMode},
    move_live,
    records::{LiveSlot, LiveStep, LiveStepUnit, Values, validate_step_units},
    transport::{child, read},
    validate_move,
};
use crate::{
    Artboard, CoreHandle, RuntimeArtboardInstanceHandle,
    source::{artboard_component_list::ArtboardComponentList, nested_artboard::NestedArtboard},
};
use anyhow::{Result, ensure};
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
struct Slot {
    node: CoreHandle,
    values: Values,
}
#[derive(Clone)]
struct Step {
    name: String,
    from: String,
    to: String,
    units: Vec<Values>,
}
#[derive(Clone)]
struct Screen {
    board: CoreHandle,
    slots: Vec<Slot>,
}
/// File plans and retained history are scoped to mounted row identities.
#[derive(Default)]
pub(crate) struct Plan {
    root: Option<CoreHandle>,
    revision: Option<u64>,
    steps: Vec<Step>,
    rows: HashMap<String, Screen>,
    // Empty home -> current holder. No instance, VM arena or retired row is
    // kept alive by history. Entries disappear as soon as either row leaves.
    homes: HashMap<CoreHandle, CoreHandle>,
    pub flights: Vec<Flight>,
    active: Option<(String, Option<String>)>,
    pub index: crate::hosted_scene::HostedIndex,
    projection: Option<(u64, Vec<CoreHandle>, std::rc::Rc<super::frame::Projection>)>,
}
impl Plan {
    pub fn prepare(&mut self, root: &RuntimeArtboardInstanceHandle) -> Result<()> {
        if let Some(scene) = crate::hosted_scene::root_scene(root) {
            scene.reconcile();
        }
        let revision = crate::hosted_scene::membership_revision(&root.core_handle());
        if self.root.as_ref() == Some(&root.core_handle()) && self.revision == Some(revision) {
            return Ok(());
        }
        if self.root.as_ref() != Some(&root.core_handle()) {
            ensure!(
                self.active.is_none(),
                "cannot change scene occurrence during a step"
            );
            self.steps = read_steps(root)?;
            self.root = Some(root.core_handle());
            self.homes.clear();
        }
        // Ordinary upstream files never acquire a page-host requirement.
        let has_slots = root
            .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
            .is_some();
        if !has_slots {
            ensure!(self.steps.is_empty(), "live steps require a screens list");
            self.revision = Some(revision);
            return Ok(());
        }
        self.rows = screen_rows(root)?;
        let mounted = self
            .rows
            .values()
            .flat_map(|r| r.slots.iter().map(|s| s.node.clone()))
            .collect::<HashSet<_>>();
        let incoming_left = self
            .active
            .as_ref()
            .is_some_and(|(incoming, _)| !self.rows.contains_key(incoming));
        // A two-version flight can land on a different Screen from its holder.
        // Cancel the whole step if either row has gone. The unused-flight pass
        // below settles surviving homes and releases orphaned instances.
        let lost_flight = self
            .flights
            .iter()
            .any(|flight| !mounted.contains(&flight.flight) || !mounted.contains(&flight.landing));
        if incoming_left || lost_flight {
            self.flights.clear();
            self.active = None;
        }
        let mut first = HashMap::new();
        let mut flight_keys = HashSet::new();
        for slot in self.rows.values().flat_map(|r| &r.slots) {
            if slot.values.role == 1 {
                ensure!(
                    flight_keys.insert(slot.values.flight_key),
                    "duplicate flightKey in file"
                );
            }
            if let Some(previous) = first.get(&slot.values.unit_key) {
                validate_move(previous, &slot.node)?;
            } else {
                first.insert(slot.values.unit_key, slot.node.clone());
            }
        }
        // Plans can outlive a removed row. Validate every plan whose two
        // Screens are still mounted without retaining the removed arenas.
        for step in &self.steps {
            if let (Some(from), Some(to)) = (self.rows.get(&step.from), self.rows.get(&step.to)) {
                for unit in &step.units {
                    let outgoing = find_slot(from, unit.outgoing_unit_key, 0)?;
                    let incoming = find_slot(to, unit.incoming_unit_key, 0)?;
                    let outgoing_flight = find_flight(from, to, unit.outgoing_flight_key)?;
                    let incoming_flight = find_flight(from, to, unit.incoming_flight_key)?;
                    ensure!(
                        outgoing_flight.values.unit_key == unit.outgoing_unit_key
                            && incoming_flight.values.unit_key == unit.incoming_unit_key,
                        "flightKey references a different unit"
                    );
                    validate_move(&outgoing.node, &outgoing_flight.node)?;
                    validate_move(&incoming.node, &incoming_flight.node)?;
                }
            }
        }
        let mut holders = HashMap::new();
        for row in self.rows.values() {
            for slot in &row.slots {
                if let Some(unit) = child(&slot.node)? {
                    holders.insert(unit.core_handle(), slot.node.clone());
                }
            }
        }
        self.homes.clear();
        for row in self.rows.values() {
            for slot in row.slots.iter().filter(|s| s.values.role == 0) {
                if child(&slot.node)?.is_some() {
                    continue;
                }
                if let Some(id) = read(&slot.node, |s| {
                    s.live_move
                        .as_ref()
                        .and_then(|state| state.away_instance.clone())
                })? {
                    if let Some(holder) = holders.get(&id) {
                        self.homes.insert(slot.node.clone(), holder.clone());
                    } else {
                        super::transport::write(&slot.node, |s| {
                            s.live_move_mut().away_instance = None
                        });
                    }
                }
            }
        }

        let unused = self
            .rows
            .values()
            .flat_map(|r| r.slots.iter())
            .filter(|s| s.values.role != 0)
            .filter(|s| !self.flights.iter().any(|f| f.flight == s.node))
            .map(|s| s.node.clone())
            .collect::<Vec<_>>();
        for slot in unused {
            if child(&slot)?.is_some() {
                // A removed step returns its surviving loan before releasing
                // unused flight content. Fresh flight mounts have no home.
                let home = self
                    .homes
                    .iter()
                    .find(|(_, holder)| **holder == slot)
                    .map(|(home, _)| home.clone());
                if let Some(home) = home {
                    self.transport(&slot, &home)?;
                    settle_slot(&home)?;
                } else {
                    empty_slot(&slot)?;
                }
            }
        }
        self.index.active(root);
        self.revision = Some(crate::hosted_scene::membership_revision(
            &root.core_handle(),
        ));
        Ok(())
    }
    fn transport(&mut self, from: &CoreHandle, to: &CoreHandle) -> Result<()> {
        move_live(from, to)?;
        self.record_transport(from, to);
        Ok(())
    }
    fn record_transport(&mut self, from: &CoreHandle, to: &CoreHandle) {
        // Every previously empty home follows the occurrence, even across
        // a cut followed by another push. A filled home needs no history.
        for holder in self.homes.values_mut() {
            if holder == from {
                *holder = to.clone();
            }
        }
        self.homes.remove(to);
        let is_home = self.rows.values().any(|r| {
            r.slots
                .iter()
                .any(|s| s.node == *from && s.values.role == 0)
        });
        if is_home {
            self.homes.insert(from.clone(), to.clone());
        }
    }
    pub fn step_begin(
        &mut self,
        root: &RuntimeArtboardInstanceHandle,
        incoming: &str,
        step_name: Option<&str>,
    ) -> Result<()> {
        ensure!(self.active.is_none(), "step already active");
        self.prepare(root)?;
        let incoming_row = self
            .rows
            .get(incoming)
            .ok_or_else(|| anyhow::anyhow!("incoming Screen not found"))?
            .clone();
        ensure!(
            crate::hosted_scene::is_active_artboard(&incoming_row.board),
            "incoming Screen is collapsed"
        );
        let step = step_name
            .map(|name| {
                self.steps
                    .iter()
                    .find(|s| s.name == name)
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("published step not found"))
            })
            .transpose()?;
        // Only a single-version arrival can bring a loan home by landing.
        // A two-version incoming instance must first return to its own slot,
        // then take off there. Plan those returns without mutating any owner.
        let mut returns = Vec::new();
        for (home, holder) in &self.homes {
            if !incoming_row.slots.iter().any(|s| s.node == *home) {
                continue;
            }
            let flies_home = step.as_ref().is_some_and(|step| {
                step.units.iter().any(|unit| {
                    unit.incoming_unit_key == unit.outgoing_unit_key
                        && incoming_row.slots.iter().any(|s| {
                            s.node == *home
                                && s.values.role == 0
                                && s.values.unit_key == unit.incoming_unit_key
                        })
                })
            });
            if !flies_home {
                validate_move(holder, home)?;
                returns.push((holder.clone(), home.clone()));
            }
        }
        let relocated = |slot: &CoreHandle| {
            returns
                .iter()
                .find(|(from, _)| from == slot)
                .map_or_else(|| slot.clone(), |(_, home)| home.clone())
        };
        let departure = |slot: &CoreHandle| {
            returns
                .iter()
                .find(|(_, home)| home == slot)
                .map_or_else(|| slot.clone(), |(from, _)| from.clone())
        };
        let mut plan = Vec::new();
        let mut fresh_destinations = Vec::new();
        let mut used = HashSet::new();
        if let Some(step) = &step {
            validate_step_units(&step.units)?;
            ensure!(step.to == incoming, "step destination mismatch");
            let outgoing = self
                .rows
                .get(&step.from)
                .ok_or_else(|| anyhow::anyhow!("outgoing Screen not mounted"))?;
            for unit in &step.units {
                let from = find_slot(outgoing, unit.outgoing_unit_key, 0)?;
                let to = find_slot(&incoming_row, unit.incoming_unit_key, 0)?;
                let two_versions = unit.incoming_unit_key != unit.outgoing_unit_key;
                let holder = relocated(self.homes.get(&from.node).unwrap_or(&from.node));
                let outgoing_flight =
                    find_flight(outgoing, &incoming_row, unit.outgoing_flight_key)?;
                let incoming_flight =
                    find_flight(outgoing, &incoming_row, unit.incoming_flight_key)?;
                let mut make_flight =
                    |slot: &Slot, owner: CoreHandle, landing: &CoreHandle| -> Result<()> {
                        ensure!(
                            crate::hosted_scene::is_active_artboard(
                                &read(&slot.node, NestedArtboard::parent_artboard_handle)?
                                    .expect("flight row")
                            ),
                            "flight Screen is collapsed"
                        );
                        validate_move(&owner, &slot.node)?;
                        validate_move(&slot.node, landing)?;
                        let mode = if slot.values.size_mode == 1 {
                            SizeMode::Layout
                        } else {
                            SizeMode::Scale
                        };
                        let mut flight = Flight::along_path(
                            slot.node.clone(),
                            from.node.clone(),
                            to.node.clone(),
                            mode,
                        )?;
                        flight.landing = landing.clone();
                        flight.use_departure(&departure(&owner))?;
                        plan.push((owner, flight));
                        Ok(())
                    };
                make_flight(
                    outgoing_flight,
                    holder.clone(),
                    if two_versions { &from.node } else { &to.node },
                )?;
                if two_versions {
                    // The incoming version takes off from its own Screen.
                    // Both versions follow the outgoing-to-incoming poses;
                    // file object order alone determines which draws above.
                    ensure!(
                        child(&departure(&to.node))?.is_some(),
                        "incoming version has no current holder"
                    );
                    make_flight(incoming_flight, to.node.clone(), &to.node)?;
                } else if holder != to.node && child(&to.node)?.is_some() {
                    fresh_destinations.push(to.node.clone());
                }
            }
        }

        for (holder, f) in &plan {
            ensure!(
                used.insert(holder.clone()) && used.insert(f.flight.clone()),
                "overlapping flight owners"
            );
            f.follow(false, None)?;
        }
        // Every return and flight has been checked before changing ownership.
        let (mut returning, mut returning_home) = (HashSet::new(), HashSet::new());
        for (holder, home) in &returns {
            ensure!(!used.contains(holder), "coming home overlaps flight");
            ensure!(
                returning.insert(holder) && returning_home.insert(home),
                "overlapping coming-home moves"
            );
        }
        let mut discarded = HashSet::new();
        for slot in &fresh_destinations {
            ensure!(
                discarded.insert(slot)
                    && !used.contains(slot)
                    && !returning.contains(slot)
                    && !returning_home.contains(slot),
                "fresh destination overlaps a planned move"
            );
        }
        for (holder, home) in returns {
            self.transport(&holder, &home)?;
            settle_slot(&home)?;
        }
        // A one-version match has one drawn occurrence throughout flight.
        // Preserve the landing box and native inline props before discarding its
        // never-shown fresh playback. Two versions both take off instead.
        for slot in fresh_destinations {
            empty_slot(&slot)?;
        }
        for (holder, mut f) in plan {
            // A returned incoming version has now settled in its own slot.
            // Scale mode preserves that departure size, not its old holder's.
            f.use_departure(&holder)?;
            super::transport::move_to_flight(&holder, &f.flight, &f.landing, &mut f.values)?;
            self.record_transport(&holder, &f.flight);
            self.flights.push(f);
        }
        self.active = Some((incoming.to_owned(), step_name.map(str::to_owned)));
        self.after_advance(root)?;
        Ok(())
    }
    pub fn step_end(
        &mut self,
        root: &RuntimeArtboardInstanceHandle,
        incoming: &str,
        name: Option<&str>,
    ) -> Result<()> {
        self.prepare(root)?;
        ensure!(
            self.active
                .as_ref()
                .is_some_and(|(s, n)| s == incoming && n.as_deref() == name),
            "step_end mismatch"
        );
        for f in &self.flights {
            validate_move(&f.flight, &f.landing)?;
            ensure!(child(&f.flight)?.is_some(), "flight lost its instance");
        }
        let flights = std::mem::take(&mut self.flights);
        for f in flights {
            self.transport(&f.flight, &f.landing)?;
            settle_slot(&f.landing)?;
        }
        self.active = None;
        self.prepare(root)?;
        Ok(())
    }
    pub fn after_advance(&mut self, root: &RuntimeArtboardInstanceHandle) -> Result<bool> {
        self.prepare(root)?;
        if self.flights.is_empty() {
            return Ok(false);
        }
        let active = self
            .index
            .active(root)
            .iter()
            .cloned()
            .collect::<HashSet<_>>();
        let mut rows = HashSet::new();
        for f in &self.flights {
            if active.contains(&f.board) {
                f.follow(true, Some(&active))?;
                rows.insert(f.board.clone());
            }
        }
        // All poses use the same completed player advance. Only then settle
        // each affected row once, including its dirty transported children.
        let changed = !rows.is_empty();
        for row in rows {
            Artboard::update_pass_handle(&row, false);
        }
        Ok(changed)
    }
    pub fn frame_scope(
        &mut self,
        root: &RuntimeArtboardInstanceHandle,
        roots: &[crate::RuntimeOwnedViewModelHandle],
    ) -> Result<Option<super::FrameScope>> {
        self.prepare(root)?;
        self.index.active(root);
        if self.steps.is_empty()
            && self.rows.values().all(|r| r.slots.is_empty())
            && !self.index.has_collapsed_rows(root)
        {
            return Ok(None);
        }
        let revision = crate::hosted_scene::membership_revision(&root.core_handle());
        let keys = roots.iter().map(|r| r.native_handle()).collect::<Vec<_>>();
        if self
            .projection
            .as_ref()
            .is_none_or(|(r, k, projection)| *r != revision || *k != keys || !projection.is_valid())
        {
            self.projection = Some((
                revision,
                keys,
                super::frame::Projection::build(
                    root.core_handle(),
                    roots,
                    self.index.inactive_models(root),
                ),
            ));
        }
        Ok(self.projection.as_ref().map(|(_, _, p)| p.enter()))
    }
    pub(crate) fn has_projection(&self) -> bool {
        self.projection.is_some()
    }
    #[cfg(feature = "test-support")]
    pub fn retained_history_len(&self) -> usize {
        self.homes.len()
    }
}
fn find_slot(row: &Screen, key: u32, role: u32) -> Result<&Slot> {
    row.slots
        .iter()
        .find(|s| s.values.unit_key == key && s.values.role == role)
        .ok_or_else(|| anyhow::anyhow!("missing unit {key} role {role}"))
}
fn find_flight<'a>(from: &'a Screen, to: &'a Screen, key: u32) -> Result<&'a Slot> {
    from.slots
        .iter()
        .chain(&to.slots)
        .find(|s| s.values.role == 1 && s.values.flight_key == key)
        .ok_or_else(|| anyhow::anyhow!("unknown flightKey {key} on step Screens"))
}
fn read_steps(root: &RuntimeArtboardInstanceHandle) -> Result<Vec<Step>> {
    let objects = root.with_artboard(|b| b.objects().iter().flatten().cloned().collect::<Vec<_>>());
    let mut result = Vec::new();
    let mut names = HashSet::new();
    for object in &objects {
        if let Some((parent, v)) =
            object.with_downcast::<LiveStep, _>(|r| (r.parent(), r.values.clone()))
        {
            ensure!(
                parent.as_ref() == Some(&root.core_handle()),
                "LiveStep must be on the page root"
            );
            ensure!(names.insert(v.name.clone()), "duplicate step name");
            let mut units = Vec::new();
            for child in &objects {
                if let Some(v) = child
                    .with_downcast::<LiveStepUnit, _>(|r| {
                        (r.parent().as_ref() == Some(object)).then(|| r.values.clone())
                    })
                    .flatten()
                {
                    units.push(v);
                }
            }
            validate_step_units(&units)?;
            result.push(Step {
                name: v.name,
                from: v.from_artboard,
                to: v.to_artboard,
                units,
            });
        }
    }
    Ok(result)
}
fn screen_rows(root: &RuntimeArtboardInstanceHandle) -> Result<HashMap<String, Screen>> {
    let rows = screens(root)?;
    let mut result = HashMap::new();
    for (name, board) in rows {
        let mut slots = Vec::new();
        let mut keys = HashSet::new();
        let objects = if crate::hosted_scene::is_screen(&board.core_handle()) {
            board.with_artboard(|b| b.objects().iter().flatten().cloned().collect::<Vec<_>>())
        } else {
            Vec::new()
        };
        for object in objects {
            if let Some((Some(node), values)) =
                object.with_downcast::<LiveSlot, _>(|r| (r.parent(), r.values.clone()))
            {
                ensure!(
                    keys.insert(if values.role == 0 {
                        (0, values.unit_key)
                    } else {
                        (1, values.flight_key)
                    }),
                    "duplicate live slot in Screen"
                );
                ensure!(
                    slots.iter().all(|slot: &Slot| slot.node != node),
                    "multiple LiveSlot records on one host"
                );
                slots.push(Slot { node, values });
            }
        }
        result.insert(
            name,
            Screen {
                board: board.core_handle(),
                slots,
            },
        );
    }
    Ok(result)
}
/// Retained row discovery is structural work, used at a step or membership
/// edit. Per-frame queries use HostedIndex instead.
pub fn screens(
    root: &RuntimeArtboardInstanceHandle,
) -> Result<HashMap<String, RuntimeArtboardInstanceHandle>> {
    let list = root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .ok_or_else(|| anyhow::anyhow!("page has no screens list"))?;
    list.with_downcast::<ArtboardComponentList, _>(|l| {
        let mut rows = HashMap::new();
        for i in 0..l.artboard_count() {
            if let Some(row) = l.artboard_instance(i as i32) {
                let name = row.with_artboard(|b| b.name().to_owned());
                // Steps name Screens by artboard; a page holds each Screen once.
                ensure!(
                    !rows.contains_key(&name),
                    "Screen artboard '{name}' appears twice in the page"
                );
                rows.insert(name, row);
            }
        }
        Ok(rows)
    })
    .unwrap()
}

fn settle_slot(slot: &CoreHandle) -> Result<()> {
    let row = read(slot, NestedArtboard::parent_artboard_handle)?.expect("slot row");
    Artboard::update_pass_handle(&row, false);
    Ok(())
}

/// Stateless scene dispatcher. All plans, flights and caches belong to the
/// native root occurrence, shared across Rust adapters and C API players.
#[derive(Default)]
pub struct LiveMoves;
impl LiveMoves {
    pub fn prepare(&mut self, root: &RuntimeArtboardInstanceHandle) -> Result<()> {
        if let Some(scene) = crate::hosted_scene::root_scene(root) {
            scene.plan.borrow_mut().prepare(root)?;
        }
        Ok(())
    }
    pub fn step_begin(
        &mut self,
        root: &RuntimeArtboardInstanceHandle,
        incoming: &str,
        name: Option<&str>,
    ) -> Result<()> {
        let scene = crate::hosted_scene::root_scene(root)
            .ok_or_else(|| anyhow::anyhow!("root is not a live-move page"))?;
        root.update_data_binds(false);
        root.update_pass(false);
        scene.plan.borrow_mut().step_begin(root, incoming, name)
    }
    pub fn step_end(
        &mut self,
        root: &RuntimeArtboardInstanceHandle,
        incoming: &str,
        name: Option<&str>,
    ) -> Result<()> {
        let scene = crate::hosted_scene::root_scene(root)
            .ok_or_else(|| anyhow::anyhow!("root is not a live-move page"))?;
        root.update_data_binds(false);
        root.update_pass(false);
        scene.plan.borrow_mut().step_end(root, incoming, name)
    }
    /// Named steps evaluate only their expanded endpoints. A cut has no
    /// published from/to record: its expanded Screens are the host's endpoints.
    pub(crate) fn evaluate_step_rows(
        &mut self,
        root: &RuntimeArtboardInstanceHandle,
        incoming: &str,
        name: Option<&str>,
    ) -> Result<Vec<crate::StateMachineReportedEvent>> {
        let scene = crate::hosted_scene::root_scene(root)
            .ok_or_else(|| anyhow::anyhow!("root is not a page"))?;
        let outgoing = name.and_then(|name| {
            scene
                .plan
                .borrow()
                .steps
                .iter()
                .find(|step| step.name == name)
                .map(|step| step.from.clone())
        });
        let list = root
            .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
            .ok_or_else(|| anyhow::anyhow!("page has no screens list"))?;
        let rows = list
            .with_downcast::<ArtboardComponentList, _>(|l| {
                l.live_move_frame_indices()
                    .iter()
                    .copied()
                    .filter(|&index| {
                        l.artboard_instance(index).is_some_and(|row| {
                            // Unrelated named rows need no visibility work.
                            // This loop already knows the row's list index;
                            // resolving it again through the artboard is linear.
                            (name.is_none()
                                || row.with_artboard(|b| {
                                    b.name() == incoming || outgoing.as_deref() == Some(b.name())
                                }))
                                && row
                                    .core_handle()
                                    .with(|object| !object.component_is_collapsed())
                                    .unwrap_or(false)
                                && !super::collapse::effectively_hidden(l, Some(index))
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Ok(super::collapse::evaluate_rows(root, &list, &rows))
    }
    pub fn after_advance(&mut self, root: &RuntimeArtboardInstanceHandle) -> Result<bool> {
        let Some(scene) = crate::hosted_scene::root_scene(root) else {
            return Ok(false);
        };
        scene.plan.borrow_mut().after_advance(root)
    }
    pub fn frame_scope(
        &mut self,
        root: &RuntimeArtboardInstanceHandle,
        roots: &[crate::RuntimeOwnedViewModelHandle],
    ) -> Result<Option<super::FrameScope>> {
        let Some(scene) = crate::hosted_scene::root_scene(root) else {
            return Ok(None);
        };
        scene.plan.borrow_mut().frame_scope(root, roots)
    }
    #[cfg(feature = "test-support")]
    pub fn flights(&self, root: &RuntimeArtboardInstanceHandle) -> Vec<Flight> {
        crate::hosted_scene::root_scene(root)
            .map_or_else(Vec::new, |s| s.plan.borrow().flights.clone())
    }
    #[cfg(feature = "test-support")]
    pub fn with_flights(
        &mut self,
        root: &RuntimeArtboardInstanceHandle,
        body: impl FnOnce(&mut Vec<Flight>),
    ) {
        let scene = crate::hosted_scene::root_scene(root).expect("live scene");
        body(&mut scene.plan.borrow_mut().flights);
    }
    #[cfg(feature = "test-support")]
    pub fn reset_for_test(&mut self, root: &RuntimeArtboardInstanceHandle) {
        let scene = crate::hosted_scene::root_scene(root).expect("live scene");
        *scene.plan.borrow_mut() = Plan::default();
    }
    #[cfg(feature = "test-support")]
    pub fn retained_history_len(&self, root: &RuntimeArtboardInstanceHandle) -> usize {
        crate::hosted_scene::root_scene(root).map_or(0, |s| s.plan.borrow().retained_history_len())
    }
}
