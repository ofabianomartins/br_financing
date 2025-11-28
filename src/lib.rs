use serde::{Serialize, Deserialize};
use rust_decimal::{ Decimal, MathematicalOps };
use rust_decimal_macros::dec;

/// Input parameters for debt trajectory calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtCalculationInput {
    pub total_amount: Decimal,
    pub interest_per_year: Decimal,
    pub total_months: u32,
}

/// Results for Price table calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthPayment {
    pub new_balance: Decimal,
    pub current_amortization: Decimal,
    pub current_interest: Decimal
}

/// Results for Price table calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceTableResult {
    pub fixed_payment: Decimal,
    pub total_paid: Decimal,
    pub amortization_curve: Vec<MonthPayment>, // Remaining balance after each payment
}

/// Results for SAC table calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SacTableResult {
    pub fixed_amortization: Decimal,
    pub first_payment: Decimal,
    pub last_payment: Decimal,
    pub total_paid: Decimal,
    pub amortization_curve: Vec<MonthPayment>, // Remaining balance after each payment
}

/// Overall debt trajectory calculation results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtTrajectoryResult {
    pub initial_total_amount: Decimal,
    pub price_table: PriceTableResult,
    pub sac_table: SacTableResult,
}

pub fn normalize_annual_interest_rate(input: Decimal) -> Decimal {
    let one = Decimal::from_str_exact("1").unwrap();
    let percent = input / Decimal::from_str_exact("100.0").unwrap();
    let twelve = Decimal::from_str_exact("12").unwrap();

    let base = one + percent;
    let exponent  = one / twelve;

    let power_result = base.powd(exponent);

    return power_result - one;
}

pub fn calculate_debt_trajectory(input: DebtCalculationInput) -> Result<DebtTrajectoryResult, anyhow::Error> {
    // Convert annual percentage to monthly decimal
    let monthly_interest_rate = normalize_annual_interest_rate(input.interest_per_year);

    let price_table = calculate_price_table(
        input.total_amount,
        monthly_interest_rate,
        input.total_months,
    )?;

    let sac_table = calculate_sac_table(
        input.total_amount,
        monthly_interest_rate,
        input.total_months,
    )?;

    Ok(DebtTrajectoryResult {
        initial_total_amount: input.total_amount,
        price_table,
        sac_table,
    })
}

/// Calculates the trajectory for the Price table.
fn calculate_price_table(
    total_amount: Decimal,
    monthly_interest_rate: Decimal,
    total_months: u32,
) -> Result<PriceTableResult, anyhow::Error> {
    if total_months == 0 {
        return Err(anyhow::anyhow!("Total months cannot be zero."));
    }
    /*
    if monthly_interest_rate <= dec!(0) {
        // Handle zero interest rate as a special case to avoid division by zero or incorrect formula application
        let fixed_payment = total_amount / Decimal::from(total_months);
        let total_paid = fixed_payment * Decimal::from(total_months);
        let mut amortization_curve = Vec::new();
        let mut remaining_balance = total_amount;
        for _ in 0..total_months {
            remaining_balance -= fixed_payment;
            amortization_curve.push(remaining_balance.max(dec!(0))); // Ensure balance doesn't go negative
        }
        return Ok(PriceTableResult {
            fixed_payment,
            total_paid,
            amortization_curve,
        });
    }
    */

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

    Ok(PriceTableResult {
        fixed_payment: fixed_payment.round_dp(2),
        total_paid: total_paid.round_dp(2),
        amortization_curve,
    })
}

/// Calculates the trajectory for the SAC table.
fn calculate_sac_table(
    total_amount: Decimal,
    monthly_interest_rate: Decimal,
    total_months: u32,
) -> Result<SacTableResult, anyhow::Error> {
    if total_months == 0 {
        return Err(anyhow::anyhow!("Total months cannot be zero."));
    }

    let fixed_amortization = total_amount / Decimal::from(total_months);
    let mut current_balance = total_amount;
    let mut first_payment: Option<Decimal> = None;
    let mut last_payment: Option<Decimal> = None;
    let mut total_paid = dec!(0);
    let mut amortization_curve = Vec::new();

    for month in 0..total_months {
        let interest_payment = current_balance * monthly_interest_rate;
        let current_payment = fixed_amortization + interest_payment;

        if month == 0 {
            first_payment = Some(current_payment);
        }
        if month == total_months - 1 {
            last_payment = Some(current_payment);
        }

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

    Ok(SacTableResult {
        fixed_amortization: fixed_amortization.round_dp(2),
        first_payment: first_payment.unwrap_or_default().round_dp(2),
        last_payment: last_payment.unwrap_or_default().round_dp(2),
        total_paid: total_paid.round_dp(2),
        amortization_curve,
    })
}
