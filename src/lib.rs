//! `br_financial` is a Rust library for calculating real estate financing in Brazil.
//!
//! It provides tools to calculate and compare financing scenarios using the two main
//! amortization systems in Brazil:
//! - **SAC (Sistema de Amortização Constante)**: Characterized by fixed amortization payments,
//!   leading to decreasing total payments over time.
//! - **Price (Sistema Francês de Amortização)**: Characterized by fixed total payments
//!   throughout the financing period.
//!
//! The library also supports:
//! - Monthly insurance cost (percentage of outstanding balance)
//! - Fixed monthly administration fee
//! - Variable monetary correction rates (BTreeMap lookup by date)
//! - Date-based installment calculation with configurable due day
//! - Internationalized error messages (PT-BR and EN)

use serde::{Serialize, Deserialize};
use rust_decimal::Decimal;

pub mod debt_calculator;
pub mod locale;
pub mod utils;

use utils::{ clean_down_payment, normalize_annual_interest_rate };
use locale::MessageKey;
pub use debt_calculator::{
    DebtCalculationInput,
    DebtCalculationType,
    MonthPayment,
    TableResult,
    calculate_table
};
pub use locale::Locale;


/// Contains the comprehensive results for both Price and SAC table calculations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtTrajectoryResult {
    /// The initial total amount of the loan.
    pub financed_amount: Decimal,
    /// The results calculated using the selected amortization method.
    pub table: TableResult,
}

/// Calculates the debt trajectory for the selected amortization system.
///
/// This is the main entry point of the library. It takes the loan parameters and
/// returns a struct containing detailed results including insurance, admin fees,
/// and monetary correction.
///
/// # Arguments
///
/// * `input` - A `DebtCalculationInput` struct containing all loan parameters.
///
/// # Errors
///
/// Returns a localized error if input validation fails.
pub fn calculate_debt_trajectory(
    input: DebtCalculationInput
) -> Result<DebtTrajectoryResult, anyhow::Error> {
    if input.total_months == 0 {
        return Err(anyhow::anyhow!(input.locale.message(MessageKey::TotalMonthsZero)));
    }

    if input.due_day < 1 || input.due_day > 31 {
        return Err(anyhow::anyhow!(input.locale.message(MessageKey::InvalidDueDay)));
    }

    if input.insurance_rate < Decimal::ZERO {
        return Err(anyhow::anyhow!(input.locale.message(MessageKey::NegativeInsuranceRate)));
    }

    if input.admin_fee < Decimal::ZERO {
        return Err(anyhow::anyhow!(input.locale.message(MessageKey::NegativeAdminFee)));
    }

    // Convert annual percentage to monthly decimal
    let monthly_interest_rate = normalize_annual_interest_rate(input.interest_per_year);
    let financed_amount = clean_down_payment(input.total_amount, input.down_payment_percent);

    let table = calculate_table(
        financed_amount,
        monthly_interest_rate,
        input.total_months,
        input.debt_type,
        input.insurance_rate,
        input.admin_fee,
        input.due_day,
        input.start_date,
        &input.monthly_correction_rates,
    );

    Ok(DebtTrajectoryResult {
        financed_amount,
        table: table.unwrap()
    })
}
