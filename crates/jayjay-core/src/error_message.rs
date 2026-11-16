const ERROR_MARKERS: [&str; 2] = ["Error:", "Caused by:"];
const WRAPPER_PREFIXES: [&str; 4] = [
    "git push failed:",
    "git fetch failed:",
    "Error:",
    "Caused by:",
];
const REMOTE_FAILURES: [&str; 3] = [
    "git push failed",
    "git fetch failed",
    "Failed to push some bookmarks",
];
const PUSH_SUMMARIES: [&str; 2] = ["Failed to push some bookmarks", "Git process failed"];
const PUSH_PROGRESS: [&str; 3] = ["Changes to push to ", "bookmark:", "Done importing changes"];
const DEBUG_HINT: &str = " (run with --debug";

/// Reduce a jj or git failure (`command failed: … Error: … Caused by: …`, push progress chatter) to the sentence a shell can show as-is.
pub fn unwrap_command_error(message: &str) -> String {
    let message = message.replace("command failed:", "");
    unwrap_git_remote_error(&message).unwrap_or_else(|| unwrap_error_chain(&message))
}

/// `Error:` and `Caused by:` lines joined into one sentence, with any `Hint:` lines kept after it.
fn unwrap_error_chain(message: &str) -> String {
    let mut parts = Vec::new();
    let mut hints = Vec::new();
    for line in message.lines().map(str::trim) {
        if let Some(part) = ERROR_MARKERS
            .iter()
            .find_map(|marker| after_marker(line, marker))
        {
            parts.push(part);
        } else if let Some(hint) = after_marker(line, "Hint:").filter(|hint| !hint.is_empty()) {
            hints.push(hint);
        }
    }
    let sentence = if parts.is_empty() {
        message.trim()
    } else {
        &parts.join(": ")
    };
    let sentence = strip_debug_hint(sentence).to_owned();
    hints.into_iter().fold(sentence, |mut text, hint| {
        text.push_str("\nHint: ");
        text.push_str(hint);
        text
    })
}

/// A failed push or fetch carries the transport's own diagnostics, so keep the git output and drop only the progress lines the shell already shows.
fn unwrap_git_remote_error(message: &str) -> Option<String> {
    if !REMOTE_FAILURES
        .iter()
        .any(|failure| message.contains(failure))
    {
        return None;
    }
    let mut summary = None;
    let mut details: Vec<&str> = Vec::new();
    for line in message
        .lines()
        .map(|line| strip_wrapper_prefixes(line.trim()))
    {
        if line.is_empty() || is_push_progress_line(line) {
            continue;
        }
        if PUSH_SUMMARIES.iter().any(|prefix| line.starts_with(prefix)) {
            summary = Some(line);
        } else if !details.contains(&line) {
            details.push(line);
        }
    }
    details.retain(|line| Some(*line) != summary);
    let lines: Vec<&str> = summary.into_iter().chain(details).collect();
    (!lines.is_empty()).then(|| strip_debug_hint(&lines.join("\n")).to_owned())
}

fn is_push_progress_line(line: &str) -> bool {
    line == "git:" || PUSH_PROGRESS.iter().any(|prefix| line.starts_with(prefix))
}

fn strip_wrapper_prefixes(mut line: &str) -> &str {
    while let Some(rest) = WRAPPER_PREFIXES
        .iter()
        .find_map(|prefix| line.strip_prefix(prefix))
    {
        line = rest.trim_start();
    }
    line
}

fn after_marker<'a>(line: &'a str, marker: &str) -> Option<&'a str> {
    line.find(marker)
        .map(|start| line[start + marker.len()..].trim())
}

fn strip_debug_hint(message: &str) -> &str {
    message
        .find(DEBUG_HINT)
        .map_or(message, |start| &message[..start])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduces_wrapped_failure_to_one_sentence() {
        assert_eq!(
            unwrap_command_error(
                "command failed: Resolving conflicts in: f.txt\nError: Failed to resolve conflicts\nCaused by: The output file is either unchanged or empty after the editor quit (run with --debug to see the exact invocation)."
            ),
            "Failed to resolve conflicts: The output file is either unchanged or empty after the editor quit"
        );
        assert_eq!(
            unwrap_command_error("command failed: Error: No conflicts found at the given path(s)"),
            "No conflicts found at the given path(s)"
        );
        assert_eq!(
            unwrap_command_error("repository is not open"),
            "repository is not open"
        );
    }

    #[test]
    fn keeps_hint_lines() {
        assert_eq!(
            unwrap_command_error(
                "jj git init: Error: Cannot create a colocated jj repo inside a Git worktree.\nHint: Run `jj git init` in the main Git repository instead."
            ),
            "Cannot create a colocated jj repo inside a Git worktree.\nHint: Run `jj git init` in the main Git repository instead."
        );
        assert_eq!(
            unwrap_command_error(
                "Error: Rebase failed (run with --debug to see the exact invocation).\nHint: Resolve the conflict first."
            ),
            "Rebase failed\nHint: Resolve the conflict first."
        );
    }

    #[test]
    fn push_failure_keeps_summary_and_transport_details() {
        assert_eq!(
            unwrap_command_error(
                "git push failed: Changes to push to origin:\n  bookmark: main [advance 0bb004e -> 7517127]\ngit: git@github.com: Permission denied (publickey).\ngit:\nError: Failed to push some bookmarks"
            ),
            "Failed to push some bookmarks\ngit: git@github.com: Permission denied (publickey)."
        );
        assert_eq!(
            unwrap_command_error(
                "git push failed: Changes to push to origin:\n  bookmark: main [add to 9d084bdd0c7e]\ngit: ssh: Could not resolve hostname invalid.invalid: nodename nor servname provided, or not known\ngit:\nError: Git process failed: External git program failed:\nfatal: Could not read from remote repository."
            ),
            "Git process failed: External git program failed:\ngit: ssh: Could not resolve hostname invalid.invalid: nodename nor servname provided, or not known\nfatal: Could not read from remote repository."
        );
    }

    #[test]
    fn fetch_failure_keeps_the_git_output_jj_leaves_out_of_its_error() {
        assert_eq!(
            unwrap_command_error(
                "git fetch failed: git: Shim: Could not create process with command '\"C:\\scoop\\apps\\git\\current\\cmd\\git.exe\" fetch'.\nError: Git process failed: External git program failed:"
            ),
            "Git process failed: External git program failed:\ngit: Shim: Could not create process with command '\"C:\\scoop\\apps\\git\\current\\cmd\\git.exe\" fetch'."
        );
    }
}
