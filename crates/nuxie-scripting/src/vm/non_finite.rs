//! A renderer call made with a number that is not finite (NaN or an
//! infinity) is skipped, and reported once per call site through the VM log
//! with the script and line that made it.
//!
//! Upstream passes such numbers to the renderer, which culls a path with NaN
//! bounds and lets the GPU drop NaN triangles, so the call draws nothing and
//! says nothing. Here the call draws nothing too, but no non-finite number
//! leaves the script, whatever reads the draw downstream, and the author is
//! told which call it was. A script commonly makes one by dividing by a
//! time of 0, such as `advance`'s `seconds` or the time between two pointer
//! events in one frame.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

use nuxie_render_api::Mat2D;

use super::logging_scripting_context::LoggingScriptingContext;
use crate::luau::Lua;

/// The first point with a coordinate that is not finite, as `(x, y)`.
pub(super) fn first_non_finite_point(
    points: impl IntoIterator<Item = (f32, f32)>,
) -> Option<String> {
    points
        .into_iter()
        .find(|(x, y)| !x.is_finite() || !y.is_finite())
        .map(|(x, y)| format!("({x}, {y})"))
}

/// The matrix, written out, when one of its values is not finite.
pub(super) fn non_finite_matrix(matrix: &Mat2D) -> Option<String> {
    matrix.0.iter().any(|value| !value.is_finite()).then(|| {
        let values: Vec<String> = matrix.0.iter().map(|value| value.to_string()).collect();
        format!("[{}]", values.join(", "))
    })
}

/// How many call sites a VM remembers having reported: what a script can
/// make the host keep stays bounded, and past this it logs no more of them.
const MAX_REPORTED_CALLS: usize = 256;

#[derive(Clone, Default)]
struct NonFiniteReports(Rc<RefCell<BTreeSet<(String, &'static str)>>>);

/// Logs, once per script line and call, that `call` was skipped
/// because `what` holds a number that is not finite.
pub(super) fn report_skipped_call(lua: &Lua, call: &'static str, what: &str) {
    report_non_finite(lua, call, "drew nothing", what);
}

/// Logs, once per script line and call, what became of `call` because
/// `what` holds a number that is not finite.
pub(super) fn report_non_finite(lua: &Lua, call: &'static str, outcome: &str, what: &str) {
    let location = caller_location(lua);
    let reports = match lua.app_data_ref::<NonFiniteReports>() {
        Some(reports) => reports.clone(),
        None => {
            let reports = NonFiniteReports::default();
            lua.set_app_data(reports.clone());
            reports
        }
    };
    {
        let mut reports = reports.0.borrow_mut();
        if reports.len() >= MAX_REPORTED_CALLS || !reports.insert((location.clone(), call)) {
            return;
        }
    }
    let Some(logging) = lua
        .app_data_ref::<LoggingScriptingContext>()
        .map(|logging| logging.clone())
    else {
        return;
    };
    let message = format!(
        "{location}: {call} {outcome}: {what} is not a finite number. \
         A division by a time of 0, such as advance's seconds or the time between two \
         pointer events in one frame, makes one."
    );
    logging.print_warning(message.as_bytes());
}

/// `script:line` of the Lua code that made the current renderer call, or
/// the script alone when the line is unknown.
fn caller_location(lua: &Lua) -> String {
    for level in 1..=32 {
        let Some(frame) = lua.inspect_stack(level) else {
            break;
        };
        let Some(source) = frame.source() else {
            continue;
        };
        let source = source
            .strip_prefix('=')
            .or_else(|| source.strip_prefix('@'))
            .unwrap_or(source);
        if source == "[C]" {
            continue;
        }
        return match frame.current_line() {
            Some(line) if line > 0 => format!("{source}:{line}"),
            _ => source.to_owned(),
        };
    }
    "a script".to_owned()
}
