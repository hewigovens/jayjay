use std::collections::HashMap;

use jayjay_i18n::FluentArgs;

#[derive(uniffi::Enum)]
pub enum LocalizedArgument {
    Text { value: String },
    Number { value: i64 },
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
