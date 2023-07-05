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

use cddl::{ast::Occurrence, visitor::Visitor, Error};

use crate::util::{is_alphaspace, split_namespaced, to_namespaced, to_pascalcase};

const MAX_ARRAYS: usize = 1 << 3;

struct GroupChoiceContext {
    in_object: bool,
    is_first: bool,
}

struct Type1Context {
    use_generic: bool,
}

pub struct Engine {
    in_comment: bool,
    nested_group_choices: Vec<GroupChoiceContext>,
    nested_type1: Vec<Type1Context>,
}

fn is_group_entry_occurence_optional(occur: &Option<cddl::ast::Occurrence<'_>>) -> bool {
    match &occur {
        Some(Occurrence { occur, .. }) => match occur {
            cddl::ast::Occur::Exact {
                lower: Some(lower), ..
            } if *lower == 0 => true,
            cddl::ast::Occur::ZeroOrMore { .. } | cddl::ast::Occur::Optional { .. } => true,
            _ => false,
        },
        None => false,
    }
}

fn calculate_array_entry_occurence(occur: &Option<cddl::ast::Occurrence<'_>>) -> (usize, usize) {
    match &occur {
        Some(Occurrence { occur, .. }) => match occur {
            cddl::ast::Occur::ZeroOrMore { .. } => (0, usize::MAX),
            cddl::ast::Occur::Exact { lower, upper, .. } => {
                (lower.unwrap_or(0), upper.unwrap_or(usize::MAX))
            }
            cddl::ast::Occur::OneOrMore { .. } => (1, usize::MAX),
            cddl::ast::Occur::Optional { .. } => (0, 1),
        },
        _ => (1, 1),
    }
}

