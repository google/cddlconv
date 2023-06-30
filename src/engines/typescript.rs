use cddl::{Error, visitor::Visitor};

use crate::util::{to_namespaced, split_namespaced, to_pascalcase, is_alpha};

#[derive(Default)]
struct GroupChoiceContext {
    in_object: bool,
    is_first: bool,
}

#[derive(Default)]
struct Type1Context {
    in_range: bool,
}

pub struct Engine<'a> {
    occurence: Option<&'a cddl::ast::Occurrence<'a>>,
    nested_groups: Vec<GroupChoiceContext>,
    nested_type1: Vec<Type1Context>,
}

impl<'a, 'b: 'a> Engine<'a> {
    pub fn new() -> Engine<'a> {
        Engine {
            occurence: None,
            nested_groups: Vec::new(),
            nested_type1: Vec::new(),
        }
    }
    /// Requires all type choices to be strings.
    fn visit_enum_type(&mut self, t: &'b cddl::ast::Type<'a>) -> cddl::visitor::Result<Error> {
        for choice in &t.type_choices {
            if let cddl::ast::Type2::TextValue { value, span } = &choice.type1.type2 {
                println!("{} = \"{}\",", to_pascalcase(value), value.to_string());
            } else {
                panic!("Called `visit_enum_type` on non-textual type");
            }
        }
        Ok(())
    }
}

