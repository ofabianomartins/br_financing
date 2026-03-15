use std::collections::BTreeMap;

use serde::{Serialize, Deserialize};
use rust_decimal::{ Decimal, MathematicalOps };
use rust_decimal_macros::dec;
use chrono::{Datelike, NaiveDate};

use crate::locale::Locale;

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
    /// The down payment as a percentage above total_amount (e.g., 5 for 5%).
    pub down_payment_percent: Decimal,
    /// The total number of months for the loan.
    pub total_months: u32,
    pub debt_type: DebtCalculationType,
    /// Insurance rate as a percentage applied over the outstanding balance each month.
    pub insurance_rate: Decimal,
    /// Fixed monthly administration fee.
    pub admin_fee: Decimal,
    /// Day of the month for installment due date (1-31, clamped to last day of month).
    pub due_day: u8,
    /// Start date of the financing.
    pub start_date: NaiveDate,
    /// Variable monthly correction rates. Key is the rate issue date, value is the rate.
    /// Lookup uses the most recent rate issued on or before the installment due date.
    pub monthly_correction_rates: BTreeMap<NaiveDate, Decimal>,
    /// Locale for error/validation messages.
    pub locale: Locale,
}


/// Internal calculator for monthly payment computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtCalculator {
    pub fixed_amortization: Decimal,
    pub fixed_payment: Decimal,
    pub monthly_interest_rate: Decimal,
    pub debt_type: DebtCalculationType
}

impl DebtCalculator {

    pub fn next_payment(
        &self,
        current_balance: Decimal,
        total_paid: Decimal,
        insurance_rate: Decimal,
        admin_fee: Decimal,
    ) -> MonthPayment {
        let insurance_cost = current_balance * insurance_rate / dec!(100);

        match self.debt_type {
            DebtCalculationType::Sac => {
                let interest_payment = current_balance * self.monthly_interest_rate;
                let current_payment = self.fixed_amortization + interest_payment + insurance_cost + admin_fee;

                MonthPayment {
                    due_date: NaiveDate::default(),
                    new_balance: current_balance.max(dec!(0)),
                    current_amortization: self.fixed_amortization,
                    current_interest: interest_payment,
                    insurance_cost,
                    admin_fee,
                    monetary_correction: dec!(0),
                    total_payment: current_payment,
                    total_paid: total_paid + current_payment,
                }
            },
            DebtCalculationType::Price => {
                let interest_payment = current_balance * self.monthly_interest_rate;
                let amortization = self.fixed_payment - interest_payment;
                let current_payment = amortization + interest_payment + insurance_cost + admin_fee;

                MonthPayment {
                    due_date: NaiveDate::default(),
                    new_balance: current_balance.max(dec!(0)),
                    current_amortization: amortization,
                    current_interest: interest_payment,
                    insurance_cost,
                    admin_fee,
                    monetary_correction: dec!(0),
                    total_payment: current_payment,
                    total_paid: total_paid + current_payment,
                }
            },
        }
    }

}

/// Represents the payment details for a single month.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthPayment {
    /// The due date for this installment.
    pub due_date: NaiveDate,
    /// The remaining balance of the loan after the payment.
    pub new_balance: Decimal,
    /// The portion of the payment that goes towards reducing the principal.
    pub current_amortization: Decimal,
    /// The portion of the payment that covers interest.
    pub current_interest: Decimal,
    /// Insurance cost for this month (percentage of pre-amortization balance).
    pub insurance_cost: Decimal,
    /// Fixed administration fee for this month.
    pub admin_fee: Decimal,
    /// Monetary correction applied to the balance after amortization.
    pub monetary_correction: Decimal,
    /// Total payment for this month (amortization + interest + insurance + admin fee).
    pub total_payment: Decimal,
    /// Cumulative total paid up to and including this month.
    pub total_paid: Decimal,
}

