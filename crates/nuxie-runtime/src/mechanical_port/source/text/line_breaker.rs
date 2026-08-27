use super::super::text_engine::{GlyphLine, GlyphRun, LineMetrics, TextAlign};

fn auto_width(width: f32) -> bool {
    width < 0.0
}

fn compute_line_metrics(
    metrics: &LineMetrics,
    custom_line_height: f32,
    font_size: f32,
) -> LineMetrics {
    if custom_line_height < 0.0 {
        return LineMetrics {
            ascent: metrics.ascent * font_size,
            descent: metrics.descent * font_size,
            cap_height: 0.0,
            x_height: 0.0,
        };
    }
    let baseline = -metrics.ascent;
    let height = baseline + metrics.descent;
    let baseline_factor = baseline / height;
    let actual_ascent = -baseline_factor * custom_line_height;
    LineMetrics {
        ascent: actual_ascent,
        descent: custom_line_height + actual_ascent,
        cap_height: 0.0,
        x_height: 0.0,
    }
}

impl GlyphLine {
    pub fn compute_max_width(lines: &[GlyphLine], runs: &[GlyphRun]) -> f32 {
        let mut max_line_width = 0.0f32;
        for line in lines {
            max_line_width = max_line_width.max(
                runs[line.end_run_index as usize].xpos[line.end_glyph_index as usize]
                    - runs[line.start_run_index as usize].xpos[line.start_glyph_index as usize],
            );
        }
        max_line_width
    }

    pub fn compute_line_spacing(
        is_first_line: bool,
        lines: &mut [GlyphLine],
        runs: &[GlyphRun],
        width: f32,
        align: TextAlign,
    ) {
        let mut first = is_first_line;
        let mut y = 0.0;
        for line in lines {
            let mut ascent = 0.0f32;
            let mut real_ascent = 0.0f32;
            let mut descent = 0.0f32;
            let mut line_height = 0.0f32;
            for i in line.start_run_index..=line.end_run_index {
                let run = &runs[i as usize];
                let metrics = compute_line_metrics(
                    run.font.as_ref().unwrap().line_metrics(),
                    run.line_height,
                    run.size,
                );
                real_ascent =
                    real_ascent.min(run.font.as_ref().unwrap().line_metrics().ascent * run.size);
                ascent = ascent.min(metrics.ascent);
                descent = descent.max(metrics.descent);
                if run.line_height >= 0.0 {
                    line_height = line_height.max(run.line_height);
                } else {
                    line_height = line_height.max(-ascent + descent);
                }
            }
            line.top = y;
            if first {
                y = -real_ascent;
                first = false;
            } else {
                y -= ascent;
            }
            line.baseline = y;
            y += descent;
            line.bottom = y;

            let line_width = runs[line.end_run_index as usize].xpos[line.end_glyph_index as usize]
                - runs[line.start_run_index as usize].xpos[line.start_glyph_index as usize];
            match align {
                TextAlign::Right => line.start_x = width - line_width,
                TextAlign::Left => line.start_x = 0.0,
                TextAlign::Center => line.start_x = width / 2.0 - line_width / 2.0,
            }
            let _ = line_height;
        }
    }

    pub fn break_lines(runs: &[GlyphRun], width: f32) -> Vec<GlyphLine> {
        let max_line_width = if auto_width(width) { f32::MAX } else { width };
        let mut lines = Vec::new();
        if runs.is_empty() {
            return lines;
        }

        let mut limit = max_line_width;
        let mut advance_word = false;
        let mut start = WordMarker {
            run: 0,
            index: u32::MAX - 1,
        };
        let mut end = WordMarker {
            run: 0,
            index: u32::MAX,
        };
        if !start.next(runs) || !end.next(runs) {
            return lines;
        }

        let mut line = GlyphLine::default();
        let mut break_index = runs[end.run].breaks[end.index as usize];
        let mut break_run = end.run;
        let mut last_end_glyph_index = end.index;
        let mut start_break_index = runs[start.run].breaks[start.index as usize];
        let mut start_break_run = start.run;
        let mut x = runs[end.run].xpos[break_index as usize];

        loop {
            if advance_word {
                last_end_glyph_index = end.index;
                if !start.next(runs) {
                    break;
                }
                if !end.next(runs) {
                    break;
                }
                advance_word = false;
                break_index = runs[end.run].breaks[end.index as usize];
                break_run = end.run;
                start_break_index = runs[start.run].breaks[start.index as usize];
                start_break_run = start.run;
                x = runs[end.run].xpos[break_index as usize];
            }

            let is_forced_break = break_run == start_break_run && break_index == start_break_index;
            if !is_forced_break && x > limit {
                let start_run_index = start.run as u32;
                if line.start_run_index == start_run_index
                    && line.start_glyph_index == start_break_index
                {
                    let mut can_break_more = true;
                    while can_break_more && x > limit {
                        let line_start = RunIterator::new(
                            runs,
                            line.start_run_index as usize,
                            line.start_glyph_index,
                        );
                        let mut line_end = RunIterator::new(
                            runs,
                            end.run,
                            runs[end.run].breaks[end.index as usize],
                        );
                        loop {
                            if !line_end.back() {
                                can_break_more = false;
                                break;
                            } else if line_end.x() <= limit {
                                if line_start == line_end && !line_end.forward() {
                                    can_break_more = false;
                                } else {
                                    line.end_run_index = line_end.run as u32;
                                    line.end_glyph_index = line_end.index;
                                }
                                break;
                            }
                        }
                        if can_break_more {
                            limit = line_end.x() + max_line_width;
                            if !line.empty() {
                                lines.push(line.clone());
                            }
                            line = GlyphLine::at(line_end.run as u32, line_end.index);
                        }
                    }
                } else {
                    let start_x =
                        runs[start.run].xpos[runs[start.run].breaks[start.index as usize] as usize];
                    limit = start_x + max_line_width;
                    if !line.empty() || start.index.wrapping_sub(last_end_glyph_index) > 1 {
                        lines.push(line.clone());
                    }
                    line = GlyphLine::at(start_run_index, start_break_index);
                }
            } else {
                line.end_run_index = end.run as u32;
                line.end_glyph_index = runs[end.run].breaks[end.index as usize];
                advance_word = true;
                if is_forced_break {
                    lines.push(line.clone());
                    let start_x = runs[start.run].xpos
                        [(runs[start.run].breaks[start.index as usize] + 1) as usize];
                    limit = start_x + max_line_width;
                    line = GlyphLine::at(start.run as u32, start_break_index + 1);
                }
            }
        }

        if !line.empty() {
            lines.push(line);
        }
        lines
    }
}

