use std::collections::HashMap;

use jayjay_i18n::FluentArgs;

#[derive(uniffi::Enum)]
pub enum LocalizedArgument {
    Text { value: String },
    Number { value: i64 },
}

#[derive(uniffi::Record)]
pub struct Language {
    pub tag: String,
    pub name: String,
}

#[uniffi::export]
fn available_languages() -> Vec<Language> {
    jayjay_i18n::Language::all()
        .map(|language| Language {
            tag: language.tag.to_owned(),
            name: language.name.to_owned(),
        })
        .collect()
}

#[uniffi::export]
fn set_preferred_languages(languages: Vec<String>) {
    jayjay_i18n::set_preferred_languages(&languages);
}

#[uniffi::export]
fn set_number_locale(locale: String) {
    jayjay_i18n::set_number_locale(&locale);
}

#[uniffi::export]
fn localized_message(id: String, args: HashMap<String, LocalizedArgument>) -> String {
    let mut fluent = FluentArgs::new();
    for (name, arg) in args {
        match arg {
            LocalizedArgument::Text { value } => fluent.set(name, value),
            LocalizedArgument::Number { value } => fluent.set(name, value),
        }
    }
    jayjay_i18n::message(&id, Some(&fluent))
}
