pub(super) use crate::commit_message::{body, summary as first_line};

/// Auto bookmark name for a change: `<slug>-<shortest-changeid>`, or the full
/// change-id when the description has no usable slug. The change-id tail keeps the
/// name unique and stable across amend/rebase, so re-running maps to the same one.
pub(super) fn bookmark_name(description: &str, change_id: &str, short_len: u32) -> String {
    let slug = slugify(&first_line(description));
    if slug.is_empty() {
        return change_id.to_owned();
    }
    let n = (short_len as usize).min(change_id.len());
    format!("{slug}-{}", &change_id[..n])
}

/// At most this many words in an auto/generated branch slug — short, readable
/// names. The change-id suffix is appended on top of this.
pub(crate) const MAX_SLUG_WORDS: usize = 5;

/// Lowercase, hyphen-joined slug of the first `MAX_SLUG_WORDS` alphanumeric words.
fn slugify(s: &str) -> String {
    let mut words: Vec<String> = Vec::new();
    let mut word = String::new();
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            word.push(ch.to_ascii_lowercase());
        } else if !word.is_empty() {
            words.push(std::mem::take(&mut word));
            if words.len() == MAX_SLUG_WORDS {
                return words.join("-");
            }
        }
    }
    if !word.is_empty() {
        words.push(word);
    }
    words.join("-")
}

/// Whether `name` is usable as a git branch / jj bookmark — a conservative subset
/// of `git check-ref-format`. Rejects empties, whitespace, the reserved ref
/// characters, `..`, and ill-formed path components, so a bad edit can't reach
/// `jj git push` after bookmarks were already moved locally.
pub fn is_valid_bookmark_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 255 || name == "@" {
        return false;
    }
    if name.starts_with('-') || name.starts_with('/') || name.ends_with('/') || name.ends_with('.')
    {
        return false;
    }
    if name.contains("..") || name.contains("//") || name.contains("@{") {
        return false;
    }
    if name.chars().any(|ch| {
        ch.is_control()
            || ch.is_whitespace()
            || matches!(ch, '~' | '^' | ':' | '?' | '*' | '[' | '\\')
    }) {
        return false;
    }
    // git ref-format rules are per slash-separated component: none may be empty,
    // dot-leading (`foo/.bar`), or end in `.lock` (`foo.lock/bar`, not just `foo.lock`).
    name.split('/')
        .all(|c| !c.is_empty() && !c.starts_with('.') && !c.ends_with(".lock"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bookmark_name_slugifies_title_and_appends_shortest_change_id() {
        let name = bookmark_name("feat: Add GitLab support!\n\nbody", "kqxoznabcd1234", 8);
        assert_eq!(name, "feat-add-gitlab-support-kqxoznab");
    }

    #[test]
    fn bookmark_name_empty_description_uses_full_change_id() {
        assert_eq!(bookmark_name("", "kqxoznabcd1234", 8), "kqxoznabcd1234");
        assert_eq!(bookmark_name("Hi", "kqxoznabcd1234", 4), "hi-kqxo");
    }

    #[test]
    fn bookmark_name_caps_slug_at_five_words() {
        let name = bookmark_name(
            "feat: support stacked PRs across many forges and remotes now",
            "kqxoznab",
            8,
        );
        // Only the first five words become the slug; the change-id is the suffix.
        assert_eq!(name, "feat-support-stacked-prs-across-kqxoznab");
    }

    #[test]
    fn validates_bookmark_names() {
        for ok in ["feat-add-x-abc123", "user/feat/thing", "v1.2-rc"] {
            assert!(is_valid_bookmark_name(ok), "{ok} should be valid");
        }
        for bad in [
            "",
            "@",
            "has space",
            "bad..dots",
            "-leading-dash",
            "trailing/",
            "ends.",
            "name.lock",
            "foo.lock/bar",
            "ti~lde",
            "co:lon",
            "a//b",
            "foo/.hidden",
            "ctrl\tchar",
        ] {
            assert!(!is_valid_bookmark_name(bad), "{bad:?} should be invalid");
        }
    }
}
