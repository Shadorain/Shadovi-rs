#![allow(clippy::string_slice)]

use crate::{highlighting, SearchDirection};
use crate::HighlightingOptions;
use std::cmp;
use termion::color;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default)]
pub struct Row {
    string: String,
    highlighting: Vec<highlighting::Type>,
    len: usize,
}

impl From<&str> for Row {
    fn from (slice: &str) -> Self {
        Self {
            string: String::from(slice),
            highlighting: Vec::new(),
            len: slice.graphemes(true).count(),
        }
    }
}

impl Row {
    pub fn render (&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.string.len());
        let start = cmp::min(start, end);
        let mut result = String::new();
        let mut current_highlighting = &highlighting::Type::None;
        #[allow(clippy::integer_arithmetic)]
        for (i, grapheme) in self.string[..].graphemes(true).enumerate()
            .skip(start).take(end - start)
        {
            if let Some(c) = grapheme.chars().next() {
                let highlighting_type = self.highlighting.get(i).unwrap_or(&highlighting::Type::None);
                if highlighting_type != current_highlighting {
                    current_highlighting = highlighting_type;
                    let start_highlight = format!("{}", termion::color::Fg(highlighting_type.to_color()));
                    result.push_str(&start_highlight[..]);
                }
                if c == '\t' { result.push_str("  "); }
                else { result.push(c); }
            }
        }
        let end_highlight = format!("{}", termion::color::Fg(color::Reset));
        result.push_str(&end_highlight[..]);
        result
    }

    pub fn len (&self) -> usize {
        self.len
    }

    pub fn is_empty (&self) -> bool {
        self.len == 0
    }

    pub fn insert (&mut self, at: usize, c: char) {
        if at >= self.len() {
            self.string.push(c);
            self.len += 1;
            return;
        }
        let mut result: String = String::new();
        let mut length = 0;
        for (i, grapheme) in self.string[..].graphemes(true).enumerate() {
            length += 1;
            if i == at {
                length += 1;
                result.push(c);
            }
            result.push_str(grapheme);
        }
        self.len = length;
        self.string = result;
    }

    pub fn delete (&mut self, at: usize) {
        if at >= self.len() { return; }
        let mut result: String = String::new();
        let mut length = 0;
        for (i, grapheme) in self.string[..].graphemes(true).enumerate() {
            if i != at {
                length += 1;
                result.push_str(grapheme);
            }
        }
        self.len = length;
        self.string = result;
    }
    pub fn append (&mut self, new: &Self) {
        self.string = format!("{}{}", self.string, new.string);
        self.len += new.len;
    }
    pub fn split (&mut self, at: usize) -> Self {
        let mut row: String = String::new();
        let mut splitted_row: String = String::new();
        let mut length = 0;
        let mut splitted_length = 0;
        for (i, grapheme) in self.string[..].graphemes(true).enumerate() {
            if i < at {
                length += 1;
                row.push_str(grapheme);
            } else {
                splitted_length += 1;
                splitted_row.push_str(grapheme);
            }
        }
        self.string = row;
        self.len = length;
        Self {
            string: splitted_row,
            highlighting: Vec::new(),
            len: splitted_length,
        }
    }

    pub fn as_bytes (&self) -> &[u8] {
        self.string.as_bytes()
    }

    pub fn find (&self, query: &str, at: usize, direction: SearchDirection) -> Option<usize> {
        if at > self.len() || query.is_empty() { return None; }
        let start = if direction == SearchDirection::Forward { at } else { 0 };
        let end = if direction == SearchDirection::Forward { self.len() } else { at };
        #[allow(clippy::integer_arithmetic)]
        let substring: String = self.string[..].graphemes(true).skip(start)
            .take(end - start).collect();
        let matching_byte_idx = if direction == SearchDirection::Forward { substring.find(query) }
            else { substring.rfind(query) };
        if let Some(matching_byte_idx) = matching_byte_idx {
            for (grapheme_idx, (byte_idx, _)) in substring[..].grapheme_indices(true).enumerate() {
                if matching_byte_idx == byte_idx {
                    #[allow(clippy::integer_arithmetic)]
                    return Some(start + grapheme_idx);
                }
            }
        }
        None
    }

    pub fn highlight_match (&mut self, word: Option<&str>) {
        if let Some(word) = word {
            if word.is_empty() { return; }
            let mut idx = 0;
            while let Some(search_match) = self.find(word, idx, SearchDirection::Forward) {
                if let Some(next_idx) = search_match.checked_add(word[..].graphemes(true).count()) {
                    #[allow(clippy::index_slicing)]
                    for i in search_match .. next_idx {
                        self.highlighting[i] = highlighting::Type::Match;
                    }
                    idx = next_idx;
                } else { break };
            }
        }
    }

    fn highlight_char (&mut self, idx: &mut usize, opts: &HighlightingOptions, c: char, chars: &[char]) -> bool {
        if opts.characters() && c == '\'' {
            if let Some(next_char) = chars.get(idx.saturating_add(1)) {
                let closing_idx = if *next_char == '\\' {
                    idx.saturating_add(3)
                } else { idx.saturating_add(2) };
                if let Some(closing_char) = chars.get(closing_idx) {
                    if *closing_char == '\'' {
                        for _ in 0 .. closing_idx.saturating_sub(*idx) {
                            self.highlighting.push(highlighting::Type::Character);
                            *idx += 1;
                        }
                        return true;
                    }
                }
            }
        }
        false
    }

    fn highlight_comment (&mut self, idx: &mut usize, opts: &HighlightingOptions, c: char, chars: &[char]) -> bool {
        if opts.comments() && c == '/'  && *idx < chars.len() {
            if let Some(next_char) = chars.get(idx.saturating_add(1)) {
                if *next_char == '/' {
                    for _ in *idx .. chars.len() {
                        self.highlighting.push(highlighting::Type::Comment);
                        *idx += 1;
                    }
                    return true;
                }
            };
        }
        false
    }

    fn highlight_str (&mut self, idx: &mut usize, substring: &str, chars: &[char], hl_type: highlighting::Type) -> bool {
        if substring.is_empty() { return false; }
        for (substring_idx, c) in substring.chars().enumerate() {
            if let Some(next_char) = chars.get(idx.saturating_add(substring_idx)) {
                if *next_char != c { return false; }
            } else { return false; }
        }
        for _ in 0 .. substring.len() {
            self.highlighting.push(hl_type);
            *idx += 1;
        }
        true
    }

    fn highlight_string (&mut self, idx: &mut usize, opts: &HighlightingOptions, c: char, chars: &[char]) -> bool {
        if opts.strings() && c == '"' {
            loop {
                self.highlighting.push(highlighting::Type::String);
                *idx += 1;
                if let Some(next_char) = chars.get(*idx) {
                    if *next_char == '"' { break; }
                } else { break; }
            }
            self.highlighting.push(highlighting::Type::String);
            *idx += 1;
            return true;
        }
        false
    }

    fn highlight_number (&mut self, idx: &mut usize, opts: &HighlightingOptions, c: char, chars: &[char]) -> bool {
        if opts.numbers() && c.is_ascii_digit() {
            if *idx > 0 {
                #[allow(clippy::index_slicing, clippy::integer_arithmetic)]
                let prev_char = chars[*idx - 1];
                if !prev_char.is_ascii_digit() && !prev_char.is_ascii_whitespace() {
                    return false;
                }
            }
            loop {
                self.highlighting.push(highlighting::Type::Number);
                *idx += 1;
                if let Some(next_char) = chars.get(*idx) {
                    if *next_char != '.' && !next_char.is_ascii_digit() { break; }
                } else { break; }
            }
            return true;
        }
        false
    }

    fn highlight_keywords_primary (&mut self, idx: &mut usize, opts: &HighlightingOptions, chars: &[char]) -> bool {
        for word in opts.keywords_primary() {
            if self.highlight_str(idx, word, chars, highlighting::Type::KeywordPrimary) {
                return true;
            }
        }
        false
    }

    pub fn highlight (&mut self, opts: &HighlightingOptions, word: Option<&str>) {
        self.highlighting = Vec::new();
        let chars: Vec<char> = self.string.chars().collect();
        let mut idx = 0;

        while let Some(c) = chars.get(idx) {
            if self.highlight_char(&mut idx, opts, *c, &chars)
                || self.highlight_comment(&mut idx, opts, *c, &chars)
                || self.highlight_string(&mut idx, opts, *c, &chars)
                || self.highlight_number(&mut idx, opts, *c, &chars)
                || self.highlight_keywords_primary(&mut idx, opts, &chars)
            { continue; }
            self.highlighting.push(highlighting::Type::None);
            idx += 1;
        }
        self.highlight_match(word);
    }
}

#[cfg(test)]
mod test_super {
    use super::*;

    #[test]
    fn test_highlight_find() {
        let mut row = Row::from("1testtest");
        row.highlighting = vec![
            highlighting::Type::Number,
            highlighting::Type::None,
            highlighting::Type::None,
            highlighting::Type::None,
            highlighting::Type::None,
            highlighting::Type::None,
            highlighting::Type::None,
            highlighting::Type::None,
            highlighting::Type::None,
        ];
        row.highlight_match(Some(&"t".to_string()));
        assert_eq!(
            vec![
                highlighting::Type::Number,
                highlighting::Type::Match,
                highlighting::Type::None,
                highlighting::Type::None,
                highlighting::Type::Match,
                highlighting::Type::Match,
                highlighting::Type::None,
                highlighting::Type::None,
                highlighting::Type::Match
            ],
            row.highlighting
        )
    }

    #[test]
    fn test_find() {
        let row = Row::from("1testtest");
        assert_eq!(row.find("t", 0, SearchDirection::Forward), Some(1));
        assert_eq!(row.find("t", 2, SearchDirection::Forward), Some(4));
        assert_eq!(row.find("t", 5, SearchDirection::Forward), Some(5));
    }
}
