//! `br_financial` is a Rust library for calculating real estate financing in Brazil.
//!
//! It provides tools to calculate and compare financing scenarios using the two main
//! amortization systems in Brazil:
//! - **SAC (Sistema de Amortização Constante)**: Characterized by fixed amortization payments,
//!   leading to decreasing total payments over time.
//! - **Price (Sistema Francês de Amortização)**: Characterized by fixed total payments
//!   throughout the financing period.
//!
//! ## Usage
//!
//! Add `br_financial` to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! br_financial = "0.1.0"
//! rust_decimal = "1.39.0"
//! rust_decimal_macros = "1.39.0"
//! ```
//!
//! Then, use the `calculate_debt_trajectory` function to get the results for both
//! SAC and Price tables:
//!
//! ```rust
//! use br_financial::{calculate_debt_trajectory, DebtCalculationInput, DebtCalculationType};
//! use rust_decimal_macros::dec;
//!
//! fn main() {
//!     let input = DebtCalculationInput {
//!         total_amount: dec!(360_000),
//!         interest_per_year: dec!(10.5),
//!         down_payment_percent: dec!(0),
//!         total_months: 420,
//!         debt_type: DebtCalculationType::Sac
//!     };
//!
//!     match calculate_debt_trajectory(input) {
//!         Ok(result) => {
//!             println!("Price Total Paid:    {:.2}", result.table.total_paid);
//!         }
//!         Err(e) => {
//!             eprintln!("Error calculating debt trajectory: {}", e);
//!         }
//!     }
//! }
//! ```

use serde::{Serialize, Deserialize};
use rust_decimal::{ Decimal, MathematicalOps };
use rust_decimal_macros::dec;

/// Input parameters for debt trajectory calculation.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum DebtCalculationType { Sac = 0, Price = 1 }

/// Input parameters for debt trajectory calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtCalculationInput {
    /// The total principal amount of the loan.
    pub total_amount: Decimal,
    /// The annual interest rate as a percentage (e.g., 10.5 for 10.5%).
    pub interest_per_year: Decimal,
    /// The down payment as a percentage above total_ammount (e.g., 5 for 5%).
        /// The percentage of the total amount that is provided as a down payment.
    pub down_payment_percent: Decimal,
    /// The total number of months for the loan.
    pub total_months: u32,
    pub debt_type: DebtCalculationType
}

/// Represents the payment details for a single month.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthPayment {
    /// The remaining balance of the loan after the payment.
    pub new_balance: Decimal,
    /// The portion of the payment that goes towards reducing the principal.
    pub current_amortization: Decimal,
    /// The portion of the payment that covers interest.
    pub current_interest: Decimal
}

/// Contains the results of a financing calculation using the Price table method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableResult {
    /// The total amount paid over the lifetime of the loan.
    pub total_paid: Decimal,
    /// A vector containing the payment details for each month.
    pub amortization_curve: Vec<MonthPayment>,
}

/// Contains the comprehensive results for both Price and SAC table calculations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtTrajectoryResult {
    /// The initial total amount of the loan.
    pub financed_amount: Decimal,
    /// The results calculated using the SAC method.
    pub table: TableResult,
}

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

/// Calculates and compares the debt trajectory for both Price and SAC amortization systems.
///
/// This is the main entry point of the library. It takes the loan parameters and
/// returns a struct containing detailed results for both financing tables.
///
/// # Arguments
///
/// * `input` - A `DebtCalculationInput` struct containing the loan amount, interest rate, and term.
///
/// # Errors
///
/// Returns an error if the `total_months` is zero.
pub fn calculate_debt_trajectory(input: DebtCalculationInput) -> Result<DebtTrajectoryResult, anyhow::Error> {
    // Convert annual percentage to monthly decimal
    let monthly_interest_rate = normalize_annual_interest_rate(input.interest_per_year);
    let financed_amount = clean_down_payment(input.total_amount, input.down_payment_percent);

    let table = calculate_table(
        financed_amount,
        monthly_interest_rate,
        input.total_months,
        input.debt_type
    )?;

    Ok(DebtTrajectoryResult { financed_amount, table })
}

/// Calculates the financing trajectory using the Price table (fixed payments).
///
/// The Price table formula is: PMT = P * [i(1 + i)^n] / [(1 + i)^n – 1]
///
/// # Arguments
///
/// * `total_amount` - The principal loan amount.
/// * `monthly_interest_rate` - The effective monthly interest rate as a decimal (not percentage).
/// * `total_months` - The total number of payments.
///
/// # Errors
///
/// Returns an error if `total_months` is zero.
pub fn calculate_table(
    total_amount: Decimal,
    monthly_interest_rate: Decimal,
    total_months: u32,
    debt_type: DebtCalculationType
) -> Result<TableResult, anyhow::Error> {
    if total_months == 0 {
        return Err(anyhow::anyhow!("Total months cannot be zero."));
    }

    if debt_type == DebtCalculationType::Sac {
        return calculate_sac_table(total_amount, monthly_interest_rate, total_months);
    } else if debt_type == DebtCalculationType::Price {
        return calculate_price_table(total_amount, monthly_interest_rate, total_months);
    } else {
        return Err(anyhow::anyhow!("DebtType is not defined."));
    }
}

