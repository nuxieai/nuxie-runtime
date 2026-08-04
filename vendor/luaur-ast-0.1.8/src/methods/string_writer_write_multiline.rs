use crate::records::string_writer::StringWriter;

#[allow(non_snake_case)]
pub fn string_writer_write_multiline(writer: &mut StringWriter, s: &str) {
    if s.is_empty() {
        return;
    }

    writer.ss.push_str(s);
    if let Some(last) = s.chars().last() {
        writer.last_char = last;
    }

    let mut num_lines = 0;
    let mut last_newline_pos = 0;

    for (i, c) in s.char_indices() {
        if c == '\n' {
            num_lines += 1;
            last_newline_pos = i + 1;
        }
    }

    writer.pos.line += num_lines as u32;
    if num_lines > 0 {
        writer.pos.column = (s.len() - last_newline_pos) as u32;
    } else {
        writer.pos.column += s.len() as u32;
    }
}
