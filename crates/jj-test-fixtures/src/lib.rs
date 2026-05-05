// Shared jj fixture builders for integration tests across the workspace.
// Used by jayjay-gpui's component tests and (eventually) jayjay-core's
// real_jj_repo.rs.

pub mod cmd;
pub mod linear;

pub use cmd::run_jj_in;
pub use linear::LinearFixture;
