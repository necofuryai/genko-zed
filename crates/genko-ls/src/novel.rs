use unicode_segmentation::UnicodeSegmentation;

const RUBY_MARKER: &str = "｜";
const RUBY_READING_OPEN: &str = "《";
const RUBY_READING_CLOSE: &str = "》";
const AOZORA_ANNOTATION_OPEN: &str = "［＃";
const AOZORA_ANNOTATION_CLOSE: &str = "］";
const HTML_COMMENT_OPEN: &str = "<!--";
const HTML_COMMENT_CLOSE: &str = "-->";

/// The observable metrics for one Genko Novel document.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NovelMetrics {
    body_character_count: usize,
}

impl NovelMetrics {
    /// Returns the number of extended grapheme clusters in the document body.
    #[must_use]
    pub const fn body_character_count(self) -> usize {
        self.body_character_count
    }
}

/// Calculates metrics for a Genko Novel document.
///
/// Body characters are extended grapheme clusters. Line breaks are not counted.
/// Complete explicit ruby, Aozora annotations, and HTML comments are interpreted;
/// malformed or incomplete markup remains literal text.
#[must_use]
pub fn novel_metrics(source: &str) -> NovelMetrics {
    let mut counter = GraphemeCounter::default();
    scan_body(source, &mut counter);

    NovelMetrics {
        body_character_count: counter.finish(),
    }
}

fn scan_body(source: &str, counter: &mut GraphemeCounter) {
    let mut cursor = 0;

    while cursor < source.len() {
        let rest = &source[cursor..];

        if let Some(line_break_length) = line_break_length(rest) {
            counter.end_line();
            cursor += line_break_length;
            continue;
        }

        if let Some(comment_length) = complete_html_comment_length(rest) {
            if contains_line_break(&rest[..comment_length]) {
                counter.end_line();
            }
            cursor += comment_length;
            continue;
        }

        if let Some(annotation_length) = complete_annotation_length(rest) {
            cursor += annotation_length;
            continue;
        }

        if let Some(ruby) = complete_explicit_ruby(rest) {
            scan_body(ruby.base, counter);
            cursor += ruby.full_length;
            continue;
        }

        let character = rest
            .chars()
            .next()
            .expect("cursor always points inside a non-empty UTF-8 string");
        counter.push(character);
        cursor += character.len_utf8();
    }
}

fn complete_html_comment_length(source: &str) -> Option<usize> {
    let after_open = source.strip_prefix(HTML_COMMENT_OPEN)?;
    let close_offset = after_open.find(HTML_COMMENT_CLOSE)?;
    Some(HTML_COMMENT_OPEN.len() + close_offset + HTML_COMMENT_CLOSE.len())
}

fn complete_annotation_length(source: &str) -> Option<usize> {
    let after_open = source.strip_prefix(AOZORA_ANNOTATION_OPEN)?;
    let close_offset = find_before_line_break(after_open, AOZORA_ANNOTATION_CLOSE)?;

    if close_offset == 0 {
        return None;
    }

    Some(AOZORA_ANNOTATION_OPEN.len() + close_offset + AOZORA_ANNOTATION_CLOSE.len())
}

#[derive(Debug)]
struct ExplicitRuby<'a> {
    base: &'a str,
    full_length: usize,
}

fn complete_explicit_ruby(source: &str) -> Option<ExplicitRuby<'_>> {
    let after_marker = source.strip_prefix(RUBY_MARKER)?;
    let reading_open_offset = find_before_line_break(after_marker, RUBY_READING_OPEN)?;
    let base = &after_marker[..reading_open_offset];

    if base.is_empty() {
        return None;
    }

    let after_reading_open = &after_marker[reading_open_offset + RUBY_READING_OPEN.len()..];
    let reading_close_offset = find_before_line_break(after_reading_open, RUBY_READING_CLOSE)?;

    if reading_close_offset == 0 {
        return None;
    }

    let full_length = RUBY_MARKER.len()
        + reading_open_offset
        + RUBY_READING_OPEN.len()
        + reading_close_offset
        + RUBY_READING_CLOSE.len();

    Some(ExplicitRuby { base, full_length })
}

