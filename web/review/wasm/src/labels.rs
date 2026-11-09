use serde_json::{Map, Value};

pub(super) fn labels(languages: &str) -> Value {
    jayjay_i18n::set_preferred_languages(&languages.split(',').collect::<Vec<_>>());
    Value::Object(
        settings_labels()
            .into_iter()
            .chain(diff_labels())
            .chain(review_labels())
            .chain(file_list_labels())
            .chain(pull_labels())
            .map(|(key, value)| (key.to_owned(), Value::String(value)))
            .chain(error_labels())
            .collect::<Map<_, _>>(),
    )
}

fn settings_labels() -> Vec<(&'static str, String)> {
    vec![
        ("close", jayjay_i18n::common::close()),
        ("fontSection", jayjay_i18n::web::font_section()),
        ("fontInterface", jayjay_i18n::web::font_interface()),
        ("increaseFontSize", jayjay_i18n::web::increase_font_size()),
        ("decreaseFontSize", jayjay_i18n::web::decrease_font_size()),
        ("fontDiff", jayjay_i18n::web::font_diff()),
        ("appearance", jayjay_i18n::settings::page_appearance()),
        (
            "appearanceSystem",
            jayjay_i18n::settings::appearance_system(),
        ),
        ("appearanceLight", jayjay_i18n::settings::appearance_light()),
        ("appearanceDark", jayjay_i18n::settings::appearance_dark()),
        ("lightTheme", jayjay_i18n::settings::light_theme()),
        ("darkTheme", jayjay_i18n::settings::dark_theme()),
        ("jayjayLight", jayjay_i18n::theme::jayjay_light()),
        ("jayjayDark", jayjay_i18n::theme::jayjay_dark()),
        ("syntaxUnavailable", jayjay_i18n::web::syntax_unavailable()),
    ]
}

fn diff_labels() -> Vec<(&'static str, String)> {
    vec![
        ("allFiles", jayjay_i18n::web::all_files()),
        ("undoImport", jayjay_i18n::web::undo_import()),
        ("result", jayjay_i18n::web::result()),
        ("ignoreWhitespace", jayjay_i18n::web::ignore_whitespace()),
        ("showSource", jayjay_i18n::web::show_source()),
        ("allLines", jayjay_i18n::web::all_lines()),
        ("eof", jayjay_i18n::web::eof()),
        ("empty", jayjay_i18n::web::empty()),
        ("emptyFile", jayjay_i18n::web::empty_file()),
        ("noTextChanges", jayjay_i18n::web::no_text_changes()),
        ("showRest", jayjay_i18n::web::show_rest()),
        ("whitespaceOnly", jayjay_i18n::web::whitespace_only()),
        ("lines", jayjay_i18n::web::lines()),
        ("renderError", jayjay_i18n::web::render_error()),
        ("workerError", jayjay_i18n::web::worker_error()),
        ("loading", jayjay_i18n::web::loading()),
        ("engineError", jayjay_i18n::web::engine_error()),
        ("private", jayjay_i18n::web::privacy()),
        ("before", jayjay_i18n::diff::before()),
        ("after", jayjay_i18n::diff::after()),
        ("unified", jayjay_i18n::diff::unified()),
        ("showAll", jayjay_i18n::diff::show_all()),
        ("added", jayjay_i18n::diff::added()),
        ("removed", jayjay_i18n::diff::removed()),
    ]
}

