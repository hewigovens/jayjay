#[cfg(feature = "storage")]
mod filesystem;
#[cfg(not(feature = "storage"))]
mod memory;

#[cfg(feature = "storage")]
pub(super) use filesystem::Persistence;
#[cfg(not(feature = "storage"))]
pub(super) use memory::Persistence;
