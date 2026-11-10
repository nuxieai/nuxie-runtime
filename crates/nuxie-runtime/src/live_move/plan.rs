use super::{
    empty_slot,
    follow::{Flight, SizeMode},
    move_live,
    records::{LiveSlot, LiveStep, LiveStepUnit, Values, validate_step_units},
    transport::{child, origin_home, read, refill_slot},
    validate_move,
};
use crate::{
    Artboard, CoreHandle, RuntimeArtboardInstanceHandle,
    source::{
        animation::{keyed_object::KeyedObject, linear_animation::LinearAnimation},
        artboard_component_list::ArtboardComponentList,
        nested_artboard::NestedArtboard,
    },
};
use anyhow::{Result, ensure};
use std::collections::{HashMap, HashSet};

pub mod zoom;

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
    /// The step's flight timeline: its index among the animations of the
    /// Screen holding the step's flight slots, as an `AnimationState`'s
    /// `animationId` names one. `u32::MAX` when the step names none.
    flight_animation: u32,
}
#[derive(Clone)]
struct Screen {
    board: CoreHandle,
    slots: Vec<Slot>,
}
/// One thing a step flies: a flight, or a zoomed pair's two.
pub enum Entry {
    One(Flight),
    Pair(zoom::Pair),
}
impl Entry {
    fn flights(&self) -> &[Flight] {
        match self {
            Self::One(flight) => std::slice::from_ref(flight),
            Self::Pair(pair) => pair.flights(),
        }
    }
}
/// One item of a step's working list before anything moves: a flight with
/// the holder it takes off from, or a zoomed pair's two with their rule.
enum Item {
    One((CoreHandle, Flight)),
    Pair(zoom::Rule, [(CoreHandle, Flight); 2]),
}
impl Item {
    fn moves(&self) -> &[(CoreHandle, Flight)] {
        match self {
            Self::One(one) => std::slice::from_ref(one),
            Self::Pair(_, two) => two,
        }
    }
}
/// File plans are scoped to mounted row identities. Between steps every slot
/// holds its own Screen's instance, so nothing is kept about where one went.
#[derive(Default)]
pub(crate) struct Plan {
    root: Option<CoreHandle>,
    revision: Option<u64>,
    steps: Vec<Step>,
    rows: HashMap<String, Screen>,
    pub flights: Vec<Entry>,
    /// The active step's slots it emptied: each slot an instance took off
    /// from, and each transferred unit's arriving slot, whose never-shown
    /// instance take-off discarded. A cancel refills those still mounted and
    /// empty (`settle`).
    emptied: Vec<CoreHandle>,
    /// The active step's slots its transferred units took off from, each
    /// refilled when the step lands (decision 297).
    departures: Vec<CoreHandle>,
    active: Option<(String, Option<String>)>,
    pub index: crate::hosted_scene::HostedIndex,
    projection: Option<(u64, Vec<CoreHandle>, std::rc::Rc<super::frame::Projection>)>,
}
impl Plan {
    /// Every flight of every entry.
    fn all_flights(&self) -> impl Iterator<Item = &Flight> {
        self.flights.iter().flat_map(Entry::flights)
    }
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
        // A flight can land on a different Screen from its flight slot.
        // Cancel the whole step if its incoming row, or any flight's flight
        // or landing slot, has gone. Every plan entry prepares first, so the
        // cancel settles before the next frame is laid out or drawn.
        let lost_flight = self
            .all_flights()
            .any(|flight| !mounted.contains(&flight.flight) || !mounted.contains(&flight.landing));
        if incoming_left || lost_flight {
            self.settle(&mounted)?;
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
        let unused = self
            .rows
            .values()
            .flat_map(|r| r.slots.iter())
            .filter(|s| s.values.role != 0)
            .filter(|s| !self.all_flights().any(|f| f.flight == s.node))
            .map(|s| s.node.clone())
            .collect::<Vec<_>>();
        for slot in unused {
            // A fresh flight mount, or what a cancel left: nothing claims it.
            if child(&slot)?.is_some() {
                empty_slot(&slot)?;
            }
        }
        self.index.active(root);
        self.revision = Some(crate::hosted_scene::membership_revision(
            &root.core_handle(),
        ));
        Ok(())
    }
    /// Puts back everything a cancelled step moved, as Hero's cancel puts
    /// every view back where it was (`HeroContext.swift:324-331`). Each
    /// instance still in the air goes back to the slot it took off from when
    /// that slot is mounted and empty, and is released otherwise; the list's
    /// hook (`settle_leaving_rows`) already moved those whose flight slot's
    /// row went. Then every mounted slot the step emptied that is still empty
    /// gets a copy of its own (decision 297), and the step ends.
    fn settle(&mut self, mounted: &HashSet<CoreHandle>) -> Result<()> {
        let entries = std::mem::take(&mut self.flights);
        for f in entries.iter().flat_map(Entry::flights) {
            if !mounted.contains(&f.flight) || child(&f.flight)?.is_none() {
                continue;
            }
            match origin_home(&f.flight)?.filter(|origin| mounted.contains(origin)) {
                Some(origin) if child(&origin)?.is_none() => {
                    move_live(&f.flight, &origin)?;
                    settle_slot(&origin)?;
                }
                _ => empty_slot(&f.flight)?,
            }
        }
        for slot in std::mem::take(&mut self.emptied) {
            if mounted.contains(&slot) && child(&slot)?.is_none() {
                refill_slot(&slot)?;
                settle_slot(&slot)?;
            }
        }
        self.departures.clear();
        self.active = None;
        Ok(())
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
        // Every slot holds its own Screen's instance between steps, so each
        // flight takes off from its own Screen's slot.
        let mut plan = Vec::new();
        let mut fresh_destinations = Vec::new();
        let mut departures = Vec::new();
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
                let two_versions = !unit.transfers();
                let holder = from.node.clone();
                ensure!(
                    child(&holder)?.is_some(),
                    "leaving copy has no instance"
                );
                let outgoing_flight =
                    find_flight(outgoing, &incoming_row, unit.outgoing_flight_key)?;
                let incoming_flight =
                    find_flight(outgoing, &incoming_row, unit.incoming_flight_key)?;
                let make_flight = |slot: &Slot, owner: CoreHandle, landing: &CoreHandle| {
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
                    flight.use_departure(&owner)?;
                    Ok::<_, anyhow::Error>((owner, flight))
                };
                let leaving = make_flight(
                    outgoing_flight,
                    holder.clone(),
                    if two_versions { &from.node } else { &to.node },
                )?;
                if two_versions {
                    // The incoming version takes off from its own Screen.
                    // Both versions follow the outgoing-to-incoming poses;
                    // file object order alone determines which draws above.
                    ensure!(
                        child(&to.node)?.is_some(),
                        "arriving copy has no instance"
                    );
                    let arriving = make_flight(incoming_flight, to.node.clone(), &to.node)?;
                    // A zoom keeps the two together: one box, one entry
                    // (docs/live-moves.md).
                    match zoom::Rule::of(unit.read_zoom()) {
                        Some(rule) => plan.push(Item::Pair(rule, [leaving, arriving])),
                        None => plan.extend([Item::One(leaving), Item::One(arriving)]),
                    }
                } else {
                    plan.push(Item::One(leaving));
                    departures.push(holder);
                    if child(&to.node)?.is_some() {
                        fresh_destinations.push(to.node.clone());
                    }
                }
            }
        }

        for item in &plan {
            for (holder, f) in item.moves() {
                ensure!(
                    used.insert(holder.clone()) && used.insert(f.flight.clone()),
                    "overlapping flight owners"
                );
            }
            // A zoomed pair refuses no end's shape: the zoom always runs, so
            // its versions skip the checks a flight's own pose makes.
            if let Item::One((_, f)) = item {
                f.validate()?;
            }
        }
        // Every flight has been checked before changing ownership.
        let mut discarded = HashSet::new();
        for slot in &fresh_destinations {
            ensure!(
                discarded.insert(slot) && !used.contains(slot),
                "fresh destination overlaps a planned move"
            );
        }
        let timeline = match &step {
            Some(step) => flight_timeline(step.flight_animation, &plan)?,
            None => None,
        };
        // What the step empties: each slot a flight takes off from, and each
        // transferred unit's arriving slot.
        let mut emptied = plan
            .iter()
            .flat_map(|item| item.moves().iter().map(|(holder, _)| holder.clone()))
            .collect::<Vec<_>>();
        emptied.extend(fresh_destinations.iter().cloned());
        // A transferred unit has one drawn occurrence throughout flight.
        // Preserve the landing box and native inline props before discarding its
        // never-shown fresh playback. Two copies both take off instead.
        for slot in fresh_destinations {
            empty_slot(&slot)?;
        }
        // The point a zoom's tracker follows on Back: the center of the
        // page's drawing space.
        let center = root.with_artboard(|b| b.bounds().center());
        for item in plan {
            match item {
                Item::One((holder, mut f)) => {
                    // Read after the fresh destinations left: scale mode keeps
                    // the size the holder has as the flight takes off.
                    f.use_departure(&holder)?;
                    super::transport::move_to_flight(
                        &holder,
                        &f.flight,
                        &f.landing,
                        &mut f.values,
                    )?;
                    self.flights.push(Entry::One(f));
                }
                Item::Pair(rule, [(leaving, mut a), (arriving, mut b)]) => {
                    // Each version's content size at its end is its departure
                    // size. Both frames are still in their holders: the pair
                    // reads its ends there, then both versions take off.
                    a.use_departure(&leaving)?;
                    b.use_departure(&arriving)?;
                    let mut pair =
                        zoom::Pair::take_off(rule, [&leaving, &arriving], [a, b], center)?;
                    for (holder, f) in [&leaving, &arriving].into_iter().zip(pair.flights_mut()) {
                        super::transport::move_to_flight(
                            holder,
                            &f.flight,
                            &f.landing,
                            &mut f.values,
                        )?;
                    }
                    self.flights.push(Entry::Pair(pair));
                }
            }
        }
        // Each flight paints where its element paints on the upper Screen
        // (docs/CSS_LAYOUT.md, 4.4). Its anchor reads the frame at rest in
        // it through the flight carrying that frame: the one that lands in
        // the anchor, else the one that left it.
        for f in self.all_flights() {
            let Some(anchor) = f.seed_placement() else {
                continue;
            };
            let mut sharing = self
                .all_flights()
                .filter(|g| g.anchor().as_ref() == Some(&anchor));
            let carrier = self
                .all_flights()
                .find(|g| g.landing == anchor && g.anchor().as_ref() == Some(&anchor))
                .or_else(|| sharing.next().filter(|_| sharing.next().is_none()));
            if let Some(carrier) = carrier {
                super::transport::write(&anchor, |s| {
                    s.live_move_mut().carried_by = Some(carrier.flight.clone())
                });
            }
        }
        // Each flight starts at its step's first frame, not at what an
        // earlier step left on its slot, before the begin's own follow.
        if let Some((board, animation)) = timeline {
            let controls = self.all_flights().map(Flight::controls).collect();
            apply_first_frame(&board, &animation, &controls);
        }
        self.emptied = emptied;
        self.departures = departures;
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
        self.land(root)
    }
    /// Lands every flight in its landing slot, as the end of a step does:
    /// each copy in its own Screen's slot, and a transferred unit in the
    /// arriving Screen's. Then the slot each transferred unit left gets a copy
    /// of its own (decision 297); a row removed mid-flight took its slot with
    /// it. Both SDKs end a step and retire the leaving row in one batch, so
    /// that copy is first drawn when that Screen is shown again.
    fn land(&mut self, root: &RuntimeArtboardInstanceHandle) -> Result<()> {
        for f in self.all_flights() {
            validate_move(&f.flight, &f.landing)?;
            ensure!(child(&f.flight)?.is_some(), "flight lost its instance");
        }
        let entries = std::mem::take(&mut self.flights);
        for f in entries.iter().flat_map(Entry::flights) {
            move_live(&f.flight, &f.landing)?;
            settle_slot(&f.landing)?;
        }
        let mounted = self
            .rows
            .values()
            .flat_map(|r| r.slots.iter().map(|s| s.node.clone()))
            .collect::<HashSet<_>>();
        for slot in std::mem::take(&mut self.departures) {
            if mounted.contains(&slot) && child(&slot)?.is_none() {
                refill_slot(&slot)?;
                settle_slot(&slot)?;
            }
        }
        self.emptied.clear();
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
        // 1. Capture controls/sizes without sampling transforms, then write
        // every active flight's values and size before entering any layout.
        // A zoomed pair lays each version out at its departure size.
        let mut frames = Vec::new();
        let mut pairs = Vec::new();
        for entry in &self.flights {
            match entry {
                Entry::One(f) if active.contains(&f.board) => {
                    frames.push((f, f.size_frame(Some(&active))?));
                }
                Entry::One(_) => {}
                Entry::Pair(pair) => pairs.push(pair),
            }
        }
        for (flight, frame) in &frames {
            let size = flight.size_to_lay_out(frame);
            flight.apply_size(frame.progress, size, frame.corners, frame.shadow)?;
        }
        for pair in &pairs {
            pair.size(&active)?;
        }
        let flown = frames
            .iter()
            .map(|(flight, _)| *flight)
            .chain(pairs.iter().flat_map(|pair| pair.awake(&active)))
            .collect::<Vec<_>>();
        let mut rows = HashSet::new();
        let mut roots = HashSet::new();
        for flight in &flown {
            rows.insert(flight.board.clone());
            roots.insert(layout_root(flight.board.clone()));
            // Plain hosts can give their unit an independent layout tree.
            if let Some(instance) = child(&flight.flight)? {
                roots.insert(layout_root(instance.core_handle()));
            }
        }
        let mut settle_order = roots.iter().cloned().collect::<Vec<_>>();
        // Independent inner trees must finish before a containing tree measures
        // them. Shared trees have already been deduplicated by their owner.
        settle_order.sort_by_key(|board| std::cmp::Reverse(artboard_depth(board)));
        // A shared solve need not move a fixed-size Screen, leaving its list
        // clean. Its dirty flight subtree still needs the ordinary row pass.
        settle_order.extend(rows.into_iter().filter(|row| !roots.contains(row)));

        // 2. Ordinary update passes solve dirty trees, settle their bounds and
        // refresh list caches/awake rows. No machine or timeline advances here.
        settle_frame(&settle_order);

        // 3. Sample every pose from the completed layout before applying any
        // pose. Projection writes only transforms/opacity, never layout inputs.
        // A zoomed pair solves each version onto its box; its writes touch
        // only its own slots and frames, which no other pose reads.
        let poses = frames
            .iter()
            .map(|(flight, frame)| flight.project_pose(frame, Some(&active)))
            .collect::<Result<Vec<_>>>()?;
        for pair in &pairs {
            pair.pose(&active)?;
        }
        for ((flight, _), pose) in frames.iter().zip(poses) {
            flight.apply_pose(pose)?;
        }
        // Then where each flight paints (docs/CSS_LAYOUT.md, 4.4): every
        // anchor's context first; when any changed, its Screen is sorted
        // with it before the search for what covers the anchor walks it.
        let mut contexts_changed = false;
        for flight in &flown {
            contexts_changed |= flight.read_context()?;
        }
        if contexts_changed {
            settle_frame(&settle_order);
        }
        for flight in &flown {
            flight.read_cover()?;
        }

        // 4. Refresh final transforms and hosted caches through the same native
        // path. Layout is now clean, so this pass does not solve it again.
        settle_frame(&settle_order);
        Ok(!flown.is_empty())
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
}
// Layout ownership is independent of where a flight's pose is authored.
fn layout_root(mut board: CoreHandle) -> CoreHandle {
    while let Some(parent) = board
        .with_downcast::<Artboard, _>(|board| {
            (!board.updates_own_layout())
                .then(|| board.parent_artboard())
                .flatten()
        })
        .flatten()
    {
        board = parent;
    }
    board
}
fn artboard_depth(board: &CoreHandle) -> usize {
    let mut depth = 0;
    let mut parent = board
        .with_downcast::<Artboard, _>(Artboard::parent_artboard)
        .flatten();
    while let Some(board) = parent {
        depth += 1;
        parent = board
            .with_downcast::<Artboard, _>(Artboard::parent_artboard)
            .flatten();
    }
    depth
}
fn settle_frame(boards: &[CoreHandle]) {
    for board in boards {
        // Size/style and binding targets use native component dirt. Skip clean
        // passes, including rows already settled by their hosting list.
        if board
            .with_downcast::<Artboard, _>(Artboard::has_component_dirt)
            .unwrap_or(false)
        {
            Artboard::update_pass_handle(board, false);
        }
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
/// The Screen holding a step's flight slots and the timeline `index` names
/// on it, read before anything moves. None when the step names none or flies
/// nothing. An index past that Screen's animations, or flights on two
/// Screens, refuses the step.
fn flight_timeline(index: u32, plan: &[Item]) -> Result<Option<(CoreHandle, CoreHandle)>> {
    if index == u32::MAX {
        return Ok(None);
    }
    let mut boards = plan
        .iter()
        .flat_map(|item| item.moves().iter().map(|(_, f)| &f.board));
    let Some(board) = boards.next() else {
        return Ok(None);
    };
    ensure!(
        boards.all(|other| other == board),
        "a step's flight timeline needs its flights on one Screen"
    );
    let animation = board
        .with_downcast::<Artboard, _>(|b| b.animation_handle_at(index as usize))
        .flatten()
        .ok_or_else(|| anyhow::anyhow!("flightAnimationId names no animation on its Screen"))?;
    Ok(Some((board.clone(), animation)))
}
/// Applies, at time 0 and in full, each keyed object of `animation` whose
/// target on `board` is one of `controls`. Before its first key a keyed
/// property applies that key's value (upstream `keyed_property.cpp:174-177`),
/// so each control takes its track's start whatever the delay. Nothing else
/// the timeline keys is touched, no instance is made or advanced and no event
/// fires: the machine's first advance plays the timeline from its start.
fn apply_first_frame(board: &CoreHandle, animation: &CoreHandle, controls: &HashSet<CoreHandle>) {
    let keyed = animation
        .with_downcast::<LinearAnimation, _>(|a| a.keyed_objects().to_vec())
        .unwrap_or_default();
    board.with_downcast_mut::<Artboard, _>(|b| {
        for object in &keyed {
            let target = object
                .with_downcast::<KeyedObject, _>(|o| o.base.object_id())
                .and_then(|id| b.resolve_handle(id));
            if target.is_some_and(|target| controls.contains(&target)) {
                KeyedObject::apply_occurrence(object, b, 0.0, 1.0, None);
            }
        }
    });
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
                flight_animation: v.flight_animation,
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

/// Settles a slot's row after transport, as a frame's Layout and Settle do:
/// the layout tree that holds the row first, so the moved unit has its new
/// box before the next draw (a Scale-mode unit lands still laid out at its
/// departure box), then the row itself.
fn settle_slot(slot: &CoreHandle) -> Result<()> {
    let row = read(slot, NestedArtboard::parent_artboard_handle)?.expect("slot row");
    // A row in its list's shared tree is solved only by that tree's owner.
    let owner = layout_root(row.clone());
    if owner != row {
        Artboard::update_pass_handle(&owner, false);
    }
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
    /// Applies the `screens` list's pending insertions, removals and moves and
    /// settles component updates, without advancing playback. Then the plan
    /// puts away every spare copy no flight claims, so nothing a spare would
    /// write is left without an owner. Begin, End and row sleep run this
    /// first, and so does a frame whose list has a change waiting.
    pub fn flush_membership(&mut self, root: &RuntimeArtboardInstanceHandle) -> Result<()> {
        #[cfg(feature = "test-support")]
        crate::hosted_scene::record_walk(crate::hosted_scene::Walk::MembershipFlush);
        root.update_data_binds(false);
        root.update_pass(false);
        self.prepare(root)
    }
    pub fn step_begin(
        &mut self,
        root: &RuntimeArtboardInstanceHandle,
        incoming: &str,
        name: Option<&str>,
    ) -> Result<()> {
        let scene = crate::hosted_scene::root_scene(root)
            .ok_or_else(|| anyhow::anyhow!("root is not a live-move page"))?;
        self.flush_membership(root)?;
        // The root's pass can leave a binding that changes the arriving
        // Screen's own layout unsettled, so that Screen runs its own update
        // pass, the one its list gives a row with work: whatever the plan
        // reads of it is where it rests. It binds and lays out, and advances
        // nothing.
        let arriving = scene
            .plan
            .borrow()
            .rows
            .get(incoming)
            .map(|row| row.board.clone());
        if let Some(board) = arriving.filter(crate::hosted_scene::is_active_artboard) {
            Artboard::update_pass_handle(&board, false);
        }
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
        self.flush_membership(root)?;
        scene.plan.borrow_mut().step_end(root, incoming, name)
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
        crate::hosted_scene::root_scene(root).map_or_else(Vec::new, |s| {
            s.plan.borrow().all_flights().cloned().collect()
        })
    }
    #[cfg(feature = "test-support")]
    pub fn with_flights(
        &mut self,
        root: &RuntimeArtboardInstanceHandle,
        body: impl FnOnce(&mut Vec<Entry>),
    ) {
        let scene = crate::hosted_scene::root_scene(root).expect("live scene");
        body(&mut scene.plan.borrow_mut().flights);
    }
    /// What each zoomed pair of the step read when it started.
    #[cfg(feature = "test-support")]
    pub fn pair_ends(&self, root: &RuntimeArtboardInstanceHandle) -> Vec<zoom::Ends> {
        crate::hosted_scene::root_scene(root).map_or_else(Vec::new, |s| {
            s.plan
                .borrow()
                .flights
                .iter()
                .filter_map(|entry| match entry {
                    Entry::One(_) => None,
                    Entry::Pair(pair) => Some(pair.ends()),
                })
                .collect()
        })
    }
    /// Where a component is drawn, from `live_move/drawn.rs`.
    #[cfg(feature = "test-support")]
    pub fn drawn(
        &self,
        component: &CoreHandle,
        when: super::When,
    ) -> Result<crate::source::math::mat2d::Mat2D> {
        super::drawn::drawn(component, when)
    }
    /// The matrix placement gives a CSS slot's frame at rest, in its Screen's
    /// space, made by the same helpers placement's resting boxes use.
    #[cfg(feature = "test-support")]
    pub fn resting_slot(&self, slot: &CoreHandle) -> Option<crate::source::math::mat2d::Mat2D> {
        super::follow::resting_slot(slot)
    }
    /// Sets a flying frame's clip, as a zoomed pair does.
    #[cfg(feature = "test-support")]
    pub fn set_flight_clip(
        &mut self,
        frame: &RuntimeArtboardInstanceHandle,
        clip: Option<crate::source::artboard::FlightClip>,
    ) {
        frame.with_artboard_mut(|artboard| artboard.set_flight_clip(clip));
    }
    /// Lands the flights a test pushed without a step, through the same
    /// landing a step's end runs.
    #[cfg(feature = "test-support")]
    pub fn land_flights(&mut self, root: &RuntimeArtboardInstanceHandle) -> Result<()> {
        let scene = crate::hosted_scene::root_scene(root).expect("live scene");
        scene.plan.borrow_mut().land(root)
    }
    #[cfg(feature = "test-support")]
    pub fn reset_for_test(&mut self, root: &RuntimeArtboardInstanceHandle) {
        let scene = crate::hosted_scene::root_scene(root).expect("live scene");
        *scene.plan.borrow_mut() = Plan::default();
    }
}
