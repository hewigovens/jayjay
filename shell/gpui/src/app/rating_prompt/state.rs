use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(default)]
pub struct RatingPromptState {
    pub action_count: u32,
    pub next_prompt_at: u32,
    pub dismissed: bool,
}

impl Default for RatingPromptState {
    fn default() -> Self {
        Self {
            action_count: 0,
            next_prompt_at: Self::FIRST_PROMPT_AT,
            dismissed: false,
        }
    }
}

impl RatingPromptState {
    const FIRST_PROMPT_AT: u32 = 5;
    const INTERVAL: u32 = 20;

    pub(super) fn record_action(&mut self, can_show: bool) -> bool {
        if self.dismissed {
            return false;
        }
        self.action_count = self.action_count.saturating_add(1);
        let due = can_show && self.action_count >= self.next_prompt_at;
        if due {
            self.next_prompt_at = self.action_count.saturating_add(Self::INTERVAL);
        }
        due
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompts_after_five_actions_then_every_twenty_until_dismissed() {
        let mut state = RatingPromptState::default();
        let due: Vec<u32> = (1..=45).filter(|_| state.record_action(true)).collect();
        assert_eq!(due, [5, 25, 45]);

        let mut blocked = RatingPromptState::default();
        let shown: Vec<bool> = (1..=6).map(|n| blocked.record_action(n != 5)).collect();
        assert_eq!(shown, [false, false, false, false, false, true]);

        state.dismissed = true;
        assert!(!(0..40).any(|_| state.record_action(true)));
    }
}
