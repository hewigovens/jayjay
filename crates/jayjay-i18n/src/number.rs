use std::sync::{PoisonError, RwLock};

use fluent_bundle::FluentValue;
use fluent_bundle::memoizer::MemoizerKind;
use icu_decimal::DecimalFormatter;
use icu_decimal::input::Decimal;
use icu_locale_core::Locale;

static NUMBER_LOCALE: RwLock<String> = RwLock::new(String::new());

/// Numbers follow the region, not the UI language: English text, German `1.234`.
pub fn set_number_locale(locale: &str) {
    *NUMBER_LOCALE
        .write()
        .unwrap_or_else(PoisonError::into_inner) = locale.to_owned();
}

pub fn decimal_formatter(locale: &str) -> DecimalFormatter {
    let (identifier, keywords) = locale.split_once('@').unwrap_or((locale, ""));
    let mut identifier = identifier
        .split('.')
        .next()
        .unwrap_or_default()
        .replace('_', "-");
    if let Some(numbering) = keywords
        .split(';')
        .find_map(|keyword| keyword.strip_prefix("numbers="))
    {
        identifier.push_str("-u-nu-");
        identifier.push_str(numbering);
    }
    let locale = identifier.parse::<Locale>().unwrap_or(Locale::UNKNOWN);
    DecimalFormatter::try_new(locale.into(), Default::default())
        .or_else(|_| DecimalFormatter::try_new(Default::default(), Default::default()))
        .expect("compiled decimal data includes the default locale")
}

pub(crate) fn format_number<M: MemoizerKind>(value: &FluentValue, _: &M) -> Option<String> {
    let FluentValue::Number(number) = value else {
        return None;
    };
    (number.value.fract() == 0.0).then(|| {
        let locale = NUMBER_LOCALE.read().unwrap_or_else(PoisonError::into_inner);
        decimal_formatter(&locale).format_to_string(&Decimal::from(number.value as i64))
    })
}
