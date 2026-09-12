//! Strict entropy consumption for the compiler's initial JPEG profile.
//! This validates structure and coefficient presence, not pixels or IDCT. The
//! unchanged portable decoder still decodes every admitted image afterward.
use crate::Diagnostic;

fn invalid(source: &str, message: &str) -> Diagnostic {
    Diagnostic::new("invalid-image", source, message)
}
fn unsupported(source: &str, message: &str) -> Diagnostic {
    Diagnostic::new("unsupported-image", source, message)
}

#[derive(Clone)]
struct Huffman {
    counts: [u8; 16],
    values: Vec<u8>,
}
impl Huffman {
    fn new(counts: [u8; 16], values: Vec<u8>, source: &str) -> Result<Self, Diagnostic> {
        if values.is_empty() || values.len() > 256 {
            return Err(invalid(
                source,
                "JPEG Huffman table must contain 1..256 symbols",
            ));
        }
        let mut code = 0u32;
        for (index, count) in counts.iter().enumerate() {
            // JPEG reserves the all-one code so final one-bit padding cannot
            // manufacture another symbol. Reject oversubscribed tables too.
            if *count != 0 && code + u32::from(*count) >= 1 << (index + 1) {
                return Err(invalid(
                    source,
                    "Oversubscribed JPEG Huffman table or reserved all-one code",
                ));
            }
            code = (code + u32::from(*count)) << 1;
        }
        Ok(Self { counts, values })
    }
    fn symbol(&self, bits: &mut Bits<'_>) -> Result<u8, Diagnostic> {
        let (mut code, mut first, mut index) = (0u32, 0u32, 0usize);
        for count in self.counts {
            code = (code << 1) | bits.read(1)?;
            if code >= first && code - first < u32::from(count) {
                return Ok(self.values[index + (code - first) as usize]);
            }
            index += usize::from(count);
            first = (first + u32::from(count)) << 1;
        }
        Err(invalid(bits.source, "Invalid JPEG Huffman symbol"))
    }
}

struct Bits<'a> {
    bytes: &'a [u8],
    source: &'a str,
    pos: usize,
    byte: u8,
    remaining: u8,
}
impl<'a> Bits<'a> {
    fn read(&mut self, count: u8) -> Result<u32, Diagnostic> {
        let mut result = 0u32;
        for _ in 0..count {
            if self.remaining == 0 {
                self.byte = *self
                    .bytes
                    .get(self.pos)
                    .ok_or_else(|| invalid(self.source, "Truncated JPEG entropy data"))?;
                if self.byte == 0xff {
                    if self.bytes.get(self.pos + 1) != Some(&0) {
                        return Err(invalid(
                            self.source,
                            "JPEG marker arrives before the declared entropy blocks are complete",
                        ));
                    }
                    self.pos += 1;
                }
                self.pos += 1;
                self.remaining = 8;
            }
            self.remaining -= 1;
            result = (result << 1) | u32::from((self.byte >> self.remaining) & 1);
        }
        Ok(result)
    }
    fn padding(&mut self) -> Result<(), Diagnostic> {
        let mask = (1u16 << self.remaining) - 1;
        if u16::from(self.byte) & mask != mask {
            return Err(invalid(
                self.source,
                "JPEG scan or restart padding must contain only one bits",
            ));
        }
        self.remaining = 0;
        Ok(())
    }
    fn restart(&mut self, expected: u8) -> Result<(), Diagnostic> {
        self.padding()?;
        if self.bytes.get(self.pos) != Some(&0xff) {
            return Err(invalid(self.source, "JPEG restart marker is missing"));
        }
        while self.bytes.get(self.pos) == Some(&0xff) {
            self.pos += 1;
        }
        if self.bytes.get(self.pos) != Some(&(0xd0 + expected)) {
            return Err(invalid(
                self.source,
                "JPEG restart marker sequence is incorrect",
            ));
        }
        self.pos += 1;
        Ok(())
    }
    fn finish(mut self) -> Result<usize, Diagnostic> {
        self.padding()?;
        if self.bytes.get(self.pos) != Some(&0xff) {
            return Err(invalid(
                self.source,
                "Extraneous JPEG entropy bytes follow the declared blocks",
            ));
        }
        Ok(self.pos)
    }
}

