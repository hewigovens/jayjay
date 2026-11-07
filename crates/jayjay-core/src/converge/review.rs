use std::collections::BTreeMap;

use super::{ConvergeItem, ConvergeItemStatus};
use crate::types::{
    ConvergeAuthorAttribute, ConvergeChoices, ConvergeDescriptionAttribute,
    ConvergeParentsAttribute, ConvergePlan, ConvergeVersion, DescriptionChoice,
};

#[derive(Debug, Clone)]
pub struct ConvergeReview {
    plan: ConvergePlan,
    selections: BTreeMap<ConvergeItem, String>,
    custom_description: Option<String>,
    focused: Option<ConvergeItem>,
}

impl ConvergeReview {
    pub fn new(plan: ConvergePlan) -> Self {
        let mut review = Self {
            plan,
            selections: BTreeMap::new(),
            custom_description: None,
            focused: None,
        };
        review.focused = review
            .unresolved_items()
            .first()
            .or(review.card_items().first())
            .copied();
        review
    }

    pub fn plan(&self) -> &ConvergePlan {
        &self.plan
    }

    pub fn versions(&self) -> &[ConvergeVersion] {
        &self.plan.versions
    }

    pub fn letter(&self, commit_id: &str) -> char {
        let index = self
            .versions()
            .iter()
            .position(|version| version.commit_id == commit_id)
            .unwrap_or(0);
        char::from(b'A' + (index % 26) as u8)
    }

    pub fn selection(&self, item: ConvergeItem) -> Option<&str> {
        self.selections.get(&item).map(String::as_str)
    }

    pub fn custom_description(&self) -> Option<&str> {
        self.custom_description.as_deref()
    }

    pub fn set_custom_description(&mut self, text: String) {
        if self.custom_description.is_some() {
            self.custom_description = Some(text);
        }
    }

    pub fn focused(&self) -> Option<ConvergeItem> {
        self.focused
    }

    pub fn focus(&mut self, item: ConvergeItem) {
        self.focused = Some(item);
    }

    fn is_solved(&self, item: ConvergeItem) -> bool {
        match item {
            ConvergeItem::Description => !matches!(
                self.plan.description,
                ConvergeDescriptionAttribute::Unsolved
            ),
            ConvergeItem::Author => !matches!(self.plan.author, ConvergeAuthorAttribute::Unsolved),
            ConvergeItem::Parents => {
                !matches!(self.plan.parents, ConvergeParentsAttribute::Unsolved)
            }
            ConvergeItem::Contents => true,
        }
    }

    fn is_same(&self, item: ConvergeItem) -> bool {
        let Some((first, rest)) = self.versions().split_first() else {
            return false;
        };
        match item {
            ConvergeItem::Description => rest.iter().all(|v| v.description == first.description),
            ConvergeItem::Author => rest.iter().all(|v| v.author == first.author),
            ConvergeItem::Parents => rest.iter().all(|v| v.parents == first.parents),
            ConvergeItem::Contents => false,
        }
    }

    fn is_chosen(&self, item: ConvergeItem) -> bool {
        self.selections.contains_key(&item)
            || (item == ConvergeItem::Description && self.custom_description.is_some())
    }

    fn is_unresolved(&self, item: ConvergeItem) -> bool {
        !self.is_solved(item) && !self.is_chosen(item)
    }

    /// The version whose value the result takes when the user makes no choice for `item`.
    fn automatic_source(&self, item: ConvergeItem) -> Option<&str> {
        let plan = &self.plan;
        let version = match item {
            ConvergeItem::Description => plan
                .description
                .solved()
                .and_then(|value| self.versions().iter().find(|v| &v.description == value)),
            ConvergeItem::Author => plan
                .author
                .solved()
                .and_then(|value| self.versions().iter().find(|v| &v.author == value)),
            ConvergeItem::Parents => plan
                .parents
                .solved()
                .and_then(|value| self.versions().iter().find(|v| &v.parents == value)),
            ConvergeItem::Contents if self.contents_follow_parents() => None,
            ConvergeItem::Contents => self
                .versions()
                .iter()
                .find(|v| plan.contents_source.as_ref() == Some(&v.commit_id)),
        };
        version.map(|version| version.commit_id.as_str())
    }

    pub(super) fn keeps(&self, item: ConvergeItem) -> Option<char> {
        if self.is_chosen(item) {
            return None;
        }
        self.automatic_source(item).map(|id| self.letter(id))
    }

    /// Without a contents choice, contents are recomputed whenever the parents differ from the previewed ones.
    fn contents_follow_parents(&self) -> bool {
        if self.selections.contains_key(&ConvergeItem::Contents) {
            return false;
        }
        let Some(solved) = self.plan.parents.solved() else {
            return true;
        };
        self.selection(ConvergeItem::Parents).is_some_and(|id| {
            self.versions()
                .iter()
                .find(|v| v.commit_id == id)
                .is_some_and(|v| &v.parents != solved)
        })
    }

    pub fn expected_conflicts(&self) -> Option<&[String]> {
        if self.selections.contains_key(&ConvergeItem::Contents) || self.contents_follow_parents() {
            return None;
        }
        self.plan
            .conflicted_paths
            .as_deref()
            .filter(|paths| !paths.is_empty())
    }

