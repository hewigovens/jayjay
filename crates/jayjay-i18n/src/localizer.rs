use std::sync::{LazyLock, PoisonError, RwLock};

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource};
use fluent_langneg::{NegotiationStrategy, negotiate_languages};
use unic_langid::LanguageIdentifier;

use crate::messages::LOCALES;
use crate::number::format_number;

const SOURCE_LOCALE: &str = "en-US";
const PSEUDO_LOCALE: &str = "en-XA";

static CURRENT: LazyLock<RwLock<Localizer>> =
    LazyLock::new(|| RwLock::new(Localizer::negotiate(&[] as &[&str])));

pub fn set_preferred_languages<S: AsRef<str>>(languages: &[S]) {
    let localizer = Localizer::negotiate(languages);
    *CURRENT.write().unwrap_or_else(PoisonError::into_inner) = localizer;
}

pub fn message(id: &str, args: Option<&FluentArgs>) -> String {
    CURRENT
        .read()
        .unwrap_or_else(PoisonError::into_inner)
        .format(id, args)
}

pub(crate) struct Localizer {
    bundles: Vec<FluentBundle<FluentResource>>,
}

impl Localizer {
    pub(crate) fn negotiate<S: AsRef<str>>(languages: &[S]) -> Self {
        if languages
            .first()
            .is_some_and(|first| first.as_ref() == PSEUDO_LOCALE)
        {
            let mut bundle = bundle(PSEUDO_LOCALE, SOURCE_LOCALE);
            bundle.set_transform(Some(|text| fluent_pseudo::transform(text, false, true)));
            return Self {
                bundles: vec![bundle],
            };
        }
        let requested: Vec<LanguageIdentifier> = languages
            .iter()
            .filter_map(|language| language.as_ref().parse().ok())
            .collect();
        let available: Vec<LanguageIdentifier> = LOCALES
            .iter()
            .map(|(locale, _)| locale.parse().unwrap())
            .collect();
        let default: LanguageIdentifier = SOURCE_LOCALE.parse().unwrap();
        let mut chosen: Vec<String> = negotiate_languages(
            &requested,
            &available,
            Some(&default),
            NegotiationStrategy::Filtering,
        )
        .into_iter()
        .filter(|locale| {
            requested
                .iter()
                .any(|language| same_script(language, locale))
        })
        .map(ToString::to_string)
        .collect();
        chosen.retain(|locale| locale != SOURCE_LOCALE);
        chosen.push(SOURCE_LOCALE.to_owned());
        Self {
            bundles: chosen.iter().map(|locale| bundle(locale, locale)).collect(),
        }
    }

    pub(crate) fn format(&self, id: &str, args: Option<&FluentArgs>) -> String {
        for bundle in &self.bundles {
            if let Some(pattern) = bundle.get_message(id).and_then(|message| message.value()) {
                return bundle
                    .format_pattern(pattern, args, &mut Vec::new())
                    .into_owned();
            }
        }
        id.to_owned()
    }
}

/// Plain negotiation would give `zh-TW` the Simplified catalog.
fn same_script(requested: &LanguageIdentifier, available: &LanguageIdentifier) -> bool {
    let (mut requested, mut available) = (requested.clone(), available.clone());
    requested.maximize();
    available.maximize();
    requested.language == available.language && requested.script == available.script
}

fn bundle(locale: &str, catalog: &str) -> FluentBundle<FluentResource> {
    let mut bundle = FluentBundle::new_concurrent(vec![locale.parse().unwrap()]);
    bundle.set_use_isolating(false);
    bundle.set_formatter(Some(format_number));
    let (_, sources) = LOCALES.iter().find(|(name, _)| *name == catalog).unwrap();
    for source in *sources {
        let resource = FluentResource::try_new((*source).to_owned()).unwrap();
        bundle.add_resource(resource).unwrap();
    }
    bundle
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_plurals_and_groups_thousands() {
        assert_eq!(
            crate::settings::review_summary(1, 1234),
            "1 mark, 1,234 notes"
        );
        assert_eq!(crate::settings::review_summary(0, 2), "0 marks, 2 notes");
        crate::set_number_locale("de_DE.UTF-8");
        assert_eq!(
            crate::settings::review_summary(1, 1234),
            "1 mark, 1.234 notes"
        );
        crate::set_number_locale("");
    }

    #[test]
    fn negotiates_with_english_fallback() {
        assert_eq!(
            Localizer::negotiate(&["fr-FR"]).format("common-cancel", None),
            "Cancel"
        );
        assert_eq!(
            Localizer::negotiate(&["en-GB", "en-XA"]).format("common-cancel", None),
            "Cancel"
        );
        assert_eq!(
            Localizer::negotiate(&["en-XA"]).format("common-cancel", None),
            "Ƈaaƞƈeeŀ"
        );
        assert_eq!(
            Localizer::negotiate(&["en-US"]).format("missing-id", None),
            "missing-id"
        );
    }
}
