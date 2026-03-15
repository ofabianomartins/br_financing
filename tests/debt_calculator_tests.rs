use std::collections::BTreeMap;
use rust_decimal_macros::dec;
use chrono::NaiveDate;
use br_financial::{
    calculate_debt_trajectory,
    debt_calculator,
    DebtCalculationInput,
    DebtCalculationType,
    Locale,
};
use br_financial::utils;

fn default_input() -> DebtCalculationInput {
    DebtCalculationInput {
        total_amount: dec!(12000),
        interest_per_year: dec!(12),
        down_payment_percent: dec!(0),
        total_months: 12,
        debt_type: DebtCalculationType::Sac,
        insurance_rate: dec!(0),
        insurance_fee: dec!(0),
        admin_fee: dec!(0),
        due_day: 15,
        start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        monthly_correction_rates: BTreeMap::new(),
        locale: Locale::En,
    }
}

// ==================== Existing tests (updated) ====================

#[test]
fn test_calculate_debt_trajectory_happy_path() {
    let input = default_input();
    let result = calculate_debt_trajectory(input).unwrap();
    assert_eq!(result.table.total_paid.round_dp(2), dec!(12740.13));
}

#[test]
fn test_calculate_debt_trajectory_with_down_payment10() {
    let mut input = default_input();
    input.down_payment_percent = dec!(10.0);
    input.debt_type = DebtCalculationType::Price;

    let result = calculate_debt_trajectory(input).unwrap();
    assert_eq!(result.financed_amount.round_dp(2), dec!(10800.00));
    assert_eq!(result.table.total_paid.round_dp(2), dec!(11477.64));
}

#[test]
fn test_calculate_debt_trajectory_with_down_payment40() {
    let mut input = default_input();
    input.down_payment_percent = dec!(40);
    input.debt_type = DebtCalculationType::Price;

    let result = calculate_debt_trajectory(input).unwrap();
    assert_eq!(result.financed_amount.round_dp(2), dec!(7200.00));
    assert_eq!(result.table.total_paid.round_dp(2), dec!(7651.76));
}

#[test]
fn test_normalize_annual_interest_rate() {
    let annual_rate = dec!(12);
    let monthly_rate = utils::normalize_annual_interest_rate(annual_rate);
    assert!(monthly_rate > dec!(0.0094) && monthly_rate < dec!(0.0095));
}

// ==================== Validation tests ====================

#[test]
fn test_zero_months_error_en() {
    let mut input = default_input();
    input.total_months = 0;
    input.locale = Locale::En;

    let result = calculate_debt_trajectory(input);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Total months cannot be zero.");
}

#[test]
fn test_zero_months_error_ptbr() {
    let mut input = default_input();
    input.total_months = 0;
    input.locale = Locale::PtBr;

    let result = calculate_debt_trajectory(input);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "O total de meses não pode ser zero."
    );
}

#[test]
fn test_invalid_due_day_zero() {
    let mut input = default_input();
    input.due_day = 0;

    let result = calculate_debt_trajectory(input);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Due day must be between 1 and 31.");
}

#[test]
fn test_negative_insurance_rate() {
    let mut input = default_input();
    input.insurance_rate = dec!(-1);

    let result = calculate_debt_trajectory(input);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Insurance rate must be greater than or equal to zero."
    );
}

#[test]
fn test_negative_admin_fee() {
    let mut input = default_input();
    input.admin_fee = dec!(-5);

    let result = calculate_debt_trajectory(input);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Administration fee must be greater than or equal to zero."
    );
}

// ==================== Insurance tests ====================

#[test]
fn test_insurance_sac_month1() {
    let mut input = default_input();
    input.insurance_rate = dec!(0.005); // 0.5% normalized

    let result = calculate_debt_trajectory(input).unwrap();
    let first_month = &result.table.amortization_curve[0];

    // Insurance on 12000 balance = 12000 * 0.005 = 60
    assert_eq!(first_month.insurance_cost.round_dp(2), dec!(60.00));
}

#[test]
fn test_insurance_price_month1() {
    let mut input = default_input();
    input.debt_type = DebtCalculationType::Price;
    input.insurance_rate = dec!(0.01); // 1% normalized

    let result = calculate_debt_trajectory(input).unwrap();
    let first_month = &result.table.amortization_curve[0];

    // Insurance on 12000 balance = 12000 * 0.01 = 120
    assert_eq!(first_month.insurance_cost.round_dp(2), dec!(120.00));
}

#[test]
fn test_total_insurance_accumulated() {
    let mut input = default_input();
    input.insurance_rate = dec!(0.005);

    let result = calculate_debt_trajectory(input).unwrap();
    assert!(result.table.total_insurance > dec!(0));

    // Insurance should decrease over time as balance decreases (SAC)
    let first = &result.table.amortization_curve[0];
    let last = &result.table.amortization_curve[11];
    assert!(first.insurance_cost > last.insurance_cost);
}