#[derive(Clone, Copy)]
struct ScanComponent {
    component: usize,
    dc: usize,
    ac: usize,
}
struct Scan {
    components: Vec<ScanComponent>,
    start: usize,
    end: usize,
    high: u8,
    low: u8,
}
struct Frame {
    progressive: bool,
    ids: [u8; 3],
    blocks: usize,
    // Only zero/nonzero affects future AC-refinement bit consumption. Retain
    // one bit per coefficient per block instead of another decoded image.
    nonzero: [Vec<u64>; 3],
    levels: [[Option<u8>; 64]; 3],
}
impl Frame {
    fn new(data: &[u8], progressive: bool, source: &str) -> Result<Self, Diagnostic> {
        if data.len() != 15 || data[0] != 8 || data[5] != 3 {
            return Err(unsupported(
                source,
                "JPEG entropy validation currently supports 8-bit three-component frames",
            ));
        }
        let width = usize::from(u16::from_be_bytes([data[3], data[4]]));
        let height = usize::from(u16::from_be_bytes([data[1], data[2]]));
        super::dimensions(width as u32, height as u32, source)?;
        let ids = [data[6], data[9], data[12]];
        if ids[0] == ids[1] || ids[0] == ids[2] || ids[1] == ids[2] {
            return Err(invalid(
                source,
                "JPEG frame component identifiers must be unique",
            ));
        }
        if [data[7], data[10], data[13]] != [0x11; 3] {
            return Err(unsupported(
                source,
                "JPEG entropy validation currently supports 4:4:4 sampling; subsampled images remain unqualified",
            ));
        }
        let blocks = width.div_ceil(8) * height.div_ceil(8);
        Ok(Self {
            progressive,
            ids,
            blocks,
            nonzero: std::array::from_fn(|_| vec![0; blocks]),
            levels: [[None; 64]; 3],
        })
    }
    fn scan(&self, data: &[u8], source: &str) -> Result<Scan, Diagnostic> {
        let count = usize::from(
            *data
                .first()
                .ok_or_else(|| invalid(source, "Empty JPEG scan header"))?,
        );
        if !(1..=3).contains(&count) || data.len() != 1 + 2 * count + 3 {
            return Err(invalid(
                source,
                "Invalid JPEG scan component count or length",
            ));
        }
        let mut components = Vec::with_capacity(count);
        for entry in data[1..1 + 2 * count].chunks_exact(2) {
            let component = self
                .ids
                .iter()
                .position(|id| *id == entry[0])
                .ok_or_else(|| invalid(source, "Unknown JPEG scan component"))?;
            if components
                .iter()
                .any(|entry: &ScanComponent| entry.component == component)
            {
                return Err(invalid(source, "Duplicate JPEG scan component"));
            }
            let (dc, ac) = (usize::from(entry[1] >> 4), usize::from(entry[1] & 15));
            if dc > 3 || ac > 3 {
                return Err(invalid(source, "JPEG Huffman table selector exceeds three"));
            }
            components.push(ScanComponent { component, dc, ac });
        }
        let tail = &data[1 + 2 * count..];
        let (start, end, high, low) = (
            usize::from(tail[0]),
            usize::from(tail[1]),
            tail[2] >> 4,
            tail[2] & 15,
        );
        if start > end || end > 63 || high > 13 || low > 13 {
            return Err(invalid(
                source,
                "Invalid JPEG spectral or approximation bounds",
            ));
        }
        if !self.progressive && (start != 0 || end != 63 || high != 0 || low != 0) {
            return Err(invalid(
                source,
                "Baseline JPEG requires complete sequential scans",
            ));
        }
        if self.progressive
            && ((start == 0 && end != 0)
                || (start != 0 && count != 1)
                || (high != 0 && high != low + 1))
        {
            return Err(invalid(
                source,
                "Invalid progressive JPEG scan shape or refinement order",
            ));
        }
        for entry in &components {
            for level in &self.levels[entry.component][start..=end] {
                if (high == 0 && level.is_some()) || (high != 0 && *level != Some(high)) {
                    return Err(invalid(
                        source,
                        "JPEG coefficient scans overlap or refine uninitialized precision",
                    ));
                }
            }
            if start != 0 && self.levels[entry.component][0].is_none() {
                return Err(invalid(source, "JPEG AC scan precedes its DC scan"));
            }
        }
        Ok(Scan {
            components,
            start,
            end,
            high,
            low,
        })
    }
}

