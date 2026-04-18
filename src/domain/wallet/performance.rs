use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::wallet::enums::TransactionType;
use crate::domain::wallet::transaction::Transaction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub portfolio_id: i32,
    pub currency: String,
    pub total_deposited: f64,
    pub total_withdrawn: f64,
    pub realized_pnl: f64,
    pub dividends_received: f64,
    pub fees_paid: f64,
    pub taxes_paid: f64,
    /// Net cash flow: deposits - withdrawals + realized gains + dividends - fees - taxes
    pub net_result: f64,
}

/// Compute a performance report from a chronologically ordered list of transactions.
///
/// Currently computes realized P&L, dividends, fees, and taxes.
/// Unrealized P&L and TWR/MWR require current market prices (future enhancement).
pub fn compute_performance(
    portfolio_id: i32,
    portfolio_currency: &str,
    transactions: &[Transaction],
) -> PerformanceReport {
    let mut total_deposited = 0.0;
    let mut total_withdrawn = 0.0;
    let mut dividends_received = 0.0;
    let mut fees_paid = 0.0;
    let mut taxes_paid = 0.0;

    // Track cost basis per stock for realized P&L (average cost method)
    let mut holdings: HashMap<i32, (f64, f64)> = HashMap::new(); // (qty, total_cost)
    let mut realized_pnl = 0.0;

    for tx in transactions {
        let rate = tx.exchange_rate;

        match tx.transaction_type {
            TransactionType::Deposit => {
                total_deposited += tx.amount.unwrap_or(0.0) * rate;
            }
            TransactionType::Withdrawal => {
                total_withdrawn += tx.amount.unwrap_or(0.0) * rate;
            }
            TransactionType::Buy => {
                if let Some(stock_id) = tx.stock_id {
                    let qty = tx.quantity.unwrap_or(0.0);
                    let price = tx.unit_price.unwrap_or(0.0);
                    let cost = qty * price * rate;
                    let fee = tx.fees * rate;

                    let entry = holdings.entry(stock_id).or_insert((0.0, 0.0));
                    entry.0 += qty;
                    entry.1 += cost + fee;

                    fees_paid += fee;
                }
            }
            TransactionType::Sell => {
                if let Some(stock_id) = tx.stock_id {
                    let qty = tx.quantity.unwrap_or(0.0);
                    let price = tx.unit_price.unwrap_or(0.0);
                    let proceeds = qty * price * rate;
                    let fee = tx.fees * rate;

                    let entry = holdings.entry(stock_id).or_insert((0.0, 0.0));
                    if entry.0 > 0.0 {
                        let avg_cost = entry.1 / entry.0;
                        let cost_basis = qty * avg_cost;
                        realized_pnl += proceeds - cost_basis - fee;

                        entry.0 -= qty;
                        entry.1 = entry.0 * avg_cost;
                        if entry.0 < 0.0 {
                            entry.0 = 0.0;
                            entry.1 = 0.0;
                        }
                    }

                    fees_paid += fee;
                }
            }
            TransactionType::Dividend => {
                let amount = tx.amount.unwrap_or(0.0) * rate;
                let tax = tx.tax * rate;
                dividends_received += amount;
                taxes_paid += tax;
            }
            TransactionType::Fee => {
                fees_paid += tx.amount.unwrap_or(0.0) * rate;
            }
            TransactionType::Split => {
                if let Some(stock_id) = tx.stock_id {
                    let from = tx.split_from.unwrap_or(1) as f64;
                    let to = tx.split_to.unwrap_or(1) as f64;
                    if from > 0.0 {
                        let entry = holdings.entry(stock_id).or_insert((0.0, 0.0));
                        entry.0 *= to / from;
                    }
                }
            }
        }
    }

    let net_result = realized_pnl + dividends_received - fees_paid - taxes_paid;

    PerformanceReport {
        portfolio_id,
        currency: portfolio_currency.to_owned(),
        total_deposited,
        total_withdrawn,
        realized_pnl,
        dividends_received,
        fees_paid,
        taxes_paid,
        net_result,
    }
}
