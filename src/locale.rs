use serde::{Serialize, Deserialize};

/// Supported locales for error and validation messages.
///
/// Used with `rust-i18n` to select the appropriate translation
/// from the `locales/` YAML files.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum Locale {
    /// Brazilian Portuguese (pt-BR).
    PtBr,
    /// English (en).
    En,
}

impl Locale {
    /// Returns the locale identifier string used by `rust-i18n`.
    pub fn as_str(&self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::PtBr => "pt-BR",
        }
    }
}
