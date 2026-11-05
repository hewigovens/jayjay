mod dispatch;
mod import;
mod parser;
mod review;
mod workspace;

pub use dispatch::run_app_cli_command;

#[cfg(test)]
pub(crate) fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}
