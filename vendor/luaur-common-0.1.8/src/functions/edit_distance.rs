#[allow(non_snake_case)]
pub fn editDistance(mut a: &[u8], mut b: &[u8]) -> usize {
    while !a.is_empty() && !b.is_empty() && a[0] == b[0] {
        a = &a[1..];
        b = &b[1..];
    }

    while !a.is_empty() && !b.is_empty() && a[a.len() - 1] == b[b.len() - 1] {
        a = &a[..a.len() - 1];
        b = &b[..b.len() - 1];
    }

    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }

    let max_distance = a.len() + b.len();
    let b_stride = b.len() + 2;
    let mut distances = alloc::vec![0; (a.len() + 2) * b_stride];

    let get_pos = |x: usize, y: usize| -> usize { (x * b_stride) + y };

    distances[0] = max_distance;

    for x in 0..=a.len() {
        distances[get_pos(x + 1, 0)] = max_distance;
        distances[get_pos(x + 1, 1)] = x;
    }

    for y in 0..=b.len() {
        distances[get_pos(0, y + 1)] = max_distance;
        distances[get_pos(1, y + 1)] = y;
    }

    let mut seen_char_to_row = [0usize; 256];

    for x in 1..=a.len() {
        let mut last_matched_y = 0;

        for y in 1..=b.len() {
            let b_seen_char_index = b[y - 1] as usize;
            let x1 = seen_char_to_row[b_seen_char_index];
            let y1 = last_matched_y;

            let mut cost = 1;
            if a[x - 1] == b[y - 1] {
                cost = 0;
                last_matched_y = y;
            }

            let transposition = distances[get_pos(x1, y1)] + (x - x1 - 1) + 1 + (y - y1 - 1);
            let substitution = distances[get_pos(x, y)] + cost;
            let insertion = distances[get_pos(x, y + 1)] + 1;
            let deletion = distances[get_pos(x + 1, y)] + 1;

            distances[get_pos(x + 1, y + 1)] = core::cmp::min(
                core::cmp::min(insertion, deletion),
                core::cmp::min(substitution, transposition),
            );
        }

        let a_seen_char_index = a[x - 1] as usize;
        seen_char_to_row[a_seen_char_index] = x;
    }

    distances[get_pos(a.len() + 1, b.len() + 1)]
}
