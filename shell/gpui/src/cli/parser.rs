/// Must accept the same `--opt value`, `--opt=value`, alias, and positional shapes with identical error text as the SwiftUI shell's parser.
pub struct ArgParser<'a> {
    arguments: &'a [String],
    consumed: Vec<bool>,
}

impl<'a> ArgParser<'a> {
    pub fn new(arguments: &'a [String]) -> Self {
        Self {
            arguments,
            consumed: vec![false; arguments.len()],
        }
    }

    pub fn flag(&mut self, name: &str) -> bool {
        let Some(index) = self.first_unconsumed_index_of(name) else {
            return false;
        };
        self.consumed[index] = true;
        true
    }

    /// Returns `Ok(None)` when `name`/`alias` is absent; `Err` only when present but its value is missing or itself looks like another flag.
    pub fn option(&mut self, name: &str, alias: Option<&str>) -> Result<Option<String>, String> {
        for index in 0..self.arguments.len() {
            if self.consumed[index] {
                continue;
            }
            let arg = self.arguments[index].as_str();
            if arg == name || Some(arg) == alias {
                let value_index = index + 1;
                let missing = self
                    .arguments
                    .get(value_index)
                    .is_none_or(|value| value.starts_with('-'));
                if missing {
                    return Err(format!("missing value for {arg}"));
                }
                self.consumed[index] = true;
                self.consumed[value_index] = true;
                return Ok(Some(self.arguments[value_index].clone()));
            }
            if let Some(value) = arg
                .strip_prefix(name)
                .and_then(|rest| rest.strip_prefix('='))
            {
                self.consumed[index] = true;
                return Ok(Some(value.to_string()));
            }
        }
        Ok(None)
    }

    pub fn positional(&mut self) -> Option<String> {
        for index in 0..self.arguments.len() {
            if self.consumed[index] || self.arguments[index].starts_with('-') {
                continue;
            }
            self.consumed[index] = true;
            return Some(self.arguments[index].clone());
        }
        None
    }

    /// Every argument must be consumed by a `flag`/`option`/`positional` call; anything left over is a typo or stray value.
    pub fn finish(&self) -> Result<(), String> {
        for (index, consumed) in self.consumed.iter().enumerate() {
            if !consumed {
                return Err(format!("unexpected argument: {}", self.arguments[index]));
            }
        }
        Ok(())
    }

    fn first_unconsumed_index_of(&self, value: &str) -> Option<usize> {
        self.arguments
            .iter()
            .enumerate()
            .find(|(index, arg)| !self.consumed[*index] && arg.as_str() == value)
            .map(|(index, _)| index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn option_reads_space_and_equals_forms() {
        let a = args(&["--repo", "/tmp/x"]);
        let mut parser = ArgParser::new(&a);
        assert_eq!(
            parser.option("--repo", None),
            Ok(Some("/tmp/x".to_string()))
        );
        assert!(parser.finish().is_ok());

        let a = args(&["--repo=/tmp/y"]);
        let mut parser = ArgParser::new(&a);
        assert_eq!(
            parser.option("--repo", None),
            Ok(Some("/tmp/y".to_string()))
        );
    }

    #[test]
    fn option_matches_alias() {
        let a = args(&["-m", "hello"]);
        let mut parser = ArgParser::new(&a);
        assert_eq!(
            parser.option("--message", Some("-m")),
            Ok(Some("hello".to_string()))
        );
    }

    #[test]
    fn option_absent_returns_none_not_error() {
        let a = args(&["notes"]);
        let mut parser = ArgParser::new(&a);
        assert_eq!(parser.option("--repo", None), Ok(None));
    }

    #[test]
    fn option_missing_value_is_an_error() {
        let a = args(&["--line"]);
        let mut parser = ArgParser::new(&a);
        assert_eq!(
            parser.option("--line", None),
            Err("missing value for --line".to_string())
        );

        let a = args(&["--line", "--side"]);
        let mut parser = ArgParser::new(&a);
        assert_eq!(
            parser.option("--line", None),
            Err("missing value for --line".to_string())
        );
    }

    #[test]
    fn flag_is_independent_of_options() {
        let a = args(&["--include-resolved"]);
        let mut parser = ArgParser::new(&a);
        assert!(parser.flag("--include-resolved"));
        assert!(!parser.flag("--include-resolved"), "already consumed");
        assert!(parser.finish().is_ok());
    }

    #[test]
    fn positional_skips_flags_and_options() {
        let a = args(&["--repo", ".", "note-id-1"]);
        let mut parser = ArgParser::new(&a);
        assert_eq!(parser.option("--repo", None), Ok(Some(".".to_string())));
        assert_eq!(parser.positional(), Some("note-id-1".to_string()));
        assert_eq!(parser.positional(), None);
    }

    #[test]
    fn finish_reports_first_unexpected_argument() {
        let a = args(&["--repo", ".", "extra"]);
        let mut parser = ArgParser::new(&a);
        let _ = parser.option("--repo", None);
        assert_eq!(
            parser.finish(),
            Err("unexpected argument: extra".to_string())
        );
    }
}
