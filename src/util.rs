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
/// `value` must be the decoded string. cddl 0.10.7 resolves escape sequences
/// in text literals at parse time, so every `\` here is a literal backslash.
/// Use [`escape_js_raw_string`] for values that still hold raw CDDL source,
/// such as byte strings.
///
/// - `\` becomes `\\`.
/// - `"` becomes `\"`.
/// - A `/` after `*` becomes `\/`, so the value can appear in a doc comment.
/// - Line terminators and control characters become escape sequences.
pub fn escape_js_string<T: AsRef<str>>(value: T) -> String {
    let value = value.as_ref();
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch == '\\' {
            out.push_str("\\\\");
        } else {
            push_escaped_char(&mut out, ch);
        }
    }
    out
}

/// Like [`escape_js_string`], but for raw CDDL source text.
///
/// The parser does not decode byte strings, so they keep escape sequences
/// such as `\"`, `\n` and `\u00e9`. These sequences are also valid in
/// JavaScript, so this function keeps them as-is. Only a `\` that does not
/// start a valid escape sequence becomes `\\`.
pub fn escape_js_raw_string<T: AsRef<str>>(value: T) -> String {
    let chars: Vec<char> = value.as_ref().chars().collect();
    let mut out = String::with_capacity(chars.len());
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if ch == '\\' {
            let len = valid_escape_len(&chars[i + 1..]);
            if len > 0 {
                out.push('\\');
                out.extend(&chars[i + 1..i + 1 + len]);
                i += len;
            } else {
                out.push_str("\\\\");
            }
        } else {
            push_escaped_char(&mut out, ch);
        }
        i += 1;
    }
    out
}

