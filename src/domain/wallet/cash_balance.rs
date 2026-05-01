use serde::{Deserialize, Serialize};

use crate::domain::wallet::enums::TransactionType;
use crate::domain::wallet::transaction::Transaction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashBalance {
    pub currency: String,
    pub balance: f64,
}

/// Compute cash balances from a list of transactions.
///
/// All amounts are converted to the portfolio's base currency using
/// the exchange_rate stored on each transaction.
/// Returns a single CashBalance in the portfolio currency.
pub fn compute_cash_balance(transactions: &[Transaction], portfolio_currency: &str) -> CashBalance {
    let mut balance = 0.0;

    for tx in transactions {
        let rate = tx.exchange_rate;

        match tx.transaction_type {
            TransactionType::Deposit => {
                balance += tx.amount.unwrap_or(0.0) * rate;
            }
            TransactionType::Withdrawal => {
                balance -= tx.amount.unwrap_or(0.0) * rate;
            }
            TransactionType::Buy => {
                let cost = tx.quantity.unwrap_or(0.0) * tx.unit_price.unwrap_or(0.0) * rate;
                let fees = tx.fees * rate;
                balance -= cost + fees;
            }
            TransactionType::Sell => {
                let proceeds = tx.quantity.unwrap_or(0.0) * tx.unit_price.unwrap_or(0.0) * rate;
                let fees = tx.fees * rate;
                balance += proceeds - fees;
            }
            TransactionType::Dividend => {
                let amount = tx.amount.unwrap_or(0.0) * rate;
                let tax = tx.tax * rate;
                balance += amount - tax;
            }
            TransactionType::Fee => {
                balance -= tx.amount.unwrap_or(0.0) * rate;
            }
            TransactionType::Split => {
                // Splits don't affect cash
            }
        }
    }

    CashBalance {
        currency: portfolio_currency.to_owned(),
        balance,
    }
}
