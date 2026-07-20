//! Inline port of `luau/tests/VecDeque.test.cpp` (`TEST_SUITE("VecDequeTests")`).
//! Tests Luau's custom ring-buffer `VecDeque`: exact capacity growth, front/back
//! queues, random access (`at`/index), contiguity, shrink-to-fit, and clone.
//!
//! Adaptations to Rust: C++ distinguishes copy-construction (preserves the raw
//! buffer layout) from copy-assignment (which may normalize to contiguous). Rust
//! has a single `Clone` that preserves the layout (see the impl), so both clones
//! stay discontiguous — these tests assert Rust's actual clone behavior. C++
//! move (which empties the source) becomes a Rust move (the source is consumed,
//! so post-move source checks are inapplicable and omitted).

#![cfg(test)]

use alloc::rc::Rc;

use crate::records::vec_deque::VecDeque;

const SSO: [&str; 10] = [
    "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
];
const LONG: [&str; 10] = [
    "Love doesn't just sit there, like a stone, it has to be made, like bread; remade all the time, made new.",
    "People who deny the existence of dragons are often eaten by dragons. From within.",
    "It is good to have an end to journey toward; but it is the journey that matters, in the end.",
    "We're each of us alone, to be sure. What can you do but hold your hand out in the dark?",
    "When you light a candle, you also cast a shadow.",
    "You cannot buy the revolution. You cannot make the revolution. You can only be the revolution. It is in your spirit, or it is nowhere.",
    "To learn which questions are unanswerable, and not to answer them: this skill is most needful in times of stress and darkness.",
    "What sane person could live in this world and not be crazy?",
    "The only thing that makes life possible is permanent, intolerable uncertainty: not knowing what comes next.",
    "My imagination makes me human and makes me a fool; it gives me all the world and exiles me from it.",
];

fn string_sets() -> [[String; 10]; 2] {
    [SSO.map(String::from), LONG.map(String::from)]
}

// ---- int queues ----

#[test]
fn forward_queue_test_no_initial_capacity() {
    let mut queue: VecDeque<i32> = VecDeque::new();
    assert!(queue.empty());
    for i in 0..10 {
        queue.push_back(i);
    }
    assert!(!queue.empty());
    assert_eq!(queue.size(), 10);
    assert_eq!(queue.capacity(), 11);
    for j in 0..10 {
        assert_eq!(*queue.front(), j);
        assert_eq!(*queue.back(), 9);
        assert!(!queue.empty());
        queue.pop_front();
    }
}

#[test]
fn forward_queue_test() {
    let mut queue: VecDeque<i32> = VecDeque::new();
    queue.reserve(5);
    assert!(queue.empty());
    for i in 0..10 {
        queue.push_back(i);
    }
    assert_eq!(queue.size(), 10);
    assert_eq!(queue.capacity(), 13);
    for j in 0..10 {
        assert_eq!(*queue.front(), j);
        assert_eq!(*queue.back(), 9);
        queue.pop_front();
    }
}

