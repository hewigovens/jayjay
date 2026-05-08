use serde_json::Value;

use crate::{SavedRevset, hash::hex_sha256};

pub fn built_in_revsets() -> Vec<SavedRevset> {
    [
        ("builtin:all", "All", "all()"),
        ("builtin:mine", "Mine", "mine()"),
        ("builtin:bookmarks", "Bookmarks", "bookmarks()"),
        ("builtin:trunk", "Trunk", "trunk()"),
        ("builtin:conflicts", "Conflicts", "conflicts()"),
        ("builtin:heads", "Heads (No Children)", "heads(all())"),
        (
            "builtin:local-stack",
            "Local Stack",
            "reachable(@, mutable())",
        ),
        (
            "builtin:current-dir",
            "Touching Current Directory",
            "files(\".\")",
        ),
        ("builtin:fork-point", "Fork Point of @", "fork_point(@)"),
        (
            "builtin:empty-mutable",
            "Empty Mutable",
            "empty() & mutable()",
        ),
    ]
    .into_iter()
    .map(|(id, name, expression)| SavedRevset {
        id: id.to_owned(),
        name: name.to_owned(),
        expression: expression.to_owned(),
    })
    .collect()
}

pub fn decode_saved_revsets_json(json: &str) -> Vec<SavedRevset> {
    serde_json::from_str::<Vec<SavedRevset>>(json).unwrap_or_else(|_| {
        serde_json::from_str::<Vec<Value>>(json)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|value| serde_json::from_value(value).ok())
            .collect()
    })
}

pub fn encode_saved_revsets_json(revsets: &[SavedRevset]) -> String {
    serde_json::to_string(revsets).unwrap_or_else(|_| "[]".to_owned())
}

pub fn upsert_saved_revset(
    mut existing: Vec<SavedRevset>,
    name: &str,
    expression: &str,
) -> Vec<SavedRevset> {
    let name = name.trim();
    let expression = expression.trim();
    if name.is_empty() || expression.is_empty() {
        return existing;
    }

    existing.retain(|item| {
        !item.name.eq_ignore_ascii_case(name) && item.expression.as_str() != expression
    });
    existing.insert(
        0,
        SavedRevset {
            id: saved_revset_id(name, expression),
            name: name.to_owned(),
            expression: expression.to_owned(),
        },
    );
    existing
}

pub fn remove_saved_revset(existing: Vec<SavedRevset>, id: &str) -> Vec<SavedRevset> {
    existing.into_iter().filter(|item| item.id != id).collect()
}

fn saved_revset_id(name: &str, expression: &str) -> String {
    let hash = hex_sha256(format!("{name}\0{expression}").as_bytes());
    format!("saved:{}", &hash[..16])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_ins_include_conflicts() {
        assert!(
            built_in_revsets()
                .iter()
                .any(|item| item.expression == "conflicts()")
        );
    }

    #[test]
    fn upsert_trims_and_dedupes_name_or_expression() {
        let existing = vec![
            SavedRevset {
                id: "a".to_owned(),
                name: "Mine".to_owned(),
                expression: "mine()".to_owned(),
            },
            SavedRevset {
                id: "b".to_owned(),
                name: "Other".to_owned(),
                expression: "all()".to_owned(),
            },
        ];

        let saved = upsert_saved_revset(existing, " mine ", " all() ");
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0].name, "mine");
        assert_eq!(saved[0].expression, "all()");
        assert!(saved[0].id.starts_with("saved:"));
    }

    #[test]
    fn upsert_ignores_empty_name_or_expression() {
        let existing = vec![SavedRevset {
            id: "a".to_owned(),
            name: "Mine".to_owned(),
            expression: "mine()".to_owned(),
        }];

        assert_eq!(
            upsert_saved_revset(existing.clone(), " ", "all()"),
            existing
        );
        assert_eq!(
            upsert_saved_revset(existing.clone(), "All", " \n\t "),
            existing
        );
    }

    #[test]
    fn json_round_trip_preserves_ids() {
        let revsets = vec![SavedRevset {
            id: "uuid-or-stable-id".to_owned(),
            name: "Stack".to_owned(),
            expression: "reachable(@, mutable())".to_owned(),
        }];
        let json = encode_saved_revsets_json(&revsets);
        assert_eq!(decode_saved_revsets_json(&json), revsets);
    }
}