    pub fn will_conflict(&self) -> bool {
        self.expected_conflicts().is_some()
    }

    pub fn card_items(&self) -> Vec<ConvergeItem> {
        ConvergeItem::ALL
            .into_iter()
            .filter(|item| !self.is_same(*item))
            .collect()
    }

    pub fn same_items(&self) -> Vec<ConvergeItem> {
        ConvergeItem::ALL
            .into_iter()
            .filter(|item| self.is_same(*item))
            .collect()
    }

    pub fn unresolved_items(&self) -> Vec<ConvergeItem> {
        ConvergeItem::ALL
            .into_iter()
            .filter(|item| self.is_unresolved(*item))
            .collect()
    }

    pub fn automatic_count(&self) -> usize {
        ConvergeItem::ALL
            .into_iter()
            .filter(|item| self.is_solved(*item))
            .count()
    }

    pub fn status(&self, item: ConvergeItem) -> ConvergeItemStatus {
        if item == ConvergeItem::Description && self.custom_description.is_some() {
            return ConvergeItemStatus::Edited;
        }
        if let Some(id) = self.selection(item) {
            return ConvergeItemStatus::Using(self.letter(id));
        }
        if self.is_unresolved(item) {
            return ConvergeItemStatus::Unresolved;
        }
        if item == ConvergeItem::Contents {
            if let Some(paths) = self.expected_conflicts() {
                return ConvergeItemStatus::WillConflict(paths.len());
            }
            if self.contents_follow_parents() {
                return ConvergeItemStatus::FollowsParents;
            }
        }
        ConvergeItemStatus::Merged
    }

    pub fn note(&self, item: ConvergeItem, changed_files: Option<usize>) -> Option<String> {
        let keeps = self
            .keeps(item)
            .map(|letter| jayjay_i18n::converge::result_keeps(&letter.to_string()));
        match item {
            ConvergeItem::Contents => match changed_files? {
                0 => Some(jayjay_i18n::converge::same(self.versions().len())),
                count => {
                    let changed = jayjay_i18n::converge::changed_files(count);
                    Some(match keeps {
                        Some(keeps) => format!("{changed} · {keeps}"),
                        None => changed,
                    })
                }
            },
            _ if self.is_unresolved(item) => Some(jayjay_i18n::converge::versions_disagree()),
            _ => keeps,
        }
    }

    pub fn hint(&self) -> Option<String> {
        match self.unresolved_items().first()? {
            ConvergeItem::Author => Some(jayjay_i18n::converge::choose_author_hint()),
            ConvergeItem::Description => Some(jayjay_i18n::converge::choose_description_hint()),
            ConvergeItem::Parents => Some(jayjay_i18n::converge::choose_parents_hint()),
            ConvergeItem::Contents => None,
        }
    }

    pub fn can_apply(&self) -> bool {
        self.plan.effects.blocked_reason.is_none() && self.unresolved_items().is_empty()
    }

    pub fn choices(&self) -> ConvergeChoices {
        let selection = |item| self.selection(item).map(str::to_owned);
        ConvergeChoices {
            author: selection(ConvergeItem::Author),
            parents: selection(ConvergeItem::Parents),
            description: self
                .custom_description
                .clone()
                .map(DescriptionChoice::Custom)
                .or_else(|| {
                    selection(ConvergeItem::Description).map(DescriptionChoice::FromVersion)
                }),
            contents: selection(ConvergeItem::Contents),
        }
    }

    pub fn use_version(&mut self, commit_id: &str, item: ConvergeItem) {
        self.selections.insert(item, commit_id.to_owned());
        self.focused = Some(item);
        if item == ConvergeItem::Description {
            self.custom_description = None;
        }
    }

    pub fn use_version_at(&mut self, index: usize) {
        let Some(item) = self.focused else {
            return;
        };
        if let Some(id) = self.versions().get(index).map(|v| v.commit_id.clone()) {
            self.use_version(&id, item);
        }
    }

    pub fn keep(&mut self, commit_id: &str) {
        for item in ConvergeItem::ALL {
            self.selections.insert(item, commit_id.to_owned());
        }
        self.custom_description = None;
    }

    pub fn edit_description(&mut self) {
        if self.custom_description.is_some() {
            return;
        }
        let source = self
            .selection(ConvergeItem::Description)
            .or(self.automatic_source(ConvergeItem::Description))
            .or(self.versions().first().map(|v| v.commit_id.as_str()));
        let text = self
            .versions()
            .iter()
            .find(|v| Some(v.commit_id.as_str()) == source)
            .map(|v| v.description.clone())
            .unwrap_or_default();
        self.custom_description = Some(text);
        self.selections.remove(&ConvergeItem::Description);
        self.focused = Some(ConvergeItem::Description);
    }

    pub fn move_focus(&mut self, delta: isize) {
        let items = self.card_items();
        if items.is_empty() {
            return;
        }
        let count = items.len() as isize;
        let current = self
            .focused
            .and_then(|item| items.iter().position(|candidate| *candidate == item))
            .map_or(if delta > 0 { -1 } else { count }, |ix| ix as isize);
        self.focused = Some(items[(current + delta).rem_euclid(count) as usize]);
    }
}
