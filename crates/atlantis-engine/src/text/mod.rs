//! Text primitives mirrored from the C++ engine.

pub mod input;
pub mod output;

pub use input::TextInput;
pub use output::{ReportWriter, RulesWriter};

/// C++: `astring.cpp` `AString::gettoken` (lines 204-251).
///
/// A comment marker at the start of the remaining input leaves it unchanged;
/// an unmatched quote clears it.
pub fn get_token(input: &mut String) -> Option<String> {
    let bytes = input.as_bytes();
    let mut start = 0;
    while start < bytes.len() && matches!(bytes[start], b' ' | b'\t') {
        start += 1;
    }
    if start == bytes.len() || bytes[start] == b';' {
        return None;
    }

    let (token, end) = if bytes[start] == b'"' {
        let content_start = start + 1;
        let Some(relative_end) = bytes[content_start..].iter().position(|byte| *byte == b'"')
        else {
            input.clear();
            return None;
        };
        let end = content_start + relative_end;
        (input[content_start..end].to_owned(), end + 1)
    } else {
        let end = bytes[start..]
            .iter()
            .position(|byte| matches!(byte, b' ' | b'\t' | b';'))
            .map_or(bytes.len(), |relative_end| start + relative_end);
        (input[start..end].to_owned(), end)
    };

    if end == bytes.len() || bytes[end] == b';' {
        input.clear();
    } else {
        *input = input[end..].to_owned();
    }
    Some(token)
}

/// C++: `astring.cpp` `AString::isEqual` (lines 136-157).
pub fn is_equal(left: &str, right: &str) -> bool {
    fn fold(byte: u8) -> u8 {
        match byte {
            b'A'..=b'Z' => byte + (b'a' - b'A'),
            b'_' => b' ',
            _ => byte,
        }
    }

    left.len() == right.len()
        && left
            .bytes()
            .zip(right.bytes())
            .all(|(left, right)| fold(left) == fold(right))
}

/// C++: `astring.cpp` `AString::value` (lines 351-362).
pub fn value(input: &str) -> i32 {
    parse_decimal_prefix(input.as_bytes())
}

/// C++: `astring.cpp` `AString::strict_value` (lines 364-382).
pub fn strict_value(input: &str) -> i32 {
    if !input.bytes().all(|byte| byte.is_ascii_digit()) {
        return -1;
    }
    parse_decimal_prefix(input.as_bytes())
}

fn parse_decimal_prefix(bytes: &[u8]) -> i32 {
    let mut value = 0_i32;
    for byte in bytes.iter().copied().take_while(u8::is_ascii_digit) {
        let Some(next) = value
            .checked_mul(10)
            .and_then(|value| value.checked_add(i32::from(byte - b'0')))
        else {
            return 0;
        };
        value = next;
    }
    value
}

/// C++: `astring.cpp` global `islegal` (lines 283-293).
pub fn is_legal(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
        || matches!(
            byte,
            b'!' | b'['
                | b']'
                | b','
                | b'.'
                | b' '
                | b'{'
                | b'}'
                | b'@'
                | b'#'
                | b'$'
                | b'%'
                | b'^'
                | b'&'
                | b'*'
                | b'-'
                | b'_'
                | b'+'
                | b'='
                | b';'
                | b':'
                | b'<'
                | b'>'
                | b'?'
                | b'/'
                | b'~'
                | b'\''
                | b'\\'
                | b'`'
        )
}

/// C++: `astring.cpp` `AString::getlegal` (lines 295-310).
pub fn get_legal(input: &str) -> Option<String> {
    let mut has_non_space = false;
    let filtered: Vec<u8> = input
        .bytes()
        .filter(|byte| {
            let legal = is_legal(*byte);
            has_non_space |= legal && *byte != b' ';
            legal
        })
        .collect();

    has_non_space.then(|| String::from_utf8(filtered).expect("legal bytes are ASCII"))
}

