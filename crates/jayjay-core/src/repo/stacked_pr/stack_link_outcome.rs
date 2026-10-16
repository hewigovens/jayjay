pub(super) enum StackLinkOutcome {
    Linked(String),
    Fallback(String),
}

impl StackLinkOutcome {
    pub(super) fn is_linked(&self) -> bool {
        matches!(self, Self::Linked(_))
    }

    pub(super) fn into_message(self) -> String {
        match self {
            Self::Linked(message) | Self::Fallback(message) => message,
        }
    }
}
