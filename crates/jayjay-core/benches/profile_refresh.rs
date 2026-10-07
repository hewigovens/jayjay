use std::hint::black_box;
use std::path::Path;
use std::time::Instant;

use jayjay_core::dag::{DagLayout, SelectionGraph};
use jayjay_core::{DEFAULT_REVSET, Repo};
use jj_lib::config::StackedConfig;
use jj_lib::default_backend_factories::default_backend_factories;
use jj_lib::repo::RepoLoader;
use jj_lib::settings::UserSettings;
use jj_lib::workspace::{DefaultWorkspaceLoaderFactory, WorkspaceLoaderFactory as _};

#[cfg(feature = "hotpath-alloc")]
#[global_allocator]
static ALLOCATOR: hotpath::CountingAllocator = hotpath::CountingAllocator::new();

fn main() {
    let args: Vec<_> = std::env::args()
        .skip(1)
        .filter(|arg| arg != "--bench")
        .collect();
    let (path, scenario) = match args.as_slice() {
        [path] => (Path::new(path), "refresh"),
        [path, scenario]
            if matches!(
                scenario.as_str(),
                "refresh" | "graph" | "bookmarks" | "overview"
            ) =>
        {
            (Path::new(path), scenario.as_str())
        }
        _ => panic!("usage: profile_refresh <repo-path> [refresh|graph|bookmarks|overview]"),
    };
    assert_single_op_head(path);

    #[cfg(feature = "hotpath")]
    let _profile = hotpath::HotpathGuardBuilder::new("profile_refresh")
        .percentiles(&[50.0, 95.0])
        .functions_limit(0)
        .build();

    // Each iteration reopens the repo, so every pass is a first load with cold caches.
    for _ in 0..5 {
        let repo = timed("open", || Repo::open(path).expect("open repo"));
        if matches!(scenario, "refresh" | "graph") {
            let entries = timed("log_graph", || {
                repo.log_graph(DEFAULT_REVSET).expect("log graph")
            });
            timed("layout", || {
                black_box((DagLayout::compute(&entries), SelectionGraph::new(&entries)))
            });
            eprintln!("rows: {}", entries.len());
        }
        if matches!(scenario, "refresh" | "bookmarks") {
            let bookmarks = timed("list_bookmarks", || {
                repo.list_bookmarks().expect("bookmarks")
            });
            timed("revset_vocabulary", || {
                black_box(repo.revset_vocabulary(&bookmarks))
            });
        }
        if scenario == "refresh" {
            timed("show(@)", || black_box(repo.show("@").expect("show")));
            timed("context", || {
                black_box((
                    repo.workspace_list().expect("workspaces"),
                    repo.pr_host_name(),
                    repo.fix_unavailable_reason(),
                    repo.current_operation_description(),
                    repo.diff_stats("@").expect("diff stats"),
                ))
            });
        }
        if scenario == "overview" {
            let snapshot = timed("overview_snapshot", || {
                repo.overview_snapshot().expect("overview")
            });
            eprintln!(
                "lanes: {}, groups: {}",
                snapshot.overview.lanes.len(),
                snapshot.groups.len()
            );
        }
        eprintln!("---");
    }
}

/// `Repo::open` merges concurrent operation heads, which writes a new operation.
fn assert_single_op_head(path: &Path) {
    let workspace = DefaultWorkspaceLoaderFactory
        .create(path)
        .expect("load workspace");
    let settings = UserSettings::from_config_and_home_dir(StackedConfig::with_defaults(), None)
        .expect("settings");
    let loader = RepoLoader::init_from_file_system(
        &settings,
        workspace.repo_path(),
        &default_backend_factories(),
    )
    .expect("load repo");
    let heads = pollster::block_on(loader.op_heads_store().get_op_heads()).expect("op heads");
    assert!(
        heads.len() == 1,
        "{} has {} operation heads; opening it would write a merge operation, so profile an isolated copy instead",
        path.display(),
        heads.len()
    );
}

fn timed<T>(label: &str, body: impl FnOnce() -> T) -> T {
    let started = Instant::now();
    let value = body();
    eprintln!(
        "{label:>18}: {:>8.1} ms",
        started.elapsed().as_secs_f64() * 1000.0
    );
    value
}