/// C++: `astring.cpp` `AString::getat` (lines 270-280).
pub fn get_at(input: &mut String) -> bool {
    let bytes = input.as_bytes();
    let mut place = 0;
    while place < bytes.len() && matches!(bytes[place], b' ' | b'\t') {
        place += 1;
    }
    if bytes.get(place) == Some(&b'@') {
        input.replace_range(place..place + 1, " ");
        true
    } else {
        false
    }
}

/// C++: `astring.cpp` `AString::Trunc` (lines 330-348).
///
/// The prefix remains in `input`; the returned value is the remainder after
/// the consumed newline, space, or hard-width boundary.
pub fn trunc(input: &mut String, width: usize, back: usize) -> Option<String> {
    if input.len() <= width {
        return None;
    }

    let bytes = input.as_bytes();
    if let Some(index) = bytes[..width]
        .iter()
        .position(|byte| matches!(byte, b'\n' | b'\r'))
    {
        let remainder = input[index + 1..].to_owned();
        input.truncate(index);
        return Some(remainder);
    }

    if back > 0 {
        let first_index = if back > width { 0 } else { width - back + 1 };
        if first_index <= width {
            for index in (first_index..=width).rev() {
                if bytes[index] == b' ' {
                    let remainder = input[index + 1..].to_owned();
                    input.truncate(index);
                    return Some(remainder);
                }
            }
        }
    }

    let boundary = if input.is_char_boundary(width) {
        width
    } else {
        input
            .char_indices()
            .map(|(index, _)| index)
            .take_while(|index| *index < width)
            .last()
            .unwrap_or(0)
    };
    let remainder = input[boundary..].to_owned();
    input.truncate(boundary);
    Some(remainder)
}

/// C++: `astring.cpp` global `plural` (lines 400-402).
pub fn plural<'a>(count: i32, one: &'a str, many: &'a str) -> &'a str {
    if count > 1 {
        many
    } else {
        one
    }
}

/// C++: `astring.cpp` global `join` (lines 404-419).
pub fn join(delimiter: &str, items: &[&str]) -> String {
    items.join(delimiter)
}

/// C++: `astring.cpp` global `join` with a final delimiter (lines 421-438).
pub fn join_with_last(delimiter: &str, last_delimiter: &str, items: &[&str]) -> String {
    let mut result = String::new();
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            result.push_str(if index == items.len() - 1 {
                last_delimiter
            } else {
                delimiter
            });
        }
        result.push_str(item);
    }
    result
}

