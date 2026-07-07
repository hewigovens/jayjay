// Shared jj test helpers for integration and component tests.

pub mod cmd;
pub mod formats;
pub mod linear;
pub mod repo;

pub use cmd::{
    configure_test_user, init_colocated, json_stdout, run_command, run_git, run_jj, run_jj_in,
};
pub use formats::FormatFixture;
pub use linear::LinearFixture;
pub use repo::{
    current_op_id, init_jj_repo, selection_for_lines, setup_source_change_with_child,
    whole_file_selection,
};