fn signed(bits: u32, count: u8) -> i32 {
    if count == 0 {
        0
    } else if bits < 1 << (count - 1) {
        bits as i32 - ((1 << count) - 1)
    } else {
        bits as i32
    }
}
fn dc(
    bits: &mut Bits<'_>,
    table: &Huffman,
    predictor: &mut i32,
    low: u8,
) -> Result<(), Diagnostic> {
    let category = table.symbol(bits)?;
    if category > 11 {
        return Err(invalid(bits.source, "Invalid 8-bit JPEG DC magnitude"));
    }
    let value = *predictor + signed(bits.read(category)?, category);
    if i16::try_from(value * (1 << low)).is_err() {
        return Err(invalid(
            bits.source,
            "JPEG DC coefficient exceeds signed 16-bit storage",
        ));
    }
    *predictor = value;
    Ok(())
}
fn initial_ac(
    bits: &mut Bits<'_>,
    table: &Huffman,
    mask: &mut u64,
    start: usize,
    end: usize,
    eob: &mut u32,
    progressive: bool,
    low: u8,
) -> Result<(), Diagnostic> {
    if *eob != 0 {
        *eob -= 1;
        return Ok(());
    }
    let mut k = start;
    while k <= end {
        let symbol = table.symbol(bits)?;
        let (run, size) = (usize::from(symbol >> 4), symbol & 15);
        if size == 0 {
            if run == 15 {
                k += 16;
                if k > end + 1 {
                    return Err(invalid(
                        bits.source,
                        "JPEG zero run exceeds its spectral band",
                    ));
                }
            } else {
                if !progressive && run != 0 {
                    return Err(invalid(
                        bits.source,
                        "Baseline JPEG cannot contain progressive EOB runs",
                    ));
                }
                *eob = (1 << run) + bits.read(run as u8)? - 1;
                break;
            }
        } else {
            if size > 10 {
                return Err(invalid(bits.source, "Invalid 8-bit JPEG AC magnitude"));
            }
            k += run;
            if k > end {
                return Err(invalid(
                    bits.source,
                    "JPEG AC coefficient exceeds its spectral band",
                ));
            }
            let value = signed(bits.read(size)?, size).unsigned_abs();
            // The decoder stores signed i16 coefficients. Reserve all remaining
            // refinement bits, so no admitted nonzero can wrap to zero or later
            // overflow. Presence alone then suffices for future bit consumption.
            if value * (1 << low) + ((1 << low) - 1) > i16::MAX as u32 {
                return Err(invalid(
                    bits.source,
                    "JPEG AC coefficient refinement exceeds signed 16-bit storage",
                ));
            }
            *mask |= 1 << k;
            k += 1;
        }
    }
    Ok(())
}
fn refine_rest(bits: &mut Bits<'_>, mask: u64, start: usize, end: usize) -> Result<(), Diagnostic> {
    for k in start..=end {
        if mask & (1 << k) != 0 {
            bits.read(1)?;
        }
    }
    Ok(())
}
fn refine_ac(
    bits: &mut Bits<'_>,
    table: &Huffman,
    mask: &mut u64,
    start: usize,
    end: usize,
    eob: &mut u32,
) -> Result<(), Diagnostic> {
    if *eob != 0 {
        *eob -= 1;
        return refine_rest(bits, *mask, start, end);
    }
    let mut k = start;
    while k <= end {
        let symbol = table.symbol(bits)?;
        let (mut run, size) = (usize::from(symbol >> 4), symbol & 15);
        if size == 0 && run != 15 {
            *eob = (1 << run) + bits.read(run as u8)? - 1;
            return refine_rest(bits, *mask, k, end);
        }
        if size > 1 {
            return Err(invalid(
                bits.source,
                "JPEG AC refinement can introduce only one-bit coefficients",
            ));
        }
        if size == 1 {
            bits.read(1)?;
        } // New coefficient's sign precedes old-coefficient refinements.
        loop {
            if k > end {
                return Err(invalid(
                    bits.source,
                    "JPEG refinement zero run exceeds its spectral band",
                ));
            }
            if *mask & (1 << k) != 0 {
                bits.read(1)?;
            } else if run == 0 {
                break;
            } else {
                run -= 1;
            }
            k += 1;
        }
        if size == 1 {
            *mask |= 1 << k;
        }
        // For F0 this is the sixteenth zero. It stays zero, but is consumed.
        k += 1;
    }
    Ok(())
}