/// Appends `ch` to `out`, escaped for a double-quoted JavaScript string.
/// The callers handle `\`.
fn push_escaped_char(out: &mut String, ch: char) {
    match ch {
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

/// Returns the names of all group rules, including `Name = ( SingleIdent )` rules that
/// cddl 0.10.7 parses as `Rule::Type(ParenthesizedType)` when `SingleIdent` is a group.
pub(crate) fn collect_group_rules(cddl: &cddl::ast::CDDL) -> std::collections::HashSet<String> {
    let mut group_rules = std::collections::HashSet::new();
    for rule in &cddl.rules {
        if let cddl::ast::Rule::Group { rule, .. } = rule {
            group_rules.insert(rule.name.ident.to_string());
        }
    }
    loop {
        let mut changed = false;
        for rule in &cddl.rules {
            if let cddl::ast::Rule::Type { rule, .. } = rule {
                if !rule.is_type_choice_alternate
                    && !group_rules.contains(rule.name.ident)
                    && type_is_group_ref(&rule.value, &group_rules)
                {
                    group_rules.insert(rule.name.ident.to_string());
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    group_rules
}

fn type_is_group_ref(t: &cddl::ast::Type, group_rules: &std::collections::HashSet<String>) -> bool {
    if t.type_choices.len() != 1 {
        return false;
    }
    let t1 = &t.type_choices[0].type1;
    if t1.operator.is_some() {
        return false;
    }
    match &t1.type2 {
        cddl::ast::Type2::Typename { ident, .. } => {
            group_rules.contains(ident.ident)
                || matches!(ident.socket, Some(cddl::token::SocketPlug::GROUP))
        }
        cddl::ast::Type2::ParenthesizedType { pt, .. } => type_is_group_ref(pt, group_rules),
        _ => false,
    }
}

/// Converts a `Name = ( SingleIdent )` rule, which cddl 0.10.7 parses as
/// `Rule::Type(ParenthesizedType)`, back to a group rule.
pub(crate) fn type_rule_as_group_rule<'a>(
    tr: &cddl::ast::TypeRule<'a>,
    group_rules: &std::collections::HashSet<String>,
) -> Option<cddl::ast::GroupRule<'a>> {
    if tr.is_type_choice_alternate || !group_rules.contains(tr.name.ident) {
        return None;
    }
    let entry = type_to_group_entry(&tr.value)?;
    Some(cddl::ast::GroupRule {
        name: tr.name.clone(),
        generic_params: tr.generic_params.clone(),
        is_group_choice_alternate: false,
        entry,
        comments_before_assigng: tr.comments_before_assignt.clone(),
        comments_after_assigng: tr.comments_after_assignt.clone(),
    })
}

fn type_to_group_entry<'a>(t: &cddl::ast::Type<'a>) -> Option<cddl::ast::GroupEntry<'a>> {
    if t.type_choices.len() != 1 {
        return None;
    }
    let t1 = &t.type_choices[0].type1;
    if t1.operator.is_some() {
        return None;
    }
    match &t1.type2 {
        cddl::ast::Type2::Typename {
            ident,
            generic_args,
            span,
        } => Some(cddl::ast::GroupEntry::TypeGroupname {
            ge: cddl::ast::TypeGroupnameEntry {
                occur: None,
                name: ident.clone(),
                generic_args: generic_args.clone(),
            },
            leading_comments: None,
            trailing_comments: None,
            span: *span,
        }),
        cddl::ast::Type2::ParenthesizedType {
            pt,
            comments_before_type,
            comments_after_type,
            span,
        } => {
            let inner_entry = type_to_group_entry(pt)?;
            Some(cddl::ast::GroupEntry::InlineGroup {
                occur: None,
                group: cddl::ast::Group {
                    group_choices: vec![cddl::ast::GroupChoice {
                        group_entries: vec![(
                            inner_entry,
                            cddl::ast::OptionalComma {
                                optional_comma: false,
                                trailing_comments: None,
                                _a: std::marker::PhantomData,
                            },
                        )],
                        span: pt.span,
                        comments_before_grpchoice: None,
                    }],
                    span: *span,
                },
                comments_before_group: comments_before_type.clone(),
                comments_after_group: comments_after_type.clone(),
                span: *span,
            })
        }
        _ => None,
    }
}

/// Returns the literal value of a member key. This includes arrow map keys
/// (`? "key" => int`), which cddl 0.10.7 parses as `MemberKey::Type1` instead of
/// `MemberKey::Value`.
pub(crate) fn member_key_literal_value<'a>(
    mk: &cddl::ast::MemberKey<'a>,
) -> Option<cddl::token::Value<'a>> {
    match mk {
        cddl::ast::MemberKey::Value { value, .. } => Some(value.clone()),
        cddl::ast::MemberKey::Type1 { t1, .. } if t1.operator.is_none() => match &t1.type2 {
            cddl::ast::Type2::IntValue { value, .. } => Some(cddl::token::Value::INT(*value)),
            cddl::ast::Type2::UintValue { value, .. } => Some(cddl::token::Value::UINT(*value)),
            cddl::ast::Type2::FloatValue { value, .. } => Some(cddl::token::Value::FLOAT(*value)),
            cddl::ast::Type2::TextValue { value, .. } => {
                Some(cddl::token::Value::TEXT(value.clone()))
            }
            cddl::ast::Type2::UTF8ByteString { value, .. } => Some(cddl::token::Value::BYTE(
                cddl::token::ByteValue::UTF8(value.clone()),
            )),
            cddl::ast::Type2::B16ByteString { value, .. } => Some(cddl::token::Value::BYTE(
                cddl::token::ByteValue::B16(value.clone()),
            )),
            cddl::ast::Type2::B64ByteString { value, .. } => Some(cddl::token::Value::BYTE(
                cddl::token::ByteValue::B64(value.clone()),
            )),
            _ => None,
        },
        _ => None,
    }
}

