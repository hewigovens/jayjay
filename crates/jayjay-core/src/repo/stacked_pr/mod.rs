mod cursor;
mod detect;
mod forge;
mod github;
mod gitlab;
mod naming;
mod native_stack_outcome;
mod submit;
mod validation;

pub use naming::branch_name_slug;

#[cfg(test)]
mod tests;
