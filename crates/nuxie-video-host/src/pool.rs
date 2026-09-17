//! Own a host's live video occurrences and reclaim before admitting replacements.
//! Budgets and decode capabilities come from the host; this pool does not infer
//! hardware acceleration from a renderer or from successful platform playback.
use nuxie_runtime::video::resources::{Allocation, DecoderBudget, DecoderRequest, allocate};
use std::collections::BTreeMap;

pub trait ManagedPlayer {
    type Error;
    fn allocation(&self) -> Allocation;
    fn owns_decoder(&self) -> bool;
    /// False after terminal failure/disposal, until an explicit source reset.
    fn can_decode(&self) -> bool {
        true
    }
    /// Success acknowledges native release. Errors must retain any unreleased
    /// resource ownership so the pool cannot silently reuse its slot.
    fn reclaim(&mut self) -> Result<(), Self::Error>;
}
impl<D: crate::scene::Decoder> ManagedPlayer for crate::scene::ScenePlayer<D> {
    type Error = crate::scene::SceneError<D::Error>;
    fn allocation(&self) -> Allocation {
        self.allocation()
    }
    fn owns_decoder(&self) -> bool {
        self.owns_decoder()
    }
    fn can_decode(&self) -> bool {
        self.can_decode()
    }
    fn reclaim(&mut self) -> Result<(), Self::Error> {
        self.reclaim()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MembershipError {
    DuplicateId,
    UnknownId,
    Capacity,
    AlreadyAllocated,
}
#[derive(Debug)]
pub struct ReclaimError<E> {
    pub id: u64,
    pub error: E,
}

struct Entry<P> {
    player: P,
    request: DecoderRequest,
}
pub struct PlayerPool<P> {
    entries: BTreeMap<u64, Entry<P>>,
}
impl<P> Default for PlayerPool<P> {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }
}
impl<P: ManagedPlayer> PlayerPool<P> {
    pub const MAX_PLAYERS: usize = 4096;
    pub fn active_decoders(&self) -> usize {
        self.entries
            .values()
            .filter(|entry| entry.player.owns_decoder())
            .count()
    }
    /// Ownership transfers only on success; an unsuccessful insert returns the
    /// player so the caller can keep or explicitly close it.
    pub fn insert(
        &mut self,
        request: DecoderRequest,
        player: P,
    ) -> Result<(), (MembershipError, P)> {
        if self.entries.contains_key(&request.id) {
            return Err((MembershipError::DuplicateId, player));
        }
        if self.entries.len() == Self::MAX_PLAYERS {
            return Err((MembershipError::Capacity, player));
        }
        if player.owns_decoder() || player.allocation() != Allocation::Poster {
            return Err((MembershipError::AlreadyAllocated, player));
        }
        self.entries.insert(request.id, Entry { player, request });
        Ok(())
    }
    pub fn update_request(&mut self, request: DecoderRequest) -> Result<(), MembershipError> {
        self.entries
            .get_mut(&request.id)
            .ok_or(MembershipError::UnknownId)?
            .request = request;
        Ok(())
    }
    pub fn get_mut(&mut self, id: u64) -> Option<&mut P> {
        self.entries.get_mut(&id).map(|entry| &mut entry.player)
    }
    /// Removing an occurrence closes it immediately; its decoder cannot escape
    /// the pool and compete with the next budget allocation.
    pub fn remove(&mut self, id: u64) -> Result<bool, ReclaimError<P::Error>> {
        let Some(entry) = self.entries.get_mut(&id) else {
            return Ok(false);
        };
        entry
            .player
            .reclaim()
            .map_err(|error| ReclaimError { id, error })?;
        self.entries.remove(&id);
        Ok(true)
    }
    /// Two phases prevent temporary oversubscription when priority changes.
    /// First release every changed allocation; only then invoke host ticks.
    /// A reclaim error aborts admission for this update. A decoder error from
    /// one tick is returned independently, allowing other occurrences to run.
    /// Host ticks must honor the supplied allocation and may not open additional
    /// decoders outside it. Software capability means selectable software decode,
    /// not a guess about what a platform-managed decoder might choose internally.
    pub fn tick<T>(
        &mut self,
        budget: DecoderBudget,
        mut tick: impl FnMut(u64, &mut P, Allocation) -> Result<T, P::Error>,
    ) -> Result<Vec<(u64, Result<T, P::Error>)>, ReclaimError<P::Error>> {
        let requests: Vec<_> = self
            .entries
            .values()
            .map(|entry| {
                let mut request = entry.request;
                request.visible &= entry.player.can_decode();
                request
            })
            .collect();
        let allocations = allocate(&requests, budget);
        for &(id, allocation) in &allocations {
            let entry = self
                .entries
                .get_mut(&id)
                .expect("allocation for registered request");
            if entry.player.allocation() != allocation {
                entry
                    .player
                    .reclaim()
                    .map_err(|error| ReclaimError { id, error })?;
            }
        }
        Ok(allocations
            .into_iter()
            .map(|(id, allocation)| {
                let entry = self
                    .entries
                    .get_mut(&id)
                    .expect("allocation for registered request");
                (id, tick(id, &mut entry.player, allocation))
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::Cell, rc::Rc};
    struct Player {
        allocation: Allocation,
        live: Rc<Cell<usize>>,
        fail_reclaim: bool,
    }
    impl Drop for Player {
        fn drop(&mut self) {
            let _ = self.reclaim();
        }
    }
    impl ManagedPlayer for Player {
        type Error = &'static str;
        fn allocation(&self) -> Allocation {
            self.allocation
        }
        fn owns_decoder(&self) -> bool {
            self.allocation != Allocation::Poster
        }
        fn reclaim(&mut self) -> Result<(), Self::Error> {
            if self.fail_reclaim {
                return Err("release failed");
            }
            if self.allocation != Allocation::Poster {
                self.live.set(self.live.get() - 1);
            }
            self.allocation = Allocation::Poster;
            Ok(())
        }
    }
    fn request(id: u64, priority: u32) -> DecoderRequest {
        DecoderRequest {
            id,
            priority,
            visible: true,
            hardware_supported: true,
            managed_supported: false,
            software_supported: false,
            pixels_per_second: 1280 * 720 * 30,
        }
    }
    fn tick(_: u64, p: &mut Player, allocation: Allocation) -> Result<Allocation, &'static str> {
        if p.allocation == Allocation::Poster && allocation != Allocation::Poster {
            p.live.set(p.live.get() + 1);
        }
        p.allocation = allocation;
        assert!(
            p.live.get() <= 1,
            "old decoder must close before higher priority admission"
        );
        Ok(allocation)
    }
    #[test]
    fn priority_visibility_and_budget_changes_never_oversubscribe() {
        let live = Rc::new(Cell::new(0));
        let mut pool = PlayerPool::default();
        for id in [1, 2] {
            assert!(
                pool.insert(
                    request(id, 0),
                    Player {
                        allocation: Allocation::Poster,
                        live: live.clone(),
                        fail_reclaim: false
                    }
                )
                .is_ok()
            );
        }
        let budget = DecoderBudget {
            max_players: 2,
            managed_players: 0,
            managed_pixels_per_second: 0,
            hardware_players: 1,
            software_pixels_per_second: 0,
        };
        let result = pool.tick(budget, tick).unwrap();
        assert_eq!(
            result,
            vec![(1, Ok(Allocation::Hardware)), (2, Ok(Allocation::Poster))]
        );
        pool.update_request(request(2, 100)).unwrap();
        let result = pool.tick(budget, tick).unwrap();
        assert_eq!(
            result,
            vec![(2, Ok(Allocation::Hardware)), (1, Ok(Allocation::Poster))]
        );
        let mut hidden = request(2, 100);
        hidden.visible = false;
        pool.update_request(hidden).unwrap();
        pool.tick(budget, tick).unwrap();
        assert_eq!(pool.get_mut(1).unwrap().allocation(), Allocation::Hardware);
        pool.tick(
            DecoderBudget {
                max_players: 0,
                managed_players: 0,
                managed_pixels_per_second: 0,
                hardware_players: 0,
                software_pixels_per_second: 0,
            },
            tick,
        )
        .unwrap();
        assert_eq!(live.get(), 0);
        pool.tick(budget, tick).unwrap();
        assert_eq!(live.get(), 1);
        assert!(pool.remove(1).unwrap());
        assert_eq!(live.get(), 0);
    }
    #[test]
    fn terminal_player_does_not_reserve_capacity_from_healthy_players() {
        struct TerminalPlayer {
            inner: Player,
            failed: bool,
        }
        impl ManagedPlayer for TerminalPlayer {
            type Error = &'static str;
            fn allocation(&self) -> Allocation {
                self.inner.allocation()
            }
            fn owns_decoder(&self) -> bool {
                self.inner.owns_decoder()
            }
            fn can_decode(&self) -> bool {
                !self.failed
            }
            fn reclaim(&mut self) -> Result<(), Self::Error> {
                self.inner.reclaim()
            }
        }
        let live = Rc::new(Cell::new(0));
        let mut pool = PlayerPool::default();
        for id in [1, 2] {
            assert!(
                pool.insert(
                    request(id, if id == 1 { 10 } else { 0 }),
                    TerminalPlayer {
                        inner: Player {
                            allocation: Allocation::Poster,
                            live: live.clone(),
                            fail_reclaim: false
                        },
                        failed: false,
                    }
                )
                .is_ok()
            );
        }
        let budget = DecoderBudget {
            max_players: 2,
            managed_players: 0,
            managed_pixels_per_second: 0,
            hardware_players: 1,
            software_pixels_per_second: 0,
        };
        pool.tick(budget, |id, p, a| tick(id, &mut p.inner, a))
            .unwrap();
        assert_eq!(pool.get_mut(1).unwrap().allocation(), Allocation::Hardware);
        pool.get_mut(1).unwrap().failed = true;
        pool.tick(budget, |id, p, a| tick(id, &mut p.inner, a))
            .unwrap();
        assert_eq!(pool.get_mut(1).unwrap().allocation(), Allocation::Poster);
        assert_eq!(pool.get_mut(2).unwrap().allocation(), Allocation::Hardware);
        assert_eq!(live.get(), 1);
        // A source reset explicitly makes the high-priority occurrence eligible again.
        pool.get_mut(1).unwrap().failed = false;
        pool.tick(budget, |id, p, a| tick(id, &mut p.inner, a))
            .unwrap();
        assert_eq!(pool.get_mut(1).unwrap().allocation(), Allocation::Hardware);
        assert_eq!(pool.get_mut(2).unwrap().allocation(), Allocation::Poster);
    }
    #[test]
    fn failed_reclaim_aborts_new_admissions() {
        let live = Rc::new(Cell::new(0));
        let mut pool = PlayerPool::default();
        assert!(
            pool.insert(
                request(1, 0),
                Player {
                    allocation: Allocation::Poster,
                    live: live.clone(),
                    fail_reclaim: false
                }
            )
            .is_ok()
        );
        pool.tick(
            DecoderBudget {
                max_players: 2,
                managed_players: 0,
                managed_pixels_per_second: 0,
                hardware_players: 1,
                software_pixels_per_second: 0,
            },
            tick,
        )
        .unwrap();
        pool.get_mut(1).unwrap().fail_reclaim = true;
        let error = pool
            .tick::<()>(
                DecoderBudget {
                    max_players: 0,
                    managed_players: 0,
                    managed_pixels_per_second: 0,
                    hardware_players: 0,
                    software_pixels_per_second: 0,
                },
                |_, _, _| panic!("admission after failed reclamation"),
            )
            .unwrap_err();
        assert_eq!(error.id, 1);
        assert_eq!(live.get(), 1);
        assert_eq!(pool.remove(1).unwrap_err().id, 1);
        assert_eq!(live.get(), 1);
        pool.get_mut(1).unwrap().fail_reclaim = false;
        assert!(pool.remove(1).unwrap());
        assert_eq!(live.get(), 0);
        assert!(!pool.remove(1).unwrap());
    }
}
