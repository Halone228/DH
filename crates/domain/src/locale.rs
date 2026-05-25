use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Locale {
    #[default]
    Ru,
    En,
}

impl Locale {
    pub fn code(&self) -> &'static str {
        match self {
            Locale::Ru => "ru",
            Locale::En => "en",
        }
    }

    pub fn from_code(code: &str) -> Self {
        match code.to_lowercase().as_str() {
            "en" | "english" => Locale::En,
            _ => Locale::Ru,
        }
    }
}
