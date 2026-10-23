use super::super::text_engine::{
    CssLineMetrics, GlyphLine, GlyphRun, LineMetrics, TextAlign, TextRun, TextWordBreak,
};

enum LineRuns<'a> {
    Fixed(&'a [GlyphRun]),
    Tabs {
        runs: &'a mut [GlyphRun],
        run: usize,
        glyph: usize,
        x: f32,
        origin: f32,
        tabs: Vec<(usize, usize)>,
    },
}
impl std::ops::Deref for LineRuns<'_> {
    type Target = [GlyphRun];
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Fixed(runs) => runs,
            Self::Tabs { runs, .. } => runs,
        }
    }
}
impl LineRuns<'_> {
    fn position_to(&mut self, end_run: usize, end_glyph: usize) {
        if let Self::Tabs {
            runs,
            run,
            glyph,
            x,
            origin,
            tabs,
        } = self
        {
            while *run < runs.len() && (*run, *glyph) <= (end_run, end_glyph) {
                runs[*run].xpos[*glyph] = *x;
                if (*run, *glyph) == (end_run, end_glyph) {
                    break;
                }
                if *glyph == runs[*run].glyphs.len() {
                    *run += 1;
                    *glyph = 0;
                    continue;
                }
                if runs[*run].is_css_tab(*glyph) {
                    tabs.push((*run, *glyph));
                }
                crate::css_text::layout_tab(&mut runs[*run], *glyph, *x - *origin);
                *x += runs[*run].advances[*glyph];
                *glyph += 1;
            }
        }
    }

    fn start_line(&mut self, run: usize, glyph: usize) {
        self.position_to(run, glyph);
        if let Self::Tabs {
            runs,
            run: next_run,
            glyph: next_glyph,
            x,
            origin,
            tabs,
        } = self
        {
            *origin = runs[run].xpos[glyph];
            // Words end before tabs. Each tab has its own shaping run, so
            // only the final tab can be inside the synthetic trailing word.
            // Invalidate that one glyph of lookahead if it wraps.
            let first = tabs.partition_point(|&position| position < (run, glyph));
            if let Some(&(run, glyph)) = tabs.get(first) {
                *next_run = run;
                *next_glyph = glyph;
                *x = runs[run].xpos[glyph];
                tabs.truncate(first);
            }
        }
    }
}

fn auto_width(width: f32) -> bool {
    width < 0.0
}

