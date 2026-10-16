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

pub fn collect_group_rules(cddl: &cddl::ast::CDDL) -> std::collections::HashSet<String> {
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

fn type_is_group_ref(
    t: &cddl::ast::Type,
    group_rules: &std::collections::HashSet<String>,
) -> bool {
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

pub fn type_rule_as_group_rule<'a>(
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

pub fn member_key_literal_value<'a>(
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

pub fn is_non_cut_non_literal_member_key(mk: &cddl::ast::MemberKey) -> bool {
    matches!(mk, cddl::ast::MemberKey::Type1 { is_cut: false, .. })
        && member_key_literal_value(mk).is_none()
}

pub fn array_group_entry_as_type_entry<'a>(
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
