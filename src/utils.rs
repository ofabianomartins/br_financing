use rust_decimal::{ Decimal, MathematicalOps };

/// Calculates the financed amount after applying the down payment percentage.
///
/// Returns `initial_amount * (1 - percent / 100)`.
///
/// # Arguments
///
/// * `initial_amount` - The total loan amount before the down payment.
/// * `percent` - The down payment percentage (e.g., `10` for 10%).
///
/// # Returns
///
/// The financed amount (total minus down payment).
pub fn clean_down_payment(initial_amount: Decimal, percent: Decimal) -> Decimal {
    let one = Decimal::from_str_exact("1").unwrap();
    let norm_percent = percent / Decimal::from_str_exact("100.0").unwrap();

    return initial_amount*(one - norm_percent);
}

/// Converts an annual interest rate percentage to a monthly decimal factor.
///
/// Uses compound interest: `monthly_rate = (1 + annual_rate / 100) ^ (1/12) - 1`.
///
/// # Arguments
///
/// * `input` - The annual interest rate as a percentage (e.g., `10.5` for 10.5%).
///
/// # Returns
///
/// The effective monthly interest rate as a decimal (e.g., ~`0.008368` for 10.5% annual).
pub fn normalize_annual_interest_rate(input: Decimal) -> Decimal {
    let one = Decimal::from_str_exact("1").unwrap();
    let percent = input / Decimal::from_str_exact("100.0").unwrap();
    let twelve = Decimal::from_str_exact("12").unwrap();

    let base = one + percent;
    let exponent  = one / twelve;

    let power_result = base.powd(exponent);

    return power_result - one;
}
