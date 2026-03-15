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


/// Input parameters for debt trajectory calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtCalculator {
    pub fixed_amortization: Decimal,
    pub fixed_payment: Decimal,
    pub monthly_interest_rate: Decimal,
    pub debt_type: DebtCalculationType
}

impl DebtCalculator {

    pub fn next_payment(&self, current_balance: Decimal, total_paid: Decimal) -> MonthPayment {
        match self.debt_type {
            DebtCalculationType::Sac => {
                let interest_payment = current_balance * self.monthly_interest_rate;
                let current_payment = self.fixed_amortization + interest_payment;

                return MonthPayment {
                    new_balance: current_balance.max(dec!(0)),
                    current_amortization: self.fixed_amortization,
                    current_interest: interest_payment,
                    total_payment: current_payment,
                    total_paid: total_paid + current_payment
                };
            },
            DebtCalculationType::Price => {
                let interest_payment = current_balance * self.monthly_interest_rate;
                let amortization = self.fixed_payment - interest_payment;
                return MonthPayment {
                    new_balance: current_balance.max(dec!(0)),
                    current_amortization: amortization,
                    current_interest: interest_payment,
                    total_payment: amortization + interest_payment,
                    total_paid: total_paid + amortization + interest_payment
                };
            },
        }
    } 

}

/// Represents the payment details for a single month.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthPayment {
    /// The remaining balance of the loan after the payment.
    pub new_balance: Decimal,
    /// The portion of the payment that goes towards reducing the principal.
    pub current_amortization: Decimal,
    /// The portion of the payment that covers interest.
    pub current_interest: Decimal,

    pub total_payment: Decimal,

    pub total_paid: Decimal
}

/// Contains the results of a financing calculation using the Price table method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableResult {
    /// The total amount paid over the lifetime of the loan.
    pub total_paid: Decimal,
    /// A vector containing the payment details for each month.
    pub amortization_curve: Vec<MonthPayment>,
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
    let mut amortization_curve = Vec::new();

    for _ in 0..total_months {
        let payment = calculation.next_payment(current_balance, total_paid);
        current_balance = payment.new_balance.clone();
        total_paid = payment.total_paid.clone();
        amortization_curve.push(payment)
    }

    Ok(TableResult {
        total_paid: total_paid.round_dp(2),
        amortization_curve,
    })
}

