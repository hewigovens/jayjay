use std::collections::HashMap;
use std::fmt::Display;
use std::process::Command;

use jj_lib::fileset::{self, FilesetDiagnostics, FilesetExpression, FilesetParseContext};
use jj_lib::fix::{LineRange, RegionsToFormat, compute_changed_ranges, compute_file_line_count};
use jj_lib::matchers::Matcher;
use jj_lib::settings::UserSettings;

use crate::repo::subprocess_command;
use crate::types::*;

pub(super) struct FixTool {
    pub(super) name: String,
    pub(super) matcher: Box<dyn Matcher>,
    command: FixCommand,
    pub(super) line_range_arg: Option<String>,
    run_tool_if_zero_line_ranges: bool,
}

impl FixTool {
    pub(super) fn command_for(
        &self,
        variables: &[(&str, &str)],
        base_content: Option<&[u8]>,
        content: &[u8],
    ) -> Option<Command> {
        let mut command = self.command.to_command(variables);
        if let Some(template) = &self.line_range_arg {
            let ranges = changed_lines(base_content, content);
            if ranges.is_empty() && !self.run_tool_if_zero_line_ranges {
                return None;
            }
            command.args(ranges.iter().map(|range| {
                template
                    .replace("$first", &range.first.to_string())
                    .replace("$last", &range.last.to_string())
            }));
        }
        Some(command)
    }
}

fn changed_lines(base_content: Option<&[u8]>, content: &[u8]) -> Vec<LineRange> {
    if content.is_empty() {
        return Vec::new();
    }
    match base_content {
        Some(base) => {
            let RegionsToFormat::LineRanges(ranges) = compute_changed_ranges(base, content);
            ranges
        }
        None => vec![LineRange::new(1, compute_file_line_count(content))],
    }
}

pub(super) fn parse_fix_tools(
    settings: &UserSettings,
    context: &FilesetParseContext,
) -> CoreResult<Vec<FixTool>> {
    let mut names: Vec<&str> = settings.table_keys("fix.tools").collect();
    names.sort_unstable();
    let mut tools = Vec::new();
    for name in names {
        let raw: RawFixTool = settings
            .get(["fix", "tools", name])
            .map_err(|error| tool_error(name, error))?;
        if raw.line_range_arg.is_none() && raw.run_tool_if_zero_line_ranges {
            return Err(tool_error(
                name,
                "run-tool-if-zero-line-ranges can only be set when line-range-arg is set",
            ));
        }
        if !raw.enabled {
            continue;
        }
        let patterns = raw
            .patterns
            .iter()
            .map(|pattern| fileset::parse(&mut FilesetDiagnostics::new(), pattern, context))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| tool_error(name, error))?;
        tools.push(FixTool {
            name: name.to_owned(),
            matcher: FilesetExpression::union_all(patterns).to_matcher(),
            command: raw.command,
            line_range_arg: raw.line_range_arg,
            run_tool_if_zero_line_ranges: raw.run_tool_if_zero_line_ranges,
        });
    }
    Ok(tools)
}

fn tool_error(name: &str, error: impl Display) -> CoreError {
    CoreError::internal(format!("fix.tools.{name}: {error}"))
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
struct RawFixTool {
    command: FixCommand,
    patterns: Vec<String>,
    #[serde(default = "enabled_by_default")]
    enabled: bool,
    #[serde(default)]
    line_range_arg: Option<String>,
    #[serde(default)]
    run_tool_if_zero_line_ranges: bool,
}

fn enabled_by_default() -> bool {
    true
}

struct FixCommand {
    program: String,
    args: Vec<String>,
    env: HashMap<String, String>,
}

impl FixCommand {
    fn to_command(&self, variables: &[(&str, &str)]) -> Command {
        let interpolate = |arg: &str| interpolate_variables(arg, variables);
        let mut command = subprocess_command(&interpolate(&self.program));
        command.args(self.args.iter().map(|arg| interpolate(arg)));
        command.envs(&self.env);
        command
    }
}

impl<'de> serde::Deserialize<'de> for FixCommand {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum Raw {
            String(String),
            Args(Vec<String>),
            Structured {
                #[serde(default)]
                env: HashMap<String, String>,
                command: Vec<String>,
            },
        }

        let (args, env) = match Raw::deserialize(deserializer)? {
            Raw::String(line) => (split_command_line(&line), HashMap::new()),
            Raw::Args(args) => (args, HashMap::new()),
            Raw::Structured { env, command } => (command, env),
        };
        let Some((program, args)) = args.split_first() else {
            return Err(serde::de::Error::custom("command must not be empty"));
        };
        Ok(Self {
            program: program.clone(),
            args: args.to_vec(),
            env,
        })
    }
}

