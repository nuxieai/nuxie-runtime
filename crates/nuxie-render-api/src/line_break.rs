//! Unicode UAX #14 line breaking, translated from rive/text/line_break.hpp and
//! src/text/line_break.cpp. Rules are evaluated in source order.

use crate::line_break_data::*;

/// LB1-resolved classes, in the generator's class order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum LineBreakClass {
    AL,
    AK,
    AP,
    AS,
    B2,
    BA,
    BB,
    BK,
    CB,
    CJ,
    CL,
    CM,
    CP,
    CR,
    EB,
    EM,
    EX,
    GL,
    H2,
    H3,
    HH,
    HL,
    HY,
    ID,
    IN,
    IS,
    JL,
    JT,
    JV,
    LF,
    NL,
    NS,
    NU,
    OP,
    PO,
    PR,
    QU,
    RI,
    SP,
    SY,
    VF,
    VI,
    WJ,
    ZW,
    ZWJ,
}

pub struct LineBreakFlags;
impl LineBreakFlags {
    pub const EAST_ASIAN: u8 = 1;
    pub const PI: u8 = 2;
    pub const PF: u8 = 4;
    pub const DOTTED_CIRCLE: u8 = 8;
    pub const PICTOGRAPHIC_CN: u8 = 16;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct LineBreakProps {
    pub cls: LineBreakClass,
    pub flags: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum LineBreak {
    None,
    Allowed,
    Mandatory,
}

pub fn line_break_props(cp: u32) -> LineBreakProps {
    if cp >= LINE_BREAK_HIGH_START {
        return LINE_BREAK_EXT_CLASSES[LINE_BREAK_DEFAULT as usize];
    }
    let mid_entries = 1u32 << (LINE_BREAK_TOP_SHIFT - LINE_BREAK_BLOCK_SHIFT);
    let mid = LINE_BREAK_TOP[(cp >> LINE_BREAK_TOP_SHIFT) as usize] as u32 * mid_entries
        + ((cp >> LINE_BREAK_BLOCK_SHIFT) & (mid_entries - 1));
    let block_size = 1u32 << LINE_BREAK_BLOCK_SHIFT;
    let data = LINE_BREAK_MID[mid as usize] as u32 * block_size + (cp & (block_size - 1));
    LINE_BREAK_EXT_CLASSES[LINE_BREAK_DATA[data as usize] as usize]
}

use LineBreakClass as C;

// One entry per codepoint surviving LB9, with combining sequences collapsed.
#[derive(Clone, Copy)]
struct Item {
    cls: C,
    flags: u8,
    after_zwj: bool,
    index: usize,
}
const INLINE_ITEMS: usize = 256;
const EMPTY_ITEM: Item = Item {
    cls: C::AL,
    flags: 0,
    after_zwj: false,
    index: 0,
};

fn is_hard_break(c: C) -> bool {
    matches!(c, C::BK | C::CR | C::LF | C::NL)
}
fn is_alphabetic(c: C) -> bool {
    matches!(c, C::AL | C::HL)
}
fn is_brahmic_base(i: &Item) -> bool {
    matches!(i.cls, C::AK | C::AS) || i.flags & LineBreakFlags::DOTTED_CIRCLE != 0
}
fn is_ak_or_dotted_circle(i: &Item) -> bool {
    i.cls == C::AK || i.flags & LineBreakFlags::DOTTED_CIRCLE != 0
}
fn is_east_asian(i: &Item) -> bool {
    i.flags & LineBreakFlags::EAST_ASIAN != 0
}
fn is_korean(c: C) -> bool {
    matches!(c, C::JL | C::JV | C::JT | C::H2 | C::H3)
}

fn break_before(
    items: &[Item],
    count: usize,
    k: usize,
    before_spaces: usize,
    regional_indicators: usize,
) -> LineBreak {
    let prev = &items[k - 1];
    let next = &items[k];
    let p = prev.cls;
    let n = next.cls;
    let ps = if before_spaces < count {
        items[before_spaces].cls
    } else {
        C::SP
    };
    let at_start = k == 1;
    // LB4, LB5
    if p == C::BK {
        return LineBreak::Mandatory;
    }
    if p == C::CR && n == C::LF {
        return LineBreak::None;
    }
    if matches!(p, C::CR | C::LF | C::NL) {
        return LineBreak::Mandatory;
    }
    // LB6, LB7
    if is_hard_break(n) || matches!(n, C::SP | C::ZW) {
        return LineBreak::None;
    }
    // LB8
    if p == C::ZW || (p == C::SP && ps == C::ZW) {
        return LineBreak::Allowed;
    }
    // LB8a
    if next.after_zwj {
        return LineBreak::None;
    }
    // Dominant pairs bypass rules that cannot affect them.
    if is_alphabetic(p) && is_alphabetic(n) {
        return LineBreak::None;
    }
    if p == C::ID && n == C::ID {
        return LineBreak::Allowed;
    }
    // LB11, LB12
    if n == C::WJ || matches!(p, C::WJ | C::GL) {
        return LineBreak::None;
    }
    // LB12a
    if n == C::GL && !matches!(p, C::SP | C::BA | C::HY | C::HH) {
        return LineBreak::None;
    }
    // LB13
    if matches!(n, C::CL | C::CP | C::EX | C::SY) {
        return LineBreak::None;
    }
    // LB14
    if p == C::OP || (p == C::SP && ps == C::OP) {
        return LineBreak::None;
    }
    // LB15a
    {
        let j = if p == C::SP { before_spaces } else { k - 1 };
        if j < count && items[j].cls == C::QU && items[j].flags & LineBreakFlags::PI != 0 {
            let mut ok = j == 0;
            if !ok {
                let b = items[j - 1].cls;
                ok = is_hard_break(b) || matches!(b, C::OP | C::QU | C::GL | C::SP | C::ZW);
            }
            if ok {
                return LineBreak::None;
            }
        }
    }
    // LB15b
    if n == C::QU && next.flags & LineBreakFlags::PF != 0 {
        let mut ok = k + 1 == count;
        if !ok {
            let a = items[k + 1].cls;
            ok = matches!(
                a,
                C::SP | C::GL | C::WJ | C::CL | C::QU | C::CP | C::EX | C::IS | C::SY | C::ZW
            ) || is_hard_break(a);
        }
        if ok {
            return LineBreak::None;
        }
    }
    // LB15c
    if p == C::SP && n == C::IS && k + 1 < count && items[k + 1].cls == C::NU {
        return LineBreak::Allowed;
    }
    // LB15d
    if n == C::IS {
        return LineBreak::None;
    }
    // LB16
    if n == C::NS && (matches!(p, C::CL | C::CP) || (p == C::SP && matches!(ps, C::CL | C::CP))) {
        return LineBreak::None;
    }
    // LB17
    if n == C::B2 && (p == C::B2 || (p == C::SP && ps == C::B2)) {
        return LineBreak::None;
    }
    // LB18
    if p == C::SP {
        return LineBreak::Allowed;
    }
    // LB19
    if n == C::QU && next.flags & LineBreakFlags::PI == 0 {
        return LineBreak::None;
    }
    if p == C::QU && prev.flags & LineBreakFlags::PF == 0 {
        return LineBreak::None;
    }
    // LB19a
    if n == C::QU && (!is_east_asian(prev) || k + 1 == count || !is_east_asian(&items[k + 1])) {
        return LineBreak::None;
    }
    if p == C::QU && (!is_east_asian(next) || at_start || !is_east_asian(&items[k - 2])) {
        return LineBreak::None;
    }
    // LB20
    if n == C::CB || p == C::CB {
        return LineBreak::Allowed;
    }
    // LB20a
    if matches!(p, C::HY | C::HH) && is_alphabetic(n) {
        let mut ok = at_start;
        if !ok {
            let b = items[k - 2].cls;
            ok = is_hard_break(b) || matches!(b, C::SP | C::ZW | C::CB | C::GL);
        }
        if ok {
            return LineBreak::None;
        }
    }
    // LB21
    if matches!(n, C::BA | C::HH | C::HY | C::NS) || p == C::BB {
        return LineBreak::None;
    }
    // LB21a
    if matches!(p, C::HY | C::HH) && n != C::HL && !at_start && items[k - 2].cls == C::HL {
        return LineBreak::None;
    }
    // LB21b
    if p == C::SY && n == C::HL {
        return LineBreak::None;
    }
    // LB22
    if n == C::IN {
        return LineBreak::None;
    }
    // LB23
    if (is_alphabetic(p) && n == C::NU) || (p == C::NU && is_alphabetic(n)) {
        return LineBreak::None;
    }
    // LB23a
    if p == C::PR && matches!(n, C::ID | C::EB | C::EM) {
        return LineBreak::None;
    }
    if matches!(p, C::ID | C::EB | C::EM) && n == C::PO {
        return LineBreak::None;
    }
    // LB24
    if (matches!(p, C::PR | C::PO) && is_alphabetic(n))
        || (is_alphabetic(p) && matches!(n, C::PR | C::PO))
    {
        return LineBreak::None;
    }
    // LB25
    if matches!(n, C::PO | C::PR | C::NU) {
        let mut j = k - 1;
        let mut searching = true;
        if n != C::NU && matches!(items[j].cls, C::CL | C::CP) {
            searching = j > 0;
            j = j.wrapping_sub(1);
        }
        while searching && matches!(items[j].cls, C::SY | C::IS) {
            searching = j > 0;
            j = j.wrapping_sub(1);
        }
        if searching && items[j].cls == C::NU {
            return LineBreak::None;
        }
    }
    if matches!(p, C::PO | C::PR) {
        if n == C::NU {
            return LineBreak::None;
        }
        if n == C::OP && k + 1 < count {
            if items[k + 1].cls == C::NU {
                return LineBreak::None;
            }
            if items[k + 1].cls == C::IS && k + 2 < count && items[k + 2].cls == C::NU {
                return LineBreak::None;
            }
        }
    }
    if matches!(p, C::HY | C::IS) && n == C::NU {
        return LineBreak::None;
    }
    // LB26
    if p == C::JL && matches!(n, C::JL | C::JV | C::H2 | C::H3) {
        return LineBreak::None;
    }
    if matches!(p, C::JV | C::H2) && matches!(n, C::JV | C::JT) {
        return LineBreak::None;
    }
    if matches!(p, C::JT | C::H3) && n == C::JT {
        return LineBreak::None;
    }
    // LB27
    if (is_korean(p) && n == C::PO) || (p == C::PR && is_korean(n)) {
        return LineBreak::None;
    }
    // LB28
    if is_alphabetic(p) && is_alphabetic(n) {
        return LineBreak::None;
    }
    // LB28a
    if p == C::AP && is_brahmic_base(next) {
        return LineBreak::None;
    }
    if is_brahmic_base(prev) && matches!(n, C::VF | C::VI) {
        return LineBreak::None;
    }
    if p == C::VI && !at_start && is_brahmic_base(&items[k - 2]) && is_ak_or_dotted_circle(next) {
        return LineBreak::None;
    }
    if is_brahmic_base(prev) && is_brahmic_base(next) && k + 1 < count && items[k + 1].cls == C::VF
    {
        return LineBreak::None;
    }
    // LB29
    if p == C::IS && is_alphabetic(n) {
        return LineBreak::None;
    }
    // LB30
    if (is_alphabetic(p) || p == C::NU) && n == C::OP && !is_east_asian(next) {
        return LineBreak::None;
    }
    if p == C::CP && !is_east_asian(prev) && (is_alphabetic(n) || n == C::NU) {
        return LineBreak::None;
    }
    // LB30a
    if p == C::RI && n == C::RI && regional_indicators & 1 != 0 {
        return LineBreak::None;
    }
    // LB30b
    if n == C::EM && (p == C::EB || prev.flags & LineBreakFlags::PICTOGRAPHIC_CN != 0) {
        return LineBreak::None;
    }
    // LB31
    LineBreak::Allowed
}

/// Computes every boundary before a codepoint. `out` must contain exactly
/// `text.len() + 1` entries. Start is always None; end is Allowed unless the
/// final codepoint is a hard break, in which case it is Mandatory.
pub fn compute_line_breaks(text: &[u32], out: &mut [LineBreak]) {
    let n = text.len();
    assert_eq!(out.len(), n + 1);
    out.fill(LineBreak::None);
    if n == 0 {
        return;
    }
    out[n] = LineBreak::Allowed;
    // LB1 is baked into the table; CJ resolves to strict NS here. LB9/LB10
    // attach combining marks and ZWJ, except after hard breaks, SP, and ZW.
    let mut inline_items = [EMPTY_ITEM; INLINE_ITEMS];
    let mut heap_items;
    let items: &mut [Item] = if n > INLINE_ITEMS {
        heap_items = vec![EMPTY_ITEM; n];
        &mut heap_items
    } else {
        &mut inline_items
    };
    let mut count = 0;
    let mut after_zwj = false;
    for (i, &cp) in text.iter().enumerate() {
        let mut props = line_break_props(cp);
        let mut cls = props.cls;
        let is_zwj = cls == C::ZWJ;
        if cls == C::CJ {
            cls = C::NS;
        }
        if matches!(cls, C::CM | C::ZWJ) {
            if count > 0 {
                let base = items[count - 1].cls;
                if !is_hard_break(base) && base != C::SP && base != C::ZW {
                    after_zwj = is_zwj;
                    continue;
                }
            }
            cls = C::AL;
            props.flags = 0;
        }
        items[count] = Item {
            cls,
            flags: props.flags,
            after_zwj,
            index: i,
        };
        count += 1;
        after_zwj = is_zwj;
    }
    let mut before_spaces = count;
    let mut regional_indicators = 0;
    for k in 1..count {
        if items[k - 1].cls != C::SP {
            before_spaces = k - 1;
        }
        regional_indicators = if items[k - 1].cls == C::RI {
            regional_indicators + 1
        } else {
            0
        };
        out[items[k].index] = break_before(items, count, k, before_spaces, regional_indicators);
    }
    if is_hard_break(items[count - 1].cls) {
        out[n] = LineBreak::Mandatory;
    }
}
