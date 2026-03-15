use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum Locale {
    PtBr,
    En,
}

impl Locale {
    pub fn as_str(&self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::PtBr => "pt-BR",
        }
    }
}
