const HISTORY_LIMIT: usize = 20;

pub(super) struct HistoryRecall {
    pub query: String,
    pub index: Option<usize>,
}

pub(super) fn record(command: &str, history: &[String]) -> Vec<String> {
    let command = command.trim();
    if command.is_empty() {
        return history.to_vec();
    }
    let mut values = vec![command.to_owned()];
    values.extend(
        history
            .iter()
            .filter(|value| value.as_str() != command)
            .cloned(),
    );
    values.truncate(HISTORY_LIMIT);
    values
}

pub(super) fn recall(
    history: &[String],
    history_index: Option<usize>,
    older: bool,
) -> Option<HistoryRecall> {
    if history.is_empty() {
        return None;
    }
    let index = if older {
        Some(
            history_index
                .map(|index| index.saturating_add(1))
                .unwrap_or(0)
                .min(history.len() - 1),
        )
    } else if let Some(index) = history_index
        && index > 0
    {
        Some(index - 1)
    } else {
        None
    };
    Some(HistoryRecall {
        query: index
            .map(|index| format!("jj {}", history[index]))
            .unwrap_or_else(|| "jj ".to_owned()),
        index,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_dedupes_and_keeps_newest_first() {
        let history = record("status", &[]);
        let history = record("log -r @", &history);
        let history = record("status", &history);

        assert_eq!(history, vec!["status", "log -r @"]);
    }

    #[test]
    fn recall_walks_older_and_newer_entries() {
        let history = vec!["status".to_owned(), "log -r @".to_owned()];

        let first = recall(&history, None, true).expect("first recall");
        assert_eq!(first.query, "jj status");
        assert_eq!(first.index, Some(0));

        let second = recall(&history, first.index, true).expect("second recall");
        assert_eq!(second.query, "jj log -r @");
        assert_eq!(second.index, Some(1));

        let newer = recall(&history, second.index, false).expect("newer recall");
        assert_eq!(newer.query, "jj status");
        assert_eq!(newer.index, Some(0));

        let live_query = recall(&history, newer.index, false).expect("live query recall");
        assert_eq!(live_query.query, "jj ");
        assert_eq!(live_query.index, None);
    }
}
