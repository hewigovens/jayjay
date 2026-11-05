use jayjay_i18n::settings;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum FlockIcon {
    Classic,
    FishOwl,
    GoldenPheasant,
    Kingfisher,
    Flycatcher,
}

impl FlockIcon {
    pub(super) const ALL: [Self; 5] = [
        Self::Classic,
        Self::FishOwl,
        Self::GoldenPheasant,
        Self::Kingfisher,
        Self::Flycatcher,
    ];

    pub(super) fn from_id(id: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|icon| icon.id() == id)
            .unwrap_or(Self::Classic)
    }

    pub(super) fn id(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::FishOwl => "fish-owl",
            Self::GoldenPheasant => "golden-pheasant",
            Self::Kingfisher => "kingfisher",
            Self::Flycatcher => "flycatcher",
        }
    }

    pub(super) fn title(self) -> String {
        match self {
            Self::Classic => settings::flock_icon_classic(),
            Self::FishOwl => settings::flock_icon_fish_owl(),
            Self::GoldenPheasant => settings::flock_icon_golden_pheasant(),
            Self::Kingfisher => settings::flock_icon_kingfisher(),
            Self::Flycatcher => settings::flock_icon_flycatcher(),
        }
    }

    pub(super) fn svgs(self) -> Option<(&'static [u8], &'static [u8])> {
        Some(match self {
            Self::Classic => return None,
            Self::FishOwl => (
                include_bytes!("../../../../../../assets/flock/flock-fish-owl-light.svg"),
                include_bytes!("../../../../../../assets/flock/flock-fish-owl-dark.svg"),
            ),
            Self::GoldenPheasant => (
                include_bytes!("../../../../../../assets/flock/flock-golden-pheasant-light.svg"),
                include_bytes!("../../../../../../assets/flock/flock-golden-pheasant-dark.svg"),
            ),
            Self::Kingfisher => (
                include_bytes!("../../../../../../assets/flock/flock-kingfisher-light.svg"),
                include_bytes!("../../../../../../assets/flock/flock-kingfisher-dark.svg"),
            ),
            Self::Flycatcher => (
                include_bytes!("../../../../../../assets/flock/flock-flycatcher-light.svg"),
                include_bytes!("../../../../../../assets/flock/flock-flycatcher-dark.svg"),
            ),
        })
    }
}
