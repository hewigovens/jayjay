pub(super) use crate::commit_message::{body, summary as first_line};

// The change-id tail keeps the name stable across amend and rebase, so a re-run maps to the same bookmark.
pub(super) fn bookmark_name(description: &str, change_id: &str, short_len: u32) -> String {
    let slug = branch_name_slug(&first_line(description));
    if slug.is_empty() {
        return change_id.to_owned();
    }
    let n = (short_len as usize).min(change_id.len());
    format!("{slug}-{}", &change_id[..n])
}

const MAX_SLUG_WORDS: usize = 5;

pub fn branch_name_slug(text: &str) -> String {
    let mut words: Vec<String> = Vec::new();
    let mut word = String::new();
    for ch in text.chars() {
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
        assert_eq!(name, "feat-support-stacked-prs-across-kqxoznab");
    }

    #[test]
    fn branch_name_slug_caps_and_sanitizes_free_text() {
        assert_eq!(
            branch_name_slug("**Add stacked PR names, safely now**"),
            "add-stacked-pr-names-safely"
        );
        assert_eq!(branch_name_slug(" -- "), "");
    }
}