/// Contains the results of a financing calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableResult {
    /// The total amount paid over the lifetime of the loan.
    pub total_paid: Decimal,
    /// Total insurance paid over the lifetime of the loan.
    pub total_insurance: Decimal,
    /// Total administration fees paid over the lifetime of the loan.
    pub total_admin_fees: Decimal,
    /// Total monetary correction applied over the lifetime of the loan.
    pub total_monetary_correction: Decimal,
    /// A vector containing the payment details for each month.
    pub amortization_curve: Vec<MonthPayment>,
}

/// Calculates the due date for a given month offset from the start date.
/// Clamps the day to the last valid day of the target month.
pub fn calculate_due_date(start_date: NaiveDate, month_offset: u32, due_day: u8) -> NaiveDate {
    let total_months = start_date.month() as u32 + month_offset;
    let target_year = start_date.year() + ((total_months - 1) / 12) as i32;
    let target_month = ((total_months - 1) % 12) + 1;

    let last_day_of_month = last_day_of_month(target_year, target_month);
    let clamped_day = (due_day as u32).min(last_day_of_month);

    NaiveDate::from_ymd_opt(target_year, target_month, clamped_day).unwrap()
}

/// Returns the last day of a given year-month.
fn last_day_of_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => unreachable!(),
    }
}

/// Looks up the most recent correction rate issued on or before the given date.
/// Returns zero if no rate is found.
pub fn lookup_correction_rate(
    rates: &BTreeMap<NaiveDate, Decimal>,
    due_date: NaiveDate,
) -> Decimal {
    rates
        .range(..=due_date)
        .next_back()
        .map(|(_, &rate)| rate)
        .unwrap_or(dec!(0))
}


/// Calculates the financing table with insurance, admin fee, and monetary correction.
pub fn calculate_table(
    total_amount: Decimal,
    monthly_interest_rate: Decimal,
    total_months: u32,
    debt_type: DebtCalculationType,
    insurance_rate: Decimal,
    admin_fee: Decimal,
    due_day: u8,
    start_date: NaiveDate,
    monthly_correction_rates: &BTreeMap<NaiveDate, Decimal>,
) -> Result<TableResult, anyhow::Error> {
    if total_months == 0 {
        return Err(anyhow::anyhow!("Total months cannot be zero."));
    }

    // Price table formula: PMT = P * [i(1 + i)^n] / [(1 + i)^n – 1]
    let i_plus_1_pow_n = (dec!(1) + monthly_interest_rate).powu(total_months.into());
    let fixed_payment =
        total_amount * (monthly_interest_rate * i_plus_1_pow_n) / (i_plus_1_pow_n - dec!(1));
    let fixed_amortization = total_amount / Decimal::from(total_months);

    let calculation = DebtCalculator {
        fixed_payment,
        fixed_amortization,
        monthly_interest_rate,
        debt_type
    };

    let mut current_balance = total_amount;
    let mut total_paid = dec!(0);
    let mut total_insurance = dec!(0);
    let mut total_admin_fees = dec!(0);
    let mut total_monetary_correction = dec!(0);
    let mut amortization_curve = Vec::new();

    for month in 1..=total_months {
        let due_date = calculate_due_date(start_date, month, due_day);

        let mut payment = calculation.next_payment(
            current_balance,
            total_paid,
            insurance_rate,
            admin_fee,
        );

        payment.due_date = due_date;

        // Apply monetary correction to the balance after amortization
        let balance_after_amortization = current_balance - payment.current_amortization;
        let correction_rate = lookup_correction_rate(monthly_correction_rates, due_date);
        let monetary_correction = balance_after_amortization * correction_rate / dec!(100);
        payment.monetary_correction = monetary_correction;
        payment.new_balance = (balance_after_amortization + monetary_correction).max(dec!(0));

        current_balance = payment.new_balance;
        total_paid = payment.total_paid;
        total_insurance += payment.insurance_cost;
        total_admin_fees += payment.admin_fee;
        total_monetary_correction += monetary_correction;

        amortization_curve.push(payment);
    }

    Ok(TableResult {
        total_paid: total_paid.round_dp(2),
        total_insurance: total_insurance.round_dp(2),
        total_admin_fees: total_admin_fees.round_dp(2),
        total_monetary_correction: total_monetary_correction.round_dp(2),
        amortization_curve,
    })
}
