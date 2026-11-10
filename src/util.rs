// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use convert_case::{Case, Casing};

fn split_identifier<T: ToString>(value: T) -> Vec<String> {
    return value.to_string().split('.').map(String::from).collect();
}

pub fn to_pascalcase<T: ToString>(value: T) -> String {
    value.to_string().to_case(Case::Pascal)
}

pub fn to_namespaced<T: ToString>(value: T) -> String {
    split_identifier(&value.to_string())
        .into_iter()
        .map(to_pascalcase)
        .collect::<Vec<String>>()
        .join(".")
}

pub fn split_namespaced<T: ToString>(value: T) -> (Vec<String>, String) {
    let mut parts = split_identifier(value.to_string())
        .into_iter()
        .map(to_pascalcase)
        .collect::<Vec<String>>();
    let value = parts.pop().unwrap();
    (parts, value)
}

/// Escapes `value` for use between double quotes in a generated JavaScript or
/// TypeScript string literal (or string literal type).
///
/// The CDDL lexer keeps escape sequences such as `\"`, `\n` and `\u00e9` in the
/// raw value. These sequences are also valid in JavaScript, so this function
/// keeps them as-is. It escapes other characters as follows:
///
/// - An unpaired `\` becomes `\\`.
/// - A raw `"` becomes `\"`.
/// - A `/` after `*` becomes `\/`, so the value can appear in a doc comment.
/// - Line terminators and control characters become escape sequences.
pub fn escape_js_string<T: AsRef<str>>(value: T) -> String {
    let chars: Vec<char> = value.as_ref().chars().collect();
    let mut out = String::with_capacity(chars.len());
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        match ch {
            '\\' => {
                let len = valid_escape_len(&chars[i + 1..]);
                if len > 0 {
                    out.push('\\');
                    out.extend(&chars[i + 1..i + 1 + len]);
                    i += len;
                } else {
                    out.push_str("\\\\");
                }
            }
            '"' => out.push_str("\\\""),
            '/' if out.ends_with('*') => out.push_str("\\/"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() || c == '\u{2028}' || c == '\u{2029}' => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
        i += 1;
    }
    out
}

/// Returns the number of characters after a `\` that form a valid JavaScript
/// escape sequence, or 0 if they do not.
fn valid_escape_len(rest: &[char]) -> usize {
    match rest.first() {
        Some('"' | '\'' | '/' | '\\' | 'b' | 'f' | 'n' | 'r' | 't') => 1,
        Some('u') => {
            if rest.len() >= 5 && rest[1..5].iter().all(char::is_ascii_hexdigit) {
                return 5;
            }
            if rest.get(1) == Some(&'{') {
                let digits = rest[2..]
                    .iter()
                    .take_while(|ch| ch.is_ascii_hexdigit())
                    .count();
                if digits > 0 && rest.get(2 + digits) == Some(&'}') {
                    // JavaScript accepts any number of digits, as long as the
                    // value is a valid code point.
                    let hex: String = rest[2..2 + digits].iter().collect();
                    if u32::from_str_radix(&hex, 16).is_ok_and(|cp| cp <= 0x10FFFF) {
                        return 3 + digits;
                    }
                }
            }
            0
        }
        _ => 0,
    }
}

pub fn is_alphaspace<T: AsRef<str>>(value: T) -> bool {
    value
        .as_ref()
        .to_ascii_lowercase()
        .bytes()
        .all(|ch| b'a' <= ch && ch <= b'z' || ch == b' ')
}

pub fn is_enum_value<T: AsRef<str>>(value: T) -> bool {
    let val = value.as_ref();
    let pascal = to_pascalcase(val);
    if pascal.is_empty() {
        return false;
    }
    let mut chars = pascal.chars();
    let first = chars.next().unwrap();
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_enum_value() {
        assert!(is_enum_value("invalid argument"));
        assert!(is_enum_value("qValue"));
        assert!(is_enum_value("qValue2"));
        assert!(is_enum_value("user-agent"));
        assert!(is_enum_value("strict"));
        assert!(is_enum_value("none"));
        assert!(!is_enum_value("2g"));
        assert!(!is_enum_value(""));
        assert!(!is_enum_value("foo.bar"));
    }

    #[test]
    fn test_escape_js_string() {
        // Plain values do not change.
        assert_eq!(escape_js_string("hello world"), "hello world");
        // Valid escape sequences from the CDDL source do not change.
        assert_eq!(escape_js_string(r#"a\"b\\c\n\/"#), r#"a\"b\\c\n\/"#);
        assert_eq!(escape_js_string(r"\u00e9\u{1F600}\'"), r"\u00e9\u{1F600}\'");
        // Code point escapes accept leading zeros, up to U+10FFFF.
        assert_eq!(
            escape_js_string(r"\u{0041}\u{000000041}"),
            r"\u{0041}\u{000000041}"
        );
        assert_eq!(escape_js_string(r"\u{10FFFF}"), r"\u{10FFFF}");
        assert_eq!(escape_js_string(r"\u{110000}"), r"\\u{110000}");
        assert_eq!(
            escape_js_string(r"\u{FFFFFFFFFF}\u{}"),
            r"\\u{FFFFFFFFFF}\\u{}"
        );
        // Raw double quotes (allowed in byte strings) are escaped.
        assert_eq!(escape_js_string(r#"'say "hi"'"#), r#"'say \"hi\"'"#);
        // An escaped backslash followed by a raw quote.
        assert_eq!(escape_js_string(r#"\\""#), r#"\\\""#);
        // Unpaired or invalid backslashes are escaped.
        assert_eq!(escape_js_string(r"a\"), r"a\\");
        assert_eq!(escape_js_string(r"\x41\u12"), r"\\x41\\u12");
        // Line terminators and control characters are escaped.
        assert_eq!(
            escape_js_string("a\nb\r\t\u{0}\u{2028}\u{2029}"),
            r"a\nb\r\t\u0000\u2028\u2029"
        );
        // Glob-like values are safe to use in doc comments.
        assert_eq!(escape_js_string("src/**/*.ts"), r"src/**\/*.ts");
    }
}
