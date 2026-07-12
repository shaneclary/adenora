use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    /// Albanian (Latin script) — Kosovo, Albania, North Macedonia
    Sq,
    /// Macedonian (Cyrillic script) — North Macedonia
    Mk,
    /// Serbian (Latin + Cyrillic) — Serbia, Kosovo minorities
    Sr,
    /// English — international, admin, diaspora
    #[default]
    En,
    /// Turkish — Kosovo/NMK minorities
    Tr,
    /// Bosnian (Latin) — Bosnia
    Bs,
    /// Croatian (Latin) — Croatia
    Hr,
    /// Montenegrin (Latin) — Montenegro
    Cnr,
    /// Bulgarian (Cyrillic) — Bulgaria
    Bg,
    /// Romanian (Latin) — Romania
    Ro,
    /// Hungarian (Latin) — Hungary
    Hu,
    /// Greek (Greek script) — Greece
    El,
    /// Russian (Cyrillic)
    Ru,
    /// Spanish (Latin) — Mexico, Honduras, Nicaragua
    Es,
    /// French (Latin) — African francophone
    Fr,
    /// Portuguese (Latin) — Angola
    Pt,
    /// Filipino/Tagalog (Latin) — Philippines
    Tl,
    /// Khmer (Khmer script) — Cambodia
    Km,
    /// Lao (Lao script) — Laos
    Lo,
    /// Vietnamese (Latin + diacritics) — Vietnam
    Vi,
    /// Chinese Simplified — China
    Zh,
}

impl Locale {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Sq => "sq",
            Self::Mk => "mk",
            Self::Sr => "sr",
            Self::En => "en",
            Self::Tr => "tr",
            Self::Bs => "bs",
            Self::Hr => "hr",
            Self::Cnr => "cnr",
            Self::Bg => "bg",
            Self::Ro => "ro",
            Self::Hu => "hu",
            Self::El => "el",
            Self::Ru => "ru",
            Self::Es => "es",
            Self::Fr => "fr",
            Self::Pt => "pt",
            Self::Tl => "tl",
            Self::Km => "km",
            Self::Lo => "lo",
            Self::Vi => "vi",
            Self::Zh => "zh",
        }
    }

    pub fn name_native(&self) -> &'static str {
        match self {
            Self::Sq => "Shqip",
            Self::Mk => "\u{041c}\u{0430}\u{043a}\u{0435}\u{0434}\u{043e}\u{043d}\u{0441}\u{043a}\u{0438}",
            Self::Sr => "\u{0421}\u{0440}\u{043f}\u{0441}\u{043a}\u{0438} / Srpski",
            Self::En => "English",
            Self::Tr => "T\u{00fc}rk\u{00e7}e",
            Self::Bs => "Bosanski",
            Self::Hr => "Hrvatski",
            Self::Cnr => "Crnogorski",
            Self::Bg => "\u{0411}\u{044a}\u{043b}\u{0433}\u{0430}\u{0440}\u{0441}\u{043a}\u{0438}",
            Self::Ro => "Rom\u{00e2}n\u{0103}",
            Self::Hu => "Magyar",
            Self::El => "\u{0395}\u{03bb}\u{03bb}\u{03b7}\u{03bd}\u{03b9}\u{03ba}\u{03ac}",
            Self::Ru => "\u{0420}\u{0443}\u{0441}\u{0441}\u{043a}\u{0438}\u{0439}",
            Self::Es => "Espa\u{00f1}ol",
            Self::Fr => "Fran\u{00e7}ais",
            Self::Pt => "Portugu\u{00ea}s",
            Self::Tl => "Filipino",
            Self::Km => "\u{1797}\u{17b6}\u{179f}\u{17b6}\u{1781}\u{17d2}\u{1798}\u{17c2}\u{179a}",
            Self::Lo => "\u{0e9e}\u{0eb2}\u{0eaa}\u{0eb2}\u{0ea5}\u{0eb2}\u{0ea7}",
            Self::Vi => "Ti\u{1ebf}ng Vi\u{1ec7}t",
            Self::Zh => "\u{4e2d}\u{6587}",
        }
    }

    pub fn uses_rtl(&self) -> bool {
        false // None of our supported languages are RTL
    }

    pub fn script(&self) -> Script {
        match self {
            Self::Mk | Self::Sr | Self::Bg | Self::Ru => Script::Cyrillic,
            Self::El => Script::Greek,
            Self::Km => Script::Khmer,
            Self::Lo => Script::Lao,
            Self::Zh => Script::Han,
            _ => Script::Latin,
        }
    }

    /// Launch locales — phase 1
    pub fn is_launch_locale(&self) -> bool {
        matches!(self, Self::Sq | Self::Mk | Self::Sr | Self::En)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Script {
    Latin,
    Cyrillic,
    Greek,
    Khmer,
    Lao,
    Han,
}

/// Translation dictionary: key -> translated string.
/// Loaded from JSON files per locale.
pub type Translations = HashMap<String, String>;

pub fn fallback_chain(locale: Locale) -> Vec<Locale> {
    match locale {
        // BCMS mutual intelligibility: fall back through each other before English
        Locale::Bs | Locale::Hr | Locale::Cnr => vec![locale, Locale::Sr, Locale::En],
        Locale::Sr => vec![Locale::Sr, Locale::En],
        // Everyone else falls back to English
        _ => vec![locale, Locale::En],
    }
}
