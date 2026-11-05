use jayjay_i18n::settings;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FlockIcon {
    Classic,
    FishOwl,
    GoldenPheasant,
    Kingfisher,
    Flycatcher,
}

impl FlockIcon {
    pub const ALL: [Self; 5] = [
        Self::Classic,
        Self::FishOwl,
        Self::GoldenPheasant,
        Self::Kingfisher,
        Self::Flycatcher,
    ];

    pub fn from_id(id: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|icon| icon.id() == id)
            .unwrap_or(Self::Classic)
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::FishOwl => "fish-owl",
            Self::GoldenPheasant => "golden-pheasant",
            Self::Kingfisher => "kingfisher",
            Self::Flycatcher => "flycatcher",
        }
    }

    pub fn title(self) -> String {
        match self {
            Self::Classic => settings::flock_icon_classic(),
            Self::FishOwl => settings::flock_icon_fish_owl(),
            Self::GoldenPheasant => settings::flock_icon_golden_pheasant(),
            Self::Kingfisher => settings::flock_icon_kingfisher(),
            Self::Flycatcher => settings::flock_icon_flycatcher(),
        }
    }
}
