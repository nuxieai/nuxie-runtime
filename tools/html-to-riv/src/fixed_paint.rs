//! Private ordinary static paint primitive; never a source/layout certificate.
//! The caller proves these root-space edges represent the authored paint at all
//! admitted viewports, and that base paint/clip/order interactions are valid.
use crate::{Diagnostic, wire::{self, Record, Value}};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Rect {
    pub geometry: u32,
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub color: u32,
}
impl Rect {
    fn painted(&self) -> bool {
        self.right > self.left && self.bottom > self.top && self.color >> 24 != 0
    }
}
/// Owns both emitted records and the unmodified descriptor sequence, including
/// invisible descriptors. Access is read-only; source identity is not inferred
/// from equal edge coordinates. This carrier establishes emission binding only.
pub(super) struct Folded {
    records: Vec<Record>,
    rects: Vec<Rect>,
    start: usize,
    base_bytes: Vec<u8>,
}
fn invalid(message: &str) -> Diagnostic {
    Diagnostic::new("fixed-paint-binding", "fixed-paint", message)
}
fn inputs(base: &[Record], rects: &[Rect]) -> Result<(), Diagnostic> {
    if base.first().map(|r| r.kind) != Some("Backboard")
        || base.get(1).map(|r| r.kind) != Some("Artboard") {
        return Err(invalid("Expected ordinary Backboard and Artboard roots"));
    }
    for rect in rects {
        if [rect.left, rect.top, rect.right, rect.bottom].iter()
            .any(|v| !(-16_384..=16_384).contains(v)) {
            return Err(invalid("Descriptor edges exceed the certified coordinate envelope"));
        }
        if !(rect.geometry as usize).checked_add(1).and_then(|i| base.get(i))
            .is_some_and(|r| r.kind == "LayoutComponent") {
            return Err(invalid("Descriptor geometry must identify a base layout component"));
        }
    }
    Ok(())
}
impl Folded {
    /// Descriptors are supplied in back-to-front paint order. Ordinary Rive
    /// root siblings draw in reverse insertion order, so emit front-to-back.
    /// No base color, clip, transform or ordering field is modified.
    pub(super) fn new(base: &[Record], rects: &[Rect]) -> Result<Self, Diagnostic> {
        inputs(base, rects)?;
        let added = rects.iter().filter(|r| r.painted()).count().checked_mul(4)
            .ok_or_else(|| invalid("Paint count overflow"))?;
        let end = base.len().checked_add(added).filter(|n| *n <= u32::MAX as usize)
            .ok_or_else(|| invalid("Paint exceeds ordinary object ID capacity"))?;
        let mut records = base.to_vec();
        for rect in rects.iter().rev().filter(|r| r.painted()) {
            let shape = records.len() as u32 - 1;
            let mut s = Record::new("Shape"); s.set("parentId", Value::Uint(0))?;
            let mut r = Record::new("Rectangle"); r.set("parentId", Value::Uint(shape))?;
            // Bounded integer sums/differences and half-integer centers are exact f32.
            r.set("x", Value::Float((rect.left + rect.right) as f32 * 0.5))?;
            r.set("y", Value::Float((rect.top + rect.bottom) as f32 * 0.5))?;
            r.set("width", Value::Float((rect.right - rect.left) as f32))?;
            r.set("height", Value::Float((rect.bottom - rect.top) as f32))?;
            let mut f = Record::new("Fill"); f.set("parentId", Value::Uint(shape))?;
            let mut c = Record::new("SolidColor"); c.set("parentId", Value::Uint(shape + 2))?;
            c.set("colorValue", Value::Color(rect.color))?;
            records.extend([s, r, f, c]);
        }
        if records.len() != end { return Err(invalid("Unexpected paint count")); }
        let folded = Self { records, rects: rects.to_vec(), start: base.len(), base_bytes: wire::encode(base)? };
        folded.validate(base, rects)?;
        Ok(folded)
    }
    pub(super) fn records(&self) -> &[Record] { &self.records }
    pub(super) fn rects(&self) -> &[Rect] { &self.rects }
    pub(super) fn start(&self) -> usize { self.start }

