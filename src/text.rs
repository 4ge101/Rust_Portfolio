use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub fn width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

pub fn wrap(text: &str, max: usize) -> Vec<String> {
    let max = max.max(1);
    let mut lines = Vec::new();

    for paragraph in text.split('\n') {
        let mut current = String::new();
        let mut current_width = 0;

        for word in paragraph.split_whitespace() {
            let word_width = width(word);

            if word_width > max {
                if current_width > 0 {
                    lines.push(std::mem::take(&mut current));
                    current_width = 0;
                }
                for piece in hard_split(word, max) {
                    lines.push(piece);
                }

                if let Some(last) = lines.pop() {
                    current_width = width(&last);
                    current = last;
                }
                continue;
            }

            let needed = if current_width == 0 {
                word_width
            } else {
                current_width + 1 + word_width
            };
            if needed <= max {
                if current_width > 0 {
                    current.push(' ');
                }
                current.push_str(word);
                current_width = needed;
            } else {
                lines.push(std::mem::take(&mut current));
                current.push_str(word);
                current_width = word_width;
            }
        }
        lines.push(current);
    }
    lines
}

fn hard_split(word: &str, max: usize) -> Vec<String> {
    let mut pieces = Vec::new();
    let mut piece = String::new();
    let mut piece_width = 0;
    for ch in word.chars() {
        let w = ch.width().unwrap_or(0);
        if piece_width + w > max && !piece.is_empty() {
            pieces.push(std::mem::take(&mut piece));
            piece_width = 0;
        }
        piece.push(ch);
        piece_width += w;
    }
    if !piece.is_empty() {
        pieces.push(piece);
    }
    pieces
}

pub fn truncate(text: &str, max: usize, ellipsis: &str) -> String {
    if width(text) <= max {
        return text.to_owned();
    }
    let budget = max.saturating_sub(width(ellipsis));
    let mut out = String::new();
    let mut used = 0;
    for ch in text.chars() {
        let w = ch.width().unwrap_or(0);
        if used + w > budget {
            break;
        }
        out.push(ch);
        used += w;
    }
    if max >= width(ellipsis) {
        out.push_str(ellipsis);
    }
    out
}

pub fn pad_right(text: &str, cells: usize) -> String {
    let used = width(text);
    format!("{text}{}", " ".repeat(cells.saturating_sub(used)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_on_word_boundaries() {
        assert_eq!(wrap("one two three four", 9), ["one two", "three", "four"]);
        assert_eq!(wrap("a b", 10), ["a b"]);
    }

    #[test]
    fn keeps_paragraph_breaks() {
        assert_eq!(wrap("a\n\nb", 5), ["a", "", "b"]);
    }

    #[test]
    fn splits_overlong_words() {
        let lines = wrap("abcdefghij", 4);
        assert_eq!(lines, ["abcd", "efgh", "ij"]);
        assert!(lines.iter().all(|line| width(line) <= 4));
    }

    #[test]
    fn continues_after_an_overlong_word() {
        assert_eq!(wrap("abcdefgh x", 5), ["abcde", "fgh x"]);
    }

    #[test]
    fn never_exceeds_width_with_wide_characters() {
        let lines = wrap("日本語のテキストを折り返す テスト", 8);
        assert!(lines.iter().all(|line| width(line) <= 8), "{lines:?}");
    }

    #[test]
    fn empty_input_gives_one_line() {
        assert_eq!(wrap("", 10), [""]);
    }

    #[test]
    fn truncates_with_ellipsis() {
        assert_eq!(truncate("hello world", 8, "…"), "hello w…");
        assert_eq!(truncate("short", 8, "…"), "short");
        assert_eq!(truncate("hello world", 6, "..."), "hel...");
        assert!(width(&truncate("日本語日本語", 5, "…")) <= 5);
    }
}
