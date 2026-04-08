use origa::domain::NativeLanguage;

#[allow(
    clippy::module_inception,
    clippy::incompatible_msrv,
    unused_imports,
    dead_code
)]
mod generated {
    include!(concat!(env!("OUT_DIR"), "/i18n/mod.rs"));
}

pub use generated::i18n::{I18nContextProvider, Locale, use_i18n};
pub use leptos_i18n::{I18nContext, t, td_string};

pub fn native_language_to_locale(lang: &NativeLanguage) -> Locale {
    match lang {
        NativeLanguage::English => Locale::en,
        NativeLanguage::Russian => Locale::ru,
    }
}

pub fn locale_to_native_language(locale: &Locale) -> NativeLanguage {
    match locale {
        Locale::en => NativeLanguage::English,
        Locale::ru => NativeLanguage::Russian,
    }
}
