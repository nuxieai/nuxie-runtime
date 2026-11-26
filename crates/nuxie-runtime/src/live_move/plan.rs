use super::{
    records::{LiveStep, LiveStepUnit, Values, validate_step_units},
    transport::{child, read, refill_slot},
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
struct Step {
    name: String,
    from: String,
    to: String,
    units: Vec<Values>,
    /// The step's flight timeline: its index among the animations of the
    /// Screen holding the step's flight records, as an `AnimationState`'s
    /// `animationId` names one. `u32::MAX` when the step names none.
    flight_animation: u32,
}
#[derive(Clone)]
struct Screen {
    board: CoreHandle,
    /// The Screen's elements by flight key, read only when a Screen of the
    /// file holds flight records, and its flight records by key, in file
    /// order.
    elements: Vec<(u32, CoreHandle)>,
    records: Vec<(u32, CoreHandle)>,
}
/// The ends a running step holds. On a `screens` page, its arriving Screen
/// by its artboard's name. On a driver page, a nameless step's arriving row,
/// or a named step's two rows, each by its list item. An item is resolved
/// from the row's key at begin, so a key removed and mounted again names a
/// new row.
#[derive(Clone, PartialEq)]
enum Ends {
    Screen(String),
    Row(CoreHandle),
    Rows {
        outgoing: CoreHandle,
        incoming: CoreHandle,
    },
}
/// The two rows a named step between rows of a driver page names, checked
/// (`Plan::between`): each row's list item and its Screen, read from its
/// own instance, and which of them is upper, the one later in the list's
/// order.
struct Between {
    step: Step,
    outgoing: (CoreHandle, Screen),
    incoming: (CoreHandle, Screen),
    incoming_upper: bool,
}
impl Between {
    /// The upper row, the only one whose flight records the step reads.
    fn upper(&self) -> &Screen {
        if self.incoming_upper {
            &self.incoming.1
        } else {
            &self.outgoing.1
        }
    }
}
/// File plans are scoped to mounted row identities. Between steps every
/// placement holds its own Screen's instance, so nothing is kept about where
/// one went.
#[derive(Default)]
pub(crate) struct Plan {
    root: Option<CoreHandle>,
    revision: Option<u64>,
    steps: Vec<Step>,
    rows: HashMap<String, Screen>,
    active: Option<(Ends, Option<String>)>,
    pub index: crate::hosted_scene::HostedIndex,
    /// The active step's flying copies (live_move/in_place.rs), and its
    /// zoomed pairs of them.
    pub lifted: Vec<super::in_place::InPlaceFlight>,
    pairs: Vec<super::in_place::InPlacePair>,
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
        let page = crate::hosted_scene::page_list(root);
        // A driver page reads its steps but no Screens by name: its rows may
        // repeat an artboard, which `screens` refuses, so a step names its
        // two rows by key (`step_begin_between`).
        let driver_list = page
            .as_ref()
            .filter(|page| page.keyed)
            .map(|page| page.list.clone());
        if self.root.as_ref() != Some(&root.core_handle()) {
            ensure!(
                self.active.is_none(),
                "cannot change scene occurrence during a step"
            );
            self.steps = read_steps(root)?;
            self.root = Some(root.core_handle());
        }
        if let Some(list) = driver_list {
            self.rows.clear();
            let mounted = page_rows(&list);
            let has = |item: &CoreHandle| mounted.iter().any(|(row, _)| row == item);
            // A row step cancels when a row it names leaves the page's list,
            // compared by list item, so a key removed and mounted again in
            // one round cancels it too. A step between rows also cancels
            // when a copy or a record it flies is gone, as on a `screens`
            // page. A row asleep stays mounted.
            let end_left = match &self.active {
                Some((Ends::Row(item), _)) => !has(item),
                Some((Ends::Rows { outgoing, incoming }, _)) => !has(outgoing) || !has(incoming),
                _ => false,
            };
            let rows = mounted
                .into_iter()
                .map(|(_, board)| board)
                .collect::<HashSet<_>>();
            let lost_copy = self.lifted.iter().any(|flight| !flight.is_whole(&rows));
            if end_left || lost_copy {
                self.settle(&rows)?;
            }
            self.index.active(root);
            self.revision = Some(crate::hosted_scene::membership_revision(
                &root.core_handle(),
            ));
            return Ok(());
        }
        // Ordinary upstream files never acquire a page-host requirement.
        if page.is_none() {
            ensure!(self.steps.is_empty(), "live steps require a screens list");
            self.revision = Some(revision);
            return Ok(());
        }
        self.rows = screen_rows(root)?;
        let incoming_left = self.active.as_ref().is_some_and(|(ends, _)| match ends {
            Ends::Screen(name) => !self.rows.contains_key(name),
            Ends::Row(_) | Ends::Rows { .. } => true,
        });
        // A step cancels when its incoming row, or a Screen, a copy or a
        // record it flies, is gone. Every plan entry prepares first, so the
        // cancel settles before the next frame is laid out or drawn.
        let rows = self
            .rows
            .values()
            .map(|row| row.board.clone())
            .collect::<HashSet<_>>();
        let lost_copy = self.lifted.iter().any(|flight| !flight.is_whole(&rows));
        if incoming_left || lost_copy {
            self.settle(&rows)?;
        }
        self.index.active(root);
        self.revision = Some(crate::hosted_scene::membership_revision(
            &root.core_handle(),
        ));
        Ok(())
    }
    /// Puts back everything a cancelled step moved, and ends the step.
    fn settle(&mut self, rows: &HashSet<CoreHandle>) -> Result<()> {
        // The cancel is the landing's clear, and each Screen still mounted
        // that drew or left out a lift settles now, as at landing, so its
        // next draw has no lift entry in it.
        let mut lifted = std::mem::take(&mut self.lifted);
        let touched = lifted
            .iter()
            .flat_map(|flight| [flight.row.clone(), flight.board.clone()])
            .filter(|row| rows.contains(row))
            .collect::<HashSet<_>>();
        // A carried instance's blend ends before its Screen settles, so the
        // first draw after the cancel shows that Screen's values.
        for flight in &mut lifted {
            flight.release_values();
            flight.clear();
        }
        for row in &touched {
            settle_row(row);
        }
        // A carried instance goes back to the placement it left, when that
        // is still on a Screen and empty, and the placement it flew in gets
        // a copy of its own. Otherwise it stays where it is, and the
        // placement it left, if still there, gets one. The list's unmount
        // hook already took one home whose arriving Screen went.
        for flight in &lifted {
            let Some(carried) = &flight.carried else {
                continue;
            };
            let (from, to) = (&carried.from, &carried.to);
            let home = |host: &CoreHandle| crate::hosted_scene::is_carry_host(host);
            if home(from) && child(from)?.is_none() && home(to) && child(to)?.is_some() {
                super::carry(to, from, false)?;
                carried.written.home();
                settle_slot(from)?;
                refill_left(to)?;
            } else {
                if home(to) && child(to)?.is_some() {
                    super::transport::land_in_place(to)?;
                }
                refill_left(from)?;
            }
        }
        drop(lifted);
        self.pairs.clear();
        self.active = None;
        Ok(())
    }
    /// Whether a step is running: begun, a nameless one (a fade or a cut)
    /// included, and not yet landed or cancelled. A Screen at 1 reads it to
    /// draw a mark of 3 above every row (css_layout/group_opacity.rs).
    pub(crate) fn step_running(&self) -> bool {
        self.active.is_some()
    }
    pub fn step_begin(
        &mut self,
        root: &RuntimeArtboardInstanceHandle,
        incoming: &str,
        step_name: Option<&str>,
    ) -> Result<()> {
        ensure!(self.active.is_none(), "step already active");
        self.prepare(root)?;
        if let Some(name) = step_name {
            return self.step_begin_in_place(root, incoming, name);
        }
        let incoming_row = self
            .rows
            .get(incoming)
            .ok_or_else(|| anyhow::anyhow!("incoming Screen not found"))?;
        ensure!(
            crate::hosted_scene::is_active_artboard(&incoming_row.board),
            "incoming Screen is collapsed"
        );
        self.active = Some((Ends::Screen(incoming.to_owned()), None));
        self.after_advance(root)?;
        Ok(())
    }
    /// Begins a nameless step on a driver page whose arriving end is the
    /// row of `list`, the page's list, mounted by `key`: it must be mounted
    /// and awake. The step holds the row's list item.
    pub fn step_begin_row(
        &mut self,
        root: &RuntimeArtboardInstanceHandle,
        list: &CoreHandle,
        key: &[u8],
    ) -> Result<()> {
        ensure!(self.active.is_none(), "step already active");
        self.prepare(root)?;
        let (item, _) = arriving_row(list, key)?;
        self.active = Some((Ends::Row(item), None));
        self.after_advance(root)?;
        Ok(())
    }
    /// Whether `key` names the row the running row step holds.
    fn holds_row(&self, list: &CoreHandle, key: &[u8]) -> Result<()> {
        let item = list
            .with_downcast::<ArtboardComponentList, _>(|list| list.keyed_row_item(key))
            .flatten();
        ensure!(
            matches!(
                (&self.active, &item),
                (Some((Ends::Row(active), None)), Some(item)) if active == item
            ),
            "step_end mismatch"
        );
        Ok(())
    }
    /// Lands the nameless step [`Self::step_begin_row`] began: `key` must
    /// still name the row it holds.
    pub fn step_end_row(
        &mut self,
        root: &RuntimeArtboardInstanceHandle,
        list: &CoreHandle,
        key: &[u8],
    ) -> Result<()> {
        self.prepare(root)?;
        self.holds_row(list, key)?;
        self.land(root)
    }
    /// The rows of `list`, the page's list, that the step `name` would
    /// begin between, checked as `step_begin_between` checks them, before
    /// anything is marked: each key mounted, two different rows, the step
    /// published and carrying no instance, the step's from and to artboards
    /// the rows' own, and both rows awake.
    fn between(
        &self,
        list: &CoreHandle,
        outgoing_key: &[u8],
        incoming_key: &[u8],
        name: &str,
    ) -> Result<Between> {
        let (outgoing_place, outgoing_item, outgoing) = step_row(list, outgoing_key, "outgoing")?;
        let (incoming_place, incoming_item, incoming) = step_row(list, incoming_key, "incoming")?;
        ensure!(
            outgoing_item != incoming_item,
            "a step's two ends name one row"
        );
        let step = self.step_named(name)?;
        // Carrying one instance between rows comes with 4.5.
        ensure!(
            !step.units.iter().any(Values::transfers),
            "step {name} carries an instance, which a step between rows does not do yet"
        );
        // The step's artboards name its kind: which component each row is.
        // A step from an artboard to itself is two rows of one component.
        let artboard = |board: &CoreHandle| {
            board
                .with_downcast::<Artboard, _>(|b| b.name().to_owned())
                .unwrap_or_default()
        };
        ensure!(
            artboard(&outgoing) == step.from && artboard(&incoming) == step.to,
            "step ends mismatch"
        );
        for board in [&outgoing, &incoming] {
            ensure!(
                crate::hosted_scene::is_active_artboard(board),
                "a row of the step is asleep"
            );
        }
        let between = Between {
            step,
            outgoing: (outgoing_item, row_screen(&outgoing)),
            incoming: (incoming_item, row_screen(&incoming)),
            incoming_upper: incoming_place > outgoing_place,
        };
        check_rows(
            &between.step,
            &between.outgoing.1,
            &between.incoming.1,
            &[between.upper()],
        )?;
        Ok(between)
    }
    /// Begins the step `name` on a driver page between the row of `list`,
    /// the page's list, mounted by `outgoing_key` and the one mounted by
    /// `incoming_key`. The rows are checked (`between`), and the step's
    /// flight records are read from the upper row alone, the one later in
    /// the list's order; each copy is read from its own row. Then it takes
    /// off as a named step on a `screens` page does. The step holds both
    /// rows' list items.
    pub fn step_begin_between(
        &mut self,
        root: &RuntimeArtboardInstanceHandle,
        list: &CoreHandle,
        outgoing_key: &[u8],
        incoming_key: &[u8],
        name: &str,
    ) -> Result<()> {
        ensure!(self.active.is_none(), "step already active");
        self.prepare(root)?;
        let between = self.between(list, outgoing_key, incoming_key, name)?;
        let (flights, pairs) = take_off(
            root,
            &between.step,
            &between.outgoing.1,
            &between.incoming.1,
            &[between.upper()],
        )?;
        self.lifted = flights;
        self.pairs = pairs;
        self.active = Some((
            Ends::Rows {
                outgoing: between.outgoing.0,
                incoming: between.incoming.0,
            },
            Some(name.to_owned()),
        ));
        self.after_advance(root)?;
        Ok(())
    }
    /// Whether the running step is the step `name` between the rows `list`
    /// mounts by `outgoing_key` and `incoming_key`, by list item.
    fn holds_rows(
        &self,
        list: &CoreHandle,
        outgoing_key: &[u8],
        incoming_key: &[u8],
        name: &str,
    ) -> Result<()> {
        let item = |key: &[u8]| {
            list.with_downcast::<ArtboardComponentList, _>(|list| list.keyed_row_item(key))
                .flatten()
        };
        ensure!(
            matches!(
                (&self.active, item(outgoing_key), item(incoming_key)),
                (
                    Some((Ends::Rows { outgoing, incoming }, Some(active))),
                    Some(leaving),
                    Some(arriving),
                ) if *outgoing == leaving && *incoming == arriving && active == name
            ),
            "step_end mismatch"
        );
        Ok(())
    }
    /// Lands the step [`Self::step_begin_between`] began: both keys must
    /// still name the rows it holds, and `name` the step.
    pub fn step_end_between(
        &mut self,
        root: &RuntimeArtboardInstanceHandle,
        list: &CoreHandle,
        outgoing_key: &[u8],
        incoming_key: &[u8],
        name: &str,
    ) -> Result<()> {
        self.prepare(root)?;
        self.holds_rows(list, outgoing_key, incoming_key, name)?;
        self.land(root)
    }
    pub fn step_end(
        &mut self,
        root: &RuntimeArtboardInstanceHandle,
        incoming: &str,
        name: Option<&str>,
    ) -> Result<()> {
        self.prepare(root)?;
        ensure!(
            self.active.as_ref().is_some_and(
                |(s, n)| *s == Ends::Screen(incoming.to_owned()) && n.as_deref() == name
            ),
            "step_end mismatch"
        );
        self.land(root)
    }
    /// Lands every flight, as the end of a step does: each copy where it
    /// rests on its own Screen, and a transferred unit on the arriving one.
    /// Then the placement each transferred unit left gets a copy of its own
    /// (decision 297); a row removed mid-flight took its placement with it.
    /// Both SDKs end a step and retire the leaving row in one batch, so
    /// that copy is first drawn when that Screen is shown again.
    fn land(&mut self, root: &RuntimeArtboardInstanceHandle) -> Result<()> {
        // Each Screen draws its own copy as authored in the same frame.
        let mut lifted = std::mem::take(&mut self.lifted);
        self.pairs.clear();
        let landed_rows = lifted
            .iter()
            .flat_map(|flight| [flight.row.clone(), flight.board.clone()])
            .collect::<HashSet<_>>();
        // A carried instance's blend ends before its Screen settles, so its
        // bindings take the arriving Screen's values in that settlement.
        for flight in &mut lifted {
            flight.release_values();
            flight.clear();
        }
        for row in &landed_rows {
            settle_row(row);
        }
        // A carried instance has landed where it flew, and the placement it
        // left gets a copy of its own (decision 297).
        let carried = lifted
            .into_iter()
            .filter_map(|flight| flight.carried)
            .map(|carried| (carried.from, carried.to))
            .collect::<Vec<_>>();
        for (from, to) in carried {
            if crate::hosted_scene::is_carry_host(&to) {
                super::transport::land_in_place(&to)?;
            }
            refill_left(&from)?;
        }
        self.active = None;
        self.prepare(root)?;
        Ok(())
    }
    pub fn after_advance(&mut self, root: &RuntimeArtboardInstanceHandle) -> Result<bool> {
        self.prepare(root)?;
        if self.lifted.is_empty() {
            return Ok(false);
        }
        self.follow_in_place(root)
    }
    /// Begins a named step (live_move/in_place.rs) on a `screens` page,
    /// whose flight records may be on either of its Screens: the leaving
    /// one is read first.
    fn step_begin_in_place(
        &mut self,
        root: &RuntimeArtboardInstanceHandle,
        incoming: &str,
        name: &str,
    ) -> Result<()> {
        let step = self.step_named(name)?;
        ensure!(step.to == incoming, "step destination mismatch");
        let outgoing = self
            .rows
            .get(&step.from)
            .ok_or_else(|| anyhow::anyhow!("outgoing Screen not mounted"))?;
        let arriving = self
            .rows
            .get(incoming)
            .ok_or_else(|| anyhow::anyhow!("incoming Screen not found"))?;
        for row in [outgoing, arriving] {
            ensure!(
                crate::hosted_scene::is_active_artboard(&row.board),
                "a Screen of the step is collapsed"
            );
        }
        let (flights, pairs) = take_off(root, &step, outgoing, arriving, &[outgoing, arriving])?;
        self.lifted = flights;
        self.pairs = pairs;
        self.active = Some((Ends::Screen(incoming.to_owned()), Some(name.to_owned())));
        self.after_advance(root)?;
        Ok(())
    }
    /// The published step named `name`, its units valid.
    fn step_named(&self, name: &str) -> Result<Step> {
        let step = self
            .steps
            .iter()
            .find(|s| s.name == name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("published step not found"))?;
        validate_step_units(&step.units)?;
        Ok(step)
    }

    /// One frame of a step, in four barriers. Size: every copy's controls and
    /// its ends' resting sizes, then its flight size, corners and shadow.
    /// Layout: both Screens of every copy settle, inner trees first. Pose:
    /// every lift's transform and opacity from the settled layout, then where
    /// each paints. Settle.
    fn follow_in_place(&mut self, root: &RuntimeArtboardInstanceHandle) -> Result<bool> {
        let active = self
            .index
            .active(root)
            .iter()
            .cloned()
            .collect::<HashSet<_>>();
        // A zoomed version is held at its departure size: it has no Size.
        let frames = self
            .lifted
            .iter()
            .map(|flight| {
                (!flight.zoomed)
                    .then(|| flight.size_frame(Some(&active)))
                    .transpose()
            })
            .collect::<Result<Vec<_>>>()?;
        for (flight, frame) in self.lifted.iter().zip(&frames) {
            if let Some(frame) = frame {
                flight.apply_size(frame, active.contains(&flight.row));
            }
        }
        let rows = self
            .lifted
            .iter()
            .flat_map(|flight| flight.rows().into_iter().cloned())
            .filter(|row| active.contains(row))
            .collect::<HashSet<_>>();
        let mut settle_order = rows
            .iter()
            .map(|row| layout_root(row.clone()))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        settle_order.sort_by_key(|board| std::cmp::Reverse(artboard_depth(board)));
        let rest = rows
            .iter()
            .filter(|row| !settle_order.contains(row))
            .cloned()
            .collect::<Vec<_>>();
        settle_order.extend(rest);
        settle_frame(&settle_order);
        let poses = self
            .lifted
            .iter()
            .zip(&frames)
            .map(|(flight, frame)| {
                frame
                    .as_ref()
                    .map(|frame| flight.project(frame, Some(&active)))
                    .transpose()
            })
            .collect::<Result<Vec<_>>>()?;
        for (flight, pose) in self.lifted.iter().zip(poses) {
            if let Some(pose) = pose {
                flight.apply_pose(pose);
            }
        }
        for pair in &self.pairs {
            let version = |copy: &CoreHandle| {
                self.lifted
                    .iter()
                    .find(|flight| flight.element == *copy)
                    .ok_or_else(|| anyhow::anyhow!("a zoomed version left its step"))
            };
            pair.pose(
                [version(&pair.copies[0])?, version(&pair.copies[1])?],
                &active,
            )?;
        }
        // A placement a transfer left sorts by the carried instance's frame:
        // when that changed how it sorts, its Screen sorts again before the
        // search for what covers each anchor walks it.
        let mut contexts_changed = false;
        for flight in &self.lifted {
            contexts_changed |= flight.read_context();
        }
        if contexts_changed {
            settle_frame(&settle_order);
        }
        let mut boards: Vec<(CoreHandle, Vec<&super::in_place::InPlaceFlight>)> = Vec::new();
        for flight in &self.lifted {
            match boards.iter_mut().find(|(seen, _)| *seen == flight.board) {
                Some((_, copies)) => copies.push(flight),
                None => boards.push((flight.board.clone(), vec![flight])),
            }
        }
        for (board, copies) in boards {
            if super::in_place::InPlaceFlight::read_covers(&board, &copies) {
                if let Some(dirty) = board.artboard_dirty_handle() {
                    dirty.mark_css_draw_order_dirty();
                }
            }
        }
        // What every lift draws moved: the upper Screens draw again.
        for board in self
            .lifted
            .iter()
            .map(|flight| &flight.board)
            .collect::<HashSet<_>>()
        {
            if let Some(dirty) = board.artboard_dirty_handle() {
                dirty.changed();
            }
        }
        settle_frame(&settle_order);
        Ok(true)
    }

    pub fn frame_scope(
        &mut self,
        root: &RuntimeArtboardInstanceHandle,
        roots: &[crate::RuntimeOwnedViewModelHandle],
    ) -> Result<Option<super::FrameScope>> {
        self.prepare(root)?;
        self.index.active(root);
        if self.steps.is_empty() && !self.index.has_collapsed_rows(root) {
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
/// The element a step unit's `key` names on `row`.
fn unit_element(row: &Screen, key: u32) -> Result<CoreHandle> {
    row.elements
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, element)| element.clone())
        .ok_or_else(|| anyhow::anyhow!("no element with flight key {key}"))
}
/// The flight record `key` names, looked for in `records` in order, and the
/// artboard that holds it.
fn flight_record(records: &[&Screen], key: u32) -> Result<(CoreHandle, CoreHandle)> {
    records
        .iter()
        .flat_map(|row| row.records.iter().map(move |record| (*row, record)))
        .find(|(_, (k, _))| *k == key)
        .map(|(row, (_, record))| (row.board.clone(), record.clone()))
        .ok_or_else(|| anyhow::anyhow!("unknown flight record {key} on step Screens"))
}
/// The step's flight timeline, its `flightAnimationId` among the animations
/// of `board`, the artboard that holds its flight records, if it names one.
fn flight_timeline(
    step: &Step,
    board: Option<&CoreHandle>,
) -> Result<Option<(CoreHandle, CoreHandle)>> {
    Ok(match (step.flight_animation, board) {
        (u32::MAX, _) | (_, None) => None,
        (index, Some(board)) => Some((
            board.clone(),
            board
                .with_downcast::<Artboard, _>(|b| b.animation_handle_at(index as usize))
                .flatten()
                .ok_or_else(|| {
                    anyhow::anyhow!("flightAnimationId names no animation on its Screen")
                })?,
        )),
    })
}
/// Every refusal [`take_off`] reads from the rows' own content, read before
/// anything is flushed or marked: each unit's elements on their rows, an
/// element named twice or inside another, an element that does not fly in
/// place, each flight record in `records`, records on two artboards, and a
/// flight timeline that names no animation. What depends on layout (a held
/// element's parent, a resting size) is read at take-off, after the flush.
/// A step between rows checks these first, so the flush never runs for a
/// step its rows' content refuses.
fn check_rows(
    step: &Step,
    outgoing: &Screen,
    arriving: &Screen,
    records: &[&Screen],
) -> Result<()> {
    use super::in_place::{Flying, is_inside};
    let mut used = HashSet::new();
    let mut boards = HashSet::new();
    for unit in &step.units {
        let from = unit_element(outgoing, unit.outgoing_unit_key)?;
        let to = unit_element(arriving, unit.incoming_unit_key)?;
        ensure!(
            used.insert(from.clone()) && used.insert(to.clone()),
            "overlapping flight owners"
        );
        Flying::of(&from)?;
        Flying::of(&to)?;
        let keys: &[u32] = if unit.transfers() {
            &[unit.incoming_flight_key]
        } else {
            &[unit.outgoing_flight_key, unit.incoming_flight_key]
        };
        for key in keys {
            boards.insert(flight_record(records, *key)?.0);
        }
    }
    ensure!(
        boards.len() <= 1,
        "a step's flight records need to be on one Screen"
    );
    for a in &used {
        ensure!(
            !used.iter().any(|b| is_inside(a, b)),
            "a flight inside another flight"
        );
    }
    flight_timeline(step, boards.iter().next())?;
    Ok(())
}
/// Takes off a named step's flights: every check before any mark, then
/// each copy's marks and the flight timeline's first frame on the flight
/// records. A refusal marks nothing. `records` are the Screens the step's
/// flight records are read from, in the order they are looked in: on a
/// `screens` page both of its Screens, the leaving one first, and on a
/// driver page the upper row alone.
fn take_off(
    root: &RuntimeArtboardInstanceHandle,
    step: &Step,
    outgoing: &Screen,
    arriving: &Screen,
    records: &[&Screen],
) -> Result<(
    Vec<super::in_place::InPlaceFlight>,
    Vec<super::in_place::InPlacePair>,
)> {
    use super::in_place::{Carried, InPlaceFlight, InPlacePair, is_inside};
    let element = unit_element;
    let record = |key: u32| flight_record(records, key);
    let mut flights = Vec::new();
    let mut zoomed = Vec::new();
    let mut used = HashSet::new();
    let mut boards = HashSet::new();
    for unit in &step.units {
        let from = element(outgoing, unit.outgoing_unit_key)?;
        let to = element(arriving, unit.incoming_unit_key)?;
        ensure!(
            used.insert(from.clone()) && used.insert(to.clone()),
            "overlapping flight owners"
        );
        // A unit that transfers flies one copy, the arriving one, which
        // the carry fills with the leaving one's instance. It never
        // zooms.
        if unit.transfers() {
            let carried = Carried::read((&from, &outgoing.board), &to)?;
            let (board, record) = record(unit.incoming_flight_key)?;
            boards.insert(board.clone());
            flights.push(
                InPlaceFlight::new(
                    record,
                    to.clone(),
                    arriving.board.clone(),
                    (from.clone(), outgoing.board.clone()),
                    (to.clone(), arriving.board.clone()),
                    board,
                    false,
                )?
                .carrying(carried),
            );
            continue;
        }
        let rule = zoom::Rule::of(unit.read_zoom());
        if let Some(rule) = rule {
            zoomed.push((rule, flights.len()));
        }
        for (key, copy, row) in [
            (unit.outgoing_flight_key, &from, &outgoing.board),
            (unit.incoming_flight_key, &to, &arriving.board),
        ] {
            let (board, record) = record(key)?;
            boards.insert(board.clone());
            flights.push(InPlaceFlight::new(
                record,
                copy.clone(),
                row.clone(),
                (from.clone(), outgoing.board.clone()),
                (to.clone(), arriving.board.clone()),
                board,
                rule.is_some(),
            )?);
        }
    }
    ensure!(
        boards.len() <= 1,
        "a step's flight records need to be on one Screen"
    );
    // A matched element inside another of the same step does not fly
    // on its own.
    for a in &used {
        ensure!(
            !used.iter().any(|b| is_inside(a, b)),
            "a flight inside another flight"
        );
    }
    for flight in flights.iter().filter(|flight| !flight.zoomed) {
        flight.project(&flight.size_frame(None)?, None)?;
    }
    // The point a zoom's tracker follows on Back: the center of the
    // page's drawing space.
    let center = root.with_artboard(|b| b.bounds().center());
    let pairs = zoomed
        .into_iter()
        .map(|(rule, at)| InPlacePair::take_off(rule, [&flights[at], &flights[at + 1]], center))
        .collect::<Result<Vec<_>>>()?;
    let timeline = flight_timeline(step, boards.iter().next())?;
    // Every check has passed. Each transferring unit's instance moves into
    // the arriving placement, which discards that placement's own
    // never-shown instance and gives it the arriving Screen's props, and its
    // Screen settles once, as after any carry. Then mark. Lifts that share
    // an anchor draw by their records' order, a later record on top, so a
    // later record is placed first.
    let order = |flight: &InPlaceFlight| {
        flight
            .board
            .with_downcast::<Artboard, _>(|board| {
                board
                    .objects()
                    .iter()
                    .position(|o| o.as_ref() == Some(&flight.record))
            })
            .flatten()
            .unwrap_or(0)
    };
    flights.sort_by_key(|flight| std::cmp::Reverse(order(flight)));
    for flight in &mut flights {
        if let Some(carried) = &mut flight.carried {
            super::transport::move_to_flight(&carried.from, &carried.to, &mut carried.values)?;
            carried.written.take_off();
            // The placement it left keeps the instance's frame at rest
            // for the step: its Screen sorts it, and the lift's cover
            // reads it, as it rested.
            carried.link();
            settle_row(&flight.row);
        }
    }
    for flight in &flights {
        flight.mark(if flight.zoomed {
            crate::css_layout::lift::OpacityRoot::Nothing
        } else {
            crate::css_layout::lift::OpacityRoot::Own
        });
    }
    if let Some((board, animation)) = timeline {
        let controls = flights.iter().map(|f| f.record.clone()).collect();
        apply_first_frame(&board, &animation, &controls);
    }
    Ok((flights, pairs))
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
        let objects = if crate::hosted_scene::is_screen(&board.core_handle()) {
            board.with_artboard(|b| b.objects().iter().flatten().cloned().collect::<Vec<_>>())
        } else {
            Vec::new()
        };
        // Only a Screen holding flight records reads them here; its file's
        // other Screens read their elements' keys below.
        let records = super::in_place::flight_records(&objects);
        let elements = if records.is_empty() {
            Vec::new()
        } else {
            super::in_place::keyed_elements(&objects)
        };
        result.insert(
            name,
            Screen {
                board: board.core_handle(),
                elements,
                records,
            },
        );
    }
    // A file with flight records flies elements of every Screen, records or
    // not. A file without any reads nothing more.
    if result.values().any(|row| !row.records.is_empty()) {
        for row in result.values_mut().filter(|row| row.records.is_empty()) {
            let objects = row
                .board
                .with_downcast::<Artboard, _>(|b| {
                    b.objects().iter().flatten().cloned().collect::<Vec<_>>()
                })
                .unwrap_or_default();
            row.elements = super::in_place::keyed_elements(&objects);
        }
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

/// Gives `placement` a copy of its own when it is still on a Screen and
/// empty, and settles its Screen.
fn refill_left(placement: &CoreHandle) -> Result<()> {
    if crate::hosted_scene::is_carry_host(placement) && child(placement)?.is_none() {
        refill_slot(placement)?;
        settle_slot(placement)?;
    }
    Ok(())
}
/// Settles a Screen after a step lands, as a frame's Layout and Settle do:
/// the layout tree that holds it first, then the Screen.
fn settle_row(row: &CoreHandle) {
    let owner = layout_root(row.clone());
    if owner != *row {
        Artboard::update_pass_handle(&owner, false);
    }
    Artboard::update_pass_handle(row, false);
}

/// Settles the row of a placement after transport, so the moved unit has its
/// new box before the next draw.
fn settle_slot(slot: &CoreHandle) -> Result<()> {
    let row = read(slot, NestedArtboard::parent_artboard_handle)?.expect("slot row");
    settle_row(&row);
    Ok(())
}

/// A step named by Screen on a driver page is refused, before anything
/// runs: its rows are named by key, a nameless step's arriving row
/// (`step_begin_row`) or a named step's two rows (`step_begin_between`).
fn refuse_driver_page(scene: &crate::hosted_scene::SceneState) -> Result<()> {
    ensure!(!scene.page.keyed, "a driver page names its rows by key");
    Ok(())
}
/// The list item and the artboard of the row of `list` mounted by `key`,
/// which a row step may arrive at: mounted and awake.
fn arriving_row(list: &CoreHandle, key: &[u8]) -> Result<(CoreHandle, CoreHandle)> {
    let (item, board) = list
        .with_downcast::<ArtboardComponentList, _>(|list| list.keyed_row(key))
        .flatten()
        .ok_or_else(|| anyhow::anyhow!("incoming row not found"))?;
    let board = board.core_handle();
    ensure!(
        crate::hosted_scene::is_active_artboard(&board),
        "incoming row is collapsed"
    );
    Ok((item, board))
}
/// The row of `list` mounted by `key` that a step between rows names: its
/// place in the list's order, its list item and its artboard.
fn step_row(list: &CoreHandle, key: &[u8], end: &str) -> Result<(usize, CoreHandle, CoreHandle)> {
    let (place, item, board) = list
        .with_downcast::<ArtboardComponentList, _>(|list| {
            let place = list.row_keys().iter().position(|row| *row == key)?;
            let (item, board) = list.keyed_row(key)?;
            Some((place, item, board))
        })
        .flatten()
        .ok_or_else(|| anyhow::anyhow!("{end} row not found"))?;
    Ok((place, item, board.core_handle()))
}
/// The rows of a driver page's list, asleep ones included, in the list's
/// order: each row's list item and its artboard.
fn page_rows(list: &CoreHandle) -> Vec<(CoreHandle, CoreHandle)> {
    let rows = list
        .with_downcast::<ArtboardComponentList, _>(|list| {
            list.row_keys()
                .into_iter()
                .filter_map(|key| list.keyed_row(key))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    rows.into_iter()
        .map(|(item, board)| (item, board.core_handle()))
        .collect()
}
/// A row of a driver page as a step reads it: its elements by flight key
/// and its flight records, from its own instance. Two rows of one artboard
/// have the same keys, each on its own objects.
fn row_screen(board: &CoreHandle) -> Screen {
    let objects = board
        .with_downcast::<Artboard, _>(|b| b.objects().iter().flatten().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    Screen {
        board: board.clone(),
        elements: super::in_place::keyed_elements(&objects),
        records: super::in_place::flight_records(&objects),
    }
}
/// A row step needs a driver page: its rows are named by key.
fn driver_page(
    root: &RuntimeArtboardInstanceHandle,
) -> Result<std::rc::Rc<crate::hosted_scene::SceneState>> {
    let scene = crate::hosted_scene::root_scene(root)
        .ok_or_else(|| anyhow::anyhow!("root is not a live-move page"))?;
    ensure!(scene.page.keyed, "a row step needs a driver page");
    Ok(scene)
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
        refuse_driver_page(&scene)?;
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
        refuse_driver_page(&scene)?;
        self.flush_membership(root)?;
        scene.plan.borrow_mut().step_end(root, incoming, name)
    }
    /// Begins a nameless step on a driver page whose arriving end is the
    /// row of the page's list mounted by `incoming_key`, as a nameless
    /// `step_begin` does for a Screen: membership flushes, the row runs its
    /// own update pass, and the row must be mounted and awake.
    pub fn step_begin_row(
        &mut self,
        root: &RuntimeArtboardInstanceHandle,
        incoming_key: &[u8],
    ) -> Result<()> {
        // Every refusal comes before the flush, so it leaves the occurrence
        // usable: a driver's row operations commit once per round, so the
        // flush mounts and removes no row.
        let scene = driver_page(root)?;
        let list = scene.page.list.clone();
        ensure!(!scene.plan.borrow().step_running(), "step already active");
        let (_, board) = arriving_row(&list, incoming_key)?;
        self.flush_membership(root)?;
        Artboard::update_pass_handle(&board, false);
        scene
            .plan
            .borrow_mut()
            .step_begin_row(root, &list, incoming_key)
    }
    /// Lands the row step `step_begin_row` began; `incoming_key` must name
    /// the same row.
    pub fn step_end_row(
        &mut self,
        root: &RuntimeArtboardInstanceHandle,
        incoming_key: &[u8],
    ) -> Result<()> {
        let scene = driver_page(root)?;
        let list = scene.page.list.clone();
        scene.plan.borrow().holds_row(&list, incoming_key)?;
        self.flush_membership(root)?;
        scene
            .plan
            .borrow_mut()
            .step_end_row(root, &list, incoming_key)
    }
    /// Begins the step `name` between two rows of a driver page, the row of
    /// the page's list mounted by `outgoing_key` and the one mounted by
    /// `incoming_key`, as `step_begin` does between two Screens:
    /// membership flushes, the arriving row runs its own update pass, and
    /// the step takes off with its flight records read from the upper row.
    pub fn step_begin_between(
        &mut self,
        root: &RuntimeArtboardInstanceHandle,
        outgoing_key: &[u8],
        incoming_key: &[u8],
        name: &str,
    ) -> Result<()> {
        // Every refusal of the rows, the step and the rows' content
        // (`check_rows`) comes before the flush, as a row step's does, so it
        // leaves the occurrence usable. Only what reads layout is refused
        // after it, at take-off.
        let scene = driver_page(root)?;
        let list = scene.page.list.clone();
        let arriving = {
            let mut plan = scene.plan.borrow_mut();
            ensure!(!plan.step_running(), "step already active");
            plan.prepare(root)?;
            let between = plan.between(&list, outgoing_key, incoming_key, name)?;
            between.incoming.1.board
        };
        self.flush_membership(root)?;
        Artboard::update_pass_handle(&arriving, false);
        scene
            .plan
            .borrow_mut()
            .step_begin_between(root, &list, outgoing_key, incoming_key, name)
    }
    /// Lands the step `step_begin_between` began; both keys and the name
    /// must name it.
    pub fn step_end_between(
        &mut self,
        root: &RuntimeArtboardInstanceHandle,
        outgoing_key: &[u8],
        incoming_key: &[u8],
        name: &str,
    ) -> Result<()> {
        let scene = driver_page(root)?;
        let list = scene.page.list.clone();
        scene
            .plan
            .borrow()
            .holds_rows(&list, outgoing_key, incoming_key, name)?;
        self.flush_membership(root)?;
        scene
            .plan
            .borrow_mut()
            .step_end_between(root, &list, outgoing_key, incoming_key, name)
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
    /// The flight record of every flight of the active step, in placing
    /// order.
    pub fn flight_controls(&self, root: &RuntimeArtboardInstanceHandle) -> Vec<CoreHandle> {
        crate::hosted_scene::root_scene(root).map_or_else(Vec::new, |s| {
            s.plan
                .borrow()
                .lifted
                .iter()
                .map(|flight| flight.record.clone())
                .collect()
        })
    }
    /// The elements flying in place in the active step, each with its
    /// Screen.
    pub fn lifted_elements(
        &self,
        root: &RuntimeArtboardInstanceHandle,
    ) -> Vec<(CoreHandle, CoreHandle)> {
        crate::hosted_scene::root_scene(root).map_or_else(Vec::new, |s| {
            s.plan
                .borrow()
                .lifted
                .iter()
                .map(|flight| (flight.element.clone(), flight.row.clone()))
                .collect()
        })
    }
    /// What each zoomed pair of the step read when it started.
    #[cfg(feature = "test-support")]
    pub fn pair_ends(&self, root: &RuntimeArtboardInstanceHandle) -> Vec<zoom::Ends> {
        crate::hosted_scene::root_scene(root).map_or_else(Vec::new, |s| {
            let plan = s.plan.borrow();
            plan.pairs.iter().map(|pair| pair.ends()).collect()
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
    /// Where a component is drawn now: `drawn(component, Live)`, and for a
    /// part of a copy flying in place, through its lift. The lift draws the
    /// copy's entries under the upper Screen's page matrix and the lift's
    /// transform, where its own Screen's draw puts them under its own page
    /// matrix, so the part's draw is U · M · L⁻¹ · `drawn`.
    #[cfg(feature = "test-support")]
    pub fn drawn_now(
        &self,
        root: &RuntimeArtboardInstanceHandle,
        component: &CoreHandle,
    ) -> Result<crate::source::math::mat2d::Mat2D> {
        let own = super::drawn::drawn(component, super::When::Live)?;
        let Some(scene) = crate::hosted_scene::root_scene(root) else {
            return Ok(own);
        };
        let plan = scene.plan.borrow();
        let under = |element: &CoreHandle| {
            let mut node = Some(component.clone());
            while let Some(current) = node {
                if current == *element {
                    return true;
                }
                // An artboard's next level is its host, as the draw's.
                node = current
                    .with(|o| match o.as_artboard() {
                        Some(board) => board.host(),
                        None => o.component_parent_handle(),
                    })
                    .flatten();
            }
            false
        };
        let Some(flight) = plan.lifted.iter().find(|flight| under(&flight.element)) else {
            return Ok(own);
        };
        let matrix = flight.lift.0.borrow().matrix;
        let (upper, _, _) = super::follow::draw_pose(&flight.board)?;
        let (row, _, _) = super::follow::draw_pose(&flight.row)?;
        let mut row_inverse = crate::source::math::mat2d::Mat2D::identity();
        ensure!(row.invert(&mut row_inverse), "singular Screen matrix");
        Ok(upper * matrix * row_inverse * own)
    }
    /// Each copy flying in place in the active step: its element, its
    /// Screen and its flight record, in placing order.
    #[cfg(feature = "test-support")]
    pub fn lifted_records(
        &self,
        root: &RuntimeArtboardInstanceHandle,
    ) -> Vec<(CoreHandle, CoreHandle, CoreHandle)> {
        crate::hosted_scene::root_scene(root).map_or_else(Vec::new, |s| {
            s.plan
                .borrow()
                .lifted
                .iter()
                .map(|flight| {
                    (
                        flight.element.clone(),
                        flight.row.clone(),
                        flight.record.clone(),
                    )
                })
                .collect()
        })
    }
    /// Sets the clip of the lift of `element`, a copy flying in place, as a
    /// zoomed pair does: a box in the element's own content space.
    #[cfg(feature = "test-support")]
    pub fn set_lift_clip(
        &mut self,
        root: &RuntimeArtboardInstanceHandle,
        element: &CoreHandle,
        clip: Option<crate::source::artboard::FlightClip>,
    ) {
        let scene = crate::hosted_scene::root_scene(root).expect("live scene");
        let plan = scene.plan.borrow();
        let flight = plan
            .lifted
            .iter()
            .find(|flight| flight.element == *element)
            .expect("the element flies in place");
        flight.lift.0.borrow_mut().clip = clip;
    }
    /// The matrix placement gives a CSS box at rest, in its Screen's space,
    /// made by the same helpers placement's resting boxes use.
    #[cfg(feature = "test-support")]
    pub fn resting_box(&self, element: &CoreHandle) -> Option<crate::source::math::mat2d::Mat2D> {
        super::follow::resting_box(element)
    }
    #[cfg(feature = "test-support")]
    pub fn reset_for_test(&mut self, root: &RuntimeArtboardInstanceHandle) {
        let scene = crate::hosted_scene::root_scene(root).expect("live scene");
        *scene.plan.borrow_mut() = Plan::default();
    }
}