fn review_labels() -> Vec<(&'static str, String)> {
    vec![
        ("files", jayjay_i18n::web::files()),
        ("resizeFiles", jayjay_i18n::web::resize_files()),
        ("filterFiles", jayjay_i18n::web::filter_files()),
        ("shortcuts", jayjay_i18n::web::shortcuts()),
        ("shortcutMark", jayjay_i18n::web::shortcut_mark()),
        ("shortcutNext", jayjay_i18n::web::shortcut_next()),
        ("shortcutPrevious", jayjay_i18n::web::shortcut_previous()),
        (
            "shortcutNextUnreviewed",
            jayjay_i18n::web::shortcut_next_unreviewed(),
        ),
        (
            "shortcutPreviousUnreviewed",
            jayjay_i18n::web::shortcut_previous_unreviewed(),
        ),
        ("hideReviewed", jayjay_i18n::web::hide_reviewed()),
        ("onlyNotes", jayjay_i18n::web::only_notes()),
        ("importChange", jayjay_i18n::web::import_change()),
        ("exportReview", jayjay_i18n::web::export_review()),
        ("importReview", jayjay_i18n::web::import_review()),
        ("reviewed", jayjay_i18n::web::reviewed()),
        ("unreviewed", jayjay_i18n::web::unreviewed()),
        ("partial", jayjay_i18n::web::partial()),
        (
            "changed_since_review",
            jayjay_i18n::web::changed_since_review(),
        ),
        ("markReviewed", jayjay_i18n::web::mark_reviewed()),
        ("group", jayjay_i18n::web::group()),
        ("notes", jayjay_i18n::web::notes()),
        ("editNote", jayjay_i18n::menu::edit_review_note()),
        ("addNote", jayjay_i18n::web::add_note()),
        ("noteBody", jayjay_i18n::web::note_body()),
        ("saveNote", jayjay_i18n::web::save_note()),
        ("cancel", jayjay_i18n::web::cancel()),
        ("resolve", jayjay_i18n::web::resolve()),
        ("current", jayjay_i18n::web::current()),
        ("stale", jayjay_i18n::web::stale()),
        ("orphaned", jayjay_i18n::web::orphaned()),
        ("resolved", jayjay_i18n::web::resolved()),
        ("loadingFile", jayjay_i18n::web::loading_file()),
        ("savedLocally", jayjay_i18n::web::saved_locally()),
        ("retry", jayjay_i18n::web::retry()),
        ("localData", jayjay_i18n::web::local_data()),
        ("loadAvatars", jayjay_i18n::web::load_avatars()),
        ("importReplaces", jayjay_i18n::web::import_replaces()),
    ]
}

fn file_list_labels() -> Vec<(&'static str, String)> {
    vec![
        ("kind.added", jayjay_i18n::diff::added()),
        ("kind.modified", jayjay_i18n::repo::modified()),
        ("kind.removed", jayjay_i18n::diff::removed()),
        ("kind.renamed", jayjay_i18n::diff::renamed()),
        ("kind.copied", jayjay_i18n::diff::copied()),
        ("sideBySide", jayjay_i18n::diff::side_by_side()),
        (
            "switchToSideBySide",
            jayjay_i18n::diff::switch_to_side_by_side(),
        ),
        ("switchToUnified", jayjay_i18n::diff::switch_to_unified()),
        ("copyPath", jayjay_i18n::diff::copy_path()),
        ("expandDescription", jayjay_i18n::repo::expand_description()),
        (
            "collapseDescription",
            jayjay_i18n::repo::collapse_description(),
        ),
        ("showingTree", jayjay_i18n::repo::showing_files_as_a_tree()),
        (
            "showingFlat",
            jayjay_i18n::repo::showing_files_as_a_flat_list(),
        ),
        ("hideReviewedFiles", jayjay_i18n::settings::hide_reviewed()),
        (
            "showingUnreviewed",
            jayjay_i18n::repo::showing_only_unreviewed_files(),
        ),
        ("noMatchingFiles", jayjay_i18n::repo::no_matching_files()),
        ("allFilesHidden", jayjay_i18n::repo::all_files_hidden()),
        ("noChangedFiles", jayjay_i18n::repo::no_files()),
        (
            "previewUnavailable",
            jayjay_i18n::diff::preview_unavailable(),
        ),
    ]
}