fn find_before_line_break(source: &str, delimiter: &str) -> Option<usize> {
    let delimiter_offset = source.find(delimiter)?;
    let line_break_offset = source
        .char_indices()
        .find_map(|(offset, character)| is_line_break(character).then_some(offset));

    match line_break_offset {
        Some(offset) if offset < delimiter_offset => None,
        _ => Some(delimiter_offset),
    }
}

fn contains_line_break(source: &str) -> bool {
    source.chars().any(is_line_break)
}

fn line_break_length(source: &str) -> Option<usize> {
    if source.starts_with("\r\n") {
        return Some(2);
    }

    let character = source.chars().next()?;
    is_line_break(character).then_some(character.len_utf8())
}

fn is_line_break(character: char) -> bool {
    matches!(
        character,
        '\n' | '\r' | '\u{0085}' | '\u{2028}' | '\u{2029}'
    )
}

#[derive(Debug, Default)]
struct GraphemeCounter {
    completed_count: usize,
    current_line: String,
}

impl GraphemeCounter {
    fn push(&mut self, character: char) {
        self.current_line.push(character);
    }

    fn end_line(&mut self) {
        self.completed_count += self.current_line.graphemes(true).count();
        self.current_line.clear();
    }

    fn finish(mut self) -> usize {
        self.end_line();
        self.completed_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn literal_count_without_line_breaks(source: &str) -> usize {
        source
            .split(is_line_break)
            .map(|line| line.graphemes(true).count())
            .sum()
    }

    #[test]
    fn counts_extended_grapheme_clusters_and_whitespace() {
        let source = "A e\u{301} 👨‍👩‍👧‍👦\tあ";

        assert_eq!(novel_metrics(source).body_character_count(), 7);
    }

    #[test]
    fn excludes_common_and_unicode_line_breaks() {
        let source = "一\r\n二\n三\r四\u{0085}五\u{2028}六\u{2029}七";

        assert_eq!(novel_metrics(source).body_character_count(), 7);
    }

    #[test]
    fn counts_explicit_ruby_base_and_excludes_markup_and_reading() {
        let source = "前｜漢字《かんじ》後 ｜e\u{301}《accent》";

        assert_eq!(novel_metrics(source).body_character_count(), 6);
    }

    #[test]
    fn does_not_interpret_implicit_ruby() {
        let source = "漢字《かんじ》";

        assert_eq!(
            novel_metrics(source).body_character_count(),
            literal_count_without_line_breaks(source)
        );
    }

    #[test]
    fn counts_incomplete_or_malformed_ruby_literally() {
        for source in [
            "｜漢字《かんじ",
            "｜漢字《》",
            "｜《かんじ》",
            "｜漢字\n《かんじ》",
        ] {
            assert_eq!(
                novel_metrics(source).body_character_count(),
                literal_count_without_line_breaks(source),
                "source: {source}"
            );
        }
    }

    #[test]
    fn excludes_complete_aozora_annotations() {
        let source = "前［＃ここは注記］後";

        assert_eq!(novel_metrics(source).body_character_count(), 2);
    }

    #[test]
    fn counts_incomplete_or_malformed_annotations_literally() {
        for source in ["前［＃未完", "前［＃］後", "前［＃未完\n本文］後"] {
            assert_eq!(
                novel_metrics(source).body_character_count(),
                literal_count_without_line_breaks(source),
                "source: {source}"
            );
        }
    }

    #[test]
    fn excludes_complete_single_and_multiline_html_comments() {
        assert_eq!(
            novel_metrics("前<!-- hidden -->後").body_character_count(),
            2
        );
        assert_eq!(
            novel_metrics("前<!-- hidden\nstill hidden -->後").body_character_count(),
            2
        );
        assert_eq!(novel_metrics("前<!---->後").body_character_count(), 2);
    }

    #[test]
    fn counts_incomplete_html_comments_literally() {
        let source = "前<!-- 未完\n後";

        assert_eq!(
            novel_metrics(source).body_character_count(),
            literal_count_without_line_breaks(source)
        );
    }

    #[test]
    fn handles_adjacent_markup_without_counting_delimiters() {
        let source = "｜猫《ねこ》［＃注記］<!-- memo --> 犬";

        assert_eq!(novel_metrics(source).body_character_count(), 3);
    }
}