pub(super) fn validate(bytes: &[u8], source: &str) -> Result<(), Diagnostic> {
    if !bytes.starts_with(&[0xff, 0xd8]) {
        return Err(invalid(source, "JPEG is missing SOI"));
    }
    let mut pos = 2usize;
    let mut frame = None;
    let mut tables: [[Option<Huffman>; 4]; 2] =
        std::array::from_fn(|_| std::array::from_fn(|_| None));
    let mut restart_interval = 0usize;
    let mut scans = 0usize;
    while pos < bytes.len() {
        if bytes[pos] != 0xff {
            return Err(invalid(
                source,
                "Expected JPEG marker after validated entropy",
            ));
        }
        while bytes.get(pos) == Some(&0xff) {
            pos += 1;
        }
        let marker = *bytes
            .get(pos)
            .ok_or_else(|| invalid(source, "Truncated JPEG marker"))?;
        pos += 1;
        if marker == 0xd9 {
            let frame: &Frame = frame
                .as_ref()
                .ok_or_else(|| invalid(source, "JPEG has no frame"))?;
            if pos != bytes.len() || scans == 0 {
                return Err(invalid(source, "JPEG requires exact EOI after its scans"));
            }
            if frame.levels.iter().flatten().any(|level| *level != Some(0)) {
                return Err(invalid(
                    source,
                    "JPEG ends before every component coefficient has complete precision",
                ));
            }
            return Ok(());
        }
        let length = bytes
            .get(pos..pos.saturating_add(2))
            .ok_or_else(|| invalid(source, "Truncated JPEG segment length"))?;
        let length = usize::from(u16::from_be_bytes([length[0], length[1]]));
        let end = pos
            .checked_add(length)
            .filter(|end| length >= 2 && *end <= bytes.len())
            .ok_or_else(|| invalid(source, "Truncated JPEG segment"))?;
        let data = &bytes[pos + 2..end];
        pos = end;
        match marker {
            0xe0 | 0xdb => {} // The enclosing container screen and portable decoder validate these.
            0xc0 | 0xc2 => {
                if frame.is_some() {
                    return Err(invalid(source, "Multiple JPEG frames are not admitted"));
                }
                frame = Some(Frame::new(data, marker == 0xc2, source)?);
            }
            0xc4 => {
                let mut offset = 0usize;
                while offset < data.len() {
                    let header = data
                        .get(offset..offset.saturating_add(17))
                        .ok_or_else(|| invalid(source, "Truncated JPEG Huffman table"))?;
                    let (class, id) = (usize::from(header[0] >> 4), usize::from(header[0] & 15));
                    if class > 1 || id > 3 {
                        return Err(invalid(source, "Invalid JPEG Huffman table identifier"));
                    }
                    let counts: [u8; 16] = header[1..].try_into().expect("checked table counts");
                    let count: usize = counts.iter().map(|count| usize::from(*count)).sum();
                    let finish = offset + 17 + count;
                    let values = data
                        .get(offset + 17..finish)
                        .ok_or_else(|| invalid(source, "Truncated JPEG Huffman symbols"))?;
                    tables[class][id] = Some(Huffman::new(counts, values.to_vec(), source)?);
                    offset = finish;
                }
            }
            0xdd => {
                if data.len() != 2 {
                    return Err(invalid(source, "Invalid JPEG restart interval"));
                }
                restart_interval = usize::from(u16::from_be_bytes([data[0], data[1]]));
            }
            0xda => {
                scans += 1;
                if scans > 64 {
                    return Err(unsupported(
                        source,
                        "JPEG entropy validation currently admits at most 64 scans",
                    ));
                }
                let frame = frame
                    .as_mut()
                    .ok_or_else(|| invalid(source, "JPEG scan precedes its frame"))?;
                let scan = frame.scan(data, source)?;
                let mut bits = Bits {
                    bytes,
                    source,
                    pos,
                    byte: 0,
                    remaining: 0,
                };
                let mut eob = 0u32;
                let mut restart = 0u8;
                let mut predictors = [0i32; 3];
                for block in 0..frame.blocks {
                    if block > 0 && restart_interval != 0 && block % restart_interval == 0 {
                        if eob != 0 {
                            return Err(invalid(source, "JPEG EOB run crosses a restart boundary"));
                        }
                        bits.restart(restart)?;
                        restart = (restart + 1) & 7;
                        predictors = [0; 3];
                    }
                    for component in &scan.components {
                        let mask = &mut frame.nonzero[component.component][block];
                        if scan.start == 0 {
                            if scan.high == 0 {
                                dc(
                                    &mut bits,
                                    tables[0][component.dc].as_ref().ok_or_else(|| {
                                        invalid(source, "Missing JPEG DC Huffman table")
                                    })?,
                                    &mut predictors[component.component],
                                    scan.low,
                                )?;
                            } else {
                                bits.read(1)?;
                            }
                        }
                        if scan.end != 0 {
                            let table = tables[1][component.ac]
                                .as_ref()
                                .ok_or_else(|| invalid(source, "Missing JPEG AC Huffman table"))?;
                            if scan.high == 0 {
                                initial_ac(
                                    &mut bits,
                                    table,
                                    mask,
                                    scan.start.max(1),
                                    scan.end,
                                    &mut eob,
                                    frame.progressive,
                                    scan.low,
                                )?;
                            } else {
                                refine_ac(&mut bits, table, mask, scan.start, scan.end, &mut eob)?;
                            }
                        }
                    }
                }
                if eob != 0 {
                    return Err(invalid(source, "JPEG EOB run exceeds its declared blocks"));
                }
                pos = bits.finish()?;
                for component in &scan.components {
                    frame.levels[component.component][scan.start..=scan.end].fill(Some(scan.low));
                }
            }
            _ => {
                return Err(unsupported(
                    source,
                    "JPEG marker is outside the validated entropy profile",
                ));
            }
        }
    }
    Err(invalid(source, "JPEG is missing its EOI marker"))
}

