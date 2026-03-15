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
use rust_decimal::Decimal;

pub mod debt_calculator;
pub mod utils;

use utils::{ clean_down_payment, normalize_annual_interest_rate };
pub use debt_calculator::{
    DebtCalculationInput,
    DebtCalculationType,
    MonthPayment,
    TableResult,
    calculate_table
};


/// Contains the comprehensive results for both Price and SAC table calculations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtTrajectoryResult {
    /// The initial total amount of the loan.
    pub financed_amount: Decimal,
    /// The results calculated using the SAC method.
    pub table: TableResult,
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
pub fn calculate_debt_trajectory(
    input: DebtCalculationInput
) -> Result<DebtTrajectoryResult, anyhow::Error> {
    if input.total_months == 0 {
        return Err(anyhow::anyhow!("Total months cannot be zero."));
    }

    // Convert annual percentage to monthly decimal
    let monthly_interest_rate = normalize_annual_interest_rate(input.interest_per_year);
    let financed_amount = clean_down_payment(input.total_amount, input.down_payment_percent);

    let table = calculate_table(
        financed_amount,
        monthly_interest_rate,
        input.total_months,
        input.debt_type
    );

    Ok(DebtTrajectoryResult {
        financed_amount,
        table: table.unwrap()
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
        assert_eq!(result.table.total_paid.round_dp(2), dec!(13366.39));
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
