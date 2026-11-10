use std::process::Command;

use crate::repo::{home_dir, is_executable_file};
use crate::tools::launcher::detach_stdio;

const BUNDLE_ID: &str = "com.superlogical.rex";
/// The app unpacks its server and CLI here whatever the app bundle is called.
const BUNDLED_CLI: &str = "Library/Application Support/com.superlogical.rex/RexServer/Versions/Current/Rex Server.app/Contents/MacOS/rex";

/// The folder-open handler adds a tab but cannot launch a command.
pub(crate) fn open(cwd: &str) -> bool {
    open_app(&[cwd])
}

pub(super) fn open_app(arguments: &[&str]) -> bool {
    detach_stdio(
        Command::new("/usr/bin/open")
            .args(["-b", BUNDLE_ID])
            .args(arguments),
    )
    .spawn()
    .is_ok()
}

pub(super) fn bundled_cli() -> Option<String> {
    let bundled = home_dir()?.join(BUNDLED_CLI);
    is_executable_file(&bundled).then(|| bundled.to_string_lossy().into_owned())
}
