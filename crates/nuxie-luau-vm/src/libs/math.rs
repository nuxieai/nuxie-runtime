//! Part of the math library (C++ `lmathlib.cpp`): the functions that are
//! one `f64` operation. Lane 5 adds `random`, `noise` and the rest.

use crate::error::VmResult;
use crate::native::{Args, BuiltinFn};
use crate::value::Value;
use crate::vm::Vm;

macro_rules! unary {
    ($name:ident, $f:expr) => {
        fn $name(vm: &mut Vm, args: Args) -> VmResult<usize> {
            let x = vm.check_number(args, 1)?;
            let f: fn(f64) -> f64 = $f;
            vm.ret(&[Value::Number(f(x))])
        }
    };
}

unary!(abs, f64::abs);
unary!(ceil, f64::ceil);
unary!(floor, f64::floor);
unary!(sqrt, f64::sqrt);
unary!(sin, f64::sin);
unary!(cos, f64::cos);
unary!(tan, f64::tan);
unary!(asin, f64::asin);
unary!(acos, f64::acos);
unary!(exp, f64::exp);
unary!(log10, f64::log10);
unary!(sinh, f64::sinh);
unary!(cosh, f64::cosh);
unary!(tanh, f64::tanh);
unary!(deg, |x| x * (180.0 / std::f64::consts::PI));
unary!(rad, |x| x * (std::f64::consts::PI / 180.0));
unary!(sign, |x| if x > 0.0 {
    1.0
} else if x < 0.0 {
    -1.0
} else {
    0.0
});
// C++ `math.round`: half away from zero.
unary!(round, f64::round);

fn atan(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let y = vm.check_number(args, 1)?;
    vm.ret(&[Value::Number(y.atan())])
}

fn atan2(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let y = vm.check_number(args, 1)?;
    let x = vm.check_number(args, 2)?;
    vm.ret(&[Value::Number(y.atan2(x))])
}

fn pow(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let x = vm.check_number(args, 1)?;
    let y = vm.check_number(args, 2)?;
    vm.ret(&[Value::Number(x.powf(y))])
}

fn fmod(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let x = vm.check_number(args, 1)?;
    let y = vm.check_number(args, 2)?;
    vm.ret(&[Value::Number(x % y)])
}

fn modf(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let x = vm.check_number(args, 1)?;
    // The fastcall body (`luauF_modf`), which the vendored VM runs for a
    // compiled `math.modf(x)` under safeenv: an integral `x` (including an
    // infinity) has a zero fraction carrying `x`'s sign. Its library body
    // gives `+0` and `NaN` there instead; see the lane 1 evidence doc.
    let ip = x.trunc();
    let fp = if x == ip { 0.0f64.copysign(x) } else { x - ip };
    vm.ret(&[Value::Number(ip), Value::Number(fp)])
}

fn log(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let x = vm.check_number(args, 1)?;
    let r = if vm.arg_or_nil(args, 2).is_nil() {
        x.ln()
    } else {
        let base = vm.check_number(args, 2)?;
        if base == 2.0 {
            x.log2()
        } else if base == 10.0 {
            x.log10()
        } else {
            x.ln() / base.ln()
        }
    };
    vm.ret(&[Value::Number(r)])
}

fn min(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let mut m = vm.check_number(args, 1)?;
    for i in 2..=args.nargs {
        let x = vm.check_number(args, i)?;
        if x < m {
            m = x;
        }
    }
    vm.ret(&[Value::Number(m)])
}

fn max(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let mut m = vm.check_number(args, 1)?;
    for i in 2..=args.nargs {
        let x = vm.check_number(args, i)?;
        if x > m {
            m = x;
        }
    }
    vm.ret(&[Value::Number(m)])
}

fn clamp(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let v = vm.check_number(args, 1)?;
    let lo = vm.check_number(args, 2)?;
    let hi = vm.check_number(args, 3)?;
    if lo > hi {
        return Err(vm.arg_error(3, "max must be greater than or equal to min"));
    }
    let r = if v < lo {
        lo
    } else if v > hi {
        hi
    } else {
        v
    };
    vm.ret(&[Value::Number(r)])
}

pub(crate) fn open(vm: &mut Vm) -> VmResult<()> {
    let functions: &[(&str, BuiltinFn)] = &[
        ("abs", abs),
        ("ceil", ceil),
        ("floor", floor),
        ("sqrt", sqrt),
        ("sin", sin),
        ("cos", cos),
        ("tan", tan),
        ("asin", asin),
        ("acos", acos),
        ("atan", atan),
        ("atan2", atan2),
        ("exp", exp),
        ("log", log),
        ("log10", log10),
        ("pow", pow),
        ("fmod", fmod),
        ("modf", modf),
        ("sinh", sinh),
        ("cosh", cosh),
        ("tanh", tanh),
        ("deg", deg),
        ("rad", rad),
        ("sign", sign),
        ("round", round),
        ("min", min),
        ("max", max),
        ("clamp", clamp),
    ];
    let t = vm.open_lib("math", functions)?;
    vm.raw_set_str(t, "pi", Value::Number(std::f64::consts::PI))?;
    vm.raw_set_str(t, "huge", Value::Number(f64::INFINITY))?;
    Ok(())
}
