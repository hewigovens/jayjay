// Shared jj test helpers for integration and component tests.

pub mod cmd;
pub mod env;
pub mod formats;
pub mod linear;
pub mod repo;
mod template;

pub use cmd::{
    configure_fix_tool, configure_test_user, git_stdout, init_colocated, run_command, run_git,
    run_jj, run_jj_in,
};
pub use env::{review_store_env, scoped_env};
pub use formats::FormatFixture;
pub use linear::LinearFixture;
pub use repo::{
    add_two_group_edit, change_by_description, changed_paths, current_op_id, find_bookmark,
    find_change, first_change, init_jj_repo, selection_for_lines, setup_source_change_with_child,
    setup_stack, whole_file_selection,
};
