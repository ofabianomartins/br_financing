use rust_i18n::t;

// Initialize i18n for integration tests
rust_i18n::i18n!("locales", fallback = "en");

#[test]
fn test_locale_en_messages() {
    let locale = "en";
    assert_eq!(t!("total_months_zero", locale = locale), "Total months cannot be zero.");
    assert_eq!(t!("invalid_due_day", locale = locale), "Due day must be between 1 and 31.");
    assert_eq!(t!("negative_insurance_rate", locale = locale), "Insurance rate must be greater than or equal to zero.");
    assert_eq!(t!("negative_admin_fee", locale = locale), "Administration fee must be greater than or equal to zero.");
    assert_eq!(t!("negative_insurance_fee", locale = locale), "Insurance fee must be greater than or equal to zero.");
}

#[test]
fn test_locale_ptbr_messages() {
    let locale = "pt-BR";
    assert_eq!(t!("total_months_zero", locale = locale), "O total de meses não pode ser zero.");
    assert_eq!(t!("invalid_due_day", locale = locale), "O dia de vencimento deve estar entre 1 e 31.");
    assert_eq!(t!("negative_insurance_rate", locale = locale), "A taxa de seguro deve ser maior ou igual a zero.");
    assert_eq!(t!("negative_admin_fee", locale = locale), "A taxa de administração deve ser maior ou igual a zero.");
    assert_eq!(t!("negative_insurance_fee", locale = locale), "A taxa fixa de seguro deve ser maior ou igual a zero.");
}
