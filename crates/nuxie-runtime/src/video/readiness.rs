//! Bounded first-frame readiness, evaluated against a caller-supplied monotonic
//! clock. Acquiring bytes and driving decoders remain host responsibilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Readiness {
    Waiting,
    Frame,
    Poster,
    Unavailable,
}
#[derive(Debug)]
pub struct FirstFrameGate {
    generation: u64,
    deadline: f64,
    wait: bool,
    optional: bool,
    resolved: Option<Readiness>,
}
impl FirstFrameGate {
    pub fn new(
        generation: u64,
        now: f64,
        timeout_seconds: f64,
        wait: bool,
        optional: bool,
    ) -> Option<Self> {
        let deadline = now + timeout_seconds;
        if !now.is_finite()
            || now < 0.0
            || !timeout_seconds.is_finite()
            || !(0.0..=60.0).contains(&timeout_seconds)
            || !deadline.is_finite()
        {
            return None;
        }
        Some(Self {
            generation,
            deadline,
            wait,
            optional,
            resolved: None,
        })
    }
    /// A timeout/failure decision is stable for this generation. A new source
    /// gets a new gate; a late old frame cannot undo the host's fallback choice.
    pub fn evaluate(
        &mut self,
        generation: u64,
        now: f64,
        has_frame: bool,
        failed: bool,
        has_poster: bool,
    ) -> Readiness {
        if let Some(resolved) = self.resolved {
            return resolved;
        }
        if generation != self.generation || !now.is_finite() {
            return Readiness::Waiting;
        }
        let state = if has_frame {
            Readiness::Frame
        } else if failed || now >= self.deadline || !self.wait {
            if has_poster || self.optional {
                Readiness::Poster
            } else {
                Readiness::Unavailable
            }
        } else {
            Readiness::Waiting
        };
        if state != Readiness::Waiting {
            self.resolved = Some(state);
        }
        state
    }
}