impl<'a, 'b: 'a, 'c> Engine {
    pub fn new() -> Engine {
        Engine {
            in_comment: false,
            nested_group_choices: Vec::new(),
            nested_type1: Vec::new(),
        }
    }
    pub fn print_preamble() {
        if cfg!(feature = "vector_groups") {
            print!(
                "export type Flatten<T extends unknown[]> = T extends (infer S)[][] \
                    ? S[] \
                    : never;"
            )
        }
    }
    /// Requires all type choices to be strings.
    fn visit_enum_type(&mut self, t: &'b cddl::ast::Type<'a>) -> cddl::visitor::Result<Error> {
        for choice in &t.type_choices {
            if let cddl::ast::Type2::TextValue { value, .. } = &choice.type1.type2 {
                println!("{} = \"{}\",", to_pascalcase(value), value.to_string());
            } else {
                panic!("Called `visit_enum_type` on non-textual type");
            }
        }
        Ok(())
    }
    fn visit_type_for_comment_inner(
        &mut self,
        t: &'b cddl::ast::Type<'a>,
    ) -> cddl::visitor::Result<Error> {
        for i in 0..t.type_choices.len() {
            self.visit_type1_for_comment(&t.type_choices[i].type1)?;
        }
        Ok(())
    }
    fn enter_comment(&mut self) {
        if !self.in_comment {
            print!("/*");
            self.in_comment = true;
        } else {
            print!(" ");
        }
        println!("*");
    }
    fn exit_comment(&mut self) {
        if self.in_comment {
            self.in_comment = false;
            println!(" */");
        }
    }
    fn visit_type_for_comment(
        &mut self,
        t: &'b cddl::ast::Type<'a>,
    ) -> cddl::visitor::Result<Error> {
        self.visit_type_for_comment_inner(&t)?;
        self.exit_comment();
        Ok(())
    }
    fn visit_type1_for_comment(
        &mut self,
        t1: &'b cddl::ast::Type1<'a>,
    ) -> cddl::visitor::Result<Error> {
        self.nested_type1.push(Type1Context { use_generic: false });
        match &t1.type2 {
            cddl::ast::Type2::ParenthesizedType { pt, .. } => {
                self.visit_type_for_comment_inner(&pt)?
            }
            _ => {}
        }
        if let Some(op) = &t1.operator {
            match op.operator {
                cddl::ast::RangeCtlOp::RangeOp { is_inclusive, .. } => {
                    self.enter_comment();
                    print!(" * Must be between `");
                    self.visit_type2(&t1.type2)?;
                    print!("` and `");
                    self.visit_type2(&op.type2)?;
                    print!("`");
                    if is_inclusive {
                        print!(", inclusive");
                    }
                    println!(".");
                }
                cddl::ast::RangeCtlOp::CtlOp { ctrl, .. } => match ctrl {
                    cddl::token::ControlOperator::DEFAULT => {
                        self.enter_comment();
                        print!(" * @defaultValue `");
                        self.visit_type2(&op.type2)?;
                        println!("`");
                    }
                    cddl::token::ControlOperator::SIZE => {
                        self.enter_comment();
                        print!(" * Must be `");
                        self.visit_type2(&op.type2)?;
                        println!("` units in length.");
                    }
                    cddl::token::ControlOperator::PCRE | cddl::token::ControlOperator::REGEXP => {
                        self.enter_comment();
                        print!(" * Must match the pattern `");
                        self.visit_type2(&op.type2)?;
                        println!("`.");
                    }
                    cddl::token::ControlOperator::LT => {
                        self.enter_comment();
                        print!(" * Must be less than `");
                        self.visit_type2(&op.type2)?;
                        println!("`.");
                    }
                    cddl::token::ControlOperator::LE => {
                        self.enter_comment();
                        print!(" * Must be less than or equal to `");
                        self.visit_type2(&op.type2)?;
                        println!("`.");
                    }
                    cddl::token::ControlOperator::GT => {
                        self.enter_comment();
                        print!(" * Must be greater than `");
                        self.visit_type2(&op.type2)?;
                        println!("`.");
                    }
                    cddl::token::ControlOperator::GE => {
                        self.enter_comment();
                        print!(" * Must be greater than or equal to `");
                        self.visit_type2(&op.type2)?;
                        println!("`.");
                    }
                    cddl::token::ControlOperator::EQ => {
                        self.enter_comment();
                        print!(" * Must be equal to `");
                        self.visit_type2(&op.type2)?;
                        println!("`.");
                    }
                    cddl::token::ControlOperator::NE => {
                        self.enter_comment();
                        print!(" * Must be not equal `");
                        self.visit_type2(&op.type2)?;
                        println!("`.");
                    }
                    _ => {}
                },
            }
        }
        self.nested_type1.pop();
        Ok(())
    }
    fn visit_value(&mut self, value: &cddl::token::Value<'a>) -> cddl::visitor::Result<Error> {
        match value {
            cddl::token::Value::INT(value) => print!("{}", value),
            cddl::token::Value::UINT(value) => print!("{}", value),
            cddl::token::Value::FLOAT(value) => print!("{}", value),
            cddl::token::Value::TEXT(value) => print!("\"{}\"", value),
            cddl::token::Value::BYTE(value) => print!("\"{}\"", value),
        }
        Ok(())
    }
    fn visit_array(&mut self, g: &'b cddl::ast::Group<'a>) -> cddl::visitor::Result<Error> {
        for (index, choice) in g.group_choices.iter().enumerate() {
            if index != 0 {
                print!("|");
            }
            self.visit_array_choice(choice)?;
        }
        Ok(())
    }
    fn visit_array_choice(
        &mut self,
        gc: &'b cddl::ast::GroupChoice<'a>,
    ) -> cddl::visitor::Result<Error> {
        self.nested_group_choices.push(GroupChoiceContext {
            in_object: false,
            is_first: true,
        });
        if gc.group_entries.is_empty() {
            self.enter_array();
        }
        for (index, (entry, _)) in gc.group_entries.iter().enumerate() {
            self.nested_group_choices.last_mut().unwrap().is_first = index == 0;
            self.visit_array_entry(entry)?;
        }
        self.exit_array();
        self.nested_group_choices.pop();
        Ok(())
    }
    fn print_group_joiner(&mut self) {
        if let Some(group) = self.nested_group_choices.last_mut() {
            if group.in_object {
                if !group.is_first {
                    print!(",");
                }
            } else {
                if !group.is_first {
                    print!("&");
                }
            }
        }
    }
    fn enter_array(&mut self) {
        if let Some(group) = self.nested_group_choices.last_mut() {
            if !group.in_object {
                println!("[");
                group.in_object = true;
            }
        }
    }
    fn exit_array(&mut self) {
        if let Some(group) = self.nested_group_choices.last_mut() {
            if group.in_object {
                group.in_object = false;
                print!("]");
            }
        }
    }
    fn enter_map(&mut self) {
        if let Some(group) = self.nested_group_choices.last_mut() {
            if !group.in_object {
                println!("{{");
                group.in_object = true;
            }
        }
    }
    fn exit_map(&mut self) {
        if let Some(group) = self.nested_group_choices.last_mut() {
            if group.in_object {
                group.in_object = false;
                print!("}}");
            }
        }
    }
    fn visit_array_entry(
        &mut self,
        entry: &'b cddl::ast::GroupEntry<'a>,
    ) -> cddl::visitor::Result<Error> {
        match entry {
            cddl::ast::GroupEntry::ValueMemberKey { ge, .. } => {
                self.visit_value_array_member_key_entry(ge)?;
            }
            cddl::ast::GroupEntry::TypeGroupname { ge, .. } => {
                self.visit_type_arrayname_entry(ge)?;
            }
            cddl::ast::GroupEntry::InlineGroup { group, .. } => {
                self.print_group_joiner();
                self.enter_array();
                print!("...(");
                self.visit_array(&group)?;
                print!(")");
            }
        }
        Ok(())
    }
    fn visit_value_array_member_key_entry(
        &mut self,
        entry: &'b cddl::ast::ValueMemberKeyEntry<'a>,
    ) -> cddl::visitor::Result<Error> {
        if let Some(mk) = &entry.member_key {
            eprintln!("Keys are not supported for arrays. Ignoring key: {}", mk);
        }
        self.print_group_joiner();
        self.enter_array();
        self.visit_type_for_comment(&entry.entry_type)?;
        match calculate_array_entry_occurence(&entry.occur) {
            (lower, upper) if lower == upper => {
                for index in 0..lower {
                    if index != 0 {
                        print!(",");
                    }
                    self.visit_type(&entry.entry_type)?
                }
            }
            (lower, upper) => {
                print!("...(");
                if upper < MAX_ARRAYS {
                    for bound in lower..upper + 1 {
                        if bound != 0 {
                            print!("|");
                        }
                        print!("[");
                        for _ in 0..bound {
                            self.visit_type(&entry.entry_type)?;
                            print!(",");
                        }
                        print!("]");
                    }
                } else {
                    self.visit_type(&entry.entry_type)?;
                    print!("[]");
                }
                print!(")");
            }
        }
        Ok(())
    }
    fn visit_type_arrayname_entry(
        &mut self,
        entry: &'b cddl::ast::TypeGroupnameEntry<'a>,
    ) -> cddl::visitor::Result<Error> {
        self.print_group_joiner();
        self.enter_array();
        match calculate_array_entry_occurence(&entry.occur) {
            (lower, upper) if lower == upper => {
                for index in 0..lower {
                    if index != 0 {
                        print!(",");
                    }
                    if cfg!(feature = "vector_groups") {
                        print!("...");
                        self.visit_identifier_with_args(&entry.name, &entry.generic_args)?;
                        print!("Vector");
                    } else {
                        print!("(");
                        self.visit_identifier_with_args(&entry.name, &entry.generic_args)?;
                        print!(")");
                    }
                }
            }
            (lower, upper) => {
                print!("...(");
                if upper < MAX_ARRAYS {
                    for bound in lower..upper + 1 {
                        if bound != 0 {
                            print!("|");
                        }
                        print!("[");
                        for _ in 0..bound {
                            if cfg!(feature = "vector_groups") {
                                print!("...");
                                self.visit_identifier_with_args(&entry.name, &entry.generic_args)?;
                                print!("Vector");
                            } else {
                                print!("(");
                                self.visit_identifier_with_args(&entry.name, &entry.generic_args)?;
                                print!(")");
                            }
                            print!(",");
                        }
                        print!("]");
                    }
                } else {
                    if cfg!(feature = "vector_groups") {
                        print!("Flatten<");
                        self.visit_identifier_with_args(&entry.name, &entry.generic_args)?;
                        print!("Vector");
                        print!("[]");
                        print!(">");
                    } else {
                        print!("(");
                        self.visit_identifier_with_args(&entry.name, &entry.generic_args)?;
                        print!(")");
                        print!("[]");
                    }
                }
                print!(")");
            }
        }
        Ok(())
    }