/// C++: `astring.cpp` global `capitalize` (lines 456-463).
pub fn capitalize(input: &str) -> String {
    let Some(first) = input.chars().next() else {
        return String::new();
    };
    if first.is_ascii() {
        format!(
            "{}{}",
            first.to_ascii_uppercase(),
            &input[first.len_utf8()..]
        )
    } else {
        input.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // C++: astring.cpp AString::gettoken, lines 204-251.
    #[test]
    fn get_token_preserves_quotes_and_discards_comments() {
        let mut line = " \tMOVE \"a ; b\" ITEM; ignored".to_owned();
        assert_eq!(get_token(&mut line).as_deref(), Some("MOVE"));
        assert_eq!(get_token(&mut line).as_deref(), Some("a ; b"));
        assert_eq!(get_token(&mut line).as_deref(), Some("ITEM"));
        assert_eq!(line, "");
        assert_eq!(get_token(&mut line), None);
    }

    // C++: astring.cpp AString::gettoken, lines 204-251.
    #[test]
    fn get_token_handles_leading_comments_and_unmatched_quotes() {
        let mut comment = "; ignored".to_owned();
        assert_eq!(get_token(&mut comment), None);
        assert_eq!(comment, "; ignored");

        let mut unmatched = "\"unfinished token".to_owned();
        assert_eq!(get_token(&mut unmatched), None);
        assert_eq!(unmatched, "");
    }

    // C++: astring.cpp AString::isEqual, lines 136-157.
    #[test]
    fn is_equal_uses_ascii_case_folding_and_maps_underscores_to_spaces() {
        assert!(is_equal("River_Guard", "river guard"));
        assert!(is_equal("FOO__BAR", "foo  bar"));
        assert!(!is_equal("river-guard", "river guard"));
        assert!(!is_equal("é", "É"));
    }

    // C++: astring.cpp AString::value/strict_value, lines 351-382.
    #[test]
    fn numeric_values_match_prefix_and_strict_parsing() {
        assert_eq!(value("123 trailing"), 123);
        assert_eq!(value("word"), 0);
        assert_eq!(value("-12"), 0);
        assert_eq!(value("2147483648"), 0);
        assert_eq!(strict_value(""), 0);
        assert_eq!(strict_value("2147483647"), i32::MAX);
        assert_eq!(strict_value("2147483648"), 0);
        assert_eq!(strict_value("12x"), -1);
    }

    // C++: astring.cpp islegal/getlegal, lines 283-310.
    #[test]
    fn legal_text_filters_characters_and_rejects_space_only_results() {
        for byte in b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789![],. {}@#$%^&*-_+=;:<>?/~'\\`" {
            assert!(is_legal(*byte), "expected {byte:?} to be legal");
        }
        for byte in b"\"()|\t\n" {
            assert!(!is_legal(*byte), "expected {byte:?} to be illegal");
        }
        assert_eq!(get_legal(" A\tB?! "), Some(" AB?! ".to_owned()));
        assert_eq!(get_legal(" \t "), None);
        assert_eq!(get_legal("\""), None);
    }

    // C++: astring.cpp AString::getat, lines 270-280.
    #[test]
    fn get_at_replaces_the_first_non_whitespace_at_sign() {
        let mut value = " \t@old".to_owned();
        assert!(get_at(&mut value));
        assert_eq!(value, " \t old");
        assert!(!get_at(&mut value));
    }

    // C++: astring.cpp AString::Trunc, lines 330-348.
    #[test]
    fn trunc_prefers_nearby_spaces_then_newlines_then_a_hard_cut() {
        let mut spaced = "0123456789 word".to_owned();
        assert_eq!(trunc(&mut spaced, 10, 5).as_deref(), Some("word"));
        assert_eq!(spaced, "0123456789");

        let mut newline = "first\nsecond".to_owned();
        assert_eq!(trunc(&mut newline, 10, 30).as_deref(), Some("second"));
        assert_eq!(newline, "first");

        let mut hard_cut = "abcdefghijklmnop".to_owned();
        assert_eq!(trunc(&mut hard_cut, 6, 3).as_deref(), Some("ghijklmnop"));
        assert_eq!(hard_cut, "abcdef");
    }

    // C++: astring.cpp AString::Trunc, lines 330-348.
    #[test]
    fn trunc_checks_the_zero_index_when_back_exceeds_width() {
        let mut input = " abcdefghij".to_owned();
        assert_eq!(trunc(&mut input, 10, 30).as_deref(), Some("abcdefghij"));
        assert_eq!(input, "");
    }

    // C++: astring.cpp plural/join/capitalize, lines 400-464.
    #[test]
    fn plural_join_and_capitalize_keep_original_edge_cases() {
        assert_eq!(plural(0, "unit", "units"), "unit");
        assert_eq!(plural(1, "unit", "units"), "unit");
        assert_eq!(plural(2, "unit", "units"), "units");
        assert_eq!(plural(-1, "unit", "units"), "unit");
        assert_eq!(join(", ", &[]), "");
        assert_eq!(join(", ", &["one"]), "one");
        assert_eq!(join(", ", &["one", "two", "three"]), "one, two, three");
        assert_eq!(
            join_with_last(", ", " and ", &["one", "two"]),
            "one and two"
        );
        assert_eq!(
            join_with_last(", ", " and ", &["one", "two", "three"]),
            "one, two and three"
        );
        assert_eq!(capitalize(""), "");
        assert_eq!(capitalize("gATE lore"), "GATE lore");
    }
}