// ==================== Admin fee tests ====================

#[test]
fn test_admin_fee_constant_sac() {
    let mut input = default_input();
    input.admin_fee = dec!(25);

    let result = calculate_debt_trajectory(input).unwrap();

    for payment in &result.table.amortization_curve {
        assert_eq!(payment.admin_fee, dec!(25));
    }
    assert_eq!(result.table.total_admin_fees, dec!(300.00)); // 25 * 12
}

#[test]
fn test_admin_fee_constant_price() {
    let mut input = default_input();
    input.debt_type = DebtCalculationType::Price;
    input.admin_fee = dec!(50);

    let result = calculate_debt_trajectory(input).unwrap();

    for payment in &result.table.amortization_curve {
        assert_eq!(payment.admin_fee, dec!(50));
    }
    assert_eq!(result.table.total_admin_fees, dec!(600.00)); // 50 * 12
}

// ==================== Monetary correction tests ====================

#[test]
fn test_monetary_correction_applied() {
    let mut input = default_input();
    let mut rates = BTreeMap::new();
    // Rate of 0.5% issued on Jan 1st, applies to all months
    rates.insert(
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        dec!(0.005),
    );
    input.monthly_correction_rates = rates;

    let result = calculate_debt_trajectory(input).unwrap();
    assert!(result.table.total_monetary_correction > dec!(0));

    // First month correction: 12000 * 0.005 = 60
    let first = &result.table.amortization_curve[0];
    assert_eq!(first.monetary_correction.round_dp(2), dec!(60.00));
}

#[test]
fn test_monetary_correction_zero_when_no_rates() {
    let input = default_input(); // empty BTreeMap

    let result = calculate_debt_trajectory(input).unwrap();
    assert_eq!(result.table.total_monetary_correction, dec!(0));

    for payment in &result.table.amortization_curve {
        assert_eq!(payment.monetary_correction, dec!(0));
    }
}

#[test]
fn test_monetary_correction_uses_most_recent_rate() {
    let mut input = default_input();
    input.total_months = 3;

    let mut rates = BTreeMap::new();
    rates.insert(
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        dec!(0.005),
    );
    rates.insert(
        NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
        dec!(0.01),
    );
    input.monthly_correction_rates = rates;

    let result = calculate_debt_trajectory(input).unwrap();

    // Month 1 due Feb 15 -> most recent <= Feb 15 is Jan 1 (0.5%)
    // Month 2 due Mar 15 -> most recent <= Mar 15 is Mar 1 (1.0%)
    let month1 = &result.table.amortization_curve[0];
    let month2 = &result.table.amortization_curve[1];

    // Compute effective rate: correction / (new_balance - correction) = correction / balance_after_amort
    let balance_after_amort_m1 = month1.new_balance - month1.monetary_correction;
    let balance_after_amort_m2 = month2.new_balance - month2.monetary_correction;
    let effective_rate_m1 = month1.monetary_correction / balance_after_amort_m1;
    let effective_rate_m2 = month2.monetary_correction / balance_after_amort_m2;

    // Month 2 uses 1.0% vs Month 1 uses 0.5%
    assert!(effective_rate_m2 > effective_rate_m1);
}

// ==================== Date calculation tests ====================

#[test]
fn test_due_date_day31_on_february() {
    let mut input = default_input();
    input.due_day = 31;
    input.start_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

    let result = calculate_debt_trajectory(input).unwrap();
    // Month 1 (offset 1) -> February 2026 (non-leap year), day 31 should clamp to 28
    let feb_payment = &result.table.amortization_curve[0];
    assert_eq!(feb_payment.due_date, NaiveDate::from_ymd_opt(2026, 2, 28).unwrap());
}

#[test]
fn test_due_date_day30_on_february_leap_year() {
    let due_date = debt_calculator::calculate_due_date(
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        1, // month offset 1 -> Feb 2023
        30,
    );
    // 2023 is not a leap year, Feb has 28 days
    assert_eq!(due_date, NaiveDate::from_ymd_opt(2023, 2, 28).unwrap());

    // 2024 is a leap year
    let due_date_leap = debt_calculator::calculate_due_date(
        NaiveDate::from_ymd_opt(2023, 12, 1).unwrap(),
        2, // month offset 2 -> Feb 2024
        30,
    );
    assert_eq!(due_date_leap, NaiveDate::from_ymd_opt(2024, 2, 29).unwrap());
}

// ==================== Rate lookup tests ====================

#[test]
fn test_lookup_correction_rate_exact_match() {
    let mut rates = BTreeMap::new();
    let date = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
    rates.insert(date, dec!(0.0075));

    let rate = debt_calculator::lookup_correction_rate(&rates, date);
    assert_eq!(rate, dec!(0.0075));
}

