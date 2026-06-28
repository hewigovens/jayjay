pub use CoreError as Error;
pub use jayjay_primitives::JayJayError as CoreError;

pub type CoreResult<T> = Result<T, Error>;
