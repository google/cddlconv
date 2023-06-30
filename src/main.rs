use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use cddl::{
    ast::{parent::Error, Identifier},
    visitor::{walk_type2, Visitor},
};
use clap::{Parser, ValueEnum};
use convert_case::Casing;

fn split_identifier<T: ToString>(value: T) -> Vec<String> {
    return value.to_string().split('.').map(String::from).collect();
}

fn to_pascalcase<T: ToString>(value: T) -> String {
    value.to_string().to_case(convert_case::Case::Pascal)
}

fn to_namespaced<T: ToString>(value: T) -> String {
    split_identifier(&value.to_string())
        .into_iter()
        .map(to_pascalcase)
        .collect::<Vec<String>>()
        .join(".")
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum EngineType {
    TypeScript,
}

#[derive(Default)]
struct GroupChoiceContext {
    in_object: bool,
    is_first: bool,
}

struct TypeScriptEngine<'a> {
    occurence: Option<&'a cddl::ast::Occurrence<'a>>,
    nested_groups: Vec<GroupChoiceContext>,
}

impl<'a, 'b: 'a> TypeScriptEngine<'a> {
    fn new() -> TypeScriptEngine<'a> {
        TypeScriptEngine {
            occurence: None,
            nested_groups: Vec::new(),
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

fn is_alpha<T: AsRef<str>>(value: T) -> bool {
    value
        .as_ref()
        .to_ascii_lowercase()
        .bytes()
        .all(|ch| b'a' <= ch && ch <= b'z')
}

impl<'a, 'b: 'a> Visitor<'a, 'b, Error> for TypeScriptEngine<'a> {
    fn visit_type_rule(&mut self, tr: &'b cddl::ast::TypeRule<'a>) -> cddl::visitor::Result<Error> {
        let mut parts = split_identifier(&tr.name)
            .into_iter()
            .map(to_pascalcase)
            .collect::<Vec<String>>();
        let value = parts.pop().unwrap();
        for part in &parts {
            println!("export namespace {} {{", part);
        }
        if tr.value.type_choices.iter().all(|choice| {
            if let cddl::ast::Type2::TextValue { value, .. } = &choice.type1.type2 {
                is_alpha(value)
            } else {
                false
            }
        }) {
            print!("export const enum {} {{", value);
            self.visit_enum_type(&tr.value)?;
            println!("}}");
        } else {
            print!("export type {} = ", value);
            self.visit_type(&tr.value)?;
            println!(";");
        }
        for _ in &parts {
            println!("}}");
        }
        Ok(())
    }
    fn visit_type(&mut self, t: &'b cddl::ast::Type<'a>) -> cddl::visitor::Result<Error> {
        for i in 0..t.type_choices.len() {
            if i != 0 {
                print!("| ");
            }
            self.visit_type1(&t.type_choices[i].type1)?;
        }
        Ok(())
    }
    fn visit_group_rule(
        &mut self,
        gr: &'b cddl::ast::GroupRule<'a>,
    ) -> cddl::visitor::Result<Error> {
        let mut parts = split_identifier(&gr.name)
            .into_iter()
            .map(to_pascalcase)
            .collect::<Vec<String>>();
        let value = parts.pop().unwrap();
        for part in &parts {
            println!("export namespace {} {{", part);
        }

        println!("export type {} = ", value);
        self.visit_group_entry(&gr.entry)?;
        println!(";");

        for _ in &parts {
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
        for i in 0..g.group_choices.len() {
            if i != 0 {
                print!("| ");
            }
            self.visit_group_choice(&g.group_choices[i])?;
        }
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
        println!("{}", to_namespaced(&entry.name));
        Ok(())
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
        self.visit_type2(&t1.type2)
    }
    fn visit_type2(&mut self, t2: &'b cddl::ast::Type2<'a>) -> cddl::visitor::Result<Error> {
        match t2 {
            cddl::ast::Type2::Typename {
                ident,
                generic_args,
                span,
            } => match ident.ident {
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
                ident => print!("{}", to_namespaced(ident)),
            },
            cddl::ast::Type2::IntValue { .. }
            | cddl::ast::Type2::UintValue { .. }
            | cddl::ast::Type2::FloatValue { .. } => {
                print!("number");
            }
            cddl::ast::Type2::TextValue { value, span } => {
                print!("'{}'", value);
            }
            // cddl::ast::Type2::UTF8ByteString { value, span } => todo!(),
            // cddl::ast::Type2::B16ByteString { value, span } => todo!(),
            // cddl::ast::Type2::B64ByteString { value, span } => todo!(),
            // cddl::ast::Type2::ParenthesizedType { pt, span, comments_before_type, comments_after_type } => todo!(),
            // cddl::ast::Type2::Map { group, span, comments_before_group, comments_after_group } => todo!(),
            cddl::ast::Type2::Array {
                group,
                span,
                comments_before_group,
                comments_after_group,
            } => {
                print!("Array<");
                self.visit_group(&group)?;
                print!(">");
            }
            // cddl::ast::Type2::Unwrap { ident, generic_args, span, comments } => todo!(),
            // cddl::ast::Type2::ChoiceFromInlineGroup { group, span, comments, comments_before_group, comments_after_group } => todo!(),
            // cddl::ast::Type2::ChoiceFromGroup { ident, generic_args, span, comments } => todo!(),
            // cddl::ast::Type2::TaggedData { tag, t, span, comments_before_type, comments_after_type } => todo!(),
            // cddl::ast::Type2::DataMajorType { mt, constraint, span } => todo!(),
            // cddl::ast::Type2::Any { span } => todo!(),
            t2 => {
                walk_type2(self, t2)?;
            }
        }
        Ok(())
    }
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File to convert.
    file: PathBuf,
    /// Format to output.
    #[arg(short, long, value_enum, default_value_t = EngineType::TypeScript)]
    format: EngineType,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let input = std::fs::read_to_string(args.file)?;
    let cddl =
        cddl::parser::cddl_from_str(&input, true).map_err(|error| anyhow::Error::msg(error))?;

    match args.format {
        EngineType::TypeScript => {
            let mut engine = TypeScriptEngine::new();
            engine.visit_cddl(&cddl)?;
        }
    };

    Ok(())
}