/// Calculates the financing trajectory using the Price table (fixed payments).
///
/// The Price table formula is: PMT = P * [i(1 + i)^n] / [(1 + i)^n – 1]
///
/// # Arguments
///
/// * `total_amount` - The principal loan amount.
/// * `monthly_interest_rate` - The effective monthly interest rate as a decimal (not percentage).
/// * `total_months` - The total number of payments.
///
/// # Errors
///
/// Returns an error if `total_months` is zero.
pub fn calculate_price_table(
    total_amount: Decimal,
    monthly_interest_rate: Decimal,
    total_months: u32,
) -> Result<TableResult, anyhow::Error> {
    if total_months == 0 {
        return Err(anyhow::anyhow!("Total months cannot be zero."));
    }

    // Price table formula: PMT = P * [i(1 + i)^n] / [(1 + i)^n – 1]
    let i_plus_1_pow_n = (dec!(1) + monthly_interest_rate).powu(total_months.into());
    let fixed_payment =
        total_amount * (monthly_interest_rate * i_plus_1_pow_n) / (i_plus_1_pow_n - dec!(1));

    let mut current_balance = total_amount;
    let mut total_paid = dec!(0);
    let mut amortization_curve = Vec::new();

    for _ in 0..total_months {
        let interest_payment = current_balance * monthly_interest_rate;
        let amortization = fixed_payment - interest_payment;
        current_balance -= amortization;
        total_paid += fixed_payment;
        amortization_curve.push(
            MonthPayment {
                new_balance: current_balance.max(dec!(0)),
                current_amortization: amortization,
                current_interest: interest_payment
            }
        );
    }

    Ok(TableResult {
        total_paid: total_paid.round_dp(2),
        amortization_curve,
    })
}

/// Calculates the financing trajectory using the SAC (Constant Amortization System).
///
/// In the SAC system, the principal portion of the payment is constant, while the
/// interest portion decreases over time, resulting in declining total payments.
///
/// # Arguments
///
/// * `total_amount` - The principal loan amount.
/// * `monthly_interest_rate` - The effective monthly interest rate as a decimal (not percentage).
/// * `total_months` - The total number of payments.
///
/// # Errors
///
/// Returns an error if `total_months` is zero.
pub fn calculate_sac_table(
    total_amount: Decimal,
    monthly_interest_rate: Decimal,
    total_months: u32,
) -> Result<TableResult, anyhow::Error> {
    if total_months == 0 {
        return Err(anyhow::anyhow!("Total months cannot be zero."));
    }

    let fixed_amortization = total_amount / Decimal::from(total_months);
    let mut current_balance = total_amount;
    let mut total_paid = dec!(0);
    let mut amortization_curve = Vec::new();

    for month in 0..total_months {
        let interest_payment = current_balance * monthly_interest_rate;
        let current_payment = fixed_amortization + interest_payment;

        current_balance -= fixed_amortization;
        total_paid += current_payment;
        amortization_curve.push(
            MonthPayment {
                new_balance: current_balance.max(dec!(0)),
                current_amortization: fixed_amortization,
                current_interest: interest_payment
            }
        );
    }

    Ok(TableResult {
        total_paid: total_paid.round_dp(2),
        amortization_curve,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_calculate_debt_trajectory_happy_path() {
        let input = DebtCalculationInput {
            total_amount: dec!(12000),
            interest_per_year: dec!(12),
            down_payment_percent: dec!(0),
            total_months: 12,
            debt_type: DebtCalculationType::Sac
        };

        let result = calculate_debt_trajectory(input).unwrap();

        // Assertions for SAC table
        assert_eq!(result.table.total_paid.round_dp(2), dec!(12740.13));
    }

    #[test]
    fn test_calculate_debt_trajectory_with_down_payment10() {
        let input = DebtCalculationInput {
            total_amount: dec!(12000),
            interest_per_year: dec!(12),
            down_payment_percent: dec!(10.0),
            total_months: 12,
            debt_type: DebtCalculationType::Price
        };

        let result = calculate_debt_trajectory(input).unwrap();

        assert_eq!(result.financed_amount.round_dp(2), dec!(10800.00));

        assert_eq!(result.table.total_paid.round_dp(2), dec!(11477.64));
    }

    #[test]
    fn test_calculate_debt_trajectory_with_down_payment40() {
        let input = DebtCalculationInput {
            total_amount: dec!(12000),
            interest_per_year: dec!(12),
            down_payment_percent: dec!(40),
            total_months: 12,
            debt_type: DebtCalculationType::Price
        };

        let result = calculate_debt_trajectory(input).unwrap();

        assert_eq!(result.financed_amount.round_dp(2), dec!(7200.00));

        // Assertions for Price table
        assert_eq!(result.table.total_paid.round_dp(2), dec!(7651.76));
    }

    #[test]
    fn test_normalize_annual_interest_rate() {
        // 12% per year should be a bit less than 1% per month when compounded.
        let annual_rate = dec!(12);
        let monthly_rate = normalize_annual_interest_rate(annual_rate);
        // Effective monthly rate for 12% annual is approx 0.9488%
        // (1.12)^(1/12) - 1 = 0.009488...
        // Let's check for a value in that range.
        assert!(monthly_rate > dec!(0.0094) && monthly_rate < dec!(0.0095));
    }

    #[test]
    fn test_zero_months_error() {
        let input = DebtCalculationInput {
            total_amount: dec!(100000),
            interest_per_year: dec!(10),
            down_payment_percent: dec!(0),
            total_months: 0,
            debt_type: DebtCalculationType::Price
        };
        let result = calculate_debt_trajectory(input);
        assert!(result.is_err());
    }
}
