use serde::Deserialize;
use serde_json::Value;

use super::super::super::Repo;
use super::super::super::environment::origin_binary;
use super::super::PrLookup;
use super::super::checks::{self, CheckState};
use crate::types::{ChecksStatus, PrInfo, PrState};

const JSON_FIELDS: &str = "number,title,url,status,ciState";

/// `origin pr list` argv with `--head` so an option-shaped bookmark is never a flag.
fn pr_list_args(bookmark: &str) -> [&str; 10] {
    [
        "pr",
        "list",
        "--state",
        "all",
        "--limit",
        "30",
        "--json",
        JSON_FIELDS,
        "--head",
        bookmark,
    ]
}

/// Confirmed list (possibly empty). `Err` means the lookup failed, so stacked submit must not create.
fn listed_prs(success: bool, stdout: &str, stderr: &str) -> Result<Vec<OriginPrResponse>, String> {
    if !success {
        return if is_no_pr_error(stderr) {
            Ok(Vec::new())
        } else {
            Err(stderr.trim().to_owned())
        };
    }
    parse_prs(stdout).ok_or_else(|| "origin pr list returned invalid JSON".to_owned())
}

fn list_prs(repo: &Repo, bookmark: &str) -> Result<Vec<OriginPrResponse>, String> {
    match repo.command_output(&origin_binary(), &pr_list_args(bookmark), "origin pr list") {
        Ok(output) => listed_prs(
            output.status.success(),
            &Repo::stdout_text(&output),
            &Repo::stderr_text(&output),
        ),
        Err(error) => Err(error.to_string()),
    }
}

pub(crate) fn pr_info(repo: &Repo, bookmark: &str) -> PrLookup {
    match list_prs(repo, bookmark).and_then(pick_pr) {
        Ok(Some(pr)) => PrLookup::Found(pr),
        Ok(None) => PrLookup::NotFound,
        Err(_) => PrLookup::Unknown,
    }
}

pub(crate) fn open_pr(repo: &Repo, bookmark: &str) -> Result<Option<(u32, String)>, String> {
    Ok(pick_pr(list_prs(repo, bookmark)?)?
        .filter(|pr| pr.state == PrState::Open)
        .map(|pr| (pr.number, pr.url)))
}

/// Origin reports a confirmed absence as an empty list or "no open or draft change".
fn is_no_pr_error(stderr: &str) -> bool {
    stderr.contains("No open or draft change found for branch")
        || stderr.contains("No matching changes")
}

fn parse_prs(json: &str) -> Option<Vec<OriginPrResponse>> {
    serde_json::from_str(json).ok()
}

