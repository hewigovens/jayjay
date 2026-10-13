use jayjay_core::{BisectPhase, BisectState, SyncToken};

#[derive(Default)]
pub struct BisectModel {
    pub state: Option<BisectState>,
    pub activity: BisectActivity,
}

#[derive(Default)]
pub enum BisectActivity {
    #[default]
    Idle,
    Transitioning,
    Running(SyncToken),
    Stopping,
}

impl BisectModel {
    pub(crate) fn is_busy(&self) -> bool {
        !matches!(self.activity, BisectActivity::Idle)
    }

    pub(crate) fn command_is_running(&self) -> bool {
        matches!(self.activity, BisectActivity::Running(_))
    }

    pub(crate) fn is_stopping(&self) -> bool {
        matches!(self.activity, BisectActivity::Stopping)
    }

    pub(crate) fn can_mark(&self) -> bool {
        !self.is_busy()
            && self.state.as_ref().is_some_and(|state| {
                state.phase == BisectPhase::Evaluating && state.candidate.is_some()
            })
    }

    pub(crate) fn can_stop(&self) -> bool {
        self.state.is_some()
            && matches!(
                self.activity,
                BisectActivity::Idle | BisectActivity::Running(_)
            )
    }

    pub(crate) fn begin_transition(&mut self) -> bool {
        self.begin(BisectActivity::Transitioning)
    }

    pub(crate) fn begin_command(&mut self, token: SyncToken) -> bool {
        self.begin(BisectActivity::Running(token))
    }

    fn begin(&mut self, activity: BisectActivity) -> bool {
        if self.is_busy() {
            return false;
        }
        self.activity = activity;
        true
    }

    pub(crate) fn begin_stop(&mut self) -> Option<SyncToken> {
        match std::mem::replace(&mut self.activity, BisectActivity::Stopping) {
            BisectActivity::Running(token) => Some(token),
            BisectActivity::Idle => None,
            other => {
                self.activity = other;
                None
            }
        }
    }

    pub(crate) fn finish_activity(&mut self) {
        self.activity = BisectActivity::Idle;
    }
}
