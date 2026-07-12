use gpui::{AppContext, Context};

use super::RepoWindow;
use super::stacked_pr::{StackedPrPhase, next_stacked_pr_generation};

impl RepoWindow {
    pub fn generate_stacked_pr_names(&mut self, cx: &mut Context<Self>) {
        let Some(state) = self.stacked_pr.as_mut() else {
            return;
        };
        if state.ai_in_flight || !matches!(state.phase, StackedPrPhase::Preview(_)) {
            return;
        }
        if self.commit_ai.provider_name.is_none() {
            self.show_toast(
                "No AI CLI found. Install codex or claude to generate names.",
                cx,
            );
            return;
        }
        let stack = state.stack().unwrap();
        let requests: Vec<_> = stack
            .layers
            .iter()
            .enumerate()
            .filter(|(_, layer)| !layer.bookmark_existed)
            .map(|(index, layer)| {
                (
                    index,
                    [layer.title.as_str(), layer.body.as_str()]
                        .into_iter()
                        .filter(|part| !part.is_empty())
                        .collect::<Vec<_>>()
                        .join("\n"),
                    layer.change_id_short.clone(),
                )
            })
            .collect();
        if requests.is_empty() {
            return;
        }
        let snapshot: Vec<_> = state
            .inputs
            .iter()
            .map(|input| input.text().to_owned())
            .collect();
        state.ai_generation = next_stacked_pr_generation();
        let generation = state.ai_generation;
        state.ai_in_flight = true;
        let provider = self.commit_ai.provider.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    requests
                        .into_iter()
                        .map(|(index, description, suffix)| {
                            provider
                                .generate_branch_name(&description)
                                .map(|slug| (index, format!("{slug}-{suffix}")))
                        })
                        .collect::<Result<Vec<_>, _>>()
                })
                .await;
            let _ = this.update(cx, |view, cx| {
                view.finish_stacked_pr_naming(generation, snapshot, result, cx);
            });
        })
        .detach();
        cx.notify();
    }

    fn finish_stacked_pr_naming(
        &mut self,
        generation: u64,
        snapshot: Vec<String>,
        result: Result<Vec<(usize, String)>, String>,
        cx: &mut Context<Self>,
    ) {
        let Some(state) = self.stacked_pr.as_mut() else {
            return;
        };
        if generation != state.ai_generation {
            return;
        }
        state.ai_in_flight = false;
        match result {
            Ok(names) => {
                for (index, name) in names {
                    if state.inputs[index].text() == snapshot[index] {
                        state.inputs[index].set_text(name);
                    }
                }
                cx.notify();
            }
            Err(message) => self.show_toast(message, cx),
        }
    }
}
