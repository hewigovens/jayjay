//! Headless CLI surface for the GPUI shell binary: `--version`, `config`, and `review ...`.

mod dispatch;
mod parser;
mod review;

pub use dispatch::run_and_exit_if_needed;
