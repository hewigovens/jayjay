use std::sync::{LazyLock, PoisonError, RwLock};

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource};
use fluent_langneg::{NegotiationStrategy, negotiate_languages};
use unic_langid::LanguageIdentifier;

use crate::messages::{LOCALES, SOURCE_LOCALE};
use crate::number::format_number;

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
            .map(|(language, _)| language.tag.parse().unwrap())
            .collect();
        Self {
            bundles: chosen_locales(&requested, &available)
                .iter()
                .map(|locale| bundle(locale, locale))
                .collect(),
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

fn chosen_locales(
    requested: &[LanguageIdentifier],
    available: &[LanguageIdentifier],
) -> Vec<String> {
    let mut chosen: Vec<String> = Vec::new();
    for language in requested {
        let compatible: Vec<LanguageIdentifier> = available
            .iter()
            .filter(|locale| same_script(language, locale))
            .cloned()
            .collect();
        for locale in negotiate_languages(
            std::slice::from_ref(language),
            &compatible,
            None,
            NegotiationStrategy::Filtering,
        ) {
            let locale = locale.to_string();
            if !chosen.contains(&locale) {
                chosen.push(locale);
            }
        }
    }
    if !chosen.iter().any(|locale| locale == SOURCE_LOCALE) {
        chosen.push(SOURCE_LOCALE.to_owned());
    }
    chosen
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
    bundle.add_builtins().unwrap();
    let (_, sources) = LOCALES
        .iter()
        .find(|(language, _)| language.tag == catalog)
        .unwrap();
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
        assert_eq!(crate::settings::font_points(14), "14 pt");
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
        assert_eq!(
            Localizer::negotiate(&["ja-JP"]).format("common-cancel", None),
            "キャンセル"
        );
    }

    #[test]
    fn keeps_english_where_the_user_ranked_it() {
        let ids = |tags: &[&str]| -> Vec<LanguageIdentifier> {
            tags.iter().map(|tag| tag.parse().unwrap()).collect()
        };
        let available = ids(&["en-US", "fr", "zh-Hans"]);
        assert_eq!(
            chosen_locales(&ids(&["en-US", "fr-FR"]), &available),
            ["en-US", "fr"]
        );
        assert_eq!(
            chosen_locales(&ids(&["fr-FR", "en-US"]), &available),
            ["fr", "en-US"]
        );
        assert_eq!(
            chosen_locales(&ids(&["fr-FR"]), &available),
            ["fr", "en-US"]
        );
        assert_eq!(chosen_locales(&ids(&["zh-TW"]), &available), ["en-US"]);
        assert_eq!(
            chosen_locales(&ids(&["zh-TW", "en-US", "zh-CN"]), &available),
            ["en-US", "zh-Hans"]
        );
    }
}
