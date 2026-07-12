use super::RepoWindow;
use super::stacked_pr::StackedPrPhase;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StackedPrSnapshot {
    Closed,
    Loading,
    Preview {
        names: Vec<String>,
        bases: Vec<String>,
        warnings: Vec<Option<String>>,
        can_submit: bool,
        ai_in_flight: bool,
    },
    Submitting,
    Results {
        outcomes: Vec<String>,
        message: String,
    },
    Error(String),
}

impl RepoWindow {
    pub fn stacked_pr_snapshot(&self) -> StackedPrSnapshot {
        let Some(state) = self.stacked_pr.as_ref() else {
            return StackedPrSnapshot::Closed;
        };
        match &state.phase {
            StackedPrPhase::Loading => StackedPrSnapshot::Loading,
            StackedPrPhase::Preview(stack) => StackedPrSnapshot::Preview {
                names: state
                    .inputs
                    .iter()
                    .map(|input| input.text().to_owned())
                    .collect(),
                bases: stack
                    .layers
                    .iter()
                    .enumerate()
                    .map(|(index, _)| {
                        if index == 0 {
                            stack.base_bookmark.clone()
                        } else {
                            state.inputs[index - 1].text().to_owned()
                        }
                    })
                    .collect(),
                warnings: (0..state.inputs.len())
                    .map(|index| state.warning(index).map(str::to_owned))
                    .collect(),
                can_submit: state.can_submit(),
                ai_in_flight: state.ai_in_flight,
            },
            StackedPrPhase::Submitting(_) => StackedPrSnapshot::Submitting,
            StackedPrPhase::Results(result) => StackedPrSnapshot::Results {
                outcomes: result
                    .layers
                    .iter()
                    .map(|layer| format!("{:?}", layer.outcome))
                    .collect(),
                message: result.message.clone(),
            },
            StackedPrPhase::Error(error) => StackedPrSnapshot::Error(error.clone()),
        }
    }
}
