use br_financial::locale::{Locale, MessageKey};

#[test]
fn test_locale_en_messages() {
    let locale = Locale::En;
    assert_eq!(locale.message(MessageKey::TotalMonthsZero), "Total months cannot be zero.");
    assert_eq!(locale.message(MessageKey::InvalidDueDay), "Due day must be between 1 and 31.");
    assert_eq!(locale.message(MessageKey::NegativeInsuranceRate), "Insurance rate must be greater than or equal to zero.");
    assert_eq!(locale.message(MessageKey::NegativeAdminFee), "Administration fee must be greater than or equal to zero.");
}

#[test]
fn test_locale_ptbr_messages() {
    let locale = Locale::PtBr;
    assert_eq!(locale.message(MessageKey::TotalMonthsZero), "O total de meses não pode ser zero.");
    assert_eq!(locale.message(MessageKey::InvalidDueDay), "O dia de vencimento deve estar entre 1 e 31.");
    assert_eq!(locale.message(MessageKey::NegativeInsuranceRate), "A taxa de seguro deve ser maior ou igual a zero.");
    assert_eq!(locale.message(MessageKey::NegativeAdminFee), "A taxa de administração deve ser maior ou igual a zero.");
}
