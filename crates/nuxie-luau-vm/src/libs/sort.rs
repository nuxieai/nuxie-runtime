//! `table.sort` (C++ `ltablib.cpp` `tsort`), phase 2 lane 4: the same
//! introsort as an explicit state machine, so a comparison that calls Lua
//! (the predicate, or `__lt`) is a frame (`call.rs`) instead of a nested
//! Rust call.
//!
//! The comparison order is C++'s exactly, so a script observes the same
//! predicate calls: quicksort with a median-of-three pivot, falling back to
//! heapsort past `1.5 log2 n` partitions. C++ recurses into the smaller
//! side and loops on the larger; here the larger side waits on `pending`
//! while the smaller one is sorted, which visits the same ranges in the
//! same order. `pending` holds at most `log2 n + 1` ranges.

use crate::call::{CallMode, Continuation, Outcome, Resume};
use crate::error::VmResult;
use crate::meta::Meta;
use crate::native::Args;
use crate::value::{TableId, Value};
use crate::vm::{MIN_STACK, Core};

pub(crate) fn sort(vm: &mut Resume<'_>, args: Args) -> VmResult<usize> {
    let t = vm.check_table(args, 1)?;
    let n = vm.heap.table(t).len();
    if vm.heap.table(t).readonly {
        return Err(vm.runtime_error("attempt to modify a readonly table"));
    }
    let has_pred = match vm.arg(args, 2) {
        None | Some(Value::Nil) => false,
        Some(Value::Function(_)) => true,
        Some(_) => return Err(vm.arg_type_error(args, 2, "function")),
    };
    if n == 0 {
        return Ok(0);
    }
    // C++ reads the border's elements straight from the array part; a border
    // that reaches into the hash part (C++ asserts there) moves them over.
    if vm.heap.table(t).array().len() < n {
        vm.resize_array(t, n)?;
    }
    let sorter = Sorter {
        t,
        size: vm.heap.table(t).array().len(),
        has_pred,
        at: At::Loop,
        l: 0,
        u: n.saturating_sub(1),
        limit: n,
        m: 0,
        i: 0,
        j: 0,
        p: 0,
        pending: Vec::new(),
        heap_root: None,
        heap_i: 0,
        sift: Sift::default(),
    };
    sorter.run(vm, args, None)
}

/// Where the sort is: the point after each comparison, named for the
/// comparison it waits on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum At {
    /// The top of `sort_rec`'s loop for `l..=u`.
    Loop,
    /// `a[u] < a[l]`.
    LessUL,
    /// `a[m] < a[l]`.
    LessML,
    /// `a[u] < a[m]`.
    LessUM,
    /// Advance `i`, then `a[i] < a[p]` (`LessIP`).
    ScanI,
    LessIP,
    /// Retreat `j`, then `a[p] < a[j]` (`LessPJ`).
    ScanJ,
    LessPJ,
    /// The next range, if any.
    Pop,
    /// `sort_heap`'s two loops.
    HeapBuild,
    HeapSort,
    HeapSortNext,
    /// `sort_siftheap`.
    Sift,
    SiftLeft,
    SiftRight,
    SiftTail,
    SiftLast,
}

/// `sort_siftheap`'s locals, and where it returns.
#[derive(Clone, Copy, Debug)]
struct Sift {
    l: usize,
    count: usize,
    root: usize,
    next: usize,
    ret: At,
}

impl Default for Sift {
    fn default() -> Sift {
        Sift {
            l: 0,
            count: 0,
            root: 0,
            next: 0,
            ret: At::Pop,
        }
    }
}

/// The sort's state. The table and the predicate stay in argument slots 1
/// and 2, so they are rooted; this holds no values.
struct Sorter {
    t: TableId,
    /// The array size when sorting started; a predicate that resizes the
    /// table is an error.
    size: usize,
    has_pred: bool,
    at: At,
    l: usize,
    u: usize,
    limit: usize,
    m: usize,
    i: usize,
    j: usize,
    p: usize,
    /// Ranges (`l`, `u`, `limit`) still to sort.
    pending: Vec<(usize, usize, usize)>,
    heap_root: Option<usize>,
    heap_i: usize,
    sift: Sift,
}

/// The sort waiting on a comparison's call, whose result is at `at`.
struct SortCont {
    sorter: Sorter,
    at: usize,
}

impl Continuation for SortCont {
    fn resume(self: Box<Self>, vm: &mut Resume<'_>, args: Args, _: Outcome) -> VmResult<usize> {
        let SortCont { sorter, at } = *self;
        let answer = vm.get(at).truthy();
        vm.top = at;
        let answer = sorter.answered(vm, answer)?;
        sorter.run(vm, args, Some(answer))
    }
}

