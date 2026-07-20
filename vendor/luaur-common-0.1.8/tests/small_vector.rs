//! Oracle validation for the hand-ported `SmallVector<T, N>`: differential fuzz
//! against `Vec<T>` (the ground-truth semantics) crossing the inline→heap
//! boundary, plus a drop-accounting check that every element is destructed
//! exactly once. Run under Miri (`cargo +nightly miri test`) to also exercise the
//! unsafe pointer/alloc paths for UB.

use std::cell::Cell;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use luaur_common::records::small_vector::SmallVector;

/// Tiny xorshift so the fuzz is deterministic without a dependency.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

fn hash_of<H: Hash>(value: &H) -> u64 {
    let mut h = DefaultHasher::new();
    value.hash(&mut h);
    h.finish()
}

#[test]
fn differential_against_vec_crossing_sbo_boundary() {
    // N = 4 so a few pushes already spill to the heap, exercising `grow`.
    let mut rng = Rng(0x1234_5678_9abc_def0);
    for _ in 0..2_000 {
        let mut sv: SmallVector<i64, 4> = SmallVector::new();
        let mut model: Vec<i64> = Vec::new();

        for _ in 0..rng.below(40) {
            match rng.below(5) {
                0 | 1 => {
                    let v = rng.next() as i64;
                    sv.push_back(v);
                    model.push(v);
                }
                2 => {
                    if !model.is_empty() {
                        sv.pop_back();
                        model.pop();
                    }
                }
                3 => {
                    let n = rng.below(12) as u32;
                    sv.resize(n);
                    model.resize(n as usize, 0);
                }
                _ => {
                    let n = rng.below(12) as u32;
                    sv.reserve(n);
                }
            }

            assert_eq!(sv.size() as usize, model.len());
            assert_eq!(sv.as_slice(), model.as_slice());
            assert!(sv.capacity() as usize >= model.len());
            if !model.is_empty() {
                assert_eq!(sv.front(), &model[0]);
                assert_eq!(sv.back(), model.last().unwrap());
                let i = rng.below(model.len() as u64) as usize;
                assert_eq!(&sv[i], &model[i]); // Index via Deref<[T]>
            }
        }

        // Trait surface the Bytecode consumers rely on.
        let clone = sv.clone();
        assert_eq!(clone, sv); // PartialEq
        assert_eq!(hash_of(&clone), hash_of(&sv)); // Hash agrees with equality
        assert_eq!(sv.iter().copied().collect::<Vec<_>>(), model); // IntoIter
    }
}

#[test]
fn upstream_basic_push_index_clear() {
    let mut sv: SmallVector<u32, 2> = SmallVector::new();
    assert!(sv.empty());
    for v in 0..10 {
        sv.push_back(v);
    }
    assert_eq!(sv.size(), 10);
    assert!(sv.capacity() >= 10);
    for v in 0..10u32 {
        assert_eq!(sv[v as usize], v);
    }
    sv.clear();
    assert!(sv.empty());
}

#[test]
fn drops_every_element_exactly_once() {
    // Rc strong-count proves no leak (would stay >1) and no double-drop (would
    // panic on underflow), across the heap-growth element moves.
    let witness = Rc::new(Cell::new(0));
    {
        let mut sv: SmallVector<Rc<Cell<i32>>, 3> = SmallVector::new();
        for _ in 0..50 {
            sv.push_back(Rc::clone(&witness));
        }
        assert_eq!(Rc::strong_count(&witness), 51);
        for _ in 0..10 {
            sv.pop_back();
        }
        assert_eq!(Rc::strong_count(&witness), 41);
        // remaining 40 dropped here when `sv` falls out of scope
    }
    assert_eq!(Rc::strong_count(&witness), 1);
}

#[test]
fn from_iter_and_equality() {
    let a: SmallVector<i32, 4> = (0..7).collect();
    let b: SmallVector<i32, 4> = (0..7).collect();
    let c: SmallVector<i32, 4> = (0..6).collect();
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_eq!(a.as_slice(), &[0, 1, 2, 3, 4, 5, 6]);
}