#[cfg(test)]
mod tests {
    use super::*;
    const BASELINE: &[u8] = include_bytes!("../fixtures/images/ordinary-r1/baseline.jpg");
    const PROGRESSIVE: &[u8] = include_bytes!("../fixtures/images/ordinary-r1/progressive.jpg");

    #[test]
    fn supplied_baseline_and_progressive_scans_finish_without_synthetic_bits() {
        for bytes in [BASELINE, PROGRESSIVE] {
            validate(bytes, "src").unwrap();
        }
    }

    #[test]
    fn missing_and_truncated_entropy_never_borrows_bits_from_markers() {
        for bytes in [BASELINE, PROGRESSIVE] {
            let sos = bytes
                .windows(2)
                .position(|bytes| bytes == [0xff, 0xda])
                .unwrap();
            let start = sos + 2 + usize::from(u16::from_be_bytes([bytes[sos + 2], bytes[sos + 3]]));
            for retained in [0, 1, 2, 4] {
                let mut truncated = bytes[..start + retained].to_vec();
                truncated.extend_from_slice(&[0xff, 0xd9]);
                assert_eq!(
                    validate(&truncated, "src").unwrap_err().code,
                    "invalid-image"
                );
            }
        }
    }

    #[test]
    fn byte_reader_rejects_markers_zero_padding_and_bad_restart_order() {
        let mut bits = Bits {
            bytes: &[0xff, 0, 0x7f, 0xff, 0xd0],
            source: "src",
            pos: 0,
            byte: 0,
            remaining: 0,
        };
        assert_eq!(bits.read(8).unwrap(), 255);
        assert_eq!(bits.read(1).unwrap(), 0);
        bits.restart(0).unwrap();
        assert_eq!(bits.pos, 5);
        assert!(bits.read(1).is_err());
        let mut bad = Bits {
            bytes: &[0, 0xff, 0xd1],
            source: "src",
            pos: 0,
            byte: 0,
            remaining: 0,
        };
        bad.read(1).unwrap();
        assert!(bad.padding().is_err());
        let mut bad = Bits {
            bytes: &[0xff, 0xd1],
            source: "src",
            pos: 0,
            byte: 0,
            remaining: 0,
        };
        assert!(bad.restart(0).is_err());
    }