impl Sorter {
    fn element(&self, vm: &mut Core, i: usize) -> VmResult<Value> {
        match vm.heap.table(self.t).array().get(i) {
            Some(&v) => Ok(v),
            None => Err(vm.lib_error("table modified during sorting")),
        }
    }

    /// C++ `sort_less`'s check after the comparison.
    fn answered(&self, vm: &mut Core, answer: bool) -> VmResult<bool> {
        if vm.heap.table(self.t).array().len() != self.size {
            return Err(vm.lib_error("table modified during sorting"));
        }
        Ok(answer)
    }

    fn swap(&self, vm: &mut Core, i: usize, j: usize) {
        let array = vm.heap.table_mut(self.t).array_mut();
        if i < array.len() && j < array.len() {
            array.swap(i, j);
        }
    }

    /// C++ `sort_less`: `a[i] < a[j]` now, or `None` when the comparison
    /// is a call, pushed at the returned slot.
    fn less(&self, vm: &mut Core, args: Args, i: usize, j: usize) -> VmResult<Result<bool, usize>> {
        let a = self.element(vm, i)?;
        let b = self.element(vm, j)?;
        let (f, x, y) = if self.has_pred {
            (vm.arg_or_nil(args, 2), a, b)
        } else {
            match vm.less_than_step(a, b)? {
                Meta::Done(v) => return self.answered(vm, v.truthy()).map(Ok),
                Meta::Call(call) => {
                    let [x, y, _] = call.args;
                    (call.f, x, y)
                }
            }
        };
        let at = vm.top;
        vm.ensure_stack(at.saturating_add(3).saturating_add(MIN_STACK))?;
        vm.set(at, f);
        vm.set(at.saturating_add(1), x);
        vm.set(at.saturating_add(2), y);
        vm.top = at.saturating_add(3);
        Ok(Err(at))
    }