fn pick_pr(prs: Vec<OriginPrResponse>) -> Result<Option<PrInfo>, String> {
    let mut fallback = None;
    for pr in prs {
        let state = pr_state(&pr.status)?;
        let info = pr.into_pr_info(state);
        if state == PrState::Open {
            return Ok(Some(info));
        }
        fallback.get_or_insert(info);
    }
    Ok(fallback)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OriginPrResponse {
    #[serde(deserialize_with = "deserialize_pr_number")]
    number: u32,
    status: String,
    title: String,
    url: String,
    #[serde(default)]
    ci_state: Option<OriginCiState>,
}

/// Origin CLI `--json` emits lowercase status strings; unknown values make the lookup uncertain so they can never trigger PR creation.
fn pr_state(status: &str) -> Result<PrState, String> {
    match status {
        "draft" | "open" => Ok(PrState::Open),
        "closed" => Ok(PrState::Closed),
        "merged" => Ok(PrState::Merged),
        _ => Err(format!(
            "origin pr list returned unsupported status {status:?}"
        )),
    }
}

impl OriginPrResponse {
    fn into_pr_info(self, state: PrState) -> PrInfo {
        let checks = self.ci_state.as_ref().map_or(ChecksStatus::None, |ci| {
            checks::rollup(
                ci.check_run_groups
                    .iter()
                    .flat_map(|group| group.check_runs.iter())
                    .map(OriginCheckRun::state),
            )
        });
        PrInfo {
            number: self.number,
            state,
            title: self.title,
            url: self.url,
            checks,
        }
    }
}

fn deserialize_pr_number<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<u32, D::Error> {
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Number(n) => n
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or_else(|| serde::de::Error::custom("pull request number out of range")),
        Value::String(s) => s
            .parse()
            .map_err(|_| serde::de::Error::custom("invalid pull request number")),
        _ => Err(serde::de::Error::custom("expected pull request number")),
    }
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct OriginCiState {
    #[serde(default)]
    check_run_groups: Vec<OriginCheckRunGroup>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct OriginCheckRunGroup {
    #[serde(default)]
    check_runs: Vec<OriginCheckRun>,
}

#[derive(Deserialize)]
struct OriginCheckRun {
    #[serde(default)]
    status: Value,
    conclusion: Option<Value>,
}

impl OriginCheckRun {
    fn state(&self) -> CheckState {
        if !is_completed(&self.status) {
            CheckState::Pending
        } else if is_success(self.conclusion.as_ref()) {
            CheckState::Success
        } else {
            CheckState::Failure
        }
    }
}

fn is_completed(status: &Value) -> bool {
    match status {
        Value::String(s) => s.eq_ignore_ascii_case("completed"),
        Value::Number(n) => n.as_i64() == Some(3),
        _ => false,
    }
}

fn is_success(conclusion: Option<&Value>) -> bool {
    match conclusion {
        Some(Value::String(s)) => s.eq_ignore_ascii_case("success"),
        Some(Value::Number(n)) => n.as_i64() == Some(1),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PrState;

    fn parse_pick(json: &str) -> Result<Option<PrInfo>, String> {
        pick_pr(parse_prs(json).unwrap())
    }

    #[test]
    fn pick_prefers_open_or_draft_then_falls_back() {
        let mixed = parse_pick(
            r#"[{"number":"1","status":"merged","title":"old","url":"https://cursor.com/codebase/o/r/pull/1"},{"number":2,"status":"draft","title":"wip","url":"https://cursor.com/codebase/o/r/pull/2"}]"#,
        )
        .unwrap()
        .unwrap();
        assert_eq!(mixed.number, 2);
        assert_eq!(mixed.state, PrState::Open);
        assert_eq!(mixed.checks, ChecksStatus::None);

        let merged = parse_pick(
            r#"[{"number":9,"status":"merged","title":"done","url":"https://cursor.com/codebase/o/r/pull/9"}]"#,
        )
        .unwrap()
        .unwrap();
        assert_eq!(merged.number, 9);
        assert_eq!(merged.state, PrState::Merged);
        assert!(parse_pick("[]").unwrap().is_none());
    }

    #[test]
    fn parse_string_and_numeric_check_states() {
        let pending = parse_pick(
            r#"[{"number":4,"status":"open","title":"feat","url":"https://cursor.com/codebase/o/r/pull/4","ciState":{"checkRunGroups":[{"checkRuns":[{"name":"ci","status":"completed","conclusion":"success"},{"name":"lint","status":2}]}]}}]"#,
        )
        .unwrap()
        .unwrap();
        assert_eq!(pending.checks, ChecksStatus::Pending);

        let failing = parse_pick(
            r#"[{"number":5,"status":"open","title":"feat","url":"https://cursor.com/codebase/o/r/pull/5","ciState":{"checkRunGroups":[{"checkRuns":[{"name":"ci","status":3,"conclusion":2}]}]}}]"#,
        )
        .unwrap()
        .unwrap();
        assert_eq!(failing.checks, ChecksStatus::Failing);
    }

    #[test]
    fn unknown_status_keeps_lookup_uncertain() {
        let error = parse_pick(
            r#"[{"number":6,"status":"superseded","title":"future","url":"https://cursor.com/codebase/o/r/pull/6"}]"#,
        )
        .unwrap_err();
        assert!(error.contains("unsupported status \"superseded\""));
    }

    #[test]
    fn list_failure_is_unknown_not_absence() {
        assert!(listed_prs(false, "", "Not authenticated. Run `origin auth login`.").is_err());
        assert!(
            listed_prs(
                false,
                "",
                "No open or draft change found for branch \"feat-x\" in acme/checkout."
            )
            .unwrap()
            .is_empty()
        );
        assert!(listed_prs(true, "[]", "").unwrap().is_empty());
        assert!(listed_prs(true, "{", "").is_err());
        assert!(is_no_pr_error(
            "No open or draft change found for branch \"feat-x\" in acme/checkout."
        ));
        assert!(!is_no_pr_error(
            "Not authenticated. Run `origin auth login`."
        ));
    }

    #[test]
    fn argv_puts_bookmark_on_head() {
        assert_eq!(pr_list_args("feat-x")[8..], ["--head", "feat-x"]);
        assert_eq!(pr_list_args("--repo=evil")[8..], ["--head", "--repo=evil"]);
    }
}
