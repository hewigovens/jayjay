pub(super) struct ArgParser<'a> {
    arguments: &'a [String],
    consumed: Vec<bool>,
    // Index after the first bare `--`; tokens from there never match flag or option names.
    literal_from: usize,
}

impl<'a> ArgParser<'a> {
    pub(super) fn new(arguments: &'a [String]) -> Self {
        let mut consumed = vec![false; arguments.len()];
        let literal_from = match arguments.iter().position(|argument| argument == "--") {
            Some(index) => {
                consumed[index] = true;
                index + 1
            }
            None => arguments.len(),
        };
        Self {
            arguments,
            consumed,
            literal_from,
        }
    }

    pub(super) fn flag(&mut self, name: &str) -> bool {
        let Some(index) = self.first_unconsumed_index_of(name) else {
            return false;
        };
        self.consumed[index] = true;
        true
    }

    pub(super) fn option(
        &mut self,
        name: &str,
        alias: Option<&str>,
    ) -> Result<Option<String>, String> {
        for index in 0..self.literal_from {
            if self.consumed[index] {
                continue;
            }
            let argument = self.arguments[index].as_str();
            if argument == name || Some(argument) == alias {
                let value_index = index + 1;
                if value_index >= self.arguments.len() || self.consumed[value_index] {
                    return Err(format!("missing value for {argument}"));
                }
                self.consumed[index] = true;
                self.consumed[value_index] = true;
                return Ok(Some(self.arguments[value_index].clone()));
            }
            let equals_value = [Some(name), alias].into_iter().flatten().find_map(|form| {
                argument
                    .strip_prefix(form)
                    .and_then(|rest| rest.strip_prefix('='))
            });
            if let Some(value) = equals_value {
                self.consumed[index] = true;
                return Ok(Some(value.to_owned()));
            }
        }
        Ok(None)
    }

    pub(super) fn positional(&mut self) -> Option<String> {
        for index in 0..self.arguments.len() {
            if self.consumed[index]
                || (index < self.literal_from && self.arguments[index].starts_with('-'))
            {
                continue;
            }
            self.consumed[index] = true;
            return Some(self.arguments[index].clone());
        }
        None
    }

    pub(super) fn finish(&self) -> Result<(), String> {
        for (index, consumed) in self.consumed.iter().enumerate() {
            if !consumed {
                return Err(format!("unexpected argument: {}", self.arguments[index]));
            }
        }
        Ok(())
    }

    fn first_unconsumed_index_of(&self, value: &str) -> Option<usize> {
        (0..self.literal_from)
            .find(|&index| !self.consumed[index] && self.arguments[index] == value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::args;

    #[test]
    fn options_support_space_equals_and_alias_forms() {
        let arguments = args(&["--repo", "/tmp/x"]);
        let mut parser = ArgParser::new(&arguments);
        assert_eq!(
            parser.option("--repo", None),
            Ok(Some("/tmp/x".to_string()))
        );
        assert_eq!(parser.option("--format", None), Ok(None));
        assert!(parser.finish().is_ok());

        let arguments = args(&["--repo=/tmp/y"]);
        let mut parser = ArgParser::new(&arguments);
        assert_eq!(
            parser.option("--repo", None),
            Ok(Some("/tmp/y".to_string()))
        );

        for message in [&["-m", "hello"][..], &["-m=hello"][..]] {
            let arguments = args(message);
            let mut parser = ArgParser::new(&arguments);
            assert_eq!(
                parser.option("--message", Some("-m")),
                Ok(Some("hello".to_string()))
            );
        }
    }

    #[test]
    fn option_values_may_begin_with_a_dash() {
        let arguments = args(&["-m", "- rename this", "--repo=-x"]);
        let mut parser = ArgParser::new(&arguments);
        assert_eq!(
            parser.option("--message", Some("-m")),
            Ok(Some("- rename this".to_string()))
        );
        assert_eq!(parser.option("--repo", None), Ok(Some("-x".to_string())));
        assert!(parser.finish().is_ok());
    }

    #[test]
    fn missing_option_value_is_an_error() {
        for values in [&["--line"][..], &["--line", "--"][..]] {
            let arguments = args(values);
            let mut parser = ArgParser::new(&arguments);
            assert_eq!(
                parser.option("--line", None),
                Err("missing value for --line".to_string())
            );
        }
    }

    #[test]
    fn double_dash_makes_the_rest_positional() {
        let arguments = args(&["--", "--repo", "-note-1"]);
        let mut parser = ArgParser::new(&arguments);
        assert_eq!(parser.option("--repo", None), Ok(None));
        assert_eq!(parser.positional(), Some("--repo".to_string()));
        assert_eq!(parser.positional(), Some("-note-1".to_string()));
        assert_eq!(parser.positional(), None);
        assert!(parser.finish().is_ok());
    }

    #[test]
    fn flags_positionals_and_extras_are_tracked() {
        let arguments = args(&["--include-resolved"]);
        let mut parser = ArgParser::new(&arguments);
        assert!(parser.flag("--include-resolved"));
        assert!(!parser.flag("--include-resolved"));
        assert!(parser.finish().is_ok());

        let arguments = args(&["--repo", ".", "note-id-1", "extra"]);
        let mut parser = ArgParser::new(&arguments);
        assert_eq!(parser.option("--repo", None), Ok(Some(".".to_string())));
        assert_eq!(parser.positional(), Some("note-id-1".to_string()));
        assert_eq!(
            parser.finish(),
            Err("unexpected argument: extra".to_string())
        );
    }
}
