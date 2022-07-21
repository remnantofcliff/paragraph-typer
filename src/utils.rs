use unicode_width::UnicodeWidthStr;

pub fn word_wrap_keep_space(text: &str, width: usize) -> Box<[&str]> {
    let mut last_space = 0;
    let mut line_start = 0;
    let mut result = Vec::with_capacity(text.chars().count() / width + 1);
    for (i, c) in text.char_indices() {
        if text[line_start..i].width() == width {
            result.push(&text[line_start..=last_space]);
            line_start = last_space + 1;
        }
        if c == ' ' {
            last_space = i;
        }
    }
    result.push(&text[line_start..]);
    result.into_boxed_slice()
}

pub fn count_spaces(string: &str) -> usize {
    string.chars().filter(|c| *c == ' ').count()
}

#[cfg(test)]
mod tests {
    use crate::utils::word_wrap_keep_space;

    use super::count_spaces;

    #[test]
    fn word_wrap_test() {
        let text = "Hello this å å å is some text";
        let wrapped = word_wrap_keep_space(text, 10);
        assert_eq!(wrapped[0], "Hello ");
        assert_eq!(wrapped[1], "this å å ");
        assert_eq!(wrapped[2], "å is some ");
        assert_eq!(wrapped[3], "text");
        let text = "Döëŝ o ţḩĩş ẃöŗk";
        let wrapped = word_wrap_keep_space(text, 7);
        assert_eq!(wrapped[0], "Döëŝ o ");
        assert_eq!(wrapped[1], "ţḩĩş ");
        assert_eq!(wrapped[2], "ẃöŗk");
    }
    #[test]
    fn count_spaces_test() {
        let text = " a b c d hello WhatAA ö ö ååä ";
        assert_eq!(count_spaces(text), 10)
    }
}
