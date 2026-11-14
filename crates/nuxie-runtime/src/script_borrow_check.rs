//! No script runs while a Core receiver on its thread is mutably lent.
//!
//! A script can reach any object in its scene. If one of them is lent out by
//! `CoreHandle::with_mut` (or a runtime artboard's or a data bind context's
//! `borrow_mut`) when a script call starts, the script's first touch of it is
//! a `RefCell` panic. Each such borrow holds a [`Lent`] while its receiver is
//! out, and the scripting facade calls [`script_entered`] before it runs Lua.
//!
//! In builds with `test-support` (test harnesses only), an entry while any
//! receiver is mutably lent writes a report naming the entry and every open
//! borrow to stderr and aborts the process, so the test run fails even when
//! the entry sits under a `catch_unwind` that would swallow a panic. With
//! `NUXIE_SCRIPT_BORROW_SURVEY=1` it reports each distinct case once and
//! carries on, which lists every entry point a suite reaches; inside
//! [`record_script_borrow_reports`] it records the report for the test to
//! assert on. Shared borrows
//! are listed in the report but are not a failure. Shipping builds keep
//! neither the list nor the check; both functions compile to nothing.

use std::panic::Location;

/// What a [`Lent`] guard lends.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Lending {
    /// `CoreHandle::with` and friends: other readers may still read it.
    Shared,
    /// `CoreHandle::with_mut`, a runtime artboard's or a bind context's
    /// `borrow_mut`: any other use of the object panics.
    Mutable,
}

/// One open receiver loan, recorded for as long as the guard lives.
#[must_use]
pub(crate) struct Lent {
    #[cfg(feature = "test-support")]
    depth: Option<usize>,
}

impl Lent {
    /// Records a loan taken at `at` (the borrowing method's caller, through
    /// `#[track_caller]` in test builds).
    #[inline]
    pub(crate) fn new(lending: Lending, at: &'static Location<'static>) -> Self {
        #[cfg(feature = "test-support")]
        {
            Self {
                depth: check::lend(lending, at),
            }
        }
        #[cfg(not(feature = "test-support"))]
        {
            let _ = (lending, at);
            Self {}
        }
    }
}

impl Drop for Lent {
    #[inline]
    fn drop(&mut self) {
        #[cfg(feature = "test-support")]
        if let Some(depth) = self.depth {
            check::release(depth);
        }
    }
}

/// Called by the scripting facade immediately before it runs Lua (a function
/// call or a resume). `entry` is the facade caller's location.
#[doc(hidden)]
#[inline]
pub fn script_entered(entry: &'static Location<'static>) {
    #[cfg(feature = "test-support")]
    check::entered(entry);
    #[cfg(not(feature = "test-support"))]
    let _ = entry;
}

/// The mutable loans open on this thread, innermost last (test builds only).
#[cfg(feature = "test-support")]
#[doc(hidden)]
pub fn open_mutable_loans() -> Vec<&'static Location<'static>> {
    check::open(Lending::Mutable)
}

/// Runs `run` with this thread's script entries recorded instead of
/// aborting: returns `run`'s result and one report per entry made while a
/// receiver was mutably lent (test builds only). Tests use it to name the
/// entry point they cover.
#[cfg(feature = "test-support")]
#[doc(hidden)]
pub fn record_script_borrow_reports<R>(run: impl FnOnce() -> R) -> (R, Vec<String>) {
    check::record(run)
}

#[cfg(feature = "test-support")]
mod check {
    use std::{
        cell::RefCell,
        collections::HashSet,
        io::Write,
        panic::Location,
        sync::{Mutex, OnceLock},
    };

    use super::Lending;

    type Loan = (Lending, &'static Location<'static>);

    thread_local! {
        static LENT: RefCell<Vec<Loan>> = const { RefCell::new(Vec::new()) };
        static RECORDED: RefCell<Option<Vec<String>>> = const { RefCell::new(None) };
    }

    pub(super) fn record<R>(run: impl FnOnce() -> R) -> (R, Vec<String>) {
        struct Restore(Option<Vec<String>>);
        impl Drop for Restore {
            fn drop(&mut self) {
                let outer = self.0.take();
                let _ = RECORDED.try_with(|recorded| *recorded.borrow_mut() = outer);
            }
        }
        let outer = RECORDED.with(|recorded| recorded.borrow_mut().replace(Vec::new()));
        let restore = Restore(outer);
        let result = run();
        let reports = RECORDED
            .with(|recorded| recorded.borrow_mut().take())
            .unwrap_or_default();
        drop(restore);
        (result, reports)
    }

    pub(super) fn lend(lending: Lending, at: &'static Location<'static>) -> Option<usize> {
        LENT.try_with(|lent| {
            let mut lent = lent.borrow_mut();
            let depth = lent.len();
            lent.push((lending, at));
            depth
        })
        .ok()
    }

