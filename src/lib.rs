//! # br_financial
//!
//! A Rust library for calculating real estate financing in Brazil.
//!
//! ## Amortization Systems
//!
//! - **SAC (Sistema de Amortização Constante)**: Fixed amortization payments with
//!   decreasing total payments over time.
//! - **Price (Sistema Francês de Amortização)**: Fixed total payments throughout
//!   the financing period (PMT calculated once upfront).
//!
//! ## Additional Features
//!
//! - **Insurance**: Monthly cost composed of a normalized rate applied to the
//!   outstanding balance plus a fixed fee (`insurance_cost = balance * rate + fee`).
//! - **Administration fee**: Fixed monthly amount added to each installment.
//! - **Monetary correction**: Variable monthly rate applied to the current balance
//!   before amortization. Rates are provided as a `BTreeMap<NaiveDate, Decimal>`,
//!   looked up by the most recent issue date on or before the installment due date.
//! - **Date-based installments**: Each payment has a due date computed from
//!   `start_date`, `due_day` (1–28), and month offset.
//! - **Internationalization**: Error messages localized via `rust-i18n` (EN and PT-BR).
//!
//! ## Example
//!
//! ```rust
//! use std::collections::BTreeMap;
//! use br_financial::{calculate_debt_trajectory, DebtCalculationInput, DebtCalculationType, Locale};
//! use rust_decimal_macros::dec;
//! use chrono::NaiveDate;
//!
//! let input = DebtCalculationInput {
//!     total_amount: dec!(360_000),
//!     interest_per_year: dec!(10.5),
//!     down_payment_percent: dec!(5),
//!     total_months: 420,
//!     debt_type: DebtCalculationType::Sac,
//!     insurance_rate: dec!(0.0003),
//!     insurance_fee: dec!(25),
//!     admin_fee: dec!(30),
//!     due_day: 15,
//!     start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
//!     monthly_correction_rates: BTreeMap::new(),
//!     locale: Locale::En,
//! };
//!
//! match calculate_debt_trajectory(input) {
//!     Ok(result) => {
//!         println!("Financed: {:.2}", result.financed_amount);
//!         println!("Total Paid: {:.2}", result.table.total_paid);
//!         println!("Total Insurance: {:.2}", result.table.total_insurance);
//!         println!("Total Admin Fees: {:.2}", result.table.total_admin_fees);
//!         println!("Total Monetary Correction: {:.2}", result.table.total_monetary_correction);
//!     }
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! ```

use serde::{Serialize, Deserialize};
use rust_decimal::Decimal;
use rust_i18n::t;

rust_i18n::i18n!("locales", fallback = "en");

pub mod debt_calculator;
pub mod locale;
pub mod utils;

use utils::{ clean_down_payment, normalize_annual_interest_rate };
pub use debt_calculator::{
    DebtCalculationInput,
    DebtCalculationType,
    MonthPayment,
    TableResult,
    calculate_table
};
pub use locale::Locale;


/// Contains the comprehensive results for a financing calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtTrajectoryResult {
    /// The financed amount after applying the down payment.
    pub financed_amount: Decimal,
    /// The detailed amortization table with monthly payments and totals.
    pub table: TableResult,
}

/// Calculates the debt trajectory for the selected amortization system.
///
/// This is the main entry point of the library. It validates the input,
/// converts the annual interest rate to a monthly rate, applies the down
/// payment, and computes the full amortization schedule including insurance,
/// administration fees, and monetary correction.
///
/// # Arguments
///
/// * `input` - A [`DebtCalculationInput`] containing all loan parameters.
///
/// # Returns
///
/// A [`DebtTrajectoryResult`] with the financed amount and the complete
/// amortization table, or a localized error if validation fails.
///
/// # Errors
///
/// Returns a localized error (`Locale::En` or `Locale::PtBr`) when:
/// - `total_months` is zero
/// - `due_day` is not between 1 and 28
/// - `insurance_rate` is negative
/// - `insurance_fee` is negative
/// - `admin_fee` is negative
pub fn calculate_debt_trajectory(
    input: DebtCalculationInput
) -> Result<DebtTrajectoryResult, anyhow::Error> {
    let locale = input.locale.as_str();

    if input.total_months == 0 {
        return Err(anyhow::anyhow!(t!("total_months_zero", locale = locale)));
    }

    if input.due_day < 1 || input.due_day > 28 {
        return Err(anyhow::anyhow!(t!("invalid_due_day", locale = locale)));
    }

    if input.insurance_rate < Decimal::ZERO {
        return Err(anyhow::anyhow!(t!("negative_insurance_rate", locale = locale)));
    }

    if input.admin_fee < Decimal::ZERO {
        return Err(anyhow::anyhow!(t!("negative_admin_fee", locale = locale)));
    }

    if input.insurance_fee < Decimal::ZERO {
        return Err(anyhow::anyhow!(t!("negative_insurance_fee", locale = locale)));
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
        input.insurance_fee,
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
