//! Mechanical translation of ore_spirv_entry_split.hpp/cpp.
#![allow(non_snake_case, non_upper_case_globals)]

pub const kSpirvVertexModel: u32 = 0;
pub const kSpirvFragmentModel: u32 = 4;
const MAGIC: u32 = 0x07230203;
const HEADER_WORDS: usize = 5;
const NO_FUNCTION: u32 = u32::MAX;
const DECLARED: u8 = 1;
const FUNCTION: u8 = 2;
const INTERFACE: u8 = 4;
const KEPT: u8 = 8;

#[derive(Default)]
pub struct Function {
    pub end: usize,
    pub callsBegin: u32,
    pub callsEnd: u32,
}

#[derive(Default)]
pub struct SpirvEntryScratch {
    pub flags: Vec<u8>,
    pub functionIndex: Vec<u32>,
    pub functions: Vec<Function>,
    pub calls: Vec<u32>,
    pub pending: Vec<u32>,
    pub words: Vec<u32>,
}

fn declaredId(op: u32, w: &[u32]) -> u32 {
    if w.len() >= 2 && ((19..=38).contains(&op) || op == 11) {
        return w[1];
    }
    if w.len() >= 3 && ((41..=52).contains(&op) || op == 1 || op == 59) {
        return w[2];
    }
    0
}
fn decorates(op: u32) -> bool {
    matches!(op, 71 | 72 | 332 | 5632 | 5633)
}
fn dropped(flags: &[u8], id: u32) -> bool {
    (id as usize) < flags.len()
        && flags[id as usize] & (FUNCTION | INTERFACE) != 0
        && flags[id as usize] & KEPT == 0
}
fn importsMath(w: &[u32]) -> bool {
    w.len() == 6
        && b"GLSL.std.450\0"
            .iter()
            .enumerate()
            .all(|(i, b)| w[2 + i / 4].to_ne_bytes()[i % 4] == *b)
}
fn namesFunctionsIndirectly(capability: u32) -> bool {
    matches!(capability, 6 | 5430 | 5432 | 5434 | 5447 | 5603 | 6246)
}
fn wordAt(code: &[u8], index: usize) -> u32 {
    u32::from_ne_bytes(code[index * 4..index * 4 + 4].try_into().unwrap())
}

/// Reads code at any alignment. The byte slice replaces the upstream pointer.
pub fn spirvEntryPointCount(code: &[u8], wordCount: usize) -> usize {
    if wordCount < HEADER_WORDS || wordCount > code.len() / 4 || wordAt(code, 0) != MAGIC {
        return 0;
    }
    let mut entries = 0;
    let mut at = HEADER_WORDS;
    while at < wordCount {
        let word = wordAt(code, at);
        let length = (word >> 16) as usize;
        let op = word & 0xffff;
        if length == 0 || op == 54 {
            break;
        }
        entries += usize::from(op == 15);
        at += length;
    }
    entries
}