/// Returns true for a non-cut `MemberKey::Type1` that is not a literal key (see
/// [`member_key_literal_value`]), such as `* text => any`.
pub(crate) fn is_non_cut_non_literal_member_key(mk: &cddl::ast::MemberKey) -> bool {
    matches!(mk, cddl::ast::MemberKey::Type1 { is_cut: false, .. })
        && member_key_literal_value(mk).is_none()
}

/// Converts a bare type name or parenthesized type inside an array, which cddl 0.10.7
/// parses as `GroupEntry::TypeGroupname` or `GroupEntry::InlineGroup`, back to a type entry.
pub(crate) fn array_group_entry_as_type_entry<'a>(
    entry: &cddl::ast::GroupEntry<'a>,
    group_rules: &std::collections::HashSet<String>,
) -> Option<cddl::ast::ValueMemberKeyEntry<'a>> {
    match entry {
        cddl::ast::GroupEntry::TypeGroupname { ge, .. } => {
            if group_rules.contains(ge.name.ident)
                || matches!(ge.name.socket, Some(cddl::token::SocketPlug::GROUP))
            {
                return None;
            }
            Some(cddl::ast::ValueMemberKeyEntry {
                occur: ge.occur.clone(),
                member_key: None,
                entry_type: typename_to_type(&ge.name, &ge.generic_args),
            })
        }
        cddl::ast::GroupEntry::InlineGroup { occur, group, .. } => {
            let entry_type = inline_group_as_type(group, group_rules)?;
            Some(cddl::ast::ValueMemberKeyEntry {
                occur: occur.clone(),
                member_key: None,
                entry_type,
            })
        }
        cddl::ast::GroupEntry::ValueMemberKey { .. } => None,
    }
}

fn typename_to_type<'a>(
    name: &cddl::ast::Identifier<'a>,
    generic_args: &Option<cddl::ast::GenericArgs<'a>>,
) -> cddl::ast::Type<'a> {
    cddl::ast::Type {
        type_choices: vec![cddl::ast::TypeChoice {
            type1: cddl::ast::Type1 {
                type2: cddl::ast::Type2::Typename {
                    ident: name.clone(),
                    generic_args: generic_args.clone(),
                    span: name.span,
                },
                operator: None,
                span: name.span,
                comments_after_type: None,
            },
            comments_before_type: None,
            comments_after_type: None,
        }],
        span: name.span,
    }
}

