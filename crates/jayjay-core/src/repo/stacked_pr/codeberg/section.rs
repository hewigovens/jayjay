pub(super) const STACK_SECTION_START: &str = "<!-- jayjay-stack -->";
pub(super) const STACK_SECTION_END: &str = "<!-- /jayjay-stack -->";

pub(super) fn with_stack_section(description: &str, current: u32, stack: &[u32]) -> String {
    let body = strip_stack_section(description);
    let items = stack
        .iter()
        .map(|number| {
            if *number == current {
                format!("**#{number}**")
            } else {
                format!("#{number}")
            }
        })
        .collect::<Vec<_>>()
        .join(" → ");
    format!("{body}\n\n{STACK_SECTION_START}\n**Stack:** {items}\n{STACK_SECTION_END}")
}

// An update keeps the PR's section, so a stack that fails before the link pass still shows its last links.
pub(super) fn with_existing_section(description: &str, previous_body: &str) -> String {
    let body = strip_stack_section(description);
    match stack_section_range(previous_body) {
        Some(range) => format!("{body}\n\n{}", &previous_body[range]),
        None => body,
    }
}

fn strip_stack_section(description: &str) -> String {
    let Some(range) = stack_section_range(description) else {
        return description.to_owned();
    };
    let stripped = format!(
        "{}{}",
        &description[..range.start],
        &description[range.end..]
    );
    stripped.trim_end().to_owned()
}

fn stack_section_range(body: &str) -> Option<std::ops::Range<usize>> {
    let start = body.find(STACK_SECTION_START)?;
    let end = body[start + STACK_SECTION_START.len()..]
        .find(STACK_SECTION_END)
        .map(|offset| start + STACK_SECTION_START.len() + offset + STACK_SECTION_END.len())?;
    Some(start..end)
}
