//! Literal replay of tests/include/pointer_log_replay.hpp at d97f3547.
//! The recorder's small JSON parser intentionally differs from general JSON.

use crate::{Action, ActionTarget, PointerCoordinate};

#[derive(Debug)]
enum Json {
    Null,
    Boolean(bool),
    Number(f64),
    String(Vec<u8>),
    Array(Vec<Json>),
    Object(Vec<(Vec<u8>, Json)>),
}

impl Json {
    fn find(&self, key: &str) -> Option<&Self> {
        match self {
            Self::Object(object) => object
                .iter()
                .find(|(name, _)| name == key.as_bytes())
                .map(|(_, value)| value),
            _ => None,
        }
    }

    fn number_or(&self, fallback: f64) -> f64 {
        match self {
            Self::Number(value) => *value,
            _ => fallback,
        }
    }

    #[allow(dead_code)]
    fn bool_or(&self, fallback: bool) -> bool {
        match self {
            Self::Boolean(value) => *value,
            _ => fallback,
        }
    }

    fn string_or<'a>(&'a self, fallback: &'a [u8]) -> &'a [u8] {
        match self {
            Self::String(value) => value,
            _ => fallback,
        }
    }
}

struct Parser<'a> {
    text: &'a [u8],
    position: usize,
}

impl<'a> Parser<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            text: text.as_bytes(),
            position: 0,
        }
    }

    fn parse(&mut self) -> Option<Json> {
        self.skip_ws();
        let value = self.parse_value()?;
        self.skip_ws();
        // Upstream accepts a parsed value without requiring end-of-input.
        Some(value)
    }

    fn at_end(&self) -> bool {
        self.position >= self.text.len()
    }

    fn peek(&self) -> u8 {
        self.text.get(self.position).copied().unwrap_or(0)
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), b' ' | b'\t' | b'\n' | b'\r') {
            self.position += 1;
        }
    }

    fn expect(&mut self, value: u8) -> Option<()> {
        if self.peek() != value {
            return None;
        }
        self.position += 1;
        Some(())
    }

    fn parse_value(&mut self) -> Option<Json> {
        self.skip_ws();
        match self.peek() {
            b'{' => self.parse_object(),
            b'[' => self.parse_array(),
            b'"' => self.parse_string().map(Json::String),
            b't' | b'f' => self.parse_bool(),
            b'n' => self.parse_null(),
            _ => self.parse_number(),
        }
    }

    fn parse_object(&mut self) -> Option<Json> {
        self.expect(b'{')?;
        let mut object = Vec::new();
        self.skip_ws();
        if self.peek() == b'}' {
            self.position += 1;
            return Some(Json::Object(object));
        }
        loop {
            self.skip_ws();
            let key = self.parse_string()?;
            self.skip_ws();
            self.expect(b':')?;
            let value = self.parse_value()?;
            object.push((key, value));
            self.skip_ws();
            if self.peek() == b',' {
                self.position += 1;
                continue;
            }
            self.expect(b'}')?;
            return Some(Json::Object(object));
        }
    }

    fn parse_array(&mut self) -> Option<Json> {
        self.expect(b'[')?;
        let mut array = Vec::new();
        self.skip_ws();
        if self.peek() == b']' {
            self.position += 1;
            return Some(Json::Array(array));
        }
        loop {
            array.push(self.parse_value()?);
            self.skip_ws();
            if self.peek() == b',' {
                self.position += 1;
                continue;
            }
            self.expect(b']')?;
            return Some(Json::Array(array));
        }
    }

    fn parse_string(&mut self) -> Option<Vec<u8>> {
        self.expect(b'"')?;
        let mut string = Vec::new();
        while !self.at_end() {
            let value = self.peek();
            self.position += 1;
            if value == b'"' {
                return Some(string);
            }
            if value == b'\\' {
                if self.at_end() {
                    break;
                }
                let escaped = self.peek();
                self.position += 1;
                string.push(match escaped {
                    b'"' => b'"',
                    b'\\' => b'\\',
                    b'/' => b'/',
                    b'n' => b'\n',
                    b't' => b'\t',
                    b'r' => b'\r',
                    b'b' => 8,
                    b'f' => 12,
                    b'u' => {
                        // Upstream skips four bytes without decoding or
                        // checking hex digits; the recorder never emits this.
                        self.position += 4.min(self.text.len() - self.position);
                        b'?'
                    }
                    other => other,
                });
            } else {
                string.push(value);
            }
        }
        None
    }

    fn parse_number(&mut self) -> Option<Json> {
        let start = self.position;
        if self.peek() == b'-' {
            self.position += 1;
        }
        let mut any = false;
        while !self.at_end() {
            if matches!(self.peek(), b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-') {
                any = true;
                self.position += 1;
            } else {
                break;
            }
        }
        if !any {
            return None;
        }
        // std::strtod accepts the valid numeric prefix of the scanned token
        // and yields zero when no conversion is possible. Preserve that
        // behavior without passing an unbounded pointer into a C parser.
        let token = &self.text[start..self.position];
        Some(Json::Number(strtod_decimal_prefix(token)))
    }

    fn parse_bool(&mut self) -> Option<Json> {
        if self.match_literal(b"true") {
            Some(Json::Boolean(true))
        } else if self.match_literal(b"false") {
            Some(Json::Boolean(false))
        } else {
            None
        }
    }

    fn parse_null(&mut self) -> Option<Json> {
        self.match_literal(b"null").then_some(Json::Null)
    }

    fn match_literal(&mut self, literal: &[u8]) -> bool {
        if self.text[self.position..].starts_with(literal) {
            self.position += literal.len();
            true
        } else {
            false
        }
    }
}

