mod codeberg;
mod cursor;
mod detect;
mod forge;
mod github;
mod gitlab;
mod naming;
mod stack_forge;
mod stack_link_outcome;
mod submit;
mod validation;

pub use naming::branch_name_slug;

#[cfg(test)]
mod tests;
