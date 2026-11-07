use super::{ConvergeItem, ConvergeItemStatus, ConvergeReview};
use crate::types::{
    ConvergeAuthor, ConvergeAuthorAttribute, ConvergeDescriptionAttribute, ConvergeEffects,
    ConvergeParentsAttribute, ConvergePlan, ConvergeVersion, DescriptionChoice,
};

fn author(name: &str) -> ConvergeAuthor {
    ConvergeAuthor {
        name: name.to_owned(),
        email: format!("{name}@example.com"),
        timestamp_millis: 0,
        timezone_offset_minutes: 0,
    }
}

fn version(id: &str, description: &str, parent: &str) -> ConvergeVersion {
    ConvergeVersion {
        commit_id: id.to_owned(),
        description: description.to_owned(),
        author: author("same"),
        parents: vec![parent.to_owned()],
        is_immutable: false,
        workspaces: Vec::new(),
        bookmarks: Vec::new(),
    }
}

fn plan() -> ConvergePlan {
    ConvergePlan {
        change_id: "change".to_owned(),
        versions: vec![version("a", "left", "p1"), version("b", "right", "p2")],
        author: ConvergeAuthorAttribute::Solved {
            value: author("same"),
        },
        description: ConvergeDescriptionAttribute::Unsolved,
        parents: ConvergeParentsAttribute::Solved {
            value: vec!["p1".to_owned()],
        },
        conflicted_paths: Some(vec!["hello.txt".to_owned()]),
        contents_source: Some("a".to_owned()),
        effects: ConvergeEffects::default(),
        basis: vec!["a".to_owned(), "b".to_owned()],
    }
}

#[test]
fn unresolved_description_gates_apply_until_chosen() {
    let mut review = ConvergeReview::new(plan());
    assert_eq!(review.focused(), Some(ConvergeItem::Description));
    assert_eq!(review.unresolved_items(), [ConvergeItem::Description]);
    assert_eq!(review.same_items(), [ConvergeItem::Author]);
    assert_eq!(review.automatic_count(), 3);
    assert_eq!(
        review.status(ConvergeItem::Description),
        ConvergeItemStatus::Unresolved
    );
    assert_eq!(review.keeps(ConvergeItem::Parents), Some('A'));
    assert!(review.hint().is_some());
    assert!(!review.can_apply());

    review.use_version_at(1);
    assert!(review.can_apply());
    assert_eq!(
        review.status(ConvergeItem::Description),
        ConvergeItemStatus::Using('B')
    );
    assert_eq!(
        review.choices().description,
        Some(DescriptionChoice::FromVersion("b".to_owned()))
    );

    let mut blocked = plan();
    blocked.effects.blocked_reason = Some("immutable".to_owned());
    let mut review = ConvergeReview::new(blocked);
    review.keep("a");
    assert!(review.unresolved_items().is_empty());
    assert!(!review.can_apply());
}

#[test]
fn contents_follow_chosen_parents_and_drop_expected_conflicts() {
    let mut review = ConvergeReview::new(plan());
    assert_eq!(review.keeps(ConvergeItem::Contents), Some('A'));
    assert_eq!(
        review.status(ConvergeItem::Contents),
        ConvergeItemStatus::WillConflict(1)
    );

    review.use_version("a", ConvergeItem::Parents);
    assert_eq!(review.expected_conflicts().map(<[_]>::len), Some(1));
    review.use_version("b", ConvergeItem::Parents);
    assert!(!review.will_conflict());
    assert_eq!(review.keeps(ConvergeItem::Contents), None);
    assert_eq!(
        review.status(ConvergeItem::Contents),
        ConvergeItemStatus::FollowsParents
    );

    review.use_version("a", ConvergeItem::Contents);
    assert!(!review.will_conflict());
    assert_eq!(
        review.status(ConvergeItem::Contents),
        ConvergeItemStatus::Using('A')
    );

    let mut unsolved = plan();
    unsolved.parents = ConvergeParentsAttribute::Unsolved;
    let review = ConvergeReview::new(unsolved);
    assert_eq!(
        review.status(ConvergeItem::Contents),
        ConvergeItemStatus::FollowsParents
    );
    assert_eq!(
        review.unresolved_items(),
        [ConvergeItem::Description, ConvergeItem::Parents]
    );
}

#[test]
fn custom_description_starts_from_the_chosen_version_and_yields_to_a_choice() {
    let mut review = ConvergeReview::new(plan());
    review.use_version("b", ConvergeItem::Description);
    review.edit_description();
    assert_eq!(review.custom_description(), Some("right"));
    assert_eq!(review.selection(ConvergeItem::Description), None);
    review.set_custom_description("merged".to_owned());
    assert_eq!(
        review.status(ConvergeItem::Description),
        ConvergeItemStatus::Edited
    );
    assert_eq!(
        review.choices().description,
        Some(DescriptionChoice::Custom("merged".to_owned()))
    );

    review.keep("a");
    assert_eq!(review.custom_description(), None);
    let choices = review.choices();
    assert_eq!(
        choices.description,
        Some(DescriptionChoice::FromVersion("a".to_owned()))
    );
    assert_eq!(choices.author.as_deref(), Some("a"));
    assert_eq!(choices.contents.as_deref(), Some("a"));
}

#[test]
fn focus_moves_through_differing_items_only() {
    let mut review = ConvergeReview::new(plan());
    assert_eq!(
        review.card_items(),
        [
            ConvergeItem::Description,
            ConvergeItem::Contents,
            ConvergeItem::Parents
        ]
    );
    review.move_focus(-1);
    assert_eq!(review.focused(), Some(ConvergeItem::Parents));
    review.move_focus(1);
    review.move_focus(1);
    assert_eq!(review.focused(), Some(ConvergeItem::Contents));
}
