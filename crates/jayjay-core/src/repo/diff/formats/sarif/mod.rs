mod model;

use crate::types::*;

use self::model::{SarifMessage, SarifReport, SarifResult, SarifRun};
use super::{
    FormatInput, ProjectionPair,
    types::{DiffFormatPlugin, project_text_pair, projection_error},
};

pub(super) struct SarifPlugin;

impl DiffFormatPlugin for SarifPlugin {
    fn id(&self) -> &'static str {
        "sarif"
    }

    fn version(&self) -> u32 {
        1
    }

    fn label(&self) -> &'static str {
        "SARIF report"
    }

    fn render_kind(&self) -> DiffRenderKind {
        DiffRenderKind::Markdown
    }

    fn matches_path(&self, path: &str) -> bool {
        let path = path.to_ascii_lowercase();
        path.ends_with(".sarif") || path.ends_with(".sarif.json")
    }

    fn virtual_path(&self, path: &str) -> String {
        format!("{path}.md")
    }

    fn project(&self, input: FormatInput<'_>) -> CoreResult<ProjectionPair> {
        project_text_pair(
            input,
            self.projection(input.path, DiffProjectionMode::Processed),
            project_sarif,
        )
    }
}

fn project_sarif(bytes: &[u8]) -> CoreResult<String> {
    let report: SarifReport = serde_json::from_slice(bytes)
        .map_err(|err| projection_error(format!("parse SARIF JSON: {err}")))?;
    let runs = report
        .runs
        .as_deref()
        .ok_or_else(|| projection_error("SARIF report has no runs array"))?;
    let version = report.version.as_deref().unwrap_or("unknown");
    let mut out = format!(
        "# SARIF Report\n\n- Version: `{}`\n- Runs: {}\n",
        inline(version),
        runs.len()
    );

    for (run_index, run) in runs.iter().enumerate() {
        out.push('\n');
        write_run(&mut out, run_index, run);
    }

    Ok(out)
}

fn write_run(out: &mut String, run_index: usize, run: &SarifRun) {
    let tool_name = run
        .tool
        .as_ref()
        .and_then(|tool| tool.driver.as_ref())
        .and_then(|driver| driver.name.as_deref())
        .unwrap_or("Unknown tool");
    let results = run.results.as_slice();
    out.push_str("## Run ");
    out.push_str(&(run_index + 1).to_string());
    out.push_str(": ");
    out.push_str(&text(tool_name));
    out.push('\n');
    out.push('\n');
    out.push_str("- Tool: `");
    out.push_str(&inline(tool_name));
    out.push_str("`\n");
    out.push_str("- Results: ");
    out.push_str(&results.len().to_string());
    out.push('\n');

    if results.is_empty() {
        return;
    }

    let rules = rule_index(run);
    out.push('\n');
    for result in results {
        write_result(out, result, &rules);
    }
}

fn write_result(out: &mut String, result: &SarifResult, rules: &[(String, RuleSummary)]) {
    let rule_id = result.rule_id.as_deref().unwrap_or("unknown-rule");
    let level = result.level.as_deref().unwrap_or("warning");
    let rule = rules
        .iter()
        .find(|(id, _)| id == rule_id)
        .map(|(_, rule)| rule);
    let title = rule
        .and_then(|rule| rule.name.as_deref())
        .unwrap_or(rule_id);
    let location = primary_location(result);

    out.push_str("### ");
    out.push_str(&text(rule_id));
    if title != rule_id {
        out.push_str(": ");
        out.push_str(&text(title));
    }
    out.push('\n');
    out.push('\n');
    out.push_str("- Level: `");
    out.push_str(&inline(level));
    out.push_str("`\n");
    if let Some(location) = location {
        out.push_str("- Location: `");
        out.push_str(&inline(&location));
        out.push_str("`\n");
    }
    if let Some(description) = rule.and_then(|rule| rule.description.as_deref()) {
        out.push_str("- Rule: ");
        out.push_str(&text(description));
        out.push('\n');
    }
    if let Some(message) = message_text(result.message.as_ref()) {
        out.push_str("- Message: ");
        out.push_str(&text(message));
        out.push('\n');
    }
    out.push('\n');
}

#[derive(Clone)]
struct RuleSummary {
    name: Option<String>,
    description: Option<String>,
}

fn rule_index(run: &SarifRun) -> Vec<(String, RuleSummary)> {
    run.tool
        .as_ref()
        .and_then(|tool| tool.driver.as_ref())
        .map(|driver| driver.rules.as_slice())
        .into_iter()
        .flatten()
        .filter_map(|rule| {
            let id = rule.id.as_ref()?.to_owned();
            let name = rule
                .name
                .as_ref()
                .cloned()
                .or_else(|| message_text(rule.short_description.as_ref()).map(str::to_owned));
            Some((
                id,
                RuleSummary {
                    name,
                    description: message_text(rule.full_description.as_ref()).map(str::to_owned),
                },
            ))
        })
        .collect()
}

fn message_text(message: Option<&SarifMessage>) -> Option<&str> {
    message.and_then(|message| message.text.as_deref())
}

fn primary_location(result: &SarifResult) -> Option<String> {
    let location = result.locations.first()?.physical_location.as_ref()?;
    let artifact = location.artifact_location.as_ref()?.uri.as_deref()?;
    let line = location
        .region
        .as_ref()
        .and_then(|region| region.start_line);
    let column = location
        .region
        .as_ref()
        .and_then(|region| region.start_column);
    Some(match (line, column) {
        (Some(line), Some(column)) => format!("{artifact}:{line}:{column}"),
        (Some(line), None) => format!("{artifact}:{line}"),
        _ => artifact.to_owned(),
    })
}

fn text(value: &str) -> String {
    value.replace('\n', " ")
}

fn inline(value: &str) -> String {
    text(value).replace('`', "\\`")
}

#[cfg(test)]
mod tests {
    use super::project_sarif;

    #[test]
    fn projects_sarif_results_as_markdown() {
        let projected = project_sarif(
            br##"{
              "version": "2.1.0",
              "runs": [{
                "tool": {
                  "driver": {
                    "name": "Example Scanner",
                    "rules": [{
                      "id": "JJ001",
                      "name": "Unsafe call",
                      "fullDescription": {"text": "Avoid this call."}
                    }]
                  }
                },
                "results": [{
                  "ruleId": "JJ001",
                  "level": "error",
                  "message": {"text": "Potential issue."},
                  "locations": [{
                    "physicalLocation": {
                      "artifactLocation": {"uri": "Sources/App.swift"},
                      "region": {"startLine": 42, "startColumn": 7}
                    }
                  }]
                }]
              }]
            }"##,
        )
        .expect("project SARIF");

        assert!(projected.contains("# SARIF Report"));
        assert!(projected.contains("## Run 1: Example Scanner"));
        assert!(projected.contains("### JJ001: Unsafe call"));
        assert!(projected.contains("- Level: `error`"));
        assert!(projected.contains("- Location: `Sources/App.swift:42:7`"));
        assert!(projected.contains("- Message: Potential issue."));
    }
}
