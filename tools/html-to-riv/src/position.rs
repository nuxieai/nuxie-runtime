//! Computed physical offsets. Static positioning retains authored values for
//! explicit inheritance but suppresses them when emitting layout properties.
use crate::{Diagnostic, wire::{Record, Value}};
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) enum Offset { #[default] Auto, Px(f32), Percent(f32) }
impl Offset {
    pub(crate) fn parse(value:&str,source:&str)->Result<Self,Diagnostic> {
        let invalid=||Diagnostic::new("unsupported-inset",source,"Inset requires auto, zero, a finite signed length up to 1000000px, or a percentage up to 10000%");
        let mut input=cssparser::ParserInput::new(value);let mut p=cssparser::Parser::new(&mut input);
        let result=match p.next() {
            Ok(cssparser::Token::Ident(v)) if v.eq_ignore_ascii_case("auto")=>Self::Auto,
            Ok(cssparser::Token::Number{value,..}) if *value==0.=>Self::Px(0.),
            Ok(cssparser::Token::Dimension{value,unit,..}) if unit.eq_ignore_ascii_case("px") && value.is_finite() && value.abs()<=1_000_000.=>Self::Px(*value),
            Ok(cssparser::Token::Percentage{unit_value,..}) if unit_value.is_finite() && unit_value.abs()<=100.=>Self::Percent(value.strip_suffix('%').and_then(|v|v.parse().ok()).unwrap_or(*unit_value*100.)),
            _=>return Err(invalid()),
        };
        p.expect_exhausted().map_err(|_|invalid())?;Ok(result)
    }
    pub(crate) fn shorthand(value:&str,source:&str)->Result<[Self;4],Diagnostic> {
        let v=value.split_ascii_whitespace().map(|v|Self::parse(v,source)).collect::<Result<Vec<_>,_>>()?;
        match v.as_slice() {
            [a]=>Ok([*a;4]),[a,b]=>Ok([*a,*b,*a,*b]),[a,b,c]=>Ok([*a,*b,*c,*b]),[a,b,c,d]=>Ok([*a,*b,*c,*d]),
            _=>Err(Diagnostic::new("unsupported-inset",source,"inset takes one to four physical offsets")),
        }
    }
    pub(crate) fn emit(self,record:&mut Record,side:&str)->Result<(),Diagnostic> {
        let (v,u)=match self{Self::Auto=>(0.,3),Self::Px(v)=>(v,1),Self::Percent(v)=>(v,2)};
        record.set(&format!("position{side}"),Value::Float(v))?;
        record.set(&format!("position{side}UnitsValue"),Value::Uint(u))
    }
}

/// Authored positioning is distinct from the legacy runtime's relative default.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum PositionMode {
    #[default]
    Static,
    Relative,
    Absolute,
}
impl PositionMode {
    pub(crate) fn is_positioned(self) -> bool { self != Self::Static }
}
