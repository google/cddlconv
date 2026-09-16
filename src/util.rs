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

/// Checks if a `Type` AST node represents a single unwrapped type reference (`~typename` or `~typename<args>`).
///
/// In CDDL grammar (`grpent = [occur S] [memberkey S] type / [occur S] groupname [genericarg]`),
/// a bare group reference (`groupname`) is parsed by the `cddl` crate into `GroupEntry::TypeGroupname`.
/// However, when the unwrap operator (`~`) is used inside a map or array (e.g., `{~Info, ...}` or `[~Tuple, ...]`),
/// the leading `~` causes the parser to take the first `grpent` alternative (`[occur] [memberkey] type`),
/// producing a `GroupEntry::ValueMemberKey` where `member_key` is `None` and `entry_type` contains `Type2::Unwrap`.
///
/// This helper extracts the target identifier and optional generic arguments from such an `entry_type`.
pub fn unwrap_entry<'a>(
    t: &'a cddl::ast::Type<'a>,
) -> Option<(
    &'a cddl::ast::Identifier<'a>,
    &'a Option<cddl::ast::GenericArgs<'a>>,
)> {
    if t.type_choices.len() == 1 {
        if let Some(tc) = t.type_choices.first() {
            if tc.type1.operator.is_none() {
                if let cddl::ast::Type2::Unwrap {
                    ident,
                    generic_args,
                    ..
                } = &tc.type1.type2
                {
                    return Some((ident, generic_args));
                }
            }
        }
    }
    None
}

/// Extracts the identifier name and source byte span from a bare group/type entry.
/// Used by `filter_group_entries` to detect duplicate tokens at the exact same source position.
fn entry_ident_span<'a>(
    entry: &'a cddl::ast::GroupEntry<'a>,
) -> Option<(&'a str, cddl::ast::Span)> {
    match entry {
        cddl::ast::GroupEntry::TypeGroupname { ge, .. } => Some((ge.name.ident, ge.name.span)),
        cddl::ast::GroupEntry::ValueMemberKey { ge, .. } if ge.member_key.is_none() => {
            if ge.entry_type.type_choices.len() == 1 {
                if let cddl::ast::Type2::Typename { ident, .. } =
                    &ge.entry_type.type_choices[0].type1.type2
                {
                    return Some((ident.ident, ident.span));
                }
            }
            None
        }
        _ => None,
    }
}

/// Workaround for a parser bug in `cddl` v0.10.1 when parsing `~ident` inside a group choice.
///
/// When `cddl::parser::Parser::parse_type2` parses `Token::UNWRAP` (`~`) without generic arguments
/// (e.g. `~browsingContext.Info`), it advances `cur_token` from `~` to the identifier (`browsingContext.Info`),
/// constructs `Type2::Unwrap`, and returns WITHOUT advancing `cur_token` past the identifier.
///
/// Because `cur_token` is left sitting on `Token::IDENT`, the enclosing loop in `parse_grpchoice`
/// does not advance the token stream either (`!matches!(self.cur_token, Token::IDENT(..))`),
/// causing the next loop iteration to re-parse the exact same identifier token at the exact same
/// byte span (`Span`) as a second phantom entry (`TypeGroupname` in maps/groups, or `Typename` in arrays).
///
/// By checking if the entry immediately following an `Unwrap` has the exact same identifier and source
/// `Span`, we reliably drop the phantom duplicate while remaining forward-compatible if a future
/// `cddl` crate version fixes token advancement (since any real subsequent entry will have a different span).
pub fn filter_group_entries<'a, 'b>(
    entries: &'b [(cddl::ast::GroupEntry<'a>, cddl::ast::OptionalComma<'a>)],
) -> Vec<&'b cddl::ast::GroupEntry<'a>> {
    let mut result = Vec::with_capacity(entries.len());
    let mut skip_next_span: Option<(&str, cddl::ast::Span)> = None;
    for (entry, _) in entries {
        if let Some((expected_ident, expected_span)) = skip_next_span.take() {
            if expected_span != (0, 0, 0) {
                if let Some((ident, span)) = entry_ident_span(entry) {
                    if ident == expected_ident && span == expected_span {
                        continue;
                    }
                }
            }
        }
        if let cddl::ast::GroupEntry::ValueMemberKey { ge, .. } = entry {
            if ge.member_key.is_none() {
                // The parser bug only occurs when generic_args is None (`~ident`).
                // When generic_args is Some (`~ident<T>`), `parse_genericargs()` consumes
                // up to `>`, advancing `cur_token` past the identifier.
                if let Some((ident, None)) = unwrap_entry(&ge.entry_type) {
                    skip_next_span = Some((ident.ident, ident.span));
                }
            }
        }
        result.push(entry);
    }
    result
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
    fn test_unwrap_entry() {
        let cddl = cddl::parser::cddl_from_str("foo = { ~bar, baz: int }\nbar = { a: int }", true)
            .unwrap();
        if let cddl::ast::Rule::Type { rule, .. } = &cddl.rules[0] {
            if let cddl::ast::Type2::Map { group, .. } = &rule.value.type_choices[0].type1.type2 {
                let entries = filter_group_entries(&group.group_choices[0].group_entries);
                assert_eq!(entries.len(), 2);
                if let cddl::ast::GroupEntry::ValueMemberKey { ge, .. } = entries[0] {
                    let (ident, generic_args) = unwrap_entry(&ge.entry_type).unwrap();
                    assert_eq!(ident.ident, "bar");
                    assert!(generic_args.is_none());
                    return;
                }
            }
        }
        panic!("Expected unwrap entry");
    }
}
