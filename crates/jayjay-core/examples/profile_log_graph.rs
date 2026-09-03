use std::path::PathBuf;
use std::time::{Duration, Instant};

use jayjay_core::{LogQuery, Repo};

const WARM_RUN_COUNT: usize = 10;
const DEFAULT_LIMIT: u32 = 20;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("jayjay_core=debug")
        .without_time()
        .init();

    let mut args = std::env::args_os().skip(1);
    let path = PathBuf::from(args.next().expect("usage: profile_log_graph REPO [LIMIT]"));
    let limit = args
        .next()
        .map(|value| {
            value
                .to_string_lossy()
                .parse()
                .expect("LIMIT must be a u32")
        })
        .unwrap_or(DEFAULT_LIMIT);

    let open_started = Instant::now();
    let repo = Repo::open(&path).expect("open repository");
    eprintln!("open_ms={:.3}", millis(open_started.elapsed()));

    repo.log_graph_page(&LogQuery::Default, limit)
        .expect("warm log graph caches");
    let mut runs = (0..WARM_RUN_COUNT)
        .map(|_| {
            let started = Instant::now();
            let page = repo
                .log_graph_page(&LogQuery::Default, limit)
                .expect("load log graph page");
            let elapsed = started.elapsed();
            eprintln!(
                "total_ms={:.3} rows={} lanes={} has_more={}",
                millis(elapsed),
                page.entries.len(),
                page.layout.logical_column_count,
                page.has_more
            );
            elapsed
        })
        .collect::<Vec<_>>();
    runs.sort_unstable();
    eprintln!("median_total_ms={:.3}", millis(runs[WARM_RUN_COUNT / 2]));
}

fn millis(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}