#[test]
fn test_lookup_correction_rate_no_match_defaults_zero() {
    let mut rates = BTreeMap::new();
    rates.insert(
        NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        dec!(0.01),
    );

    // Query before any rate exists
    let rate = debt_calculator::lookup_correction_rate(
        &rates,
        NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
    );
    assert_eq!(rate, dec!(0));
}

// ==================== Total payment formula tests ====================

#[test]
fn test_total_payment_includes_all_components_sac() {
    let mut input = default_input();
    input.insurance_rate = dec!(0.005);
    input.admin_fee = dec!(25);

    let result = calculate_debt_trajectory(input).unwrap();

    for payment in &result.table.amortization_curve {
        let expected_total = payment.current_amortization
            + payment.current_interest
            + payment.insurance_cost
            + payment.admin_fee
            + payment.monetary_correction;
        assert_eq!(
            payment.total_payment.round_dp(6),
            expected_total.round_dp(6)
        );
    }
}

#[test]
fn test_total_payment_includes_all_components_price() {
    let mut input = default_input();
    input.debt_type = DebtCalculationType::Price;
    input.insurance_rate = dec!(0.003);
    input.admin_fee = dec!(10);

    let result = calculate_debt_trajectory(input).unwrap();

    for payment in &result.table.amortization_curve {
        let expected_total = payment.current_amortization
            + payment.current_interest
            + payment.insurance_cost
            + payment.admin_fee
            + payment.monetary_correction;
        assert_eq!(
            payment.total_payment.round_dp(6),
            expected_total.round_dp(6)
        );
    }
}

// ==================== Full trajectory tests ====================

#[test]
fn test_full_sac_trajectory_with_all_features() {
    let mut input = default_input();
    input.insurance_rate = dec!(0.005);
    input.admin_fee = dec!(25);
    let mut rates = BTreeMap::new();
    rates.insert(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), dec!(0.003));
    input.monthly_correction_rates = rates;

    let result = calculate_debt_trajectory(input).unwrap();

    assert!(result.table.total_paid > dec!(12740.13)); // More than without extras
    assert!(result.table.total_insurance > dec!(0));
    assert_eq!(result.table.total_admin_fees, dec!(300.00));
    assert!(result.table.total_monetary_correction > dec!(0));
    assert_eq!(result.table.amortization_curve.len(), 12);
}

#[test]
fn test_full_price_trajectory_with_all_features() {
    let mut input = default_input();
    input.debt_type = DebtCalculationType::Price;
    input.insurance_rate = dec!(0.005);
    input.admin_fee = dec!(25);
    let mut rates = BTreeMap::new();
    rates.insert(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), dec!(0.003));
    input.monthly_correction_rates = rates;

    let result = calculate_debt_trajectory(input).unwrap();

    assert!(result.table.total_paid > dec!(0));
    assert!(result.table.total_insurance > dec!(0));
    assert_eq!(result.table.total_admin_fees, dec!(300.00));
    assert!(result.table.total_monetary_correction > dec!(0));
    assert_eq!(result.table.amortization_curve.len(), 12);
}

// ==================== Insurance fee tests ====================

#[test]
fn test_insurance_fee_constant_sac() {
    let mut input = default_input();
    input.insurance_fee = dec!(30);

    let result = calculate_debt_trajectory(input).unwrap();

    for payment in &result.table.amortization_curve {
        // With rate=0, insurance_cost should equal the fixed fee
        assert_eq!(payment.insurance_cost, dec!(30));
    }
    assert_eq!(result.table.total_insurance, dec!(360.00)); // 30 * 12
}

#[test]
fn test_insurance_fee_constant_price() {
    let mut input = default_input();
    input.debt_type = DebtCalculationType::Price;
    input.insurance_fee = dec!(45);

    let result = calculate_debt_trajectory(input).unwrap();

    for payment in &result.table.amortization_curve {
        assert_eq!(payment.insurance_cost, dec!(45));
    }
    assert_eq!(result.table.total_insurance, dec!(540.00)); // 45 * 12
}

#[test]
fn test_insurance_fee_combined_with_rate() {
    let mut input = default_input();
    input.insurance_rate = dec!(0.005); // 0.5% of balance
    input.insurance_fee = dec!(20);

    let result = calculate_debt_trajectory(input).unwrap();
    let first_month = &result.table.amortization_curve[0];

    // First month: 12000 * 0.005 + 20 = 60 + 20 = 80
    assert_eq!(first_month.insurance_cost.round_dp(2), dec!(80.00));

    // Insurance cost should decrease over time (rate portion decreases, fee stays)
    let last_month = &result.table.amortization_curve[11];
    assert!(first_month.insurance_cost > last_month.insurance_cost);

    // But last month should still be >= fee (the fixed part)
    assert!(last_month.insurance_cost >= dec!(20));
}

#[test]
fn test_negative_insurance_fee() {
    let mut input = default_input();
    input.insurance_fee = dec!(-10);

    let result = calculate_debt_trajectory(input);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Insurance fee must be greater than or equal to zero."
    );
}
