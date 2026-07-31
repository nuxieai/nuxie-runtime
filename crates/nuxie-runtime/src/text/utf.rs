fn character_index_for_cluster(text: &str, cluster: u32) -> usize {
    let cluster = cluster as usize;
    text.char_indices()
        .take_while(|(byte_index, _)| *byte_index <= cluster)
        .count()
        .saturating_sub(1)
}
fn char_byte_index(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(byte_index, _)| byte_index)
        .unwrap_or(text.len())
}