fn compute_line_metrics(
    metrics: &LineMetrics,
    custom_line_height: f32,
    font_size: f32,
    css: Option<CssLineMetrics>,
) -> LineMetrics {
    if let Some(metrics) = css {
        let height = if custom_line_height < 0.0 {
            metrics.ascent + metrics.descent + metrics.line_gap
        } else {
            custom_line_height
        };
        let baseline = metrics.ascent + ((height - metrics.ascent - metrics.descent) / 2.0).floor();
        return LineMetrics {
            ascent: -baseline,
            descent: height - baseline,
            cap_height: 0.0,
            x_height: 0.0,
        };
    }
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
                    None,
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
                TextAlign::Unknown(_) => {}
            }
            let _ = line_height;
        }
    }

    pub(crate) fn compute_css_line_spacing(
        lines: &mut [GlyphLine],
        runs: &[GlyphRun],
        width: f32,
        align: TextAlign,
        strut: &TextRun,
    ) {
        let font = strut.font.as_ref().expect("text strut retains its font");
        let strut = compute_line_metrics(
            font.line_metrics(),
            strut.line_height,
            strut.size,
            Some(font.css_line_metrics(strut.size)),
        );
        let mut y = 0.0;
        for line in lines {
            let mut ascent = strut.ascent;
            let mut descent = strut.descent;
            for i in line.start_run_index..=line.end_run_index {
                // The exclusive line end can be the beginning of another run.
                if i == line.end_run_index && line.end_glyph_index == 0 && i != line.start_run_index
                {
                    continue;
                }
                let run = &runs[i as usize];
                let font = run.font.as_ref().expect("shaped text retains its font");
                let metrics = compute_line_metrics(
                    font.line_metrics(),
                    run.line_height,
                    run.size,
                    Some(font.css_line_metrics(run.size)),
                );
                ascent = ascent.min(metrics.ascent);
                descent = descent.max(metrics.descent);
            }
            line.top = y;
            line.baseline = y - ascent;
            y += descent - ascent;
            line.bottom = y;

            let line_width = runs[line.end_run_index as usize].xpos[line.end_glyph_index as usize]
                - runs[line.start_run_index as usize].xpos[line.start_glyph_index as usize];
            match align {
                TextAlign::Right => line.start_x = width - line_width,
                TextAlign::Left => line.start_x = 0.0,
                TextAlign::Center => line.start_x = width / 2.0 - line_width / 2.0,
                TextAlign::Unknown(_) => {}
            }
        }
    }

    /// Source default argument: TextWordBreak::breakWord.
    pub fn break_lines(runs: &[GlyphRun], width: f32) -> Vec<GlyphLine> {
        Self::break_lines_with_word_break(runs, width, TextWordBreak::BreakWord)
    }

    pub fn break_lines_with_word_break(
        runs: &[GlyphRun],
        width: f32,
        word_break: TextWordBreak,
    ) -> Vec<GlyphLine> {
        Self::break_lines_impl(&mut LineRuns::Fixed(runs), width, word_break)
    }

    pub(crate) fn break_lines_with_tabs(
        runs: &mut [GlyphRun],
        width: f32,
        word_break: TextWordBreak,
    ) -> Vec<GlyphLine> {
        let x = runs.first().map_or(0.0, |run| run.xpos[0]);
        Self::break_lines_impl(
            &mut LineRuns::Tabs {
                runs,
                run: 0,
                glyph: 0,
                x,
                origin: x,
                tabs: Vec::new(),
            },
            width,
            word_break,
        )
    }

    fn break_lines_impl(
        runs: &mut LineRuns<'_>,
        width: f32,
        word_break: TextWordBreak,
    ) -> Vec<GlyphLine> {
        let max_line_width = if auto_width(width) { f32::MAX } else { width };
        let mut lines = Vec::new();
        if runs.is_empty() {
            return lines;
        }

        runs.start_line(0, 0);
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
        runs.position_to(break_run, break_index as usize);
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
                runs.position_to(break_run, break_index as usize);
                x = runs[end.run].xpos[break_index as usize];
            }

            let is_forced_break = break_run == start_break_run && break_index == start_break_index;
            let start_run_index = start.run as u32;
            // A word already alone on its line cannot be knocked further down.
            let word_starts_line = line.start_glyph_index == start_break_index
                && line.start_run_index == start_run_index;
            // Normal treats an overflowing lone word as fitting, allowing the
            // word iterator to advance instead of rebuilding an empty line.
            let overflows = !is_forced_break
                && x > limit
                && !(word_break == TextWordBreak::Normal && word_starts_line);
            if overflows {
                let mut split_inside_word = word_starts_line;
                if !split_inside_word && word_break == TextWordBreak::BreakAll {
                    // At least one cluster must fit; otherwise the unbounded
                    // walk-back could cut inside the whitespace before this word.
                    let mut first_cluster = RunIterator::new(runs, start.run, start_break_index);
                    split_inside_word = first_cluster.forward() && first_cluster.x() <= limit;
                }
                if split_inside_word {
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
                            runs.start_line(
                                line.start_run_index as usize,
                                line.start_glyph_index as usize,
                            );
                            runs.position_to(break_run, break_index as usize);
                            x = runs[break_run].xpos[break_index as usize];
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
                    runs.start_line(start_run_index as usize, start_break_index as usize);
                    runs.position_to(break_run, break_index as usize);
                    x = runs[break_run].xpos[break_index as usize];
                }
            } else {
                line.end_run_index = end.run as u32;
                line.end_glyph_index = runs[end.run].breaks[end.index as usize];
                advance_word = true;
                if is_forced_break {
                    lines.push(line.clone());
                    runs.position_to(start.run, (start_break_index + 1) as usize);
                    let start_x = runs[start.run].xpos[(start_break_index + 1) as usize];
                    limit = start_x + max_line_width;
                    line = GlyphLine::at(start.run as u32, start_break_index + 1);
                    runs.start_line(start.run, (start_break_index + 1) as usize);
                }
            }
        }

        if !line.empty() {
            lines.push(line);
        }
        let last = runs.len() - 1;
        runs.position_to(last, runs[last].glyphs.len());
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
        while self.index > 0
            && run.text_indices[self.index as usize] == run.text_indices[self.index as usize - 1]
        {
            self.index -= 1;
        }
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

        while self.index > 0
            && (self.index as usize) < self.runs[self.run].glyphs.len()
            && self.runs[self.run].text_indices[self.index as usize]
                == self.runs[self.run].text_indices[self.index as usize - 1]
        {
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
