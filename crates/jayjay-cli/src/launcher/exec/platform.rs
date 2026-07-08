#[cfg(unix)]
mod unix;

#[cfg(windows)]
mod windows;

#[cfg(unix)]
pub use unix::exec;

#[cfg(windows)]
pub use windows::exec;