    fn visit_identifier_with_params(
        &mut self,
        ident: &cddl::ast::Identifier<'a>,
        params: &Option<cddl::ast::GenericParams<'a>>,
    ) -> cddl::visitor::Result<Error> {
        self.visit_identifier(ident)?;
        if let Some(params) = params {
            print!("<");
            for param in &params.params {
                self.visit_identifier(&param.param)?;
                print!(",")
            }
            print!(">");
        }
        Ok(())
    }

    fn visit_identifier_with_args(
        &mut self,
        ident: &cddl::ast::Identifier<'a>,
        params: &Option<cddl::ast::GenericArgs<'a>>,
    ) -> cddl::visitor::Result<Error> {
        self.visit_identifier(ident)?;
        if let Some(params) = params {
            print!("<");
            for param in &params.args {
                self.visit_type1(&param.arg)?;
                print!(",")
            }
            print!(">");
        }
        Ok(())
    }
}

impl<'a, 'b: 'a> Visitor<'a, 'b, Error> for Engine {
    fn visit_identifier(
        &mut self,
        ident: &cddl::ast::Identifier<'a>,
    ) -> cddl::visitor::Result<Error> {
        match ident.ident {
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
            "any" => print!("any"),
            "nil" | "null" => print!("null"),
            "true" => print!("true"),
            "uri" => print!("URL"),
            "regexp" => print!("RegExp"),
            "false" => print!("false"),
            "undefined" => print!("undefined"),
            ident => print!("{}", to_namespaced(ident)),
        }
        Ok(())
    }
    fn visit_type_rule(&mut self, tr: &'b cddl::ast::TypeRule<'a>) -> cddl::visitor::Result<Error> {
        let (namespaces, type_name) = split_namespaced(&tr.name);
        for namespace in &namespaces {
            println!("export namespace {} {{", namespace);
        }
        if tr.value.type_choices.iter().all(|choice| {
            if let cddl::ast::Type2::TextValue { value, .. } = &choice.type1.type2 {
                is_alphaspace(value)
            } else {
                false
            }
        }) {
            print!("export const enum {} {{", type_name);
            self.visit_enum_type(&tr.value)?;
            println!("}}");
        } else {
            self.visit_type_for_comment(&tr.value)?;
            print!("export type ");
            self.visit_identifier_with_params(
                &cddl::ast::Identifier {
                    ident: &type_name,
                    socket: None,
                    span: Default::default(),
                },
                &tr.generic_params,
            )?;
            print!(" = ");
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

        let choice = cddl::ast::GroupChoice {
            group_entries: vec![(
                gr.entry.clone(),
                cddl::ast::OptionalComma {
                    optional_comma: false,
                    trailing_comments: None,
                    _a: std::marker::PhantomData,
                },
            )],
            span: Default::default(),
            comments_before_grpchoice: None,
        };

        // Group rules are objects that behave depending on their context.
        // Specifically, if a group rule is composed inside a map or an array,
        // it behaves as though it is a map or an array respectively.
        //
        // This requires us to build to types in case of usage: one for use as a
        // map and the other for use as an array.
        println!("export type {} = ", type_name);
        self.visit_group_choice(&choice)?;
        println!(";");

        if cfg!(feature = "vector_groups") {
            println!("export type {}Vector = ", type_name);
            self.visit_array_choice(&choice)?;
            println!(";");
        }

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
            cddl::ast::GroupEntry::ValueMemberKey { ge, .. } => {
                self.visit_value_member_key_entry(ge)?;
            }
            cddl::ast::GroupEntry::TypeGroupname { ge, .. } => {
                self.visit_type_groupname_entry(ge)?;
            }
            cddl::ast::GroupEntry::InlineGroup { group, .. } => {
                self.exit_map();
                self.print_group_joiner();
                self.visit_group(group)?;
            }
        }
        Ok(())
    }
    fn visit_value_member_key_entry(
        &mut self,
        entry: &'b cddl::ast::ValueMemberKeyEntry<'a>,
    ) -> cddl::visitor::Result<Error> {
        let mk = match &entry.member_key {
            Some(mk) => mk,
            None => {
                eprintln!(
                    "Expected key for value of type {} since this is a map. \
                    Did you mean to declare {} with parenthesis (`( .. )`) \
                    instead of brackets (`{{ .. }}`)?",
                    entry.entry_type, entry.entry_type
                );
                // TODO: This is a temporary fix for situations where a typename
                // should be used instead of a groupname.
                self.exit_map();
                self.print_group_joiner();
                if is_group_entry_occurence_optional(&entry.occur) {
                    print!("Partial<");
                    self.visit_type(&entry.entry_type)?;
                    print!(">");
                } else {
                    self.visit_type(&entry.entry_type)?;
                }
                return Ok(());
            }
        };
        self.print_group_joiner();
        self.enter_map();
        print!("  ");
        self.visit_type_for_comment(&entry.entry_type)?;
        self.visit_memberkey(&mk)?;
        if is_group_entry_occurence_optional(&entry.occur)
            && !matches!(&mk, cddl::ast::MemberKey::Type1 { is_cut: false, .. })
        {
            print!("?");
        }
        print!(":");
        self.visit_type(&entry.entry_type)?;
        Ok(())
    }
    fn visit_type_groupname_entry(
        &mut self,
        entry: &'b cddl::ast::TypeGroupnameEntry<'a>,
    ) -> cddl::visitor::Result<Error> {
        self.exit_map();
        self.print_group_joiner();
        if is_group_entry_occurence_optional(&entry.occur) {
            print!("Partial<");
            self.visit_identifier_with_args(&entry.name, &entry.generic_args)?;
            print!(">");
        } else {
            self.visit_identifier_with_args(&entry.name, &entry.generic_args)?;
        }
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
    fn visit_group_choice(
        &mut self,
        gc: &'b cddl::ast::GroupChoice<'a>,
    ) -> cddl::visitor::Result<Error> {
        self.nested_group_choices.push(GroupChoiceContext {
            in_object: false,
            is_first: true,
        });
        if gc.group_entries.is_empty() {
            self.enter_map();
        }
        for (index, (entry, _)) in gc.group_entries.iter().enumerate() {
            self.nested_group_choices.last_mut().unwrap().is_first = index == 0;
            self.visit_group_entry(entry)?;
        }
        self.exit_map();
        self.nested_group_choices.pop();
        Ok(())
    }
    fn visit_memberkey(
        &mut self,
        mk: &'b cddl::ast::MemberKey<'a>,
    ) -> cddl::visitor::Result<Error> {
        match mk {
            cddl::ast::MemberKey::Type1 { t1, is_cut, .. } => {
                print!("[");
                if !is_cut {
                    print!("key: ");
                    self.visit_type1(t1)?;
                } else {
                    self.visit_type1(t1)?;
                }
                print!("]");
            }
            cddl::ast::MemberKey::Bareword { ident, .. } => {
                print!("\"{}\"", &ident);
            }
            cddl::ast::MemberKey::Value { value, .. } => {
                print!("[");
                self.visit_value(value)?;
                print!("]");
            }
            cddl::ast::MemberKey::NonMemberKey { .. } => {
                unimplemented!()
            }
        }
        Ok(())
    }
    fn visit_type1(&mut self, t1: &'b cddl::ast::Type1<'a>) -> cddl::visitor::Result<Error> {
        self.nested_type1.push(Type1Context {
            use_generic: matches!(
                t1.operator,
                Some(cddl::ast::Operator {
                    operator: cddl::ast::RangeCtlOp::RangeOp { .. },
                    ..
                })
            ),
        });
        self.visit_type2(&t1.type2)?;
        if let Some(cddl::ast::Operator {
            operator: cddl::ast::RangeCtlOp::CtlOp { ctrl, .. },
            type2,
            ..
        }) = &t1.operator
        {
            match ctrl {
                cddl::token::ControlOperator::WITHIN => {
                    println!(" extends ");
                    self.visit_type2(&type2)?;
                    println!(" ? ");
                    self.visit_type2(&t1.type2)?;
                    println!(" : never");
                }
                cddl::token::ControlOperator::AND => {
                    println!(" & ");
                    self.visit_type2(&type2)?;
                }
                _ => {}
            }
        }
        self.nested_type1.pop();
        Ok(())
    }
    fn visit_type2(&mut self, t2: &'b cddl::ast::Type2<'a>) -> cddl::visitor::Result<Error> {
        match t2 {
            cddl::ast::Type2::Typename {
                ident,
                generic_args,
                ..
            } => {
                self.visit_identifier_with_args(&ident, &generic_args)?;
            }
            cddl::ast::Type2::Array { group, .. } => {
                self.visit_array(&group)?;
            }
            cddl::ast::Type2::Any { .. } => print!("any"),
            // The default has the correct behavior for the rest of the cases.
            t2 => {
                cddl::visitor::walk_type2(self, t2)?;
            }
        }
        Ok(())
    }

    fn visit_value(&mut self, value: &cddl::token::Value<'a>) -> cddl::visitor::Result<Error> {
        if self.nested_type1.last().unwrap().use_generic {
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
