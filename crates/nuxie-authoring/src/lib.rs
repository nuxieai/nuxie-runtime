//! Editor-authoring contract for dynamic Nuxie scenes.
//!
//! The complete Scene interface is re-exported as one deep module so stable
//! identity, transaction validation, lowering, materialization, and authored
//! observations do not fracture across shallow packages. UNIV-1627 moves its
//! implementation to the editor workspace; the current `nuxie` exports remain
//! temporary compatibility paths until that coordinated move.

pub use nuxie::authoring::*;

#[cfg(test)]
mod tests {
    use super::Scene;

    #[test]
    fn scene_contract_keeps_legacy_type_identity() {
        fn accepts_legacy(value: nuxie::Scene) -> Scene {
            value
        }

        let _ = accepts_legacy;
    }
}
