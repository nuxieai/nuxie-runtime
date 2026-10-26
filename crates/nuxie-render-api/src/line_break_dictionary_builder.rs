//! Mechanical translation of src/text/line_break_dictionary_builder.cpp.
use crate::line_break_dictionary::LineBreakDictionary;
use std::collections::HashMap;

const MAX_BITS: usize = 15;
struct Huffman {
    count: [u16; MAX_BITS + 1],
    symbol: [u16; 288],
}
impl Default for Huffman {
    fn default() -> Self {
        Self {
            count: [0; MAX_BITS + 1],
            symbol: [0; 288],
        }
    }
}
struct Inflater<'a> {
    input: &'a [u8],
    input_pos: usize,
    bit_buffer: u32,
    bit_count: u32,
    output: &'a mut [u8],
    output_pos: usize,
    failed: bool,
}
impl<'a> Inflater<'a> {
    fn new(input: &'a [u8], output: &'a mut [u8]) -> Self {
        Self {
            input,
            input_pos: 0,
            bit_buffer: 0,
            bit_count: 0,
            output,
            output_pos: 0,
            failed: false,
        }
    }
    fn run(&mut self) -> bool {
        let mut last = false;
        while !last && !self.failed {
            last = self.bits(1) != 0;
            let ok = match self.bits(2) {
                0 => self.stored(),
                1 => self.fixed(),
                2 => self.dynamic(),
                _ => false,
            };
            if !ok {
                return false;
            }
        }
        !self.failed && self.output_pos == self.output.len()
    }
    fn bits(&mut self, n: u32) -> u32 {
        while self.bit_count < n {
            if self.input_pos == self.input.len() {
                self.failed = true;
                return 0;
            }
            self.bit_buffer |= u32::from(self.input[self.input_pos]) << self.bit_count;
            self.input_pos += 1;
            self.bit_count += 8;
        }
        let value = self.bit_buffer & ((1u32 << n) - 1);
        self.bit_buffer >>= n;
        self.bit_count -= n;
        value
    }
    fn stored(&mut self) -> bool {
        self.bit_buffer = 0;
        self.bit_count = 0;
        if self.input.len() - self.input_pos < 4 {
            return false;
        }
        let length = usize::from(self.input[self.input_pos])
            | (usize::from(self.input[self.input_pos + 1]) << 8);
        let complement = usize::from(self.input[self.input_pos + 2])
            | (usize::from(self.input[self.input_pos + 3]) << 8);
        self.input_pos += 4;
        if length != (!complement & 0xffff)
            || self.input.len() - self.input_pos < length
            || self.output.len() - self.output_pos < length
        {
            return false;
        }
        self.output[self.output_pos..self.output_pos + length]
            .copy_from_slice(&self.input[self.input_pos..self.input_pos + length]);
        self.input_pos += length;
        self.output_pos += length;
        true
    }
    fn construct(huffman: &mut Huffman, lengths: &[u8]) -> bool {
        huffman.count.fill(0);
        for &length in lengths {
            huffman.count[length as usize] += 1;
        }
        let mut left = 1i32;
        for length in 1..=MAX_BITS {
            left = (left << 1) - i32::from(huffman.count[length]);
            if left < 0 {
                return false;
            }
        }
        let mut offsets = [0u16; MAX_BITS + 1];
        for length in 1..MAX_BITS {
            offsets[length + 1] = offsets[length] + huffman.count[length];
        }
        for (i, &length) in lengths.iter().enumerate() {
            if length != 0 {
                huffman.symbol[offsets[length as usize] as usize] = i as u16;
                offsets[length as usize] += 1;
            }
        }
        true
    }
    fn decode(&mut self, huffman: &Huffman) -> i32 {
        let (mut code, mut first, mut index) = (0i32, 0i32, 0i32);
        for length in 1..=MAX_BITS {
            code |= self.bits(1) as i32;
            let count = i32::from(huffman.count[length]);
            if code - count < first {
                return i32::from(huffman.symbol[(index + code - first) as usize]);
            }
            index += count;
            first = (first + count) << 1;
            code <<= 1;
        }
        -1
    }
    fn codes(&mut self, length_code: &Huffman, distance_code: &Huffman) -> bool {
        const LENGTH_BASE: [u16; 29] = [
            3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99,
            115, 131, 163, 195, 227, 258,
        ];
        const LENGTH_EXTRA: [u8; 29] = [
            0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
        ];
        const DISTANCE_BASE: [u16; 30] = [
            1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025,
            1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
        ];
        const DISTANCE_EXTRA: [u8; 30] = [
            0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12,
            12, 13, 13,
        ];
        while !self.failed {
            let mut symbol = self.decode(length_code);
            if symbol < 0 {
                return false;
            }
            if symbol < 256 {
                if self.output_pos == self.output.len() {
                    return false;
                }
                self.output[self.output_pos] = symbol as u8;
                self.output_pos += 1;
                continue;
            }
            if symbol == 256 {
                return !self.failed;
            }
            symbol -= 257;
            if symbol >= 29 {
                return false;
            }
            let length = (u32::from(LENGTH_BASE[symbol as usize])
                + self.bits(u32::from(LENGTH_EXTRA[symbol as usize])))
                as usize;
            let distance_symbol = self.decode(distance_code);
            if !(0..30).contains(&distance_symbol) {
                return false;
            }
            let distance = (u32::from(DISTANCE_BASE[distance_symbol as usize])
                + self.bits(u32::from(DISTANCE_EXTRA[distance_symbol as usize])))
                as usize;
            if distance > self.output_pos || length > self.output.len() - self.output_pos {
                return false;
            }
            for _ in 0..length {
                self.output[self.output_pos] = self.output[self.output_pos - distance];
                self.output_pos += 1;
            }
        }
        false
    }
    fn fixed(&mut self) -> bool {
        let mut lengths = [0u8; 318];
        for (i, length) in lengths[..288].iter_mut().enumerate() {
            *length = if i < 144 {
                8
            } else if i < 256 {
                9
            } else if i < 280 {
                7
            } else {
                8
            };
        }
        lengths[288..].fill(5);
        let mut length_code = Huffman::default();
        let mut distance_code = Huffman::default();
        Self::construct(&mut length_code, &lengths[..288]);
        Self::construct(&mut distance_code, &lengths[288..]);
        self.codes(&length_code, &distance_code)
    }
    fn dynamic(&mut self) -> bool {
        const ORDER: [usize; 19] = [
            16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
        ];
        let length_count = self.bits(5) as usize + 257;
        let distance_count = self.bits(5) as usize + 1;
        let code_count = self.bits(4) as usize + 4;
        if length_count > 286 || distance_count > 30 {
            return false;
        }
        let mut code_lengths = [0u8; 19];
        for i in 0..code_count {
            code_lengths[ORDER[i]] = self.bits(3) as u8;
        }
        let mut code_code = Huffman::default();
        if !Self::construct(&mut code_code, &code_lengths) {
            return false;
        }
        let mut lengths = [0u8; 316];
        let total = length_count + distance_count;
        let mut index = 0;
        while index < total {
            let symbol = self.decode(&code_code);
            if symbol < 0 || self.failed {
                return false;
            }
            if symbol < 16 {
                lengths[index] = symbol as u8;
                index += 1;
                continue;
            }
            let mut repeat = 0;
            let times = if symbol == 16 {
                if index == 0 {
                    return false;
                }
                repeat = lengths[index - 1];
                3 + self.bits(2) as usize
            } else if symbol == 17 {
                3 + self.bits(3) as usize
            } else {
                11 + self.bits(7) as usize
            };
            if index + times > total {
                return false;
            }
            for _ in 0..times {
                lengths[index] = repeat;
                index += 1;
            }
        }
        if lengths[256] == 0 {
            return false;
        }
        let mut length_code = Huffman::default();
        let mut distance_code = Huffman::default();
        if !Self::construct(&mut length_code, &lengths[..length_count])
            || !Self::construct(&mut distance_code, &lengths[length_count..total])
        {
            return false;
        }
        self.codes(&length_code, &distance_code)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Edge {
    symbol: u8,
    word_end: bool,
    target: u32,
}
struct TrieBuilder {
    states: Vec<Vec<Edge>>,
    free: Vec<u32>,
    path: Vec<u32>,
    // Hash buckets retain state IDs, avoiding a self-referential hasher while
    // comparing exactly the source's outgoing-edge vectors.
    registry: HashMap<usize, Vec<u32>>,
}
impl TrieBuilder {
    fn new() -> Self {
        Self {
            states: vec![Vec::new()],
            free: Vec::new(),
            path: vec![0],
            registry: HashMap::with_capacity(1024),
        }
    }
    fn state_hash(&self, state: u32) -> usize {
        let mut hash = 14695981039346656037u64;
        for edge in &self.states[state as usize] {
            let value = u64::from(edge.symbol)
                | if edge.word_end { 1u64 << 8 } else { 0 }
                | (u64::from(edge.target) << 9);
            hash = (hash ^ value).wrapping_mul(1099511628211);
        }
        (hash ^ (hash >> 32)) as usize
    }
    fn new_state(&mut self) -> u32 {
        if let Some(state) = self.free.pop() {
            self.states[state as usize].clear();
            state
        } else {
            self.states.push(Vec::new());
            (self.states.len() - 1) as u32
        }
    }
    fn minimize(&mut self, depth: usize) {
        for d in (depth + 1..self.path.len()).rev() {
            let state = self.path[d];
            let hash = self.state_hash(state);
            let found = self.registry.get(&hash).and_then(|bucket| {
                bucket
                    .iter()
                    .copied()
                    .find(|&other| self.states[state as usize] == self.states[other as usize])
            });
            if let Some(found) = found {
                self.states[self.path[d - 1] as usize]
                    .last_mut()
                    .unwrap()
                    .target = found;
                self.free.push(state);
            } else {
                self.registry.entry(hash).or_default().push(state);
            }
        }
    }
    fn add(&mut self, word: &[u8], shared: usize) {
        self.minimize(shared);
        self.path.resize(shared + 1, 0);
        for i in shared..word.len() {
            let state = self.new_state();
            self.states[*self.path.last().unwrap() as usize].push(Edge {
                symbol: word[i],
                word_end: i + 1 == word.len(),
                target: state,
            });
            self.path.push(state);
        }
    }
    fn finish(mut self) -> Vec<u32> {
        self.minimize(0);
        let mut first_edge = vec![0u32; self.states.len()];
        let mut seen = vec![false; self.states.len()];
        let mut order = vec![0usize];
        seen[0] = true;
        let mut edge_count = 0u32;
        let mut i = 0;
        while i < order.len() {
            let edges = &self.states[order[i]];
            if !edges.is_empty() {
                first_edge[order[i]] = edge_count;
                edge_count = edge_count.wrapping_add(edges.len() as u32);
                for edge in edges {
                    if !seen[edge.target as usize] {
                        seen[edge.target as usize] = true;
                        order.push(edge.target as usize);
                    }
                }
            }
            i += 1;
        }
        let mut result = Vec::with_capacity(edge_count as usize);
        for state in order {
            let edges = &self.states[state];
            for (i, edge) in edges.iter().enumerate() {
                result.push(
                    u32::from(edge.symbol)
                        | if i + 1 == edges.len() {
                            LineBreakDictionary::LAST_EDGE
                        } else {
                            0
                        }
                        | if edge.word_end {
                            LineBreakDictionary::WORD_END
                        } else {
                            0
                        }
                        | (first_edge[edge.target as usize] << LineBreakDictionary::TARGET_SHIFT),
                );
            }
        }
        result
    }
}

pub fn build_line_break_dictionary_edges(dictionary: &LineBreakDictionary<'_>) -> Vec<u32> {
    let mut coded = vec![0u8; dictionary.words_size as usize];
    if !Inflater::new(dictionary.words, &mut coded).run() {
        return Vec::new();
    }
    let mut builder = TrieBuilder::new();
    let mut word = [0u8; 256];
    let mut length = 0;
    let mut i = 0;
    while i < coded.len() {
        let shared = coded[i] as usize;
        i += 1;
        if shared > length {
            return Vec::new();
        }
        length = shared;
        while i < coded.len() && coded[i] != 0 {
            if length == word.len() || coded[i] as usize > dictionary.alphabet.len() {
                return Vec::new();
            }
            word[length] = coded[i] - 1;
            length += 1;
            i += 1;
        }
        if i == coded.len() || length == shared {
            return Vec::new();
        }
        i += 1;
        builder.add(&word[..length], shared);
    }
    let edges = builder.finish();
    let mut checksum = 2166136261u32;
    for &edge in &edges {
        checksum = (checksum ^ edge).wrapping_mul(16777619);
    }
    if edges.len() != dictionary.edge_count as usize || checksum != dictionary.edge_checksum {
        return Vec::new();
    }
    edges
}