pub fn spirvEntryModule(
    words: &[u32],
    executionModel: u32,
    name: Option<&str>,
    scratch: &mut SpirvEntryScratch,
) -> bool {
    if words.len() < HEADER_WORDS || words[0] != MAGIC || name.is_none() {
        return false;
    }
    let name = name.unwrap().as_bytes();
    // Upstream takes a C string, so only the prefix before a NUL participates.
    let name = &name[..name.iter().position(|b| *b == 0).unwrap_or(name.len())];
    let count = words.len();
    let bound = words[3] as usize;
    if bound == 0 || bound > count {
        return false;
    }
    let SpirvEntryScratch {
        flags,
        functionIndex,
        functions,
        calls,
        pending,
        words: out,
    } = scratch;
    flags.clear();
    flags.resize(bound, 0);
    functionIndex.clear();
    functionIndex.resize(bound, NO_FUNCTION);
    functions.clear();
    calls.clear();
    let mut entries = 0;
    let mut entryAt = 0;
    let mut entryFunction = 0usize;
    let mut interfaceBegin = 0;
    let mut interfaceEnd = 0;
    let mut inFunction = false;
    let mut at = HEADER_WORDS;
    while at < count {
        let length = (words[at] >> 16) as usize;
        let op = words[at] & 0xffff;
        if length == 0 || length > count - at {
            return false;
        }
        let w = &words[at..at + length];
        if op == 54 {
            if inFunction || length < 3 || w[2] as usize >= bound || flags[w[2] as usize] != 0 {
                return false;
            }
            flags[w[2] as usize] |= DECLARED | FUNCTION;
            functionIndex[w[2] as usize] = functions.len() as u32;
            functions.push(Function {
                end: 0,
                callsBegin: calls.len() as u32,
                callsEnd: 0,
            });
            inFunction = true;
        } else if op == 56 {
            if !inFunction {
                return false;
            }
            let function = functions.last_mut().unwrap();
            function.end = at + length;
            function.callsEnd = calls.len() as u32;
            inFunction = false;
        } else if inFunction {
            if op == 57 && length >= 4 {
                calls.push(w[3]);
            }
        } else if matches!(op, 73 | 74 | 75)
            || (op == 11 && !importsMath(w))
            || (op == 17 && length >= 2 && namesFunctionsIndirectly(w[1]))
        {
            return false;
        } else if op == 15 {
            if length < 4 {
                return false;
            }
            let byteAt = |i: usize| w[3 + i / 4].to_ne_bytes()[i % 4];
            let Some(nameBytes) = (0..(length - 3) * 4).find(|i| byteAt(*i) == 0) else {
                return false;
            };
            entries += 1;
            if w[1] == executionModel
                && name.len() == nameBytes
                && name.iter().enumerate().all(|(i, b)| byteAt(i) == *b)
            {
                entryAt = at;
                entryFunction = w[2] as usize;
                interfaceBegin = at + 3 + nameBytes / 4 + 1;
                interfaceEnd = at + length;
            }
        } else {
            let id = declaredId(op, w) as usize;
            if id != 0 {
                if id >= bound {
                    return false;
                }
                flags[id] |= DECLARED;
                if op == 59 && length >= 4 && matches!(w[3], 1 | 3) {
                    flags[id] |= INTERFACE;
                }
            }
        }
        at += length;
    }
    if inFunction
        || entries < 2
        || entryAt == 0
        || entryFunction >= bound
        || functionIndex[entryFunction] == NO_FUNCTION
    {
        return false;
    }
    pending.clear();
    pending.push(entryFunction as u32);
    while let Some(id) = pending.pop() {
        let id = id as usize;
        if id >= bound || flags[id] & KEPT != 0 || functionIndex[id] == NO_FUNCTION {
            continue;
        }
        flags[id] |= KEPT;
        let function = &functions[functionIndex[id] as usize];
        pending.extend_from_slice(&calls[function.callsBegin as usize..function.callsEnd as usize]);
    }
    for &id in &words[interfaceBegin..interfaceEnd] {
        if (id as usize) < bound {
            flags[id as usize] |= KEPT;
        }
    }
    out.clear();
    out.extend_from_slice(&words[..HEADER_WORDS]);
    out.reserve(count.saturating_sub(out.len()));
    let mut at = HEADER_WORDS;
    while at < count {
        let length = (words[at] >> 16) as usize;
        let op = words[at] & 0xffff;
        let w = &words[at..at + length];
        if op == 54 {
            let end = functions[functionIndex[w[2] as usize] as usize].end;
            if flags[w[2] as usize] & KEPT != 0 {
                out.extend_from_slice(&words[at..end]);
            }
            at = end;
            continue;
        }
        let mut copy = true;
        if op == 15 {
            copy = at == entryAt;
        } else if op == 16 || op == 331 {
            copy = length >= 2 && w[1] as usize == entryFunction;
        } else if op == 59 {
            copy = length < 3 || !dropped(flags, w[2]);
        } else if op == 5 || op == 6 {
            copy = false;
        } else if decorates(op) && length >= 2 {
            if dropped(flags, w[1]) {
                copy = false;
            } else if w[1] as usize >= bound || flags[w[1] as usize] & DECLARED == 0 {
                return false;
            }
            if op == 332 {
                for &id in &w[3.min(length)..] {
                    if dropped(flags, id) {
                        return false;
                    }
                }
            }
        }
        if copy {
            out.extend_from_slice(w);
        }
        at += length;
    }
    true
}