/// jj splits a string command on shell quoting when it contains quotes, and on single spaces otherwise.
fn split_command_line(line: &str) -> Vec<String> {
    let quoted = line.contains('"') || line.contains('\'');
    if quoted && let Ok(parts) = shell_words::split(line) {
        return parts;
    }
    line.split(' ').map(str::to_owned).collect()
}

/// jj's `\$([a-z0-9_]+)\b` interpolation: a known name is replaced, anything else stays as written.
fn interpolate_variables(arg: &str, variables: &[(&str, &str)]) -> String {
    let is_name = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_';
    let mut pieces = arg.split('$');
    let mut result = pieces.next().unwrap_or_default().to_owned();
    for piece in pieces {
        let (name, tail) = piece.split_at(piece.find(|c| !is_name(c)).unwrap_or(piece.len()));
        match variables.iter().find(|(key, _)| *key == name) {
            Some((_, value)) if !tail.starts_with(char::is_alphanumeric) => {
                result.push_str(value);
                result.push_str(tail);
            }
            _ => {
                result.push('$');
                result.push_str(piece);
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use jj_lib::config::{ConfigLayer, ConfigSource, StackedConfig};
    use jj_lib::fileset::{FilesetAliasesMap, FilesetParseContext};
    use jj_lib::settings::UserSettings;
    use jj_lib::ui_path::RepoPathUiConverter;

    use super::*;

    fn parse(config_text: &str) -> CoreResult<Vec<FixTool>> {
        let mut config = StackedConfig::with_defaults();
        config.add_layer(
            ConfigLayer::parse(ConfigSource::User, config_text).expect("parse test config"),
        );
        let settings = UserSettings::from_config(config).expect("build user settings");
        let path_converter = RepoPathUiConverter::Fs {
            cwd: PathBuf::from("/repo"),
            base: PathBuf::from("/repo"),
        };
        let context = FilesetParseContext {
            aliases_map: &FilesetAliasesMap::new(),
            path_converter: &path_converter,
        };
        parse_fix_tools(&settings, &context)
    }

    fn tools(config_text: &str) -> Vec<FixTool> {
        parse(config_text).expect("parse fix tools")
    }

    #[test]
    fn enabled_tools_sort_by_name_and_accept_every_command_form() {
        let parsed = tools(
            r#"
            [fix.tools.sorter]
            command = "sort -u"
            patterns = ["glob:'**/*.txt'"]

            [fix.tools.off]
            command = ["cat"]
            patterns = ["glob:'**/*.md'"]
            enabled = false

            [fix.tools.formatter]
            command = { env = { LANG = "C" }, command = ["fmt", "--stdin"] }
            patterns = ["glob:'**/*.rs'", "glob:'**/*.toml'"]
            "#,
        );

        let names: Vec<&str> = parsed.iter().map(|tool| tool.name.as_str()).collect();
        assert_eq!(names, ["formatter", "sorter"]);
        assert_eq!(parsed[1].command.program, "sort");
        assert_eq!(parsed[1].command.args, ["-u"]);
        assert_eq!(
            parsed[0].command.env.get("LANG").map(String::as_str),
            Some("C")
        );
    }

    #[test]
    fn no_table_and_all_disabled_tools_parse_as_no_tools() {
        assert!(tools("").is_empty());
        assert!(
            tools(
                r#"
                [fix.tools.off]
                command = ["cat"]
                patterns = ["glob:'**/*.txt'"]
                enabled = false
                "#,
            )
            .is_empty()
        );
    }

    #[test]
    fn invalid_tool_config_names_the_tool() {
        for config_text in [
            "[fix.tools.broken]\ncommand = [\"cat\"]\npatterns = [\"glob:'**/*\"]",
            "[fix.tools.broken]\ncommand = [\"cat\"]\npatterns = []\nrun-tool-if-zero-line-ranges = true",
        ] {
            let error = parse(config_text)
                .err()
                .expect("an invalid tool must not parse");
            assert!(error.to_string().contains("fix.tools.broken"), "{error}");
        }
    }

    #[test]
    fn interpolation_replaces_known_variables_only() {
        let variables = [("path", "src/main.rs"), ("root", "/repo")];
        assert_eq!(
            interpolate_variables("--stdinpath=$path", &variables),
            "--stdinpath=src/main.rs"
        );
        assert_eq!(
            interpolate_variables("$root/$path", &variables),
            "/repo/src/main.rs"
        );
        for unchanged in ["$PATH", "$unknown", "$pathX", "literal"] {
            assert_eq!(interpolate_variables(unchanged, &variables), unchanged);
        }
    }

    #[cfg(unix)]
    #[test]
    fn tools_start_in_the_app_subprocess_environment() {
        let tool = tools("[fix.tools.shell]\ncommand = [\"sh\"]\npatterns = []").remove(0);
        let command = tool.command.to_command(&[]);
        let expected = subprocess_command("sh");
        assert_eq!(command.get_program(), expected.get_program());
        assert!(command.get_envs().eq(expected.get_envs()));
    }
}
