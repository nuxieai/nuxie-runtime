//! Source correspondence: rive/text/line_break_dictionary.hpp and
//! src/text/line_break_dictionary.cpp. Optional dictionary word segmentation.
use crate::line_break::{LineBreak, LineBreakClass, line_break_props};

#[derive(Clone, Copy)]
pub struct LineBreakDictionary<'a> {
    pub ranges: &'a [u32],
    pub alphabet: &'a [u32],
    pub no_break_before: &'a [u32],
    pub no_break_after: &'a [u32],
    pub suffixes: &'a [u32],
    pub coda_marks: &'a [u32],
    pub coda_skip: &'a [u32],
    #[cfg(feature = "text-dict-compressed")]
    pub words: &'a [u8],
    #[cfg(feature = "text-dict-compressed")]
    pub words_size: u32,
    #[cfg(not(feature = "text-dict-compressed"))]
    pub edges: &'a [u32],
    pub edge_count: u32,
    pub edge_checksum: u32,
}
impl LineBreakDictionary<'_> {
    pub const SYMBOL_MASK: u32 = 0x7f;
    pub const LAST_EDGE: u32 = 1 << 7;
    pub const WORD_END: u32 = 1 << 8;
    pub const TARGET_SHIFT: u32 = 9;
}
static DICTIONARIES: &[&LineBreakDictionary<'static>] = &[
    #[cfg(feature = "text-dict-thai")]
    &crate::line_break_dict_thai::K_LINE_BREAK_DICTIONARY_THAI,
    #[cfg(feature = "text-dict-lao")]
    &crate::line_break_dict_lao::K_LINE_BREAK_DICTIONARY_LAO,
    #[cfg(feature = "text-dict-khmer")]
    &crate::line_break_dict_khmer::K_LINE_BREAK_DICTIONARY_KHMER,
    #[cfg(feature = "text-dict-burmese")]
    &crate::line_break_dict_burmese::K_LINE_BREAK_DICTIONARY_BURMESE,
];
fn in_ranges(d: &LineBreakDictionary<'_>, cp: u32) -> bool {
    if cp < d.ranges[0] || cp > d.ranges[d.ranges.len() - 1] {
        return false;
    }
    d.ranges.chunks_exact(2).any(|r| cp >= r[0] && cp <= r[1])
}
fn is_joiner(cp: u32) -> bool {
    cp == 0x200c || cp == 0x200d
}
fn contains(sorted: &[u32], cp: u32) -> bool {
    sorted.binary_search(&cp).is_ok()
}
fn is_mark(cp: u32) -> bool {
    is_joiner(cp) || line_break_props(cp).cls == LineBreakClass::CM
}
fn is_final_consonant(d: &LineBreakDictionary<'_>, run: &[u32], i: usize) -> bool {
    let mut j = i + 1;
    while j < run.len() && contains(d.coda_skip, run[j]) {
        j += 1;
    }
    j < run.len() && contains(d.coda_marks, run[j])
}
fn can_break(d: &LineBreakDictionary<'_>, run: &[u32], i: usize) -> bool {
    let prev = run[i - 1];
    let next = run[i];
    !is_joiner(prev)
        && !is_mark(next)
        && !contains(d.no_break_before, next)
        && !contains(d.suffixes, next)
        && !contains(d.no_break_after, prev)
        && !is_final_consonant(d, run, i)
}
#[derive(Clone, Copy)]
struct Cell {
    symbol: i32,
    can_break: bool,
    word: bool,
    unknown: u32,
    segments: u32,
    from: u32,
}
const EMPTY_CELL: Cell = Cell {
    symbol: -1,
    can_break: false,
    word: false,
    unknown: u32::MAX,
    segments: 0,
    from: 0,
};
fn relax(cells: &mut [Cell], from: usize, to: usize, unknown: u32, word: bool) {
    let total = cells[from].unknown.wrapping_add(unknown);
    let segments = cells[from].segments.wrapping_add(1);
    let cell = &mut cells[to];
    if total < cell.unknown || (total == cell.unknown && segments < cell.segments) {
        cell.unknown = total;
        cell.segments = segments;
        cell.from = from as u32;
        cell.word = word;
    }
}
fn edge_symbol(edge: u32) -> u32 {
    edge & LineBreakDictionary::SYMBOL_MASK
}
fn find_edge(edges: &[u32], first: u32, symbol: u32, root_count: u32) -> Option<usize> {
    if first == 0 {
        let root = &edges[..root_count as usize];
        let found = root.partition_point(|&e| edge_symbol(e) < symbol);
        return (found < root.len() && edge_symbol(root[found]) == symbol).then_some(found);
    }
    for i in first as usize..edges.len() {
        let edge = edges[i];
        if edge_symbol(edge) == symbol {
            return Some(i);
        }
        if edge_symbol(edge) > symbol || edge & LineBreakDictionary::LAST_EDGE != 0 {
            break;
        }
    }
    None
}
fn extend_word_end(
    d: &LineBreakDictionary<'_>,
    run: &[u32],
    cells: &[Cell],
    mut end: usize,
) -> usize {
    while !cells[end].can_break {
        if contains(d.suffixes, run[end]) {
            end += 1;
        } else if !is_mark(run[end]) && is_final_consonant(d, run, end) {
            end += 1;
            while end < run.len() && is_mark(run[end]) {
                end += 1;
            }
        } else {
            return 0;
        }
    }
    end
}
fn segment_run(d: &LineBreakDictionary<'_>, edges: &[u32], run: &[u32], out: &mut [LineBreak]) {
    let count = run.len();
    let mut inline = [EMPTY_CELL; 129];
    let mut heap = Vec::new();
    let cells = if count > 128 {
        heap.resize(count + 1, EMPTY_CELL);
        &mut heap[..]
    } else {
        &mut inline[..count + 1]
    };
    for i in 0..=count {
        cells[i] = EMPTY_CELL;
        if i < count {
            let found = d.alphabet.partition_point(|&cp| cp < run[i]);
            if found < d.alphabet.len() && d.alphabet[found] == run[i] {
                cells[i].symbol = found as i32;
            }
        }
        cells[i].can_break = i == 0 || i == count || can_break(d, run, i);
    }
    cells[0].unknown = 0;
    let mut root_count = 1;
    while edges[root_count - 1] & LineBreakDictionary::LAST_EDGE == 0 {
        root_count += 1;
    }
    for i in 0..count {
        if cells[i].unknown == u32::MAX {
            continue;
        }
        let mut state = 0;
        for j in i..count {
            if cells[j].symbol < 0 {
                break;
            }
            let Some(edge) = find_edge(edges, state, cells[j].symbol as u32, root_count as u32)
            else {
                break;
            };
            let value = edges[edge];
            if value & LineBreakDictionary::WORD_END != 0 {
                let end = extend_word_end(d, run, cells, j + 1);
                if end != 0 {
                    relax(cells, i, end, (end - j - 1) as u32, true);
                }
            }
            state = value >> LineBreakDictionary::TARGET_SHIFT;
            if state == 0 {
                break;
            }
        }
        let mut next = i + 1;
        while !cells[next].can_break {
            next += 1;
        }
        relax(cells, i, next, (next - i) as u32, false);
    }
    let mut end = count;
    while end > 0 {
        let start = cells[end].from as usize;
        if start > 0 && (cells[end].word || cells[start].word) && out[start] == LineBreak::None {
            out[start] = LineBreak::Allowed;
        }
        end = start;
    }
}
#[cfg(feature = "text-dict-compressed")]
pub fn line_break_dictionary_edges<'a>(dictionary: &'a LineBreakDictionary<'_>) -> &'a [u32] {
    static BUILT: [std::sync::OnceLock<Vec<u32>>; DICTIONARIES.len()] =
        [const { std::sync::OnceLock::new() }; DICTIONARIES.len()];
    for (i, d) in DICTIONARIES.iter().enumerate() {
        if std::ptr::eq(*d, dictionary) {
            return BUILT[i].get_or_init(|| {
                crate::line_break_dictionary_builder::build_line_break_dictionary_edges(dictionary)
            });
        }
    }
    &[]
}
#[cfg(not(feature = "text-dict-compressed"))]
pub fn line_break_dictionary_edges<'a>(dictionary: &'a LineBreakDictionary<'_>) -> &'a [u32] {
    dictionary.edges
}
pub fn line_break_dictionary_for(codepoint: u32) -> Option<&'static LineBreakDictionary<'static>> {
    DICTIONARIES
        .iter()
        .copied()
        .find(|d| in_ranges(d, codepoint))
}
pub fn apply_line_break_dictionaries(text: &[u32], out: &mut [LineBreak]) {
    let mut first = u32::MAX;
    let mut last = 0;
    for d in DICTIONARIES {
        first = first.min(d.ranges[0]);
        last = last.max(d.ranges[d.ranges.len() - 1]);
    }
    let mut i = 0;
    while i < text.len() {
        if text[i] < first || text[i] > last {
            i += 1;
            continue;
        }
        let Some(d) = line_break_dictionary_for(text[i]) else {
            i += 1;
            continue;
        };
        if is_mark(text[i]) {
            i += 1;
            continue;
        }
        let mut end = i + 1;
        while end < text.len() && (in_ranges(d, text[end]) || is_joiner(text[end])) {
            end += 1;
        }
        if end - i > 1 {
            let edges = line_break_dictionary_edges(d);
            if !edges.is_empty() {
                segment_run(d, edges, &text[i..end], &mut out[i..]);
            }
        }
        i = end;
    }
}
