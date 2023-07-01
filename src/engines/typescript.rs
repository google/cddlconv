use cddl::{ast::Occurrence, visitor::Visitor, Error};

use crate::util::{is_alpha, split_namespaced, to_namespaced, to_pascalcase};

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

impl<'a, 'b: 'a> Engine {
    pub fn new() -> Engine {
        Engine {
            in_comment: false,
            nested_group_choices: Vec::new(),
            nested_type1: Vec::new(),
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
    fn visit_memberkey_with_occurence(
        &mut self,
        mk: &'b cddl::ast::MemberKey<'a>,
        occurence: Option<&'a cddl::ast::Occurrence<'a>>,
    ) -> cddl::visitor::Result<Error> {
        self.visit_memberkey(&mk)?;
        if matches!(mk, cddl::ast::MemberKey::Type1 { is_cut: true, .. }) {
            match occurence {
                Some(cddl::ast::Occurrence {
                    occur: cddl::ast::Occur::Optional { .. },
                    ..
                })
                | Some(cddl::ast::Occurrence {
                    occur: cddl::ast::Occur::ZeroOrMore { .. },
                    ..
                }) => print!("?"),
                Some(cddl::ast::Occurrence {
                    occur:
                        cddl::ast::Occur::Exact {
                            lower: Some(lower), ..
                        },
                    ..
                }) if *lower == 0 => print!("?"),
                _ => {}
            }
        }
        print!(":");
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
    fn visit_type_for_comment(
        &mut self,
        t: &'b cddl::ast::Type<'a>,
    ) -> cddl::visitor::Result<Error> {
        self.visit_type_for_comment_inner(&t)?;
        if self.in_comment {
            self.in_comment = false;
            println!(" */");
        }
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
                    if !self.in_comment {
                        print!("/*");
                        self.in_comment = true;
                    } else {
                        print!(" ");
                    }
                    println!("*");
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
                        if !self.in_comment {
                            print!("/*");
                            self.in_comment = true;
                        } else {
                            print!(" ");
                        }
                        println!("*");
                        print!(" * @defaultValue `");
                        self.visit_type2(&op.type2)?;
                        println!("`");
                    }
                    cddl::token::ControlOperator::SIZE => {
                        if !self.in_comment {
                            print!("/*");
                            self.in_comment = true;
                        } else {
                            print!(" ");
                        }
                        println!("*");
                        print!(" * Must be `");
                        self.visit_type2(&op.type2)?;
                        println!("` units in length.");
                    }
                    cddl::token::ControlOperator::PCRE | cddl::token::ControlOperator::REGEXP => {
                        if !self.in_comment {
                            print!("/*");
                            self.in_comment = true;
                        } else {
                            print!(" ");
                        }
                        println!("*");
                        print!(" * Must match the pattern `");
                        self.visit_type2(&op.type2)?;
                        println!("`.");
                    }
                    cddl::token::ControlOperator::LT => {
                        if !self.in_comment {
                            print!("/*");
                            self.in_comment = true;
                        } else {
                            print!(" ");
                        }
                        println!("*");
                        print!(" * Must be less than `");
                        self.visit_type2(&op.type2)?;
                        println!("`.");
                    }
                    cddl::token::ControlOperator::LE => {
                        if !self.in_comment {
                            print!("/*");
                            self.in_comment = true;
                        } else {
                            print!(" ");
                        }
                        println!("*");
                        print!(" * Must be less than or equal to `");
                        self.visit_type2(&op.type2)?;
                        println!("`.");
                    }
                    cddl::token::ControlOperator::GT => {
                        if !self.in_comment {
                            print!("/*");
                            self.in_comment = true;
                        } else {
                            print!(" ");
                        }
                        println!("*");
                        print!(" * Must be greater than `");
                        self.visit_type2(&op.type2)?;
                        println!("`.");
                    }
                    cddl::token::ControlOperator::GE => {
                        if !self.in_comment {
                            print!("/*");
                            self.in_comment = true;
                        } else {
                            print!(" ");
                        }
                        println!("*");
                        print!(" * Must be greater than or equal to `");
                        self.visit_type2(&op.type2)?;
                        println!("`.");
                    }
                    cddl::token::ControlOperator::EQ => {
                        if !self.in_comment {
                            print!("/*");
                            self.in_comment = true;
                        } else {
                            print!(" ");
                        }
                        println!("*");
                        print!(" * Must be equal to `");
                        self.visit_type2(&op.type2)?;
                        println!("`.");
                    }
                    cddl::token::ControlOperator::NE => {
                        if !self.in_comment {
                            print!("/*");
                            self.in_comment = true;
                        } else {
                            print!(" ");
                        }
                        println!("*");
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
    fn visit_array_group(&mut self, g: &'b cddl::ast::Group<'a>) -> cddl::visitor::Result<Error> {
        for i in 0..g.group_choices.len() {
            if i != 0 {
                print!("|");
            }
            self.visit_array_group_choice(&g.group_choices[i])?;
        }
        Ok(())
    }
    fn visit_array_group_choice(
        &mut self,
        gc: &'b cddl::ast::GroupChoice<'a>,
    ) -> cddl::visitor::Result<Error> {
        self.nested_group_choices.push(GroupChoiceContext {
            in_object: false,
            is_first: false,
        });
        if gc.group_entries.is_empty() {
            let group = self.nested_group_choices.last_mut().unwrap();
            group.in_object = true;
            print!("[");
        }
        for i in 0..gc.group_entries.len() {
            self.nested_group_choices.last_mut().unwrap().is_first = i == 0;
            self.visit_array_group_entry(&gc.group_entries[i].0)?;
        }
        let group = self.nested_group_choices.last_mut().unwrap();
        if group.in_object {
            group.in_object = false;
            print!("] ");
        }
        self.nested_group_choices.pop();
        Ok(())
    }
    fn visit_array_group_entry(
        &mut self,
        entry: &'b cddl::ast::GroupEntry<'a>,
    ) -> cddl::visitor::Result<Error> {
        match entry {
            cddl::ast::GroupEntry::ValueMemberKey { ge, .. } => {
                self.visit_array_value_member_key_entry(ge)?;
            }
            cddl::ast::GroupEntry::TypeGroupname { ge, .. } => {
                self.visit_array_type_groupname_entry(ge)?;
            }
            cddl::ast::GroupEntry::InlineGroup { group, .. } => {
                if let Some(group) = self.nested_group_choices.last_mut() {
                    if group.in_object {
                        group.in_object = false;
                        print!("] ");
                    }
                    if !group.is_first {
                        print!("& ");
                    }
                }
                self.visit_array_group(&group)?;
            }
        }
        Ok(())
    }
    fn visit_array_value_member_key_entry(
        &mut self,
        entry: &'b cddl::ast::ValueMemberKeyEntry<'a>,
    ) -> cddl::visitor::Result<Error> {
        match &entry.occur {
            Some(_) => {
                self.visit_array_occurence_with(&entry.occur, Self::visit_type, &entry.entry_type)?;
            }
            None => {
                if let Some(group) = self.nested_group_choices.last_mut() {
                    if !group.in_object {
                        if !group.is_first {
                            print!("& ");
                        }
                        println!("[");
                        group.in_object = true;
                    }
                }
                if let Some(mk) = &entry.member_key {
                    print!("  ");
                    self.visit_memberkey_with_occurence(&mk, None)?;
                }
                self.visit_type(&entry.entry_type)?;
                println!(",");
            }
        }
        Ok(())
    }
    fn visit_array_type_groupname_entry(
        &mut self,
        entry: &'b cddl::ast::TypeGroupnameEntry<'a>,
    ) -> cddl::visitor::Result<Error> {
        self.visit_array_occurence_with(&entry.occur, Self::visit_identifier, &entry.name)
    }

    fn visit_array_occurence_with<T, F: Fn(&mut Self, &'b T) -> cddl::visitor::Result<Error>>(
        &mut self,
        occurence: &Option<Occurrence<'_>>,
        func: F,
        value: &'b T,
    ) -> cddl::visitor::Result<Error> {
        let bounds: (usize, Option<usize>);
        if let Some(Occurrence { occur, .. }) = occurence {
            match occur {
                cddl::ast::Occur::ZeroOrMore { .. } => {
                    bounds = (0, None);
                }
                cddl::ast::Occur::Exact { lower, upper, .. } => {
                    bounds = (lower.unwrap_or(0), *upper);
                }
                cddl::ast::Occur::OneOrMore { .. } => {
                    bounds = (1, None);
                }
                cddl::ast::Occur::Optional { .. } => {
                    print!("[");
                    func(self, value)?;
                    print!("?]");
                    return Ok(());
                }
            }
        } else {
            if let Some(group) = self.nested_group_choices.last_mut() {
                if !group.in_object {
                    if !group.is_first {
                        print!("& ");
                    }
                    println!("[");
                    group.in_object = true;
                }
            }
            func(self, value)?;
            print!(",");
            return Ok(());
        }
        if let Some(group) = self.nested_group_choices.last_mut() {
            if group.in_object {
                group.in_object = false;
                print!("] ");
            }
            if !group.is_first {
                print!("& ");
            }
        }
        let lower = bounds.0;
        match bounds.1 {
            Some(upper) if upper - lower < 10 => {
                for bound in lower..upper + 1 {
                    if bound != lower {
                        print!("|");
                    }
                    print!("[");
                    for _ in 0..bound {
                        func(self, value)?;
                        print!(",");
                    }
                    print!("]");
                }
            }
            _ => {
                if lower > 0 {
                    print!("[");
                    for _ in 0..lower {
                        func(self, value)?;
                        print!(",");
                    }
                    print!("] & ");
                }
                func(self, value)?;
                print!("[]");
            }
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
                is_alpha(value)
            } else {
                false
            }
        }) {
            print!("export const enum {} {{", type_name);
            self.visit_enum_type(&tr.value)?;
            println!("}}");
        } else {
            self.visit_type_for_comment(&tr.value)?;
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
            cddl::ast::GroupEntry::ValueMemberKey { ge, .. } => {
                self.visit_value_member_key_entry(ge)?;
            }
            cddl::ast::GroupEntry::TypeGroupname { ge, .. } => {
                if let Some(group) = self.nested_group_choices.last_mut() {
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
            cddl::ast::GroupEntry::InlineGroup { group, .. } => {
                if let Some(group) = self.nested_group_choices.last_mut() {
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
    fn visit_value_member_key_entry(
        &mut self,
        entry: &'b cddl::ast::ValueMemberKeyEntry<'a>,
    ) -> cddl::visitor::Result<Error> {
        if let Some(mk) = &entry.member_key {
            if let Some(group) = self.nested_group_choices.last_mut() {
                if !group.in_object {
                    if !group.is_first {
                        print!("& ");
                    }
                    println!("{{");
                    group.in_object = true;
                }
            }
            print!("  ");
            self.visit_type_for_comment(&entry.entry_type)?;
            self.visit_memberkey_with_occurence(&mk, entry.occur.as_ref())?;
            self.visit_type(&entry.entry_type)?;
            println!(",");
        } else {
            if let Some(group) = self.nested_group_choices.last_mut() {
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
            is_first: false,
        });
        if gc.group_entries.is_empty() {
            let group = self.nested_group_choices.last_mut().unwrap();
            group.in_object = true;
            print!("{{");
        }
        for i in 0..gc.group_entries.len() {
            self.nested_group_choices.last_mut().unwrap().is_first = i == 0;
            self.visit_group_entry(&gc.group_entries[i].0)?;
        }
        let group = self.nested_group_choices.last_mut().unwrap();
        if group.in_object {
            group.in_object = false;
            print!("}} ");
        }
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
            operator:
                cddl::ast::RangeCtlOp::CtlOp {
                    ctrl: cddl::token::ControlOperator::WITHIN | cddl::token::ControlOperator::AND,
                    ..
                },
            type2,
            ..
        }) = &t1.operator
        {
            println!(" & ");
            self.visit_type2(&type2)?;
        }
        self.nested_type1.pop();
        Ok(())
    }
    fn visit_type2(&mut self, t2: &'b cddl::ast::Type2<'a>) -> cddl::visitor::Result<Error> {
        match t2 {
            cddl::ast::Type2::Typename { ident, .. } => {
                self.visit_identifier(&ident)?;
            }
            cddl::ast::Type2::Array { group, .. } => {
                self.visit_array_group(&group)?;
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
