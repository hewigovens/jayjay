use gpui::App;
use jayjay_core::CliStatus;

use crate::ui::logo::Logo;

pub(crate) struct OnboardingState {
    pub(crate) page: OnboardingPage,
    pub(crate) jj: JjCheckState,
    pub(crate) logo: Logo,
}

impl OnboardingState {
    pub(crate) fn new(cx: &mut App) -> Self {
        Self {
            page: OnboardingPage::Welcome,
            jj: JjCheckState::Checking,
            logo: Logo::load(cx),
        }
    }
}

#[derive(Clone)]
pub(crate) enum JjCheckState {
    Checking,
    Loaded(CliStatus),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OnboardingPage {
    Welcome,
    JjCheck,
    Ready,
}

impl OnboardingPage {
    pub(crate) fn previous(self) -> Option<Self> {
        match self {
            Self::Welcome => None,
            Self::JjCheck => Some(Self::Welcome),
            Self::Ready => Some(Self::JjCheck),
        }
    }

    pub(crate) fn next(self) -> Option<Self> {
        match self {
            Self::Welcome => Some(Self::JjCheck),
            Self::JjCheck => Some(Self::Ready),
            Self::Ready => None,
        }
    }

    pub(crate) fn all() -> [Self; 3] {
        [Self::Welcome, Self::JjCheck, Self::Ready]
    }

    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::Welcome => "welcome",
            Self::JjCheck => "jj-check",
            Self::Ready => "ready",
        }
    }
}