    pub(super) fn release(depth: usize) {
        let _ = LENT.try_with(|lent| lent.borrow_mut().truncate(depth));
    }

    pub(super) fn open(lending: Lending) -> Vec<&'static Location<'static>> {
        LENT.try_with(|lent| {
            lent.borrow()
                .iter()
                .filter(|(kind, _)| *kind == lending)
                .map(|(_, at)| *at)
                .collect()
        })
        .unwrap_or_default()
    }

    /// Whether an unrecorded entry outside a survey aborts. Off while the
    /// entry points the suites reach are still being moved out of borrows.
    const ENFORCED: bool = false;

    fn survey() -> bool {
        static SURVEY: OnceLock<bool> = OnceLock::new();
        *SURVEY.get_or_init(|| {
            std::env::var_os("NUXIE_SCRIPT_BORROW_SURVEY").is_some_and(|value| value == "1")
        })
    }

    /// The report for a script entered at `entry` now; `None` when no
    /// receiver is mutably lent.
    pub(super) fn report(entry: &'static Location<'static>) -> Option<String> {
        let loans: Vec<Loan> = LENT
            .try_with(|lent| lent.borrow().clone())
            .unwrap_or_default();
        if !loans.iter().any(|(lending, _)| *lending == Lending::Mutable) {
            return None;
        }
        let mut report = format!(
            "SCRIPT-BORROW: a script was entered at {entry} while a Core receiver was mutably lent; open loans, outermost first:\n"
        );
        for (lending, at) in &loans {
            report.push_str(&format!("SCRIPT-BORROW:   {lending:?} at {at}\n"));
        }
        Some(report)
    }

    pub(super) fn entered(entry: &'static Location<'static>) {
        let Some(report) = report(entry) else {
            return;
        };
        let recorded = RECORDED
            .try_with(|recorded| {
                recorded
                    .borrow_mut()
                    .as_mut()
                    .map(|recorded| recorded.push(report.clone()))
                    .is_some()
            })
            .unwrap_or(false);
        if recorded {
            return;
        }
        if survey() || !ENFORCED {
            static SEEN: Mutex<Option<HashSet<String>>> = Mutex::new(None);
            let fresh = SEEN
                .lock()
                .map(|mut seen| seen.get_or_insert_with(HashSet::new).insert(report.clone()))
                .unwrap_or(true);
            if fresh {
                let _ = std::io::stderr().write_all(report.as_bytes());
            }
            return;
        }
        // Written straight to stderr, not through `eprintln!`, so the test
        // harness's output capture cannot hold it back from the abort.
        let backtrace = std::backtrace::Backtrace::force_capture();
        let _ = writeln!(std::io::stderr(), "{report}{backtrace}");
        std::process::abort();
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use std::panic::Location;

    use super::{Lending, Lent, check};

    #[test]
    fn a_mutable_loan_is_reported_with_its_place_and_a_shared_one_is_not() {
        let entry = Location::caller();
        assert!(check::report(entry).is_none());
        let shared_at = Location::caller();
        let shared = Lent::new(Lending::Shared, shared_at);
        assert!(check::report(entry).is_none(), "a shared loan alone is not a failure");
        let mutable_at = Location::caller();
        let report = {
            let _mutable = Lent::new(Lending::Mutable, mutable_at);
            check::report(entry).expect("a mutable loan is reported")
        };
        assert!(report.contains(&format!("entered at {entry}")), "{report}");
        assert!(report.contains(&format!("Shared at {shared_at}")), "{report}");
        assert!(report.contains(&format!("Mutable at {mutable_at}")), "{report}");
        assert!(check::report(entry).is_none(), "the loan ends with its guard");
        drop(shared);
    }

    #[test]
    fn a_recorded_entry_is_returned_instead_of_aborting() {
        let entry = Location::caller();
        let ((), reports) = super::record_script_borrow_reports(|| {
            super::script_entered(entry);
            let _mutable = Lent::new(Lending::Mutable, Location::caller());
            super::script_entered(entry);
        });
        assert_eq!(reports.len(), 1, "{reports:?}");
        assert!(reports[0].contains(&format!("entered at {entry}")));
    }

    #[test]
    fn core_with_mut_lends_mutably_for_exactly_its_callback() {
        let arena = crate::source::core::CoreArena::default();
        let node = arena.insert(crate::source::node::Node::default());
        assert!(super::open_mutable_loans().is_empty());
        let inside = node.with_mut(|_| super::open_mutable_loans()).expect("live node");
        assert_eq!(inside.len(), 1);
        assert_eq!(inside[0].file(), file!(), "the loan names the with_mut caller");
        assert!(node.with(|_| super::open_mutable_loans()).expect("live").is_empty());
        assert!(super::open_mutable_loans().is_empty());
    }
}
