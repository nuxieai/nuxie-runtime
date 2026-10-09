//! Explicit Formula occurrence ownership at pinned160085.
use nuxie_runtime::source::{
    core::CoreArena,
    data_bind::converters::{
        data_converter_formula::DataConverterFormula,
        formula::formula_token_value::FormulaTokenValue,
    },
};

#[test]
fn formula_explicit_removal_retires_only_the_source_owned_token_list() {
    for is_instance in [false, true] {
        let arena = CoreArena::default();
        let authored = arena.insert(FormulaTokenValue::default());
        let output = arena.insert(FormulaTokenValue::default());
        let mut formula = DataConverterFormula::default();
        formula.set_is_instance(is_instance);
        formula.add_token(authored.clone());
        formula.add_output_token(output.clone(), 0);
        let formula = arena.insert(formula);
        assert!(formula.remove_occurrence());
        assert_eq!(
            authored.is_alive(),
            is_instance,
            "authored tokens are owned by definitions"
        );
        assert_eq!(
            output.is_alive(),
            !is_instance,
            "output tokens are owned by instances"
        );
    }
}