impl<'a, 'b: 'a> Visitor<'a, 'b, Error> for Engine<'a> {
    fn visit_identifier(
        &mut self,
        ident: &cddl::ast::Identifier<'a>,
    ) -> cddl::visitor::Result<Error> {
        print!("{}", to_namespaced(ident.ident));
        Ok(())
    }
    fn visit_type_rule(&mut self, tr: &'b cddl::ast::TypeRule<'a>) -> cddl::visitor::Result<Error> {
        let (namespaces, type_name) = split_namespaced(&tr.name);
        for namespace in &namespaces {
            println!("export namespace {} {{", namespace);
        }
        if tr.value.type_choices.iter().all(|choice| {
            if let cddl::ast::Type2::TextValue { value, .. } = &choice.type1.type2 {
                is_alpha(value)
            } else {
                false
            }
        }) {
            print!("export const enum {} {{", type_name);
            self.visit_enum_type(&tr.value)?;
            println!("}}");
        } else {
            print!("export type {} = ", type_name);
            self.visit_type(&tr.value)?;
            println!(";");
        }
        for _ in &namespaces {
            println!("}}");
        }
        Ok(())
    }
    fn visit_type(&mut self, t: &'b cddl::ast::Type<'a>) -> cddl::visitor::Result<Error> {
        print!("(");
        for i in 0..t.type_choices.len() {
            if i != 0 {
                print!("| ");
            }
            self.visit_type1(&t.type_choices[i].type1)?;
        }
        print!(")");
        Ok(())
    }
    fn visit_group_rule(
        &mut self,
        gr: &'b cddl::ast::GroupRule<'a>,
    ) -> cddl::visitor::Result<Error> {
        let (namespaces, type_name) = split_namespaced(&gr.name);
        for namespace in &namespaces {
            println!("export namespace {} {{", namespace);
        }

        println!("export type {} = ", type_name);
        self.visit_group_entry(&gr.entry)?;
        println!(";");

        for _ in &namespaces {
            println!("}}");
        }
        Ok(())
    }
    fn visit_group_entry(
        &mut self,
        entry: &'b cddl::ast::GroupEntry<'a>,
    ) -> cddl::visitor::Result<Error> {
        match entry {
            cddl::ast::GroupEntry::ValueMemberKey {
                ge,
                span,
                leading_comments,
                trailing_comments,
            } => {
                self.visit_value_member_key_entry(ge)?;
            }
            cddl::ast::GroupEntry::TypeGroupname {
                ge,
                span,
                leading_comments,
                trailing_comments,
            } => {
                if let Some(group) = self.nested_groups.last_mut() {
                    if group.in_object {
                        group.in_object = false;
                        print!("}} ");
                    }
                    if !group.is_first {
                        print!("& ");
                    }
                }
                self.visit_type_groupname_entry(ge)?;
            }
            cddl::ast::GroupEntry::InlineGroup {
                occur,
                group,
                span,
                comments_before_group,
                comments_after_group,
            } => {
                if let Some(group) = self.nested_groups.last_mut() {
                    if group.in_object {
                        group.in_object = false;
                        print!("}} ");
                    }
                    if !group.is_first {
                        print!("& ");
                    }
                }
                self.visit_group(group)?;
            }
        }
        Ok(())
    }
    fn visit_group_choice(
        &mut self,
        gc: &'b cddl::ast::GroupChoice<'a>,
    ) -> cddl::visitor::Result<Error> {
        self.nested_groups.push(Default::default());
        if gc.group_entries.is_empty() {
            let group = self.nested_groups.last_mut().unwrap();
            group.in_object = true;
            print!("{{");
        }
        for i in 0..gc.group_entries.len() {
            self.nested_groups.last_mut().unwrap().is_first = i == 0;
            self.visit_group_entry(&gc.group_entries[i].0)?;
        }
        let group = self.nested_groups.last_mut().unwrap();
        if group.in_object {
            group.in_object = false;
            print!("}} ");
        }
        self.nested_groups.pop();
        Ok(())
    }
    fn visit_group(&mut self, g: &'b cddl::ast::Group<'a>) -> cddl::visitor::Result<Error> {
        print!("(");
        for i in 0..g.group_choices.len() {
            if i != 0 {
                print!("| ");
            }
            self.visit_group_choice(&g.group_choices[i])?;
        }
        print!(")");
        Ok(())
    }
    fn visit_value_member_key_entry(
        &mut self,
        entry: &'b cddl::ast::ValueMemberKeyEntry<'a>,
    ) -> cddl::visitor::Result<Error> {
        self.occurence = entry.occur.as_ref();
        if let Some(mk) = &entry.member_key {
            if let Some(group) = self.nested_groups.last_mut() {
                if !group.in_object {
                    if !group.is_first {
                        print!("& ");
                    }
                    println!("{{");
                    group.in_object = true;
                }
            }
            print!("  ");
            self.visit_memberkey(&mk)?;
            self.visit_type(&entry.entry_type)?;
            println!(",");
        } else {
            if let Some(group) = self.nested_groups.last_mut() {
                if group.in_object {
                    group.in_object = false;
                    print!("}} ");
                }
                if !group.is_first {
                    print!("& ");
                }
            }
            self.visit_type(&entry.entry_type)?;
        }
        Ok(())
    }
    fn visit_type_groupname_entry(
        &mut self,
        entry: &'b cddl::ast::TypeGroupnameEntry<'a>,
    ) -> cddl::visitor::Result<Error> {
        self.visit_identifier(&entry.name)
    }
    fn visit_memberkey(
        &mut self,
        mk: &'b cddl::ast::MemberKey<'a>,
    ) -> cddl::visitor::Result<Error> {
        match mk {
            cddl::ast::MemberKey::Type1 {
                t1,
                is_cut,
                span,
                comments_before_cut,
                comments_after_cut,
                comments_after_arrowmap,
            } => {
                print!("[key: ",);
                self.visit_type1(t1)?;
                print!("]: ");
            }
            cddl::ast::MemberKey::Bareword {
                ident,
                span,
                comments,
                comments_after_colon,
            } => {
                print!(
                    "{}{}: ",
                    &ident,
                    match self.occurence {
                        Some(cddl::ast::Occurrence {
                            occur: cddl::ast::Occur::Optional { .. },
                            ..
                        }) => "?",
                        _ => "",
                    }
                );
            }
            cddl::ast::MemberKey::Value {
                value,
                span,
                comments,
                comments_after_colon,
            } => {}
            cddl::ast::MemberKey::NonMemberKey {
                non_member_key,
                comments_before_type_or_group,
                comments_after_type_or_group,
            } => {
                println!("{:?}", non_member_key);
            }
        }
        Ok(())
    }
    fn visit_type1(&mut self, t1: &'b cddl::ast::Type1<'a>) -> cddl::visitor::Result<Error> {
        self.nested_type1.push(Type1Context {
            in_range: matches!(
                t1.operator,
                Some(cddl::ast::Operator {
                    operator: cddl::ast::RangeCtlOp::RangeOp { .. },
                    ..
                })
            ),
        });
        self.visit_type2(&t1.type2)?;
        self.nested_type1.pop();
        Ok(())
    }
    fn visit_type2(&mut self, t2: &'b cddl::ast::Type2<'a>) -> cddl::visitor::Result<Error> {
        match t2 {
            cddl::ast::Type2::Typename { ident, .. } => match ident.ident {
                "bool" => print!("boolean"),
                "uint" | "nint" | "int" | "float16" | "float32" | "float64" | "float16-32"
                | "float32-64" | "float" | "number" => {
                    print!("number")
                }
                "biguint" | "bignint" | "bigint" => {
                    print!("bigint")
                }
                "bstr" | "bytes" => print!("Uint8Array"),
                "tstr" | "text" => print!("string"),
                "any" => print!("never"),
                "nil" | "null" => print!("null"),
                "true" => print!("true"),
                "uri" => print!("URL"),
                "regexp" => print!("RegExp"),
                "false" => print!("false"),
                "undefined" => print!("undefined"),
                _ => cddl::visitor::walk_type2(self, t2)?,
            },
            // cddl::ast::Type2::ParenthesizedType { pt, span, comments_before_type, comments_after_type } => todo!(),
            // cddl::ast::Type2::Map { group, span, comments_before_group, comments_after_group } => todo!(),
            cddl::ast::Type2::Array { .. } => {
                print!("Array<");
                cddl::visitor::walk_type2(self, t2)?;
                print!(">");
            }
            // cddl::ast::Type2::Unwrap { ident, generic_args, span, comments } => todo!(),
            // cddl::ast::Type2::ChoiceFromInlineGroup { group, span, comments, comments_before_group, comments_after_group } => todo!(),
            // cddl::ast::Type2::ChoiceFromGroup { ident, generic_args, span, comments } => todo!(),
            // cddl::ast::Type2::TaggedData { tag, t, span, comments_before_type, comments_after_type } => todo!(),
            // cddl::ast::Type2::DataMajorType { mt, constraint, span } => todo!(),
            // cddl::ast::Type2::Any { span } => todo!(),
            t2 => {
                cddl::visitor::walk_type2(self, t2)?;
            }
        }
        Ok(())
    }

    fn visit_value(&mut self, value: &cddl::token::Value<'a>) -> cddl::visitor::Result<Error> {
        if self.nested_type1.last().unwrap().in_range {
            match value {
                cddl::token::Value::INT(_)
                | cddl::token::Value::UINT(_)
                | cddl::token::Value::FLOAT(_) => print!("number"),
                cddl::token::Value::TEXT(_) | cddl::token::Value::BYTE(_) => print!("string"),
            }
        } else {
            match value {
                cddl::token::Value::INT(value) => print!("{}", value),
                cddl::token::Value::UINT(value) => print!("{}", value),
                cddl::token::Value::FLOAT(value) => print!("{}", value),
                cddl::token::Value::TEXT(value) => print!("\"{}\"", value),
                cddl::token::Value::BYTE(value) => print!("\"{}\"", value),
            }
        }
        Ok(())
    }
}
