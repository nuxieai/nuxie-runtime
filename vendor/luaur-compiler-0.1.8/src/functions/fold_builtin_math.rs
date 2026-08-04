use crate::functions::cnum::cnum;
use crate::functions::cvar::cvar;
use crate::records::constant::Constant;
use luaur_ast::records::ast_name::AstName;

pub(crate) const kPi: f64 = core::f64::consts::PI;
pub(crate) const kNan: f64 = core::f64::NAN;
pub(crate) const kE: f64 = core::f64::consts::E;
pub(crate) const kPhi: f64 = 1.61803398874989484820;
pub(crate) const kSqrt2: f64 = core::f64::consts::SQRT_2;
pub(crate) const kTau: f64 = 6.28318530717958647693;

pub(crate) fn fold_builtin_math(index: AstName) -> Constant {
    if index.operator_eq_c_char(c"pi".as_ptr()) {
        return cnum(kPi);
    }

    if index.operator_eq_c_char(c"huge".as_ptr()) {
        return cnum(f64::INFINITY);
    }

    if index.operator_eq_c_char(c"nan".as_ptr()) {
        return cnum(kNan);
    }

    if index.operator_eq_c_char(c"e".as_ptr()) {
        return cnum(kE);
    }

    if index.operator_eq_c_char(c"phi".as_ptr()) {
        return cnum(kPhi);
    }

    if index.operator_eq_c_char(c"sqrt2".as_ptr()) {
        return cnum(kSqrt2);
    }

    if index.operator_eq_c_char(c"tau".as_ptr()) {
        return cnum(kTau);
    }

    cvar()
}
