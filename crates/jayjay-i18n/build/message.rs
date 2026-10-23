use std::path::Path;

use fluent_syntax::ast::{self, Expression, InlineExpression, Pattern, PatternElement, VariantKey};

use crate::SOURCE_LOCALE;
use crate::identifier::Identifier;

const PLURAL_CATEGORIES: [&str; 5] = ["zero", "one", "two", "few", "many"];

pub(crate) struct Message {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) args: Vec<Arg>,
    references: Vec<String>,
}

#[derive(Clone)]
pub(crate) struct Arg {
    pub(crate) name: String,
    pub(crate) number: bool,
}

impl Message {
    pub(crate) fn read(path: &Path, prefix: &str, message: &ast::Message<&str>) -> Self {
        let id = message.id.name.to_owned();
        let name = id
            .strip_prefix(prefix)
            .and_then(|rest| rest.strip_prefix('-'))
            .unwrap_or_else(|| panic!("{}: `{id}` must start with `{prefix}-`", path.display()))
            .to_owned();
        Identifier::Message.check(path, &name);
        let pattern = message
            .value
            .as_ref()
            .unwrap_or_else(|| panic!("{}: `{id}` needs a value", path.display()));
        let mut message = Self {
            id,
            name,
            args: Vec::new(),
            references: Vec::new(),
        };
        message.collect_pattern(pattern);
        for arg in &message.args {
            Identifier::Variable.check(path, &arg.name);
        }
        message
    }

    pub(crate) fn resolve_references<'a>(messages: impl Iterator<Item = &'a mut Self>) {
        let mut messages: Vec<&mut Self> = messages.collect();
        loop {
            let known: Vec<(String, Vec<Arg>)> = messages
                .iter()
                .map(|m| (m.id.clone(), m.args.clone()))
                .collect();
            let mut changed = false;
            for message in &mut messages {
                for reference in message.references.clone() {
                    let (_, args) = known
                        .iter()
                        .find(|(id, _)| *id == reference)
                        .unwrap_or_else(|| {
                            panic!("`{}` references unknown `{reference}`", message.id)
                        });
                    for arg in args {
                        changed |= message.add_arg(&arg.name, arg.number);
                    }
                }
            }
            if !changed {
                break;
            }
        }
    }

    pub(crate) fn check_translation(&self, locale: &str, source: &[Self]) {
        let original = source
            .iter()
            .find(|m| m.id == self.id)
            .unwrap_or_else(|| panic!("{locale}: `{}` is not in {SOURCE_LOCALE}", self.id));
        for arg in &self.args {
            assert!(
                original.args.iter().any(|a| a.name == arg.name),
                "{locale}: `{}` uses `${}`, which {SOURCE_LOCALE} does not pass",
                self.id,
                arg.name
            );
        }
    }

    fn collect_pattern(&mut self, pattern: &Pattern<&str>) {
        for element in &pattern.elements {
            if let PatternElement::Placeable { expression } = element {
                self.collect_expression(expression);
            }
        }
    }

    /// Only plural or numeric variants make a selector numeric; `[file] / *[other]` stays text.
    fn collect_expression(&mut self, expression: &Expression<&str>) {
        match expression {
            Expression::Select { selector, variants } => {
                let plural = variants.iter().any(|variant| match &variant.key {
                    VariantKey::NumberLiteral { .. } => true,
                    VariantKey::Identifier { name } => PLURAL_CATEGORIES.contains(name),
                });
                self.collect_inline(selector, plural);
                for variant in variants {
                    self.collect_pattern(&variant.value);
                }
            }
            Expression::Inline(inline) => self.collect_inline(inline, false),
        }
    }

    fn collect_inline(&mut self, inline: &InlineExpression<&str>, number: bool) {
        match inline {
            InlineExpression::VariableReference { id } => {
                self.add_arg(id.name, number);
            }
            InlineExpression::FunctionReference { id, arguments } => {
                for argument in &arguments.positional {
                    self.collect_inline(argument, number || id.name == "NUMBER");
                }
            }
            InlineExpression::MessageReference { id, .. } => {
                self.references.push(id.name.to_owned());
            }
            InlineExpression::Placeable { expression } => self.collect_expression(expression),
            _ => {}
        }
    }

    fn add_arg(&mut self, name: &str, number: bool) -> bool {
        match self.args.iter_mut().find(|arg| arg.name == name) {
            Some(arg) if number && !arg.number => arg.number = true,
            Some(_) => return false,
            None => self.args.push(Arg {
                name: name.to_owned(),
                number,
            }),
        }
        true
    }
}
