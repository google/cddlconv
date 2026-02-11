#![allow(unused_must_use)]

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

use std::io::Write;

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

pub struct Engine<Stdout, Stderr>
where
    Stdout: Write,
    Stderr: Write,
{
    nested_group_choices: Vec<GroupChoiceContext>,
    nested_type1: Vec<Type1Context>,
    #[allow(dead_code)]
    postamble_options: PostambleOptions,
    stdout: Stdout,
    #[allow(dead_code)]
    stderr: Stderr,
}

fn calculate_occurrence(occur: &Option<cddl::ast::Occurrence<'_>>) -> (usize, usize) {
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

impl<'a, 'b: 'a, 'c, Stdout: Write, Stderr: Write> Engine<Stdout, Stderr> {
    pub fn with_writers(stdout: Stdout, stderr: Stderr) -> Engine<Stdout, Stderr> {
        Engine {
            nested_group_choices: Vec::new(),
            nested_type1: Vec::new(),
            postamble_options: PostambleOptions {
                #[cfg(feature = "vector_groups")]
                print_flatten: false,
            },
            stdout,
            stderr,
        }
    }
    pub fn into_writers(self) -> (Stdout, Stderr) {
        (self.stdout, self.stderr)
    }
    pub fn print_preamble(&mut self) {
        writeln!(
            self.stdout,
            "// eslint-disable-next-line @typescript-eslint/ban-ts-comment"
        );
        writeln!(self.stdout, "// @ts-nocheck Some types may be circular.");
        writeln!(self.stdout);
        writeln!(self.stdout, "import * as z from 'zod';");
        writeln!(self.stdout);
    }
    pub fn print_postamble(&mut self) {
        #[cfg(feature = "vector_groups")]
        if self.postamble_options.print_flatten {
            unimplemented!();
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
            write!(self.stdout, "z.enum([");
            for type2 in t.type_choices.iter().map(|choice| &choice.type1.type2) {
                if let cddl::ast::Type2::TextValue { value, .. } = type2 {
                    write!(self.stdout, "\"{}\",", value);
                }
            }
            write!(self.stdout, "])");
            true
        } else {
            false
        }
    }
    fn visit_array(&mut self, g: &'b cddl::ast::Group<'a>) -> cddl::visitor::Result<Error> {
        if g.group_choices.len() != 1 {
            write!(self.stdout, "z.union([");
        }
        for (index, choice) in g.group_choices.iter().enumerate() {
            if index != 0 {
                write!(self.stdout, ",");
            }
            self.visit_array_choice(choice)?;
        }
        if g.group_choices.len() != 1 {
            write!(self.stdout, "])");
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
    fn in_tuple(&self) -> bool {
        if let Some(group) = self.nested_group_choices.last() {
            group.in_object
        } else {
            false
        }
    }
    fn enter_tuple(&mut self) {
        if let Some(group) = self.nested_group_choices.last_mut() {
            if !group.in_object {
                writeln!(self.stdout, "z.tuple([");
                group.in_object = true;
            }
        }
    }
    fn exit_tuple(&mut self) {
        if let Some(group) = self.nested_group_choices.last_mut() {
            if group.in_object {
                group.in_object = false;
                write!(self.stdout, "])");
            }
        }
    }
    fn print_group_joiner(&mut self) {
        if let Some(group) = self.nested_group_choices.last_mut() {
            if group.in_object {
                if !group.is_first {
                    write!(self.stdout, ",");
                }
            } else {
                if group.in_and {
                    group.in_and = false;
                    writeln!(self.stdout, ")");
                }
                if !group.is_first {
                    self.enter_and();
                }
            }
        }
    }
    fn enter_and(&mut self) {
        if let Some(group) = self.nested_group_choices.last_mut() {
            if !group.in_and {
                writeln!(self.stdout, ".and(");
                group.in_and = true;
            }
        }
    }
    fn exit_and(&mut self) {
        if let Some(group) = self.nested_group_choices.last_mut() {
            if group.in_and {
                group.in_and = false;
                writeln!(self.stdout, ")");
            }
        }
    }
    fn enter_map(&mut self) {
        if let Some(group) = self.nested_group_choices.last_mut() {
            if !group.in_object {
                writeln!(self.stdout, "z.object({{");
                group.in_object = true;
            }
        }
    }
    fn exit_map(&mut self) {
        if let Some(group) = self.nested_group_choices.last_mut() {
            if group.in_object {
                group.in_object = false;
                write!(self.stdout, "}})");
            }
        }
        self.exit_and();
    }
    fn enter_record(&mut self) {
        if let Some(group) = self.nested_group_choices.last_mut() {
            if !group.in_record {
                writeln!(self.stdout, "z.record(");
                group.in_record = true;
            }
        }
    }
    fn exit_record(&mut self) {
        if let Some(group) = self.nested_group_choices.last_mut() {
            if group.in_record {
                group.in_record = false;
                write!(self.stdout, ")");
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
                if self.in_tuple() {
                    return Err(Error::CDDL(
                        "Zod cannot mix array members and array groups. Use one or the other"
                            .to_string(),
                    ));
                }
                let (lower, upper) = calculate_occurrence(&ge.occur);
                if lower != upper || lower != 1 {
                    return Err(Error::CDDL(
                        "Multiplicity for array types is not supported.".to_string(),
                    ));
                }
                self.visit_type_arrayname_entry(ge)?;
            }
            cddl::ast::GroupEntry::InlineGroup { occur, group, .. } => {
                if self.in_tuple() {
                    return Err(Error::CDDL(
                        "Zod cannot mix array members and array groups. Use one or the other"
                            .to_string(),
                    ));
                }
                let (lower, upper) = calculate_occurrence(&occur);
                if lower != upper || lower != 1 {
                    return Err(Error::CDDL(
                        "Multiplicity for array types is not supported.".to_string(),
                    ));
                }
                self.visit_array(&group)?;
            }
        }
        Ok(())
    }

    fn visit_value_array_member_key_entry(
        &mut self,
        entry: &'b cddl::ast::ValueMemberKeyEntry<'a>,
    ) -> cddl::visitor::Result<Error> {
        let (lower, upper) = calculate_occurrence(&entry.occur);
        if lower == upper {
            self.print_group_joiner();
            self.enter_tuple();
            for index in 0..lower {
                if index != 0 {
                    write!(self.stdout, ",");
                }
                self.visit_type(&entry.entry_type)?
            }
        } else {
            if self.in_tuple() {
                return Err(Error::CDDL(
                    "Zod cannot mix array members (e.g. `int`) with varying occurrence array members (e.g. `* text`). Use one or the other."
                        .to_string(),
                ));
            }
            if upper < MAX_ARRAYS {
                write!(self.stdout, "z.union([");
                for bound in lower..upper + 1 {
                    if bound != 0 {
                        write!(self.stdout, ",");
                    }
                    write!(self.stdout, "z.tuple([");
                    for index in 0..bound {
                        if index != 0 {
                            write!(self.stdout, ",");
                        }
                        self.visit_type(&entry.entry_type)?;
                    }
                    write!(self.stdout, "])");
                }
                write!(self.stdout, "])");
            } else {
                write!(self.stdout, "z.array(");
                self.visit_type(&entry.entry_type)?;
                write!(self.stdout, ")");
                if lower > 0 {
                    write!(self.stdout, ".min({})", lower);
                }
                if upper < usize::MAX {
                    write!(self.stdout, ".max({})", upper);
                }
            }
        }
        Ok(())
    }

    fn visit_type_arrayname_entry(
        &mut self,
        entry: &'b cddl::ast::TypeGroupnameEntry<'a>,
    ) -> cddl::visitor::Result<Error> {
        self.visit_identifier_with_args(&entry.name, &entry.generic_args)
    }

    fn visit_identifier_with_params(
        &mut self,
        ident: &cddl::ast::Identifier<'a>,
        _params: &Option<cddl::ast::GenericParams<'a>>,
    ) -> cddl::visitor::Result<Error> {
        self.visit_identifier(ident)
    }

    fn visit_identifier_with_args(
        &mut self,
        ident: &cddl::ast::Identifier<'a>,
        _params: &Option<cddl::ast::GenericArgs<'a>>,
    ) -> cddl::visitor::Result<Error> {
        self.visit_identifier(ident)
    }
}

impl<'a, 'b: 'a, Stdout: Write, Stderr: Write> Visitor<'a, 'b, Error> for Engine<Stdout, Stderr> {
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
                    write!(self.stdout, "null");
                    return Ok(());
                }
                "true" => {
                    write!(self.stdout, "true");
                    return Ok(());
                }
                "false" => {
                    write!(self.stdout, "false");
                    return Ok(());
                }
                "undefined" => {
                    write!(self.stdout, "undefined");
                    return Ok(());
                }
                _ => {}
            }
        }
        match ident.ident {
            "bool" => write!(self.stdout, "z.boolean()"),
            "uint" => {
                write!(self.stdout, "z.number().int().nonnegative()")
            }
            "nint" => {
                write!(self.stdout, "z.number().int().negative()")
            }
            "int" => {
                write!(self.stdout, "z.number().int()")
            }
            "float16" | "float32" | "float64" | "float16-32" | "float32-64" | "float"
            | "number" => {
                write!(self.stdout, "z.number()")
            }
            "biguint" => {
                write!(self.stdout, "z.bigint().nonnegative()")
            }
            "bignint" => {
                write!(self.stdout, "z.bigint().negative()")
            }
            "bigint" => {
                write!(self.stdout, "z.bigint()")
            }
            "bstr" | "bytes" => write!(self.stdout, "z.string()"),
            "tstr" | "text" => write!(self.stdout, "z.string()"),
            "any" => write!(self.stdout, "z.any()"),
            "nil" | "null" => write!(self.stdout, "z.null()"),
            "true" => write!(self.stdout, "z.literal(true)"),
            "false" => write!(self.stdout, "z.literal(false)"),
            "undefined" => write!(self.stdout, "z.undefined()"),
            "uri" => write!(self.stdout, "z.url()"),
            "regexp" => write!(self.stdout, "z.string()"),
            ident => write!(self.stdout, "{}Schema", to_namespaced(ident)),
        };
        Ok(())
    }
    fn visit_type_rule(&mut self, tr: &'b cddl::ast::TypeRule<'a>) -> cddl::visitor::Result<Error> {
        let (namespaces, type_name) = split_namespaced(&tr.name);
        for namespace in &namespaces {
            writeln!(self.stdout, "export namespace {} {{", namespace);
        }
        write!(self.stdout, "export const ");
        self.visit_identifier_with_params(
            &cddl::ast::Identifier {
                ident: &type_name,
                socket: None,
                span: Default::default(),
            },
            &tr.generic_params,
        )?;
        write!(self.stdout, " = ");
        if tr.value.type_choices.len() == 1
            && is_primitive_type(&tr.value.type_choices.first().unwrap().type1.type2)
        {
            self.visit_type(&tr.value)?;
        } else {
            write!(self.stdout, "z.lazy(() => ");
            self.visit_type(&tr.value)?;
            write!(self.stdout, ")");
        }
        writeln!(self.stdout, ";");
        for _ in &namespaces {
            writeln!(self.stdout, "}}");
        }
        Ok(())
    }
    fn visit_type(&mut self, t: &'b cddl::ast::Type<'a>) -> cddl::visitor::Result<Error> {
        if self.visit_maybe_enum_type(&t) {
            return Ok(());
        }
        if t.type_choices.len() != 1 {
            write!(self.stdout, "z.union([");
        }
        for i in 0..t.type_choices.len() {
            if i != 0 {
                write!(self.stdout, ",");
            }
            self.visit_type1(&t.type_choices[i].type1)?;
        }
        if t.type_choices.len() != 1 {
            write!(self.stdout, "])");
        }
        Ok(())
    }
    fn visit_group_rule(
        &mut self,
        gr: &'b cddl::ast::GroupRule<'a>,
    ) -> cddl::visitor::Result<Error> {
        let (namespaces, type_name) = split_namespaced(&gr.name);
        for namespace in &namespaces {
            writeln!(self.stdout, "export namespace {} {{", namespace);
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
        writeln!(self.stdout, "export const ");
        self.visit_identifier_with_params(
            &cddl::ast::Identifier {
                ident: &type_name,
                socket: None,
                span: Default::default(),
            },
            &gr.generic_params,
        )?;
        write!(self.stdout, " = z.lazy(() => ");
        self.visit_group_choice(&choice)?;
        writeln!(self.stdout, ");");

        if cfg!(feature = "vector_groups") {
            unimplemented!();
        }

        for _ in &namespaces {
            writeln!(self.stdout, "}}");
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
                self.exit_map();
                self.print_group_joiner();
                if matches!(calculate_occurrence(&ge.occur), (0, max) if max > 0) {
                    self.visit_type_groupname_entry(ge)?;
                    write!(self.stdout, ".or(z.object({{}}))");
                } else {
                    self.visit_type_groupname_entry(ge)?;
                }
            }
            cddl::ast::GroupEntry::InlineGroup { occur, group, .. } => {
                self.exit_map();
                self.print_group_joiner();
                if matches!(calculate_occurrence(&occur), (0, max) if max > 0) {
                    self.visit_group(group)?;
                    write!(self.stdout, ".or(z.object({{}}))");
                } else {
                    self.visit_group(group)?;
                }
            }
        }
        Ok(())
    }
    fn visit_value_member_key_entry(
        &mut self,
        entry: &'b cddl::ast::ValueMemberKeyEntry<'a>,
    ) -> cddl::visitor::Result<Error> {
        let mk = entry.member_key.as_ref().expect(&format!(
            "Expected member key for type {} since the current ambient rule is a map. \
            Did you mean to declare {} with parenthesis (`( .. )`) \
            instead of brackets (`{{ .. }}`)?",
            entry.entry_type, entry.entry_type
        ));
        self.visit_memberkey(&mk)?;
        self.visit_type(&entry.entry_type)?;
        if matches!(calculate_occurrence(&entry.occur), (0, max) if max > 0)
            && !matches!(&mk, cddl::ast::MemberKey::Type1 { is_cut: false, .. })
        {
            write!(self.stdout, ".optional()");
        }
        self.exit_record();
        Ok(())
    }
    fn visit_type_groupname_entry(
        &mut self,
        entry: &'b cddl::ast::TypeGroupnameEntry<'a>,
    ) -> cddl::visitor::Result<Error> {
        self.visit_identifier_with_args(&entry.name, &entry.generic_args)?;
        Ok(())
    }
    fn visit_group(&mut self, g: &'b cddl::ast::Group<'a>) -> cddl::visitor::Result<Error> {
        if g.group_choices.len() != 1 {
            write!(self.stdout, "z.union([");
        }
        for i in 0..g.group_choices.len() {
            if i != 0 {
                write!(self.stdout, ",");
            }
            self.visit_group_choice(&g.group_choices[i])?;
        }
        if g.group_choices.len() != 1 {
            write!(self.stdout, "])");
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
                write!(self.stdout, ",");
            }
            cddl::ast::MemberKey::Bareword { ident, .. } => {
                self.print_group_joiner();
                self.enter_map();
                write!(self.stdout, "\"{}\":", &ident);
            }
            cddl::ast::MemberKey::Value { value, .. } => {
                self.print_group_joiner();
                self.enter_map();
                write!(self.stdout, "[");
                self.visit_value(value)?;
                write!(self.stdout, "]:");
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
                        write!(self.stdout, ".gte(");
                        self.visit_type2(&t1.type2)?;
                        write!(self.stdout, ").lte(");
                        self.visit_type2(&op.type2)?;
                        write!(self.stdout, ")");
                    } else {
                        write!(self.stdout, ".gt(");
                        self.visit_type2(&t1.type2)?;
                        write!(self.stdout, ").lt(");
                        self.visit_type2(&op.type2)?;
                        write!(self.stdout, ")");
                    }
                }
                cddl::ast::RangeCtlOp::CtlOp { ctrl, .. } => match ctrl {
                    cddl::token::ControlOperator::DEFAULT => {
                        write!(self.stdout, ".default(");
                        self.visit_type2(&op.type2)?;
                        write!(self.stdout, ")");
                    }
                    cddl::token::ControlOperator::SIZE => {
                        write!(self.stdout, ".length(");
                        self.visit_type2(&op.type2)?;
                        write!(self.stdout, ")");
                    }
                    cddl::token::ControlOperator::PCRE | cddl::token::ControlOperator::REGEXP => {
                        write!(self.stdout, ".regex(new RegExp(");
                        self.visit_type2(&op.type2)?;
                        write!(self.stdout, "))");
                    }
                    cddl::token::ControlOperator::LT => {
                        write!(self.stdout, ".lt(");
                        self.visit_type2(&op.type2)?;
                        write!(self.stdout, ")");
                    }
                    cddl::token::ControlOperator::LE => {
                        write!(self.stdout, ".lte(");
                        self.visit_type2(&op.type2)?;
                        write!(self.stdout, ")");
                    }
                    cddl::token::ControlOperator::GT => {
                        write!(self.stdout, ".gt(");
                        self.visit_type2(&op.type2)?;
                        write!(self.stdout, ")");
                    }
                    cddl::token::ControlOperator::GE => {
                        write!(self.stdout, ".gte(");
                        self.visit_type2(&op.type2)?;
                        write!(self.stdout, ")");
                    }
                    cddl::token::ControlOperator::EQ | cddl::token::ControlOperator::NE => {
                        unimplemented!();
                    }
                    cddl::token::ControlOperator::WITHIN | cddl::token::ControlOperator::AND => {
                        write!(self.stdout, ".and(");
                        self.visit_type2(&op.type2)?;
                        write!(self.stdout, ")");
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
            cddl::ast::Type2::Any { .. } => {
                write!(self.stdout, "z.unknown()");
            }
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
                cddl::token::Value::INT(value) => write!(self.stdout, "z.literal({})", value),
                cddl::token::Value::UINT(value) => write!(self.stdout, "z.literal({})", value),
                cddl::token::Value::FLOAT(value) => write!(self.stdout, "z.literal({})", value),
                cddl::token::Value::TEXT(value) => write!(self.stdout, "z.literal(\"{}\")", value),
                cddl::token::Value::BYTE(value) => write!(self.stdout, "z.literal(\"{}\")", value),
            },
            ValueMode::Generic => match value {
                cddl::token::Value::INT(_) => write!(self.stdout, "z.number().int()"),
                cddl::token::Value::UINT(_) => {
                    write!(self.stdout, "z.number().int().nonnegative()")
                }
                cddl::token::Value::FLOAT(_) => write!(self.stdout, "z.number()"),
                cddl::token::Value::TEXT(_) => write!(self.stdout, "z.string()"),
                cddl::token::Value::BYTE(_) => write!(self.stdout, "z.string()"),
            },
            ValueMode::JavaScript => match value {
                cddl::token::Value::INT(value) => write!(self.stdout, "{}", value),
                cddl::token::Value::UINT(value) => write!(self.stdout, "{}", value),
                cddl::token::Value::FLOAT(value) => write!(self.stdout, "{}", value),
                cddl::token::Value::TEXT(value) => write!(self.stdout, "\"{}\"", value),
                cddl::token::Value::BYTE(value) => write!(self.stdout, "\"{}\"", value),
            },
        };
        Ok(())
    }
}

fn is_primitive_type(type2: &cddl::ast::Type2) -> bool {
    !matches!(
        type2,
        cddl::ast::Type2::Typename { .. }
            | cddl::ast::Type2::ParenthesizedType { .. }
            | cddl::ast::Type2::Array { .. }
            | cddl::ast::Type2::Map { .. }
            | cddl::ast::Type2::Unwrap { .. }
            | cddl::ast::Type2::ChoiceFromInlineGroup { .. }
            | cddl::ast::Type2::ChoiceFromGroup { .. }
            | cddl::ast::Type2::TaggedData { .. }
            | cddl::ast::Type2::DataMajorType { .. }
            | cddl::ast::Type2::Any { .. }
    )
}