fn strtod_decimal_prefix(token: &[u8]) -> f64 {
    let mut end = usize::from(matches!(token.first(), Some(b'+' | b'-')));
    let mut digits = 0;
    while token.get(end).is_some_and(u8::is_ascii_digit) {
        end += 1;
        digits += 1;
    }
    if token.get(end) == Some(&b'.') {
        end += 1;
        while token.get(end).is_some_and(u8::is_ascii_digit) {
            end += 1;
            digits += 1;
        }
    }
    if digits == 0 {
        return 0.0;
    }
    if matches!(token.get(end), Some(b'e' | b'E')) {
        let exponent = end;
        end += 1;
        if matches!(token.get(end), Some(b'+' | b'-')) {
            end += 1;
        }
        let start_digits = end;
        while token.get(end).is_some_and(u8::is_ascii_digit) {
            end += 1;
        }
        if end == start_digits {
            end = exponent;
        }
    }
    std::str::from_utf8(&token[..end])
        .expect("number scanner accepts ASCII only")
        .parse()
        .expect("decimal prefix has a valid numeric grammar")
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct PointerLogReplayOptions {
    pub interval: f32,
    pub fps: f32,
    pub stride: i32,
    pub quantize: f32,
}

impl Default for PointerLogReplayOptions {
    fn default() -> Self {
        Self {
            interval: 0.0,
            fps: 0.0,
            stride: 1,
            quantize: 0.001,
        }
    }
}

struct Replay {
    actions: Vec<Action>,
    interval: f32,
    stride: i32,
    quantize: f32,
    group_key: f64,
    group_q: f32,
    have_group: bool,
    group_count: i32,
}

impl Replay {
    fn bucket_for(&self, dt: f64) -> (f64, f32) {
        if self.quantize > 0.0 {
            // C++ divides in double, lrounds to long, then multiplies in float.
            let bucket = ((dt / f64::from(self.quantize)).round() as i64).max(1);
            (bucket as f64, bucket as f32 * self.quantize)
        } else {
            (dt, dt as f32)
        }
    }

    fn advance(&mut self, seconds: f32) {
        self.actions.push(Action::Advance {
            target: ActionTarget::StateMachine,
            seconds,
        });
    }

    fn flush_group(&mut self) {
        if !self.have_group || self.group_count == 0 {
            self.have_group = false;
            self.group_count = 0;
            return;
        }
        let q = self.group_q;
        let step = if self.interval > 0.0 {
            // The division and rounding here use float, unlike bucket_for.
            ((self.interval / q).round() as i64 as i32).max(1)
        } else {
            self.stride
        };
        let periods = self.group_count / step;
        let remainder = self.group_count - periods * step;
        for _ in 0..periods {
            self.actions.push(Action::Frame);
            for _ in 0..step {
                self.advance(q);
            }
            self.actions.push(Action::Draw);
        }
        for _ in 0..remainder {
            self.advance(q);
        }
        self.have_group = false;
        self.group_count = 0;
    }

    fn handle_advance(&mut self, dt: f64) {
        if dt <= 0.0 {
            self.flush_group();
            self.advance(0.0);
            return;
        }
        let (key, q) = self.bucket_for(dt);
        if !self.have_group {
            self.have_group = true;
            self.group_key = key;
            self.group_q = q;
            self.group_count = 1;
        } else if key == self.group_key {
            self.group_count += 1;
        } else {
            self.flush_group();
            self.have_group = true;
            self.group_key = key;
            self.group_q = q;
            self.group_count = 1;
        }
    }
}

pub(crate) fn replay_actions(
    text: &str,
    opts: PointerLogReplayOptions,
) -> anyhow::Result<Vec<Action>> {
    let root = Parser::new(text)
        .parse()
        .ok_or_else(|| anyhow::anyhow!("failed to parse pointer log"))?;
    let entries = root
        .find("entries")
        .ok_or_else(|| anyhow::anyhow!("pointer log is missing entries"))?;
    let Json::Array(entries) = entries else {
        anyhow::bail!("pointer log entries must be an array");
    };
    let mut replay = Replay {
        actions: Vec::new(),
        interval: if opts.fps > 0.0 {
            1.0 / opts.fps
        } else {
            opts.interval
        },
        stride: opts.stride.max(1),
        quantize: opts.quantize,
        group_key: 0.0,
        group_q: 0.0,
        have_group: false,
        group_count: 0,
    };
    for entry in entries {
        let Some(op) = entry.find("op") else {
            continue;
        };
        let number = |key| entry.find(key).map_or(0.0, |value| value.number_or(0.0));
        match op.string_or(b"") {
            b"pointer" => {
                replay.flush_group();
                let pointer_type = entry
                    .find("type")
                    .map_or(b"".as_slice(), |v| v.string_or(b""));
                let x = PointerCoordinate::Literal(number("x") as f32);
                let y = PointerCoordinate::Literal(number("y") as f32);
                let pointer_id = number("id") as i32;
                let action = match pointer_type {
                    b"down" => Some(Action::PointerDown { x, y, pointer_id }),
                    b"up" => Some(Action::PointerUp { x, y, pointer_id }),
                    b"move" => Some(Action::PointerMove {
                        x,
                        y,
                        seconds: 0.0,
                        pointer_id,
                    }),
                    b"exit" => Some(Action::PointerExit { x, y, pointer_id }),
                    _ => None,
                };
                if let Some(action) = action {
                    replay.actions.push(action);
                }
            }
            b"advance" => replay.handle_advance(number("dt")),
            b"multiAdvance" => {
                let dt = number("dt");
                let count = number("count") as i64;
                for _ in 0..count {
                    replay.handle_advance(dt);
                }
            }
            _ => {}
        }
    }
    replay.flush_group();
    Ok(replay.actions)
}