/// Strings for `web/diff`, the GitHub pull request viewer.
fn pull_labels() -> Vec<(&'static str, String)> {
    vec![
        ("pullHeading", jayjay_i18n::web::pull_heading()),
        ("pullLedeLead", jayjay_i18n::web::pull_lede_lead()),
        ("pullLede", jayjay_i18n::web::pull_lede()),
        ("pullFeatureHide", jayjay_i18n::web::pull_feature_hide()),
        (
            "pullFeatureHideDetail",
            jayjay_i18n::web::pull_feature_hide_detail(),
        ),
        ("pullFeatureTree", jayjay_i18n::web::pull_feature_tree()),
        (
            "pullFeatureTreeDetail",
            jayjay_i18n::web::pull_feature_tree_detail(),
        ),
        ("pullUrl", jayjay_i18n::web::pull_url()),
        ("pullOpen", jayjay_i18n::web::pull_open()),
        ("pullExtension", jayjay_i18n::web::pull_extension()),
        (
            "pullExtensionDetail",
            jayjay_i18n::web::pull_extension_detail(),
        ),
        ("pullInvalid", jayjay_i18n::web::pull_invalid()),
        ("pullUnsupported", jayjay_i18n::web::pull_unsupported()),
        ("pullSupported", jayjay_i18n::web::pull_supported()),
        ("pullTwoDot", jayjay_i18n::web::pull_two_dot()),
        ("pullOnGitHub", jayjay_i18n::web::pull_on_github()),
        ("pullSignIn", jayjay_i18n::web::pull_sign_in()),
        ("pullDeviceCode", jayjay_i18n::web::pull_device_code()),
        ("pullCopyOpen", jayjay_i18n::web::pull_copy_open()),
        ("pullWaiting", jayjay_i18n::web::pull_waiting()),
        ("pullSignInFailed", jayjay_i18n::web::pull_sign_in_failed()),
        ("pullSignOut", jayjay_i18n::web::pull_sign_out()),
        (
            "pullError.unauthorized",
            jayjay_i18n::web::pull_unauthorized(),
        ),
        ("pullRefresh", jayjay_i18n::web::pull_refresh()),
        ("pullShareNotes", jayjay_i18n::web::pull_share_notes()),
        ("pullShareCopied", jayjay_i18n::web::pull_share_copied()),
        ("pullShareLink", jayjay_i18n::web::pull_share_link()),
        (
            "pullShareCopyFailed",
            jayjay_i18n::web::pull_share_copy_failed(),
        ),
        ("pullShareFailed", jayjay_i18n::web::pull_share_failed()),
        ("pullSamples", jayjay_i18n::web::pull_samples()),
        ("pullHow", jayjay_i18n::web::pull_how()),
        ("pullHowMove", jayjay_i18n::web::pull_how_move()),
        (
            "pullHowMoveDetail",
            jayjay_i18n::web::pull_how_move_detail(),
        ),
        ("pullHowMark", jayjay_i18n::web::pull_how_mark()),
        (
            "pullHowMarkDetail",
            jayjay_i18n::web::pull_how_mark_detail(),
        ),
        ("pullFeatureStacks", jayjay_i18n::web::pull_feature_stacks()),
        (
            "pullFeatureStacksDetail",
            jayjay_i18n::web::pull_feature_stacks_detail(),
        ),
        ("pullFeatureSync", jayjay_i18n::web::pull_feature_sync()),
        (
            "pullFeatureSyncDetail",
            jayjay_i18n::web::pull_feature_sync_detail(),
        ),
        ("pullSource", jayjay_i18n::web::pull_source()),
        ("pullFooterLead", jayjay_i18n::web::pull_footer_lead()),
        ("pullFooter", jayjay_i18n::web::pull_footer()),
        ("pullCommits", jayjay_i18n::web::pull_commits()),
        ("pullAllCommits", jayjay_i18n::web::pull_all_commits()),
        ("pullStack", jayjay_i18n::web::pull_stack()),
        ("pullMerged", jayjay_i18n::web::pull_merged()),
        ("pullClosed", jayjay_i18n::web::pull_closed()),
        ("pullSyncViewed", jayjay_i18n::web::pull_sync_viewed()),
        (
            "pullSyncViewedScope",
            jayjay_i18n::web::pull_sync_viewed_scope(),
        ),
        ("pullSyncingViewed", jayjay_i18n::web::pull_syncing_viewed()),
        (
            "pullSyncViewedFailed",
            jayjay_i18n::web::pull_sync_viewed_failed(),
        ),
        (
            "pullError.rate-limited",
            jayjay_i18n::web::pull_rate_limited(),
        ),
    ]
}

fn error_labels() -> impl Iterator<Item = (String, Value)> {
    ["pull", "compare", "commit"].into_iter().flat_map(|kind| {
        [
            ("not-found", jayjay_i18n::web::pull_not_found(kind)),
            ("too-large", jayjay_i18n::web::pull_too_large(kind)),
            ("unavailable", jayjay_i18n::web::pull_unavailable(kind)),
        ]
        .map(|(code, text)| (format!("{kind}Error.{code}"), Value::String(text)))
    })
}
