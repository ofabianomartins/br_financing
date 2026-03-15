use rust_decimal::{ Decimal, MathematicalOps };



///
/// Calculates the down payment amount based on a total amount and a percentage.
///
/// This function ensures that the calculated down payment does not exceed the
/// `initial_amount`.
///
/// Arguments:
///
/// * `initial_amount` - The total amount from which the down payment is calculated.
/// * `percent` - The down payment percentage.
///
/// Returns:
///
/// The calculated down payment amount, limited by `initial_amount`.
pub fn clean_down_payment(initial_amount: Decimal, percent: Decimal) -> Decimal {
    let one = Decimal::from_str_exact("1").unwrap();
    let norm_percent = percent / Decimal::from_str_exact("100.0").unwrap();

    return initial_amount*(one - norm_percent);
}

/// Normalizes an annual interest rate percentage to a monthly decimal factor.
///
/// This function converts a rate like 10.5% per year into its equivalent monthly multiplier
/// for use in compound interest calculations.
pub fn normalize_annual_interest_rate(input: Decimal) -> Decimal {
    let one = Decimal::from_str_exact("1").unwrap();
    let percent = input / Decimal::from_str_exact("100.0").unwrap();
    let twelve = Decimal::from_str_exact("12").unwrap();

    let base = one + percent;
    let exponent  = one / twelve;

    let power_result = base.powd(exponent);

    return power_result - one;
}
