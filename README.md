# br_financial

[![Crates.io](https://img.shields.io/crates/v/br_financial.svg)](https://crates.io/crates/br_financial)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Docs.rs](https://docs.rs/br_financial/badge.svg)](https://docs.rs/br_financial)

A library to calculate real estate financing in Brazil.

## Description

This library provides tools to calculate the installments of real estate financing using the two main amortization systems in Brazil: SAC (Sistema de Amortização Constante) and Price (Sistema Francês de Amortização).

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
br_financial = "0.1.0"
```

And then in your code:

```rust
// Example usage will be added here.
// For now, please refer to the documentation on docs.rs.


fn main() {
    let input = DebtCalculationInput {
        total_amount: dec!(360_000),
        interest_per_year: dec!(10.5),
        total_months: 420,
    };

    match calculate_debt_trajectory(input) {
        Ok(result) => {
            println!("SAC First Payment: {:.2}", result.sac_table.first_payment);
            println!("SAC Last Payment:  {:.2}", result.sac_table.last_payment);
            println!("SAC Total Paid:    {:.2}", result.sac_table.total_paid);

            println!("Price Fixed Payment: {:.2}", result.price_table.fixed_payment);
            println!("Price Total Paid:    {:.2}", result.price_table.total_paid);
        }
        Err(e) => {
            eprintln!("Error calculating debt trajectory: {}", e);
        }
    }
}
```

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a pull request.