#[test]
fn forward_queue_test_initializer_list() {
    let mut queue = VecDeque::from_init_list(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    assert!(!queue.empty());
    assert_eq!(queue.size(), 10);
    assert_eq!(queue.capacity(), 10);
    for j in 0..10 {
        assert_eq!(*queue.front(), j);
        assert_eq!(*queue.back(), 9);
        queue.pop_front();
    }
}

#[test]
fn reverse_queue_test() {
    let mut queue: VecDeque<i32> = VecDeque::new();
    queue.reserve(5);
    for i in 0..10 {
        queue.push_front(i);
    }
    assert_eq!(queue.size(), 10);
    assert_eq!(queue.capacity(), 13);
    for j in 0..10 {
        assert_eq!(*queue.front(), 9);
        assert_eq!(*queue.back(), j);
        queue.pop_back();
    }
}

#[test]
fn random_access_queue_test() {
    let mut queue: VecDeque<i32> = VecDeque::new();
    queue.reserve(5);
    for i in 0..10 {
        queue.push_back(i);
    }
    assert_eq!(queue.size(), 10);
    for j in 0..10usize {
        assert_eq!(*queue.at(j), j as i32);
        assert_eq!(*queue.operator_index(j), j as i32);
    }
}

#[test]
fn clear_works_on_queue() {
    let mut queue: VecDeque<i32> = VecDeque::new();
    queue.reserve(5);
    for i in 0..10 {
        queue.push_back(i);
    }
    assert_eq!(queue.size(), 10);
    for j in 0..10usize {
        assert_eq!(*queue.operator_index(j), j as i32);
    }
    queue.clear();
    assert!(queue.empty());
    assert_eq!(queue.size(), 0);
}

#[test]
fn pop_front_at_end() {
    let mut queue: VecDeque<i32> = VecDeque::new();
    queue.reserve(5);
    queue.push_front(0);
    for i in 1..10 {
        queue.push_back(i);
    }
    assert_eq!(queue.size(), 10);
    for j in 0..10 {
        assert_eq!(*queue.front(), j);
        assert_eq!(*queue.back(), 9);
        queue.pop_front();
    }
}

#[test]
fn pop_back_at_front() {
    let mut queue: VecDeque<i32> = VecDeque::new();
    queue.reserve(5);
    queue.push_back(0);
    for i in 1..10 {
        queue.push_front(i);
    }
    assert_eq!(queue.size(), 10);
    for j in 0..10 {
        assert_eq!(*queue.front(), 9);
        assert_eq!(*queue.back(), j);
        queue.pop_back();
    }
}

#[test]
fn queue_is_contiguous() {
    let mut queue: VecDeque<i32> = VecDeque::new();
    for i in 0..10 {
        queue.push_back(i);
    }
    assert_eq!(queue.size(), 10);
    assert_eq!(queue.capacity(), 11);
    assert!(queue.is_contiguous());
}

#[test]
fn queue_is_not_contiguous() {
    let mut queue: VecDeque<i32> = VecDeque::new();
    for i in 5..10 {
        queue.push_back(i);
    }
    for i in (0..5).rev() {
        queue.push_front(i);
    }
    assert_eq!(queue.size(), 10);
    assert_eq!(queue.capacity(), 11);
    assert!(!queue.is_contiguous());
    for j in 0..10usize {
        assert_eq!(*queue.operator_index(j), j as i32);
    }
}

#[test]
fn shrink_to_fit_works() {
    let mut queue: VecDeque<i32> = VecDeque::new();
    for i in 5..10 {
        queue.push_back(i);
    }
    for i in (0..5).rev() {
        queue.push_front(i);
    }
    assert_eq!(queue.size(), 10);
    assert_eq!(queue.capacity(), 11);
    assert!(!queue.is_contiguous());
    for j in 0..10usize {
        assert_eq!(*queue.operator_index(j), j as i32);
    }
    queue.shrink_to_fit();
    assert!(queue.is_contiguous());
    assert_eq!(queue.capacity(), queue.size());
    for j in 0..10usize {
        assert_eq!(*queue.operator_index(j), j as i32);
    }
}

// ---- string queues (run over both the SSO and long-string sets) ----

#[test]
fn string_queue_test_no_initial_capacity() {
    for ts in string_sets() {
        let mut queue: VecDeque<String> = VecDeque::new();
        for i in 0..10 {
            queue.push_back(ts[i].clone());
        }
        assert_eq!(queue.size(), 10);
        assert_eq!(queue.capacity(), 11);
        for j in 0..10 {
            assert_eq!(*queue.front(), ts[j]);
            assert_eq!(*queue.back(), ts[9]);
            queue.pop_front();
        }
    }
}

#[test]
fn string_queue_test() {
    for ts in string_sets() {
        let mut queue: VecDeque<String> = VecDeque::new();
        queue.reserve(5);
        for i in 0..10 {
            queue.push_back(ts[i].clone());
        }
        assert_eq!(queue.size(), 10);
        assert_eq!(queue.capacity(), 13);
        for j in 0..10 {
            assert_eq!(*queue.front(), ts[j]);
            assert_eq!(*queue.back(), ts[9]);
            queue.pop_front();
        }
    }
}

#[test]
fn string_queue_test_initializer_list() {
    for ts in string_sets() {
        let mut queue = VecDeque::from_init_list(ts.to_vec());
        assert_eq!(queue.size(), 10);
        assert_eq!(queue.capacity(), 10);
        for j in 0..10 {
            assert_eq!(*queue.front(), ts[j]);
            assert_eq!(*queue.back(), ts[9]);
            queue.pop_front();
        }
    }
}

#[test]
fn reverse_string_queue_test() {
    for ts in string_sets() {
        let mut queue: VecDeque<String> = VecDeque::new();
        queue.reserve(5);
        for i in 0..10 {
            queue.push_front(ts[i].clone());
        }
        assert_eq!(queue.size(), 10);
        assert_eq!(queue.capacity(), 13);
        for j in 0..10 {
            assert_eq!(*queue.front(), ts[9]);
            assert_eq!(*queue.back(), ts[j]);
            queue.pop_back();
        }
    }
}

#[test]
fn random_access_string_queue_test() {
    for ts in string_sets() {
        let mut queue: VecDeque<String> = VecDeque::new();
        queue.reserve(5);
        for i in 0..10 {
            queue.push_back(ts[i].clone());
        }
        for j in 0..10usize {
            assert_eq!(*queue.at(j), ts[j]);
            assert_eq!(*queue.operator_index(j), ts[j]);
        }
    }
}

#[test]
fn clear_works_on_string_queue() {
    for ts in string_sets() {
        let mut queue: VecDeque<String> = VecDeque::new();
        queue.reserve(5);
        for i in 0..10 {
            queue.push_back(ts[i].clone());
        }
        for j in 0..10usize {
            assert_eq!(*queue.operator_index(j), ts[j]);
        }
        queue.clear();
        assert!(queue.empty());
        assert_eq!(queue.size(), 0);
    }
}

#[test]
fn pop_front_string_at_end() {
    for ts in string_sets() {
        let mut queue: VecDeque<String> = VecDeque::new();
        queue.reserve(5);
        queue.push_front(ts[0].clone());
        for i in 1..10 {
            queue.push_back(ts[i].clone());
        }
        assert_eq!(queue.size(), 10);
        for j in 0..10 {
            assert_eq!(*queue.front(), ts[j]);
            assert_eq!(*queue.back(), ts[9]);
            queue.pop_front();
        }
    }
}

#[test]
fn pop_back_string_at_front() {
    for ts in string_sets() {
        let mut queue: VecDeque<String> = VecDeque::new();
        queue.reserve(5);
        queue.push_back(ts[0].clone());
        for i in 1..10 {
            queue.push_front(ts[i].clone());
        }
        assert_eq!(queue.size(), 10);
        for j in 0..10 {
            assert_eq!(*queue.front(), ts[9]);
            assert_eq!(*queue.back(), ts[j]);
            queue.pop_back();
        }
    }
}

#[test]
fn string_queue_is_contiguous() {
    for ts in string_sets() {
        let mut queue: VecDeque<String> = VecDeque::new();
        for i in 0..10 {
            queue.push_back(ts[i].clone());
        }
        assert_eq!(queue.size(), 10);
        assert_eq!(queue.capacity(), 11);
        assert!(queue.is_contiguous());
        for j in 0..10usize {
            assert_eq!(*queue.operator_index(j), ts[j]);
        }

        // Clone preserves layout + capacity (C++ copy construction).
        let queue2 = queue.clone();
        assert_eq!(queue2.size(), 10);
        assert_eq!(queue2.capacity(), 11);
        assert!(queue2.is_contiguous());
        for j in 0..10usize {
            assert_eq!(*queue2.operator_index(j), ts[j]);
        }

        // Move (C++ move construction); source is consumed.
        let queue4 = queue2;
        assert_eq!(queue4.size(), 10);
        assert_eq!(queue4.capacity(), 11);
        assert!(queue4.is_contiguous());
        for j in 0..10usize {
            assert_eq!(*queue4.operator_index(j), ts[j]);
        }
    }
}

#[test]
fn string_queue_is_not_contiguous() {
    for ts in string_sets() {
        let mut queue: VecDeque<String> = VecDeque::new();
        for i in 5..10 {
            queue.push_back(ts[i].clone());
        }
        for i in (0..5).rev() {
            queue.push_front(ts[i].clone());
        }
        assert_eq!(queue.size(), 10);
        assert_eq!(queue.capacity(), 11);
        assert!(!queue.is_contiguous());
        for j in 0..10usize {
            assert_eq!(*queue.operator_index(j), ts[j]);
        }

        // Rust clone preserves the discontiguous layout (unlike C++ copy-
        // assignment, which normalizes — Rust has only one Clone).
        let queue2 = queue.clone();
        assert!(!queue2.is_contiguous());
        for j in 0..10usize {
            assert_eq!(*queue2.operator_index(j), ts[j]);
        }

        // Move from discontiguous, then grow — must stay correct.
        let mut queue4 = queue;
        assert!(!queue4.is_contiguous());
        queue4.push_back(String::from("zero"));
        queue4.push_back(String::from("?"));
        for j in 0..10usize {
            assert_eq!(*queue4.operator_index(j), ts[j]);
        }
        assert_eq!(*queue4.operator_index(10), "zero");
        assert_eq!(*queue4.operator_index(11), "?");

        // Reserve from discontiguous — must stay correct.
        let mut queue5 = queue2;
        queue5.reserve(20);
        for j in 0..10usize {
            assert_eq!(*queue5.operator_index(j), ts[j]);
        }
    }
}

#[test]
fn shrink_to_fit_works_with_strings() {
    for ts in string_sets() {
        let mut queue: VecDeque<String> = VecDeque::new();
        for i in 5..10 {
            queue.push_back(ts[i].clone());
        }
        for i in (0..5).rev() {
            queue.push_front(ts[i].clone());
        }
        assert_eq!(queue.size(), 10);
        assert_eq!(queue.capacity(), 11);
        assert!(!queue.is_contiguous());
        for j in 0..10usize {
            assert_eq!(*queue.operator_index(j), ts[j]);
        }
        queue.shrink_to_fit();
        assert!(queue.is_contiguous());
        assert_eq!(queue.capacity(), queue.size());
        for j in 0..10usize {
            assert_eq!(*queue.operator_index(j), ts[j]);
        }
    }
}

struct TestStruct;

#[test]
fn push_front_elements_are_destroyed_correctly() {
    let t = Rc::new(TestStruct);
    {
        let mut queue: VecDeque<Rc<TestStruct>> = VecDeque::new();
        queue.reserve(10);
        queue.push_front(t.clone());
        queue.push_front(t.clone());
        assert_eq!(Rc::strong_count(&t), 3);

        let _queue2 = queue.clone();
        let _queue3 = queue.clone();
        // queue, _queue2, _queue3 all dropped at scope end.
    }
    assert_eq!(Rc::strong_count(&t), 1);
}
