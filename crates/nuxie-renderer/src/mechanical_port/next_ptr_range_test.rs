//! tests/unit_tests/renderer/next_ptr_range_test.cpp at 51b35dc1.
use crate::mechanical_port::source::renderer::include::rive::renderer::{
    next_ptr_range_hpp::{NextPtrNode, NextPtrNodeMut, NextPtrRange, NextPtrRangeMut},
    render_context_hpp::BlockAllocatedLinkedList,
};
use core::ptr::{self, NonNull};

#[derive(Debug)]
struct Node {
    value: i32,
    next: Option<NonNull<Node>>,
}
impl NextPtrNode for Node {
    fn next_ptr(&self) -> Option<NonNull<Self>> {
        self.next
    }
}
#[derive(Debug)]
struct ConstLinkedNode {
    value: i32,
    next: *const ConstLinkedNode,
}
impl NextPtrNode for ConstLinkedNode {
    fn next_ptr(&self) -> Option<NonNull<Self>> {
        NonNull::new(self.next.cast_mut())
    }
}
impl NextPtrNodeMut for ConstLinkedNode {
    fn set_next_ptr(&mut self, next: Option<NonNull<Self>>) {
        self.next = next.map_or(ptr::null(), |n| n.as_ptr());
    }
}

fn node(value: i32, next: Option<NonNull<Node>>) -> Node {
    Node { value, next }
}
fn values(range: NextPtrRange<'_, Node>) -> Vec<i32> {
    range.into_iter().map(|n| n.value).collect()
}

#[test]
fn empty() {
    // SAFETY: the empty chain contains no pointers to dereference.
    let range = unsafe { NextPtrRange::<Node>::new(ptr::null(), ptr::null()) };
    assert!(range.begin() == range.end());
    assert!(values(range).is_empty());
}
#[test]
fn walks_every_node_in_order() {
    let mut c = node(3, None);
    let mut b = node(2, Some(NonNull::from(&mut c)));
    let a = node(1, Some(NonNull::from(&mut b)));
    // SAFETY: all three stack nodes remain live and unmodified during iteration.
    assert_eq!(
        values(unsafe { NextPtrRange::new(&a, ptr::null()) }),
        vec![1, 2, 3]
    );
    let const_a: *const Node = &a;
    assert_eq!(
        values(unsafe { NextPtrRange::new(const_a, ptr::null()) }),
        vec![1, 2, 3]
    );
}
#[test]
fn stops_before_non_null_end() {
    let mut d = node(4, None);
    let mut c = node(3, Some(NonNull::from(&mut d)));
    let mut b = node(2, Some(NonNull::from(&mut c)));
    let a = node(1, Some(NonNull::from(&mut b)));
    // SAFETY: each end is reachable in the live stack-node chain.
    assert_eq!(values(unsafe { NextPtrRange::new(&a, &c) }), vec![1, 2]);
    assert_eq!(values(unsafe { NextPtrRange::new(&b, &c) }), vec![2]);
    assert!(values(unsafe { NextPtrRange::new(&c, &c) }).is_empty());
}
#[test]
fn const_next_pointers() {
    let b = ConstLinkedNode {
        value: 2,
        next: ptr::null(),
    };
    let a = ConstLinkedNode { value: 1, next: &b };
    // SAFETY: nodes are retained for the full immutable traversal.
    let values: Vec<_> = unsafe { NextPtrRange::new(&a, ptr::null()) }
        .into_iter()
        .map(|n| n.value)
        .collect();
    assert_eq!(values, vec![1, 2]);
}
#[test]
fn writes_through_mutable_range() {
    let mut c = node(3, None);
    let mut b = node(2, Some(NonNull::from(&mut c)));
    let mut a = node(1, Some(NonNull::from(&mut b)));
    // SAFETY: this finite, acyclic chain is exclusively accessed by this loop.
    for n in unsafe { NextPtrRangeMut::new(&mut a, ptr::null_mut()) } {
        n.value *= 10;
    }
    assert_eq!(
        values(unsafe { NextPtrRange::new(&a, ptr::null()) }),
        vec![10, 20, 30]
    );
}
#[test]
fn block_allocated_linked_list() {
    // The existing Rust list owns stable boxed nodes instead of borrowing an
    // external TrivialBlockAllocator. Exercise its real iteration owner.
    let mut list = BlockAllocatedLinkedList::<ConstLinkedNode>::default();
    // SAFETY: the empty list has no linked nodes to dereference.
    assert!(
        unsafe { list.iter() }
            .map(|n| n.value)
            .collect::<Vec<_>>()
            .is_empty()
    );
    for value in 1..=3 {
        list.push_back(ConstLinkedNode {
            value,
            next: ptr::null(),
        });
    }
    // SAFETY: the list owns all nodes, linked by push_back, and remains immutable during traversal.
    assert_eq!(
        unsafe { list.iter() }.map(|n| n.value).collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    let const_list = &list;
    // SAFETY: the shared list reference keeps its owned finite node chain live.
    assert_eq!(
        unsafe { const_list.iter() }
            .map(|n| n.value)
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
}