#[derive(Clone, Copy)]
struct WordMarker {
    run: usize,
    index: u32,
}

impl WordMarker {
    fn next(&mut self, runs: &[GlyphRun]) -> bool {
        self.index = self.index.wrapping_add(2);
        while self.index as usize >= runs[self.run].breaks.len() {
            self.index = self.index.wrapping_sub(runs[self.run].breaks.len() as u32);
            self.run += 1;
            if self.run == runs.len() {
                return false;
            }
        }
        true
    }
}

#[derive(Clone, Copy)]
struct RunIterator<'a> {
    runs: &'a [GlyphRun],
    run: usize,
    index: u32,
}

impl PartialEq for RunIterator<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.run == other.run && self.index == other.index
    }
}

impl<'a> RunIterator<'a> {
    fn new(runs: &'a [GlyphRun], run: usize, index: u32) -> Self {
        Self { runs, run, index }
    }

    fn back(&mut self) -> bool {
        if self.index == 0 {
            if self.run == 0 {
                return false;
            }
            self.run -= 1;
            if self.runs[self.run].glyphs.is_empty() {
                self.index = 0;
                return self.back();
            }
            self.index = if self.runs[self.run].glyphs.is_empty() {
                0
            } else {
                self.runs[self.run].glyphs.len() as u32 - 1
            };
        } else {
            self.index -= 1;
        }

        let run = &self.runs[self.run];
        if !run.joiners.is_empty() && run.text_indices[self.index as usize] > 0 {
            let joiners = &run.joiners;
            let word_joiner_index = run.text_indices[self.index as usize] - 1;
            let mut start = 0usize;
            let mut end = joiners.len();
            if word_joiner_index >= joiners[start] && word_joiner_index < joiners[end - 1] {
                while start < end {
                    let mid = (start + end) >> 1;
                    let joiner_candidate = joiners[mid];
                    if joiner_candidate == word_joiner_index {
                        let mut is_joiner = true;
                        let mut current_joiner_index = mid;
                        while is_joiner {
                            if self.index == 0 {
                                return self.back();
                            }
                            if self.runs[self.run].text_indices[self.index as usize - 1]
                                == self.runs[self.run].joiners[current_joiner_index]
                            {
                                self.index -= 1;
                                if current_joiner_index > 0 {
                                    current_joiner_index -= 1;
                                } else {
                                    is_joiner = false;
                                }
                            } else {
                                is_joiner = false;
                            }
                        }
                        return self.back();
                    }
                    if joiner_candidate < word_joiner_index {
                        start = mid + 1;
                    } else {
                        end = mid;
                    }
                }
            }
        }
        true
    }

    fn forward(&mut self) -> bool {
        if self.index as usize == self.runs[self.run].glyphs.len() {
            if self.run == self.runs.len() {
                return false;
            }
            self.run += 1;
            self.index = 0;
            if self.index as usize == self.runs[self.run].glyphs.len() {
                return self.forward();
            }
        } else {
            self.index += 1;
        }

        if !self.runs[self.run].joiners.is_empty()
            && (self.index as usize) < self.runs[self.run].text_indices.len()
        {
            let word_joiner_index = self.runs[self.run].text_indices[self.index as usize];
            let mut start = 0usize;
            let mut end = self.runs[self.run].joiners.len();
            if word_joiner_index >= self.runs[self.run].joiners[start]
                && word_joiner_index < self.runs[self.run].joiners[end - 1]
            {
                while start < end {
                    let mid = (start + end) >> 1;
                    let joiner_candidate = self.runs[self.run].joiners[mid];
                    if joiner_candidate == word_joiner_index {
                        let mut is_joiner = true;
                        let mut current_joiner_index = mid;
                        while is_joiner {
                            if self.index as usize == self.runs[self.run].glyphs.len() {
                                return self.forward();
                            }
                            if current_joiner_index < self.runs[self.run].joiners.len()
                                && self.runs[self.run].text_indices[self.index as usize]
                                    == self.runs[self.run].joiners[current_joiner_index]
                            {
                                self.index += 1;
                                if current_joiner_index < self.runs[self.run].joiners.len() {
                                    current_joiner_index += 1;
                                } else {
                                    is_joiner = false;
                                }
                            } else {
                                is_joiner = false;
                            }
                        }
                        return self.forward();
                    }
                    if joiner_candidate < word_joiner_index {
                        start = mid + 1;
                    } else {
                        end = mid;
                    }
                }
            }
        }
        true
    }

    fn x(&self) -> f32 {
        self.runs[self.run].xpos[self.index as usize]
    }
}