    fn segment(marker: u8, data: &[u8]) -> Vec<u8> {
        let mut out = vec![0xff, marker];
        out.extend_from_slice(&((data.len() + 2) as u16).to_be_bytes());
        out.extend_from_slice(data);
        out
    }
    fn header(progressive: bool, width: u16) -> Vec<u8> {
        let mut bytes = vec![0xff, 0xd8];
        let [w1, w2] = width.to_be_bytes();
        bytes.extend(segment(
            if progressive { 0xc2 } else { 0xc0 },
            &[8, 0, 8, w1, w2, 3, 1, 0x11, 0, 2, 0x11, 0, 3, 0x11, 0],
        ));
        // DC: category0 => 0. AC: EOB=>0, EOB-run=>10, new1=>110, ZRL=>1110.
        let mut tables = vec![0];
        tables.extend_from_slice(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        tables.push(0);
        tables.push(0x10);
        tables.extend_from_slice(&[1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        tables.extend_from_slice(&[0, 0x10, 1, 0xf0]);
        bytes.extend(segment(0xc4, &tables));
        bytes
    }
    fn scan(
        bytes: &mut Vec<u8>,
        ids: &[u8],
        start: u8,
        end: u8,
        approximation: u8,
        entropy: &[u8],
    ) {
        let mut header = vec![ids.len() as u8];
        for id in ids {
            header.extend_from_slice(&[*id, 0]);
        }
        header.extend_from_slice(&[start, end, approximation]);
        bytes.extend(segment(0xda, &header));
        bytes.extend_from_slice(entropy);
    }
    fn finish(mut bytes: Vec<u8>) -> Vec<u8> {
        bytes.extend_from_slice(&[0xff, 0xd9]);
        bytes
    }

    #[test]
    fn synthetic_sequential_mcus_require_exact_entropy_and_restarts() {
        // Three DC0+EOB pairs per MCU => six zero bits, then two one padding bits.
        let mut good = header(false, 16);
        good.extend(segment(0xdd, &[0, 1]));
        scan(&mut good, &[1, 2, 3], 0, 63, 0, &[0x03, 0xff, 0xd0, 0x03]);
        let good = finish(good);
        validate(&good, "src").unwrap();
        let restart = good.windows(2).position(|b| b == [0xff, 0xd0]).unwrap();
        let mut wrong = good.clone();
        wrong[restart + 1] = 0xd1;
        assert!(validate(&wrong, "src").is_err());
        let mut missing = good.clone();
        missing.drain(restart..restart + 2);
        assert!(validate(&missing, "src").is_err());
        let mut bad_padding = good.clone();
        bad_padding[restart - 1] = 0;
        assert!(validate(&bad_padding, "src").is_err());
        let mut extra = good.clone();
        extra.insert(extra.len() - 2, 0);
        assert!(validate(&extra, "src").is_err());
    }

    #[test]
    fn synthetic_progressive_eob_runs_cannot_cross_scan_or_restart_bounds() {
        let create = |run_bits: u8, restart: bool| {
            let mut bytes = header(true, 16);
            scan(&mut bytes, &[1, 2, 3], 0, 0, 0, &[0x03]); // Two MCUs, six DC0 symbols.
            if restart {
                bytes.extend(segment(0xdd, &[0, 1]));
            }
            for id in [1, 2, 3] {
                scan(&mut bytes, &[id], 1, 63, 0, &[run_bits]);
            }
            finish(bytes)
        };
        validate(&create(0x9f, false), "src").unwrap(); // 10 + 0 => EOB for two blocks.
        assert!(validate(&create(0xbf, false), "src").is_err()); // 10 + 1 => three blocks.
        assert!(validate(&create(0x9f, true), "src").is_err());
    }

    #[test]
    fn progressive_refinement_consumes_existing_coefficients_and_new_sign_bits() {
        let mut bytes = header(true, 8);
        scan(&mut bytes, &[1, 2, 3], 0, 0, 0, &[0x1f]); // Three DC0 symbols.
        scan(&mut bytes, &[1], 1, 63, 1, &[0xd7]); // 110 (new1), sign1, EOB0, padding111.
        for id in [2, 3] {
            scan(&mut bytes, &[id], 1, 63, 0, &[0x7f]);
        }
        // new1 code110, sign0, refine coefficient1 with1, then EOB0; padding11.
        scan(&mut bytes, &[1], 1, 63, 0x10, &[0xcb]);
        validate(&finish(bytes.clone()), "src").unwrap();
        // A repeated first scan cannot silently overwrite initialized precision.
        scan(&mut bytes, &[1], 1, 63, 0, &[0x7f]);
        assert!(validate(&finish(bytes), "src").is_err());

        let counts = [1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let table = Huffman::new(counts, vec![0, 0x10, 1, 0xf0], "src").unwrap();
        let mut bits = Bits {
            bytes: &[0xcb, 0xff, 0xd9],
            source: "src",
            pos: 0,
            byte: 0,
            remaining: 0,
        };
        let mut mask = 1 << 1;
        refine_ac(&mut bits, &table, &mut mask, 1, 63, &mut 0).unwrap();
        assert_eq!(mask, (1 << 1) | (1 << 2));
        assert_eq!(bits.finish().unwrap(), 1);
    }

    #[test]
    fn reserved_huffman_codes_and_unimplemented_sampling_diagnose() {
        assert!(
            Huffman::new(
                [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                vec![0, 1],
                "src"
            )
            .is_err()
        );
        assert!(Huffman::new([0; 16], vec![], "src").is_err());
        let mut bytes = BASELINE.to_vec();
        let sof = bytes.windows(2).position(|b| b == [0xff, 0xc0]).unwrap();
        bytes[sof + 4 + 7] = 0x22;
        assert_eq!(
            validate(&bytes, "src").unwrap_err().code,
            "unsupported-image"
        );
    }

    #[test]
    fn coefficient_bounds_prevent_nonzero_state_from_wrapping() {
        let counts = [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let dc_table = Huffman::new(counts, vec![11], "src").unwrap();
        // Code0 then eleven one bits => +2047. Baseline decoder wraps this sum.
        let mut bits = Bits {
            bytes: &[0x7f, 0xf0],
            source: "src",
            pos: 0,
            byte: 0,
            remaining: 0,
        };
        assert!(dc(&mut bits, &dc_table, &mut 32760, 0).is_err());
        let ac_table = Huffman::new(counts, vec![10], "src").unwrap();
        // Code0, ten one bits => +1023 at Al6 cannot fit the target coefficient.
        let mut bits = Bits {
            bytes: &[0x7f, 0xe0],
            source: "src",
            pos: 0,
            byte: 0,
            remaining: 0,
        };
        assert!(initial_ac(&mut bits, &ac_table, &mut 0, 1, 63, &mut 0, true, 6).is_err());
    }

    #[test]
    fn truncating_each_real_progressive_scan_retains_later_markers_but_fails() {
        let bytes = PROGRESSIVE;
        let mut pos = 2usize;
        let mut scans = 0usize;
        while bytes[pos + 1] != 0xd9 {
            let marker = bytes[pos + 1];
            let length = usize::from(u16::from_be_bytes([bytes[pos + 2], bytes[pos + 3]]));
            pos += 2 + length;
            if marker != 0xda {
                continue;
            }
            let start = pos;
            while bytes[pos] != 0xff || bytes[pos + 1] == 0 {
                pos += if bytes[pos] == 0xff { 2 } else { 1 };
            }
            let end = pos;
            assert!(end > start);
            for cut in [start, start + (end - start) / 2, end - 1] {
                let mut truncated = bytes[..cut].to_vec();
                truncated.extend_from_slice(&bytes[end..]);
                assert!(
                    validate(&truncated, "src").is_err(),
                    "scan {scans} retained {}/{} bytes",
                    cut - start,
                    end - start
                );
            }
            scans += 1;
        }
        assert_eq!(scans, 10);
    }
}
