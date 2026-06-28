use super::{app, path};

pub(crate) fn launch_app(repo: Option<String>, path: Option<String>) {
    let repo_path = path::repo_path(repo, path);

    if let Some(bundle) = app::running_app_bundle() {
        app::open_running(repo_path.as_deref(), &bundle);
        return;
    }

    let Some(app_path) = app::find_app() else {
        eprintln!("error: JayJay.app not found");
        eprintln!("Install it to /Applications or build with: just build");
        std::process::exit(1);
    };

    match app::open_app(&app_path, repo_path.as_deref()) {
        Ok(status) if status.success() => {}
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(error) => {
            eprintln!("error: failed to launch JayJay: {error}");
            std::process::exit(1);
        }
    }
}
