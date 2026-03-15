# br_financial

[![Crates.io](https://img.shields.io/crates/v/br_financial.svg)](https://crates.io/crates/br_financial)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Docs.rs](https://docs.rs/br_financial/badge.svg)](https://docs.rs/br_financial)

A Rust library for calculating real estate financing in Brazil, supporting SAC and Price amortization systems with insurance, administration fees, monetary correction, and internationalized error messages.

## Features

- **SAC (Sistema de Amortização Constante)**: Fixed amortization with decreasing total payments over time.
- **Price (Sistema Francês de Amortização)**: Fixed total payments throughout the financing period.
- **Insurance**: Monthly cost composed of a normalized rate on the outstanding balance plus a fixed fee.
- **Administration fee**: Fixed monthly amount added to each installment.
- **Monetary correction**: Variable monthly rate applied to the current balance before amortization, with rates looked up by issue date from a `BTreeMap<NaiveDate, Decimal>`.
- **Date-based installments**: Each payment has a due date computed from `start_date` and `due_day` (1–28).
- **Internationalization (i18n)**: Error messages in English and Brazilian Portuguese via `rust-i18n`.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
br_financial = "0.5.0"
rust_decimal = "1.39.0"
rust_decimal_macros = "1.39.0"
chrono = "0.4"
```

## Usage

```rust
use std::collections::BTreeMap;
use br_financial::{calculate_debt_trajectory, DebtCalculationInput, DebtCalculationType, Locale};
use rust_decimal_macros::dec;
use chrono::NaiveDate;

fn main() {
    let input = DebtCalculationInput {
        total_amount: dec!(360_000),
        interest_per_year: dec!(10.5),
        down_payment_percent: dec!(5),
        total_months: 420,
        debt_type: DebtCalculationType::Sac,
        insurance_rate: dec!(0.0003),   // 0.03% of balance per month
        insurance_fee: dec!(25),         // R$ 25 fixed monthly insurance fee
        admin_fee: dec!(30),             // R$ 30 fixed monthly admin fee
        due_day: 15,
        start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        monthly_correction_rates: BTreeMap::new(),
        locale: Locale::En,
    };

    match calculate_debt_trajectory(input) {
        Ok(result) => {
            println!("Financed Amount:          R$ {:.2}", result.financed_amount);
            println!("Total Paid:               R$ {:.2}", result.table.total_paid);
            println!("Total Insurance:          R$ {:.2}", result.table.total_insurance);
            println!("Total Admin Fees:         R$ {:.2}", result.table.total_admin_fees);
            println!("Total Monetary Correction: R$ {:.2}", result.table.total_monetary_correction);
            println!("Number of Installments:   {}", result.table.amortization_curve.len());
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
```

## Monthly Calculation Flow

For each installment:

1. **Insurance**: `balance * insurance_rate + insurance_fee`
2. **Interest**: `balance * monthly_interest_rate`
3. **Amortization**: SAC (fixed) or Price (PMT - interest)
4. **Monetary correction**: `balance * correction_rate` (looked up from `BTreeMap`)
5. **Total payment**: amortization + interest + insurance + admin_fee + monetary_correction
6. **New balance**: balance + monetary_correction - amortization

## Input Parameters

| Parameter | Type | Description |
|---|---|---|
| `total_amount` | `Decimal` | Total loan principal |
| `interest_per_year` | `Decimal` | Annual interest rate as percentage (e.g., `10.5`) |
| `down_payment_percent` | `Decimal` | Down payment as percentage (e.g., `5` for 5%) |
| `total_months` | `u32` | Number of monthly installments |
| `debt_type` | `DebtCalculationType` | `Sac` or `Price` |
| `insurance_rate` | `Decimal` | Normalized monthly rate (e.g., `0.0003` for 0.03%) |
| `insurance_fee` | `Decimal` | Fixed monthly insurance fee |
| `admin_fee` | `Decimal` | Fixed monthly administration fee |
| `due_day` | `u8` | Installment due day (1–28) |
| `start_date` | `NaiveDate` | Financing start date |
| `monthly_correction_rates` | `BTreeMap<NaiveDate, Decimal>` | Correction rates by issue date (normalized) |
| `locale` | `Locale` | `Locale::En` or `Locale::PtBr` for error messages |

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a pull request.