fn inline_group_as_type<'a>(
    group: &cddl::ast::Group<'a>,
    group_rules: &std::collections::HashSet<String>,
) -> Option<cddl::ast::Type<'a>> {
    if group.group_choices.len() != 1 {
        return None;
    }
    let choice = &group.group_choices[0];
    if choice.group_entries.len() != 1 {
        return None;
    }
    let (inner_entry, _) = &choice.group_entries[0];
    match inner_entry {
        cddl::ast::GroupEntry::ValueMemberKey { ge, .. } => {
            if ge.occur.is_none() && ge.member_key.is_none() {
                Some(ge.entry_type.clone())
            } else {
                None
            }
        }
        cddl::ast::GroupEntry::TypeGroupname { ge, .. } => {
            if ge.occur.is_some()
                || group_rules.contains(ge.name.ident)
                || matches!(ge.name.socket, Some(cddl::token::SocketPlug::GROUP))
            {
                return None;
            }
            Some(typename_to_type(&ge.name, &ge.generic_args))
        }
        cddl::ast::GroupEntry::InlineGroup {
            occur,
            group: inner_group,
            span,
            comments_before_group,
            comments_after_group,
        } => {
            if occur.is_some() {
                return None;
            }
            let pt = inline_group_as_type(inner_group, group_rules)?;
            Some(cddl::ast::Type {
                type_choices: vec![cddl::ast::TypeChoice {
                    type1: cddl::ast::Type1 {
                        type2: cddl::ast::Type2::ParenthesizedType {
                            pt,
                            comments_before_type: comments_before_group.clone(),
                            comments_after_type: comments_after_group.clone(),
                            span: *span,
                        },
                        operator: None,
                        span: *span,
                        comments_after_type: None,
                    },
                    comments_before_type: None,
                    comments_after_type: None,
                }],
                span: *span,
            })
        }
    }
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
        assert_eq!(escape_js_string("unicode é"), "unicode é");
        // Every backslash is a literal backslash.
        assert_eq!(escape_js_string(r"\d+\.\d+"), r"\\d+\\.\\d+");
        assert_eq!(escape_js_string(r"\n\b\u00e9"), r"\\n\\b\\u00e9");
        assert_eq!(escape_js_string(r"a\"), r"a\\");
        // Double quotes are escaped.
        assert_eq!(escape_js_string(r#"a"b"#), r#"a\"b"#);
        assert_eq!(escape_js_string(r#"\""#), r#"\\\""#);
        // Line terminators and control characters are escaped.
        assert_eq!(
            escape_js_string("a\nb\r\t\u{0}\u{8}\u{2028}\u{2029}"),
            r"a\nb\r\t\u0000\u0008\u2028\u2029"
        );
        // Glob-like values are safe to use in doc comments.
        assert_eq!(escape_js_string("src/**/*.ts"), r"src/**\/*.ts");
    }

    #[test]
    fn test_escape_js_raw_string() {
        // Plain values do not change.
        assert_eq!(escape_js_raw_string("hello world"), "hello world");
        // Valid escape sequences from the CDDL source do not change.
        assert_eq!(escape_js_raw_string(r#"a\"b\\c\n\/"#), r#"a\"b\\c\n\/"#);
        assert_eq!(escape_js_raw_string(r"\u00e9\u{1F600}\'"), r"\u00e9\u{1F600}\'");
        // Code point escapes accept leading zeros, up to U+10FFFF.
        assert_eq!(
            escape_js_raw_string(r"\u{0041}\u{000000041}"),
            r"\u{0041}\u{000000041}"
        );
        assert_eq!(escape_js_raw_string(r"\u{10FFFF}"), r"\u{10FFFF}");
        assert_eq!(escape_js_raw_string(r"\u{110000}"), r"\\u{110000}");
        assert_eq!(
            escape_js_raw_string(r"\u{FFFFFFFFFF}\u{}"),
            r"\\u{FFFFFFFFFF}\\u{}"
        );
        // Raw double quotes (allowed in byte strings) are escaped.
        assert_eq!(escape_js_raw_string(r#"'say "hi"'"#), r#"'say \"hi\"'"#);
        // An escaped backslash followed by a raw quote.
        assert_eq!(escape_js_raw_string(r#"\\""#), r#"\\\""#);
        // Unpaired or invalid backslashes are escaped.
        assert_eq!(escape_js_raw_string(r"a\"), r"a\\");
        assert_eq!(escape_js_raw_string(r"\x41\u12"), r"\\x41\\u12");
        // Line terminators and control characters are escaped.
        assert_eq!(
            escape_js_raw_string("a\nb\r\t\u{0}\u{2028}\u{2029}"),
            r"a\nb\r\t\u0000\u2028\u2029"
        );
        // Glob-like values are safe to use in doc comments.
        assert_eq!(escape_js_raw_string("src/**/*.ts"), r"src/**\/*.ts");
    }

    #[test]
    fn test_collect_group_rules_and_type_rule_as_group_rule() {
        let input = r#"
            BrowserCommand = ( browser.Close )
            browser.Close = ( method: "browser.close" )
            NetworkResult = ( network.AddInterceptResult )
            network.AddInterceptResult = { intercept: text }
            BrowserCommand /= ( browser.Close )
        "#;
        let cddl = cddl::parser::cddl_from_str(input, true).unwrap();
        let group_rules = collect_group_rules(&cddl);

        assert!(group_rules.contains("browser.Close"));
        assert!(group_rules.contains("BrowserCommand"));
        assert!(!group_rules.contains("network.AddInterceptResult"));
        assert!(!group_rules.contains("NetworkResult"));

        if let cddl::ast::Rule::Type { rule, .. } = &cddl.rules[0] {
            let gr = type_rule_as_group_rule(rule, &group_rules).unwrap();
            assert!(!gr.is_group_choice_alternate);
        } else {
            panic!("Expected BrowserCommand to parse as Rule::Type in cddl 0.10.7");
        }

        if let cddl::ast::Rule::Type { rule, .. } = &cddl.rules[2] {
            assert!(type_rule_as_group_rule(rule, &group_rules).is_none());
        } else {
            panic!("Expected NetworkResult to parse as Rule::Type");
        }

        if let cddl::ast::Rule::Type { rule, .. } = &cddl.rules[4] {
            assert!(type_rule_as_group_rule(rule, &group_rules).is_none());
        } else {
            panic!("Expected BrowserCommand /= to parse as Rule::Type");
        }
    }

    #[test]
    fn test_member_key_literal_value() {
        let input = r#"
            m = {
                ? "optional-key" => int,
                ? "cut-key" ^ => int,
                * text => any,
            }
        "#;
        let cddl = cddl::parser::cddl_from_str(input, true).unwrap();
        let cddl::ast::Rule::Type { rule, .. } = &cddl.rules[0] else {
            panic!("Expected type rule");
        };
        let cddl::ast::Type2::Map { group, .. } = &rule.value.type_choices[0].type1.type2 else {
            panic!("Expected map");
        };
        let entries = &group.group_choices[0].group_entries;
        let cddl::ast::GroupEntry::ValueMemberKey { ge: first, .. } = &entries[0].0 else {
            panic!("Expected first entry");
        };
        let first_mk = first.member_key.as_ref().unwrap();
        assert_eq!(
            member_key_literal_value(first_mk),
            Some(cddl::token::Value::TEXT("optional-key".into()))
        );
        assert!(!is_non_cut_non_literal_member_key(first_mk));

        let cddl::ast::GroupEntry::ValueMemberKey { ge: second, .. } = &entries[1].0 else {
            panic!("Expected second entry");
        };
        let second_mk = second.member_key.as_ref().unwrap();
        assert_eq!(
            member_key_literal_value(second_mk),
            Some(cddl::token::Value::TEXT("cut-key".into()))
        );
        assert!(!is_non_cut_non_literal_member_key(second_mk));

        let cddl::ast::GroupEntry::ValueMemberKey { ge: third, .. } = &entries[2].0 else {
            panic!("Expected third entry");
        };
        let third_mk = third.member_key.as_ref().unwrap();
        assert_eq!(member_key_literal_value(third_mk), None);
        assert!(is_non_cut_non_literal_member_key(third_mk));
    }

    #[test]
    fn test_array_group_entry_as_type_entry() {
        let input = r#"
            Arr = [* MyType, *(uint / text), *(uint / text,), * MyGroup]
            MyType = text
            MyGroup = ( a: int )
        "#;
        let cddl = cddl::parser::cddl_from_str(input, true).unwrap();
        let group_rules = collect_group_rules(&cddl);
        let cddl::ast::Rule::Type { rule, .. } = &cddl.rules[0] else {
            panic!("Expected type rule");
        };
        let cddl::ast::Type2::Array { group, .. } = &rule.value.type_choices[0].type1.type2 else {
            panic!("Expected array");
        };
        let entries = &group.group_choices[0].group_entries;
        assert!(array_group_entry_as_type_entry(&entries[0].0, &group_rules).is_some());
        assert!(array_group_entry_as_type_entry(&entries[1].0, &group_rules).is_some());
        assert!(array_group_entry_as_type_entry(&entries[2].0, &group_rules).is_some());
        assert!(array_group_entry_as_type_entry(&entries[3].0, &group_rules).is_none());
    }
}