    /// Independent closed suffix reader, not re-emission. Exact fields and
    /// float bits are checked, including explicit zeros and absent defaults.
    pub(super) fn validate(&self, base: &[Record], rects: &[Rect]) -> Result<(), Diagnostic> {
        inputs(base, rects)?;
        if self.rects != rects || self.start != base.len() || self.start > self.records.len()
            || wire::encode(base)? != self.base_bytes
            || wire::encode(&self.records[..self.start])? != self.base_bytes {
            return Err(invalid("Base prefix or source descriptor identity changed"));
        }
        let mut reader = Reader { records: &self.records, next: self.start };
        // Deliberately use edge arithmetic in f64 independently of the emitter.
        for rect in rects.iter().rev() {
            if rect.right <= rect.left || rect.bottom <= rect.top || rect.color & 0xff00_0000 == 0 { continue; }
            let shape = reader.take("Shape", &[("parentId", Value::Uint(0))])?;
            reader.take("Rectangle", &[
                ("parentId", Value::Uint(shape)),
                ("x", Value::Float(((f64::from(rect.left) + f64::from(rect.right)) / 2.) as f32)),
                ("y", Value::Float(((f64::from(rect.top) + f64::from(rect.bottom)) / 2.) as f32)),
                ("width", Value::Float((i64::from(rect.right) - i64::from(rect.left)) as f32)),
                ("height", Value::Float((i64::from(rect.bottom) - i64::from(rect.top)) as f32)),
            ])?;
            let fill = reader.take("Fill", &[("parentId", Value::Uint(shape))])?;
            reader.take("SolidColor", &[("parentId", Value::Uint(fill)), ("colorValue", Value::Color(rect.color))])?;
        }
        if reader.next != self.records.len() { return Err(invalid("Unexpected paint suffix records")); }
        Ok(())
}
}
struct Reader<'a> { records: &'a [Record], next: usize }
impl Reader<'_> {
    fn take(&mut self, kind: &str, fields: &[(&str, Value)]) -> Result<u32, Diagnostic> {
        let record = self.records.get(self.next).ok_or_else(|| invalid("Truncated paint suffix"))?;
        let same = |a: Option<&Value>, b: &Value| match (a, b) {
            (Some(Value::Uint(a)), Value::Uint(b)) | (Some(Value::Color(a)), Value::Color(b)) => a == b,
            (Some(Value::Float(a)), Value::Float(b)) => a.to_bits() == b.to_bits(),
            _ => false,
        };
        if record.kind != kind || !record.has_only_properties(&fields.iter().map(|(k, _)| *k).collect::<Vec<_>>())
            || !fields.iter().all(|(k, v)| same(record.get(k), v)) {
            return Err(invalid("Paint suffix has unexpected type, field, value or order"));
        }
        let id = self.next.checked_sub(1).and_then(|i| u32::try_from(i).ok())
            .ok_or_else(|| invalid("Invalid paint object ID"))?;
        self.next += 1;
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn base() -> Vec<Record> {
        vec![Record::new("Backboard"), Record::new("Artboard"), Record::new("LayoutComponent"), Record::new("LayoutComponent")]
    }
    fn rect() -> Rect { Rect { geometry: 1, left: -11, top: -6, right: 8, bottom: 9, color: 0x80ee5533 } }
    #[test]
    fn signed_half_centers_full_envelope_and_back_to_front_order() {
        let b = base(); let bytes = wire::encode(&b).unwrap();
        let a = rect(); let z = Rect { geometry: 2, left: -16384, top: -16384, right: 16384, bottom: 16384, color: 0xff2167b1 };
        let f = Folded::new(&b, &[a,z]).unwrap();
        assert_eq!(f.start(), 4); assert_eq!(f.records().len(),12); assert_eq!(f.rects(), &[a,z]);
        assert!(matches!(f.records[5].get("width"), Some(Value::Float(v)) if *v==32768.));
        assert!(matches!(f.records[9].get("x"), Some(Value::Float(v)) if *v== -1.5));
        assert!(matches!(f.records[9].get("y"), Some(Value::Float(v)) if *v==1.5));
        assert!(matches!(f.records[7].get("colorValue"), Some(Value::Color(v)) if *v==z.color));
        assert!(matches!(f.records[11].get("colorValue"), Some(Value::Color(v)) if *v==a.color));
        assert_eq!(wire::encode(&f.records[..4]).unwrap(),bytes); wire::encode(f.records()).unwrap();
        assert!(f.validate(&b,&[z,a]).is_err());
    }
    #[test]
    fn invisible_descriptors_still_bind_source_identity() {
        let b=base(); let a=rect();
        let descriptors=[Rect{color:0x00ff0000,..a},Rect{right:a.left,..a},Rect{bottom:a.top-1,..a}];
        let f=Folded::new(&b,&descriptors).unwrap(); assert_eq!(f.records.len(),b.len());
        let mut changed=descriptors;changed[0].geometry=2;assert!(f.validate(&b,&changed).is_err());
        let f=Folded::new(&b,&[a]).unwrap();assert!(f.validate(&b,&[Rect{geometry:2,..a}]).is_err());
        assert_eq!(Folded::new(&b,&[]).unwrap().records.len(),b.len());
    }
    #[test]
    fn every_emitted_field_extra_default_and_suffix_order_are_bound() {
        let b=base();let a=rect();
        for (index,key,value) in [
            (4,"parentId",Value::Uint(1)),(5,"parentId",Value::Uint(0)),
            (5,"x",Value::Float(0.)),(5,"y",Value::Float(0.)),
            (5,"width",Value::Float(18.)),(5,"height",Value::Float(14.)),
            (6,"parentId",Value::Uint(0)),(7,"parentId",Value::Uint(0)),
            (7,"colorValue",Value::Color(0)),(4,"x",Value::Float(0.)),
        ] {
            let mut f=Folded::new(&b,&[a]).unwrap();f.records[index].set(key,value).unwrap();
            assert!(f.validate(&b,&[a]).is_err(),"{index}.{key}");
        }
        let mut f=Folded::new(&b,&[a]).unwrap();f.records[5]=Record::new("Rectangle");assert!(f.validate(&b,&[a]).is_err());
        let mut f=Folded::new(&b,&[a]).unwrap();f.records.swap(5,6);assert!(f.validate(&b,&[a]).is_err());
        let mut f=Folded::new(&b,&[a]).unwrap();f.records.pop();assert!(f.validate(&b,&[a]).is_err());
        let mut f=Folded::new(&b,&[a]).unwrap();f.records.push(Record::new("Shape"));assert!(f.validate(&b,&[a]).is_err());
        let mut f=Folded::new(&b,&[a]).unwrap();f.records[2].set("x",Value::Float(1.)).unwrap();assert!(f.validate(&b,&[a]).is_err());
    }
    #[test]
    fn rejects_bounds_bad_owner_and_roots_without_mutation() {
        let b=base();let before=wire::encode(&b).unwrap();
        for a in [Rect{left:-16385,..rect()},Rect{right:16385,..rect()},Rect{top:i32::MIN,..rect()},Rect{bottom:i32::MAX,..rect()},Rect{geometry:u32::MAX,..rect()},Rect{geometry:0,..rect()}] {
            assert!(Folded::new(&b,&[a]).is_err());assert_eq!(wire::encode(&b).unwrap(),before);
        }
        let mut changed=b.clone();changed[1]=Record::new("Node");assert!(Folded::new(&changed,&[]).is_err());
        let f=Folded::new(&b,&[rect()]).unwrap();let mut changed=b.clone();changed[2].set("x",Value::Float(2.)).unwrap();assert!(f.validate(&changed,&[rect()]).is_err());
    }
}