    /// Runs from `self.at`, `answer` being the result of the comparison it
    /// waits on, until the sort is done or needs a call.
    fn run(mut self, vm: &mut Resume<'_>, args: Args, mut answer: Option<bool>) -> VmResult<usize> {
        // Asks `a[i] < a[j]` and continues at `next` with the answer.
        macro_rules! ask {
            ($i:expr, $j:expr, $next:expr) => {{
                self.at = $next;
                match self.less(vm, args, $i, $j)? {
                    Ok(result) => {
                        answer = Some(result);
                        continue;
                    }
                    Err(at) => {
                        // C++ `lua_call`: counted, not yieldable.
                        let cont = Box::new(SortCont { sorter: self, at });
                        return vm.request_call(at, 2, 1, CallMode::Counted, cont);
                    }
                }
            }};
        }
        loop {
            let yes = answer.take().unwrap_or(false);
            match self.at {
                At::Pop => match self.pending.pop() {
                    Some((l, u, limit)) => {
                        (self.l, self.u, self.limit) = (l, u, limit);
                        self.at = At::Loop;
                    }
                    None => return Ok(0),
                },
                At::Loop => {
                    if self.l >= self.u {
                        self.at = At::Pop;
                        continue;
                    }
                    if self.limit == 0 {
                        let count = self.u.saturating_sub(self.l).saturating_add(1);
                        self.heap_root = (count / 2).checked_sub(1);
                        self.at = At::HeapBuild;
                        continue;
                    }
                    ask!(self.u, self.l, At::LessUL);
                }
                At::LessUL => {
                    if yes {
                        self.swap(vm, self.u, self.l);
                    }
                    if self.u.saturating_sub(self.l) == 1 {
                        self.at = At::Pop;
                        continue;
                    }
                    self.m = self.l.saturating_add(self.u.saturating_sub(self.l) >> 1);
                    ask!(self.m, self.l, At::LessML);
                }
                At::LessML => {
                    if yes {
                        self.swap(vm, self.m, self.l);
                        self.pivot(vm);
                    } else {
                        ask!(self.u, self.m, At::LessUM);
                    }
                }
                At::LessUM => {
                    if yes {
                        self.swap(vm, self.m, self.u);
                    }
                    self.pivot(vm);
                }
                At::ScanI => {
                    self.i = self.i.saturating_add(1);
                    ask!(self.i, self.p, At::LessIP);
                }
                At::LessIP => {
                    if !yes {
                        self.at = At::ScanJ;
                    } else if self.i >= self.u {
                        return Err(vm.lib_error("invalid order function for sorting"));
                    } else {
                        self.at = At::ScanI;
                    }
                }
                At::ScanJ => {
                    self.j = self.j.saturating_sub(1);
                    ask!(self.p, self.j, At::LessPJ);
                }
                At::LessPJ => {
                    if yes {
                        if self.j <= self.l {
                            return Err(vm.lib_error("invalid order function for sorting"));
                        }
                        self.at = At::ScanJ;
                    } else if self.j < self.i {
                        self.partitioned(vm);
                    } else {
                        self.swap(vm, self.i, self.j);
                        self.at = At::ScanI;
                    }
                }
                At::HeapBuild => match self.heap_root {
                    Some(r) => {
                        self.heap_root = r.checked_sub(1);
                        self.start_sift(self.l, self.u, r, At::HeapBuild);
                    }
                    None => {
                        self.heap_i = self.u.saturating_sub(self.l);
                        self.at = At::HeapSort;
                    }
                },
                At::HeapSort => {
                    if self.heap_i > 0 {
                        let top = self.l.saturating_add(self.heap_i);
                        self.swap(vm, self.l, top);
                        self.start_sift(self.l, top.saturating_sub(1), 0, At::HeapSortNext);
                    } else {
                        self.at = At::Pop;
                    }
                }
                At::HeapSortNext => {
                    self.heap_i = self.heap_i.saturating_sub(1);
                    self.at = At::HeapSort;
                }
                At::Sift => {
                    let s = self.sift;
                    if s.root.saturating_mul(2).saturating_add(2) < s.count {
                        self.sift.next = s.root;
                        let left = s.root.saturating_mul(2).saturating_add(1);
                        ask!(
                            s.l.saturating_add(s.root),
                            s.l.saturating_add(left),
                            At::SiftLeft
                        );
                    }
                    self.at = At::SiftTail;
                }
                At::SiftLeft => {
                    let s = self.sift;
                    let left = s.root.saturating_mul(2).saturating_add(1);
                    if yes {
                        self.sift.next = left;
                    }
                    let right = left.saturating_add(1);
                    ask!(
                        s.l.saturating_add(self.sift.next),
                        s.l.saturating_add(right),
                        At::SiftRight
                    );
                }
                At::SiftRight => {
                    let s = self.sift;
                    if yes {
                        self.sift.next = s.root.saturating_mul(2).saturating_add(2);
                    }
                    let next = self.sift.next;
                    if next == s.root {
                        self.at = At::SiftTail;
                    } else {
                        self.swap(vm, s.l.saturating_add(s.root), s.l.saturating_add(next));
                        self.sift.root = next;
                        self.at = At::Sift;
                    }
                }
                At::SiftTail => {
                    let s = self.sift;
                    let last_left = s.root.saturating_mul(2).saturating_add(1);
                    if last_left == s.count.saturating_sub(1) {
                        ask!(
                            s.l.saturating_add(s.root),
                            s.l.saturating_add(last_left),
                            At::SiftLast
                        );
                    }
                    self.at = s.ret;
                }
                At::SiftLast => {
                    let s = self.sift;
                    if yes {
                        let last_left = s.root.saturating_mul(2).saturating_add(1);
                        self.swap(
                            vm,
                            s.l.saturating_add(s.root),
                            s.l.saturating_add(last_left),
                        );
                    }
                    self.at = s.ret;
                }
            }
        }
    }

    /// After the median of three: stop for three or fewer elements, else
    /// move the pivot to `p` and start the partition.
    fn pivot(&mut self, vm: &mut Core) {
        if self.u.saturating_sub(self.l) == 2 {
            self.at = At::Pop;
            return;
        }
        self.p = self.u.saturating_sub(1);
        self.swap(vm, self.m, self.p);
        self.i = self.l;
        self.j = self.p;
        self.at = At::ScanI;
    }

    /// After the partition: the pivot goes to `i`; the smaller side is
    /// sorted next and the larger waits (C++ recurses into the smaller).
    fn partitioned(&mut self, vm: &mut Core) {
        self.swap(vm, self.p, self.i);
        self.limit = (self.limit >> 1).saturating_add(self.limit >> 2);
        let (l, u, i, limit) = (self.l, self.u, self.i, self.limit);
        if i.saturating_sub(l) < u.saturating_sub(i) {
            self.pending.push((i.saturating_add(1), u, limit));
            match i.checked_sub(1) {
                Some(upper) => self.u = upper,
                None => {
                    self.at = At::Pop;
                    return;
                }
            }
        } else {
            if let Some(upper) = i.checked_sub(1) {
                self.pending.push((l, upper, limit));
            }
            self.l = i.saturating_add(1);
        }
        self.at = At::Loop;
    }

    fn start_sift(&mut self, l: usize, u: usize, root: usize, ret: At) {
        self.sift = Sift {
            l,
            count: u.saturating_sub(l).saturating_add(1),
            root,
            next: root,
            ret,
        };
        self.at = At::Sift;
    }
}
