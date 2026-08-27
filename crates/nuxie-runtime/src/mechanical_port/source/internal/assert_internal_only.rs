// The C++ header rejects inclusion outside the private build. Rust enforces
// the same boundary with crate visibility rather than a build-definition flag.
pub(crate) const INTERNAL_ONLY: () = ();
