pub fn trim_length_left(s: &str, width: usize) -> &str {
    let len = s.len();
    if len > width {
        for i in len - width..len {
            if s.is_char_boundary(i) {
                return &s[i..];
            }
        }
    }

    s
}

pub fn truncate(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        None => s,
        Some((idx, _)) => &s[..idx],
    }
}

#[inline]
pub fn split_by_len(needle: &str, page: usize) -> Vec<String> {
    needle
        .chars()
        .collect::<Vec<char>>()
        .chunks(page)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect()
}
