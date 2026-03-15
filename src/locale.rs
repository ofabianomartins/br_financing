use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum Locale {
    PtBr,
    En,
}

impl Locale {
    pub fn message(&self, key: MessageKey) -> &'static str {
        match (self, key) {
            (Locale::En, MessageKey::TotalMonthsZero) => "Total months cannot be zero.",
            (Locale::PtBr, MessageKey::TotalMonthsZero) => "O total de meses não pode ser zero.",

            (Locale::En, MessageKey::InvalidDueDay) => "Due day must be between 1 and 31.",
            (Locale::PtBr, MessageKey::InvalidDueDay) => "O dia de vencimento deve estar entre 1 e 31.",

            (Locale::En, MessageKey::NegativeInsuranceRate) => "Insurance rate must be greater than or equal to zero.",
            (Locale::PtBr, MessageKey::NegativeInsuranceRate) => "A taxa de seguro deve ser maior ou igual a zero.",

            (Locale::En, MessageKey::NegativeAdminFee) => "Administration fee must be greater than or equal to zero.",
            (Locale::PtBr, MessageKey::NegativeAdminFee) => "A taxa de administração deve ser maior ou igual a zero.",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MessageKey {
    TotalMonthsZero,
    InvalidDueDay,
    NegativeInsuranceRate,
    NegativeAdminFee,
}
