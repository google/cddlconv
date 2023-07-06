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

use crate::util::{split_namespaced, to_namespaced};

const MAX_ARRAYS: usize = 1 << 3;

#[repr(packed)]
struct GroupChoiceContext {
    in_object: bool,
    is_first: bool,
    in_record: bool,
    in_and: bool,
}

#[derive(Copy, Clone)]
enum ValueMode {
    Literal,
    Generic,
    JavaScript,
}

struct Type1Context {
    value_mode: ValueMode,
}

struct PostambleOptions {
    #[cfg(feature = "vector_groups")]
    print_flatten: bool,
}

pub struct Engine {
    nested_group_choices: Vec<GroupChoiceContext>,
    nested_type1: Vec<Type1Context>,
    #[allow(dead_code)]
    postamble_options: PostambleOptions,
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
            nested_group_choices: Vec::new(),
            nested_type1: Vec::new(),
            postamble_options: PostambleOptions {
                #[cfg(feature = "vector_groups")]
                print_flatten: false,
            },
        }
    }
    pub fn print_preamble() {
        println!("import z from 'zod';");
    }
    pub fn print_postamble(&mut self) {
        #[cfg(feature = "vector_groups")]
        if self.postamble_options.print_flatten {
            print!(
                "export type Flatten<T extends unknown[]> = T extends (infer S)[][] \
                    ? S[] \
                    : never;"
            )
        }
    }
    fn visit_maybe_enum_type(&mut self, t: &'b cddl::ast::Type<'a>) -> bool {
        // Special case for string enums
        if t.type_choices.len() > 1
            && t.type_choices
                .iter()
                .map(|choice| &choice.type1.type2)
                .all(|type2| matches!(type2, cddl::ast::Type2::TextValue { .. }))
        {
            print!("z.enum([");
            for type2 in t.type_choices.iter().map(|choice| &choice.type1.type2) {
                if let cddl::ast::Type2::TextValue { value, .. } = type2 {
                    print!("\"{}\",", value);
                }
            }
            print!("])");
            true
        } else {
            false
        }
    }
    fn visit_array(&mut self, g: &'b cddl::ast::Group<'a>) -> cddl::visitor::Result<Error> {
        if g.group_choices.len() != 1 {
            print!("z.union([");
        }
        for (index, choice) in g.group_choices.iter().enumerate() {
            if index != 0 {
                print!(",");
            }
            self.visit_array_choice(choice)?;
        }
        if g.group_choices.len() != 1 {
            print!("])");
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
            in_record: false,
            in_and: false,
        });
        if gc.group_entries.is_empty() {
            self.enter_tuple();
        }
        for (index, (entry, _)) in gc.group_entries.iter().enumerate() {
            self.nested_group_choices.last_mut().unwrap().is_first = index == 0;
            self.visit_array_entry(entry)?;
        }
        self.exit_tuple();
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
                if group.in_and {
                    group.in_and = false;
                    println!(")");
                }
                if !group.is_first {
                    self.enter_and();
                }
            }
        }
    }
    fn enter_tuple(&mut self) {
        if let Some(group) = self.nested_group_choices.last_mut() {
            if !group.in_object {
                println!("z.tuple([");
                group.in_object = true;
            }
        }
    }
    fn exit_tuple(&mut self) {
        if let Some(group) = self.nested_group_choices.last_mut() {
            if group.in_object {
                group.in_object = false;
                print!("])");
            }
        }
        self.exit_and();
    }
    fn enter_and(&mut self) {
        if let Some(group) = self.nested_group_choices.last_mut() {
            if !group.in_and {
                println!(".and(");
                group.in_and = true;
            }
        }
    }
    fn exit_and(&mut self) {
        if let Some(group) = self.nested_group_choices.last_mut() {
            if group.in_and {
                group.in_and = false;
                println!(")");
            }
        }
    }
    fn enter_map(&mut self) {
        if let Some(group) = self.nested_group_choices.last_mut() {
            if !group.in_object {
                println!("z.object({{");
                group.in_object = true;
            }
        }
    }
    fn exit_map(&mut self) {
        if let Some(group) = self.nested_group_choices.last_mut() {
            if group.in_object {
                group.in_object = false;
                print!("}})");
            }
        }
        self.exit_and();
    }
    fn enter_record(&mut self) {
        if let Some(group) = self.nested_group_choices.last_mut() {
            if !group.in_record {
                println!("z.record(");
                group.in_record = true;
            }
        }
    }
    fn exit_record(&mut self) {
        if let Some(group) = self.nested_group_choices.last_mut() {
            if group.in_record {
                group.in_record = false;
                print!(")");
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
                self.exit_tuple();
                self.print_group_joiner();
                self.visit_array(&group)?;
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
        match calculate_array_entry_occurence(&entry.occur) {
            (lower, upper) if lower == upper => {
                self.print_group_joiner();
                self.enter_tuple();
                for index in 0..lower {
                    if index != 0 {
                        print!(",");
                    }
                    self.visit_type(&entry.entry_type)?
                }
            }
            (lower, upper) => {
                self.exit_tuple();
                self.print_group_joiner();
                if upper < MAX_ARRAYS {
                    print!("z.union([");
                    for bound in lower..upper + 1 {
                        if bound != 0 {
                            print!(",");
                        }
                        print!("z.tuple([");
                        for index in 0..bound {
                            if index != 0 {
                                print!(",");
                            }
                            self.visit_type(&entry.entry_type)?;
                        }
                        print!("])");
                    }
                    print!("])");
                } else {
                    print!("z.array(");
                    self.visit_type(&entry.entry_type)?;
                    print!(")");
                }
            }
        }
        Ok(())
    }
    fn visit_type_arrayname_entry(
        &mut self,
        entry: &'b cddl::ast::TypeGroupnameEntry<'a>,
    ) -> cddl::visitor::Result<Error> {
        match calculate_array_entry_occurence(&entry.occur) {
            (lower, upper) if lower == upper => {
                self.print_group_joiner();
                self.enter_tuple();
                for index in 0..lower {
                    if index != 0 {
                        print!(",");
                    }
                    if cfg!(feature = "vector_groups") {
                        unimplemented!();
                    } else {
                        self.visit_identifier_with_args(&entry.name, &entry.generic_args)?;
                    }
                }
            }
            (lower, upper) => {
                self.exit_tuple();
                self.print_group_joiner();
                if upper < MAX_ARRAYS {
                    print!("z.union([");
                    for bound in lower..upper + 1 {
                        if bound != 0 {
                            print!(",");
                        }
                        print!("z.tuple([");
                        for index in 0..bound {
                            if index != 0 {
                                print!(",");
                            }
                            if cfg!(feature = "vector_groups") {
                                unimplemented!();
                            } else {
                                self.visit_identifier_with_args(&entry.name, &entry.generic_args)?;
                            }
                        }
                        print!("])");
                    }
                    print!("])");
                } else {
                    #[cfg(feature = "vector_groups")]
                    {
                        unimplemented!();
                    }
                    #[cfg(not(feature = "vector_groups"))]
                    {
                        print!("z.array(");
                        self.visit_identifier_with_args(&entry.name, &entry.generic_args)?;
                        print!(")");
                    }
                }
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
        if matches!(
            self.nested_type1
                .last()
                .map(|context| context.value_mode)
                .unwrap_or(ValueMode::Generic),
            ValueMode::JavaScript
        ) {
            match ident.ident {
                "null" => {
                    print!("null");
                    return Ok(());
                }
                "true" => {
                    print!("true");
                    return Ok(());
                }
                "false" => {
                    print!("false");
                    return Ok(());
                }
                "undefined" => {
                    print!("undefined");
                    return Ok(());
                }
                _ => {}
            }
        }
        match ident.ident {
            "bool" => print!("z.boolean()"),
            "uint" => {
                print!("z.number().int().nonnegative()")
            }
            "nint" => {
                print!("z.number().int().negative()")
            }
            "int" => {
                print!("z.number().int()")
            }
            "float16" | "float32" | "float64" | "float16-32" | "float32-64" | "float"
            | "number" => {
                print!("z.number()")
            }
            "biguint" => {
                print!("z.bigint().nonnegative()")
            }
            "bignint" => {
                print!("z.bigint().negative()")
            }
            "bigint" => {
                print!("z.bigint()")
            }
            "bstr" | "bytes" => print!("z.string()"),
            "tstr" | "text" => print!("z.string()"),
            "any" => print!("z.any()"),
            "nil" | "null" => print!("z.null()"),
            "true" => print!("z.literal(true)"),
            "false" => print!("z.literal(false)"),
            "undefined" => print!("z.undefined()"),
            "uri" => print!("z.string().url()"),
            "regexp" => print!("z.string()"),
            ident => print!("{}Schema", to_namespaced(ident)),
        }
        Ok(())
    }
    fn visit_type_rule(&mut self, tr: &'b cddl::ast::TypeRule<'a>) -> cddl::visitor::Result<Error> {
        let (namespaces, type_name) = split_namespaced(&tr.name);
        for namespace in &namespaces {
            println!("export namespace {} {{", namespace);
        }
        print!("export const ");
        self.visit_identifier_with_params(
            &cddl::ast::Identifier {
                ident: &type_name,
                socket: None,
                span: Default::default(),
            },
            &tr.generic_params,
        )?;
        print!(" = z.lazy(() => ");
        self.visit_type(&tr.value)?;
        println!(");");
        for _ in &namespaces {
            println!("}}");
        }
        Ok(())
    }
    fn visit_type(&mut self, t: &'b cddl::ast::Type<'a>) -> cddl::visitor::Result<Error> {
        if self.visit_maybe_enum_type(&t) {
            return Ok(());
        }
        if t.type_choices.len() != 1 {
            print!("z.union([");
        }
        for i in 0..t.type_choices.len() {
            if i != 0 {
                print!(",");
            }
            self.visit_type1(&t.type_choices[i].type1)?;
        }
        if t.type_choices.len() != 1 {
            print!("])");
        }
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
        println!("export const ");
        self.visit_identifier_with_params(
            &cddl::ast::Identifier {
                ident: &type_name,
                socket: None,
                span: Default::default(),
            },
            &gr.generic_params,
        )?;
        print!(" = z.lazy(() => ");
        self.visit_group_choice(&choice)?;
        println!(");");

        if cfg!(feature = "vector_groups") {
            unimplemented!();
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
                    self.visit_type(&entry.entry_type)?;
                    print!(".partial()");
                } else {
                    self.visit_type(&entry.entry_type)?;
                }
                return Ok(());
            }
        };
        print!("  ");
        self.visit_memberkey(&mk)?;
        self.visit_type(&entry.entry_type)?;
        if is_group_entry_occurence_optional(&entry.occur)
            && !matches!(&mk, cddl::ast::MemberKey::Type1 { is_cut: false, .. })
        {
            print!(".optional()");
        }
        self.exit_record();
        Ok(())
    }
    fn visit_type_groupname_entry(
        &mut self,
        entry: &'b cddl::ast::TypeGroupnameEntry<'a>,
    ) -> cddl::visitor::Result<Error> {
        self.exit_map();
        self.print_group_joiner();
        if is_group_entry_occurence_optional(&entry.occur) {
            self.visit_identifier_with_args(&entry.name, &entry.generic_args)?;
            print!(".partial()");
        } else {
            self.visit_identifier_with_args(&entry.name, &entry.generic_args)?;
        }
        Ok(())
    }
    fn visit_group(&mut self, g: &'b cddl::ast::Group<'a>) -> cddl::visitor::Result<Error> {
        if g.group_choices.len() != 1 {
            print!("z.union([");
        }
        for i in 0..g.group_choices.len() {
            if i != 0 {
                print!(",");
            }
            self.visit_group_choice(&g.group_choices[i])?;
        }
        if g.group_choices.len() != 1 {
            print!("])");
        }
        Ok(())
    }
    fn visit_group_choice(
        &mut self,
        gc: &'b cddl::ast::GroupChoice<'a>,
    ) -> cddl::visitor::Result<Error> {
        self.nested_group_choices.push(GroupChoiceContext {
            in_object: false,
            is_first: true,
            in_record: false,
            in_and: false,
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
            cddl::ast::MemberKey::Type1 { t1, .. } => {
                self.exit_map();
                self.print_group_joiner();
                self.enter_record();
                self.visit_type1(t1)?;
                print!(",");
            }
            cddl::ast::MemberKey::Bareword { ident, .. } => {
                self.print_group_joiner();
                self.enter_map();
                print!("\"{}\":", &ident);
            }
            cddl::ast::MemberKey::Value { value, .. } => {
                self.print_group_joiner();
                self.enter_map();
                print!("[");
                self.visit_value(value)?;
                print!("]:");
            }
            cddl::ast::MemberKey::NonMemberKey { .. } => {
                unimplemented!()
            }
        }
        Ok(())
    }
    fn visit_type1(&mut self, t1: &'b cddl::ast::Type1<'a>) -> cddl::visitor::Result<Error> {
        self.nested_type1.push(Type1Context {
            value_mode: ValueMode::Generic,
        });
        if let Some(op) = &t1.operator {
            self.visit_type2(&t1.type2)?;

            self.nested_type1.last_mut().unwrap().value_mode = ValueMode::JavaScript;
            match op.operator {
                cddl::ast::RangeCtlOp::RangeOp { is_inclusive, .. } => {
                    if is_inclusive {
                        print!(".gte(");
                        self.visit_type2(&t1.type2)?;
                        print!(").lte(");
                        self.visit_type2(&op.type2)?;
                        print!(")");
                    } else {
                        print!(".gt(");
                        self.visit_type2(&t1.type2)?;
                        print!(").lt(");
                        self.visit_type2(&op.type2)?;
                        print!(")");
                    }
                }
                cddl::ast::RangeCtlOp::CtlOp { ctrl, .. } => match ctrl {
                    cddl::token::ControlOperator::DEFAULT => {
                        print!(".default(");
                        self.visit_type2(&op.type2)?;
                        print!(")");
                    }
                    cddl::token::ControlOperator::SIZE => {
                        print!(".length(");
                        self.visit_type2(&op.type2)?;
                        print!(")");
                    }
                    cddl::token::ControlOperator::PCRE | cddl::token::ControlOperator::REGEXP => {
                        print!(".regex(/");
                        self.visit_type2(&op.type2)?;
                        print!("/)");
                    }
                    cddl::token::ControlOperator::LT => {
                        print!(".lt(");
                        self.visit_type2(&op.type2)?;
                        print!(")");
                    }
                    cddl::token::ControlOperator::LE => {
                        print!(".lte(");
                        self.visit_type2(&op.type2)?;
                        print!(")");
                    }
                    cddl::token::ControlOperator::GT => {
                        print!(".gt(");
                        self.visit_type2(&op.type2)?;
                        print!(")");
                    }
                    cddl::token::ControlOperator::GE => {
                        print!(".gte(");
                        self.visit_type2(&op.type2)?;
                        print!(")");
                    }
                    cddl::token::ControlOperator::EQ | cddl::token::ControlOperator::NE => {
                        unimplemented!();
                    }
                    cddl::token::ControlOperator::WITHIN | cddl::token::ControlOperator::AND => {
                        print!(".and(");
                        self.visit_type2(&op.type2)?;
                        print!(")");
                    }
                    _ => unimplemented!(),
                },
            }
        } else {
            self.nested_type1.last_mut().unwrap().value_mode = ValueMode::Literal;
            self.visit_type2(&t1.type2)?;
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
            cddl::ast::Type2::Any { .. } => print!("z.unknown()"),
            // The default has the correct behavior for the rest of the cases.
            t2 => {
                cddl::visitor::walk_type2(self, t2)?;
            }
        }
        Ok(())
    }

    fn visit_value(&mut self, value: &cddl::token::Value<'a>) -> cddl::visitor::Result<Error> {
        match self.nested_type1.last().unwrap().value_mode {
            ValueMode::Literal => match value {
                cddl::token::Value::INT(value) => print!("z.literal({})", value),
                cddl::token::Value::UINT(value) => print!("z.literal({})", value),
                cddl::token::Value::FLOAT(value) => print!("z.literal({})", value),
                cddl::token::Value::TEXT(value) => print!("z.literal(\"{}\")", value),
                cddl::token::Value::BYTE(value) => print!("z.literal(\"{}\")", value),
            },
            ValueMode::Generic => match value {
                cddl::token::Value::INT(_) => print!("z.number().int()"),
                cddl::token::Value::UINT(_) => print!("z.number().int().nonnegative()"),
                cddl::token::Value::FLOAT(_) => print!("z.number()"),
                cddl::token::Value::TEXT(_) => print!("z.string()"),
                cddl::token::Value::BYTE(_) => print!("z.string()"),
            },
            ValueMode::JavaScript => match value {
                cddl::token::Value::INT(value) => print!("{}", value),
                cddl::token::Value::UINT(value) => print!("{}", value),
                cddl::token::Value::FLOAT(value) => print!("{}", value),
                cddl::token::Value::TEXT(value) => print!("\"{}\"", value),
                cddl::token::Value::BYTE(value) => print!("\"{}\"", value),
            },
        }
        Ok(())
    }
}
