mod group_coverage;
mod range_mapping;

pub(crate) use group_coverage::selection_covers_whole_change_group;
pub(crate) use range_mapping::display_range_to_diff_edit_range;

#[cfg(test)]
mod tests;
