mod package;
#[cfg(target_arch = "wasm32")]
mod route;
mod search;

pub(crate) use package::Package;
#[cfg(target_arch = "wasm32")]
pub(crate) use route::serve;
pub(crate) use search::SearchRequest;

#[cfg(test)]
mod tests;
