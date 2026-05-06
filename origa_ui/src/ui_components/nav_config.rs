#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NavRoute {
    Home,
    Words,
    Grammar,
    Kanji,
    Phrases,
    Profile,
}

impl NavRoute {
    pub const fn href(self) -> &'static str {
        match self {
            Self::Home => "/home",
            Self::Words => "/words",
            Self::Grammar => "/grammar",
            Self::Kanji => "/kanji",
            Self::Phrases => "/phrases",
            Self::Profile => "/profile",
        }
    }

    pub const fn icon(self) -> icondata::Icon {
        match self {
            Self::Home => icondata::LuHouse,
            Self::Words => icondata::LuLanguages,
            Self::Grammar => icondata::LuPencilLine,
            Self::Kanji => icondata::LuBookOpen,
            Self::Phrases => icondata::LuMessageSquare,
            Self::Profile => icondata::LuUser,
        }
    }

    pub const fn use_logo(self) -> bool {
        matches!(self, Self::Home)
    }

    pub fn label(self, i18n: &leptos_i18n::I18nContext<crate::i18n::Locale>) -> String {
        let keys = i18n.get_keys();
        match self {
            Self::Home => keys.home().home_tab().inner().to_string(),
            Self::Words => keys.home().words().inner().to_string(),
            Self::Grammar => keys.home().grammar().inner().to_string(),
            Self::Kanji => keys.home().kanji().inner().to_string(),
            Self::Phrases => keys.home().phrases().inner().to_string(),
            Self::Profile => keys.home().profile().inner().to_string(),
        }
    }

    pub fn is_active(self, path: &str) -> bool {
        match self {
            Self::Home => path.starts_with("/home") || path == "/" || path.is_empty(),
            Self::Words => path.starts_with("/words") || path.starts_with("/sets"),
            Self::Grammar => path.starts_with("/grammar"),
            Self::Kanji => path.starts_with("/kanji"),
            Self::Phrases => path.starts_with("/phrases"),
            Self::Profile => path.starts_with("/profile"),
        }
    }

    pub const fn all() -> &'static [NavRoute; 6] {
        &[
            Self::Home,
            Self::Words,
            Self::Grammar,
            Self::Kanji,
            Self::Phrases,
            Self::Profile,
        ]
    }

    pub const fn sidebar_routes() -> &'static [NavRoute; 5] {
        &[
            Self::Home,
            Self::Words,
            Self::Grammar,
            Self::Kanji,
            Self::Phrases,
        ]
    }

    pub const fn test_id_suffix(self) -> &'static str {
        match self {
            Self::Home => "tab-home",
            Self::Words => "tab-words",
            Self::Grammar => "tab-grammar",
            Self::Kanji => "tab-kanji",
            Self::Phrases => "tab-phrases",
            Self::Profile => "tab-profile",
        }
    }
}
