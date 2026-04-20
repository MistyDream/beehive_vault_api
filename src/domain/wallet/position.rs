use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::market::stock::Stock;
use crate::domain::wallet::enums::TransactionType;
use crate::domain::wallet::transaction::Transaction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub stock: Stock,
    pub quantity: f64,
    pub average_cost: f64,
    pub total_cost: f64,
    pub currency: String,
    /// Share of the portfolio's total invested cost, as a fraction in [0.0, 1.0].
    pub weight: f64,
}

/// Compute open positions from a chronologically ordered list of transactions.
///
/// Uses weighted average cost method. Splits adjust quantity and average cost.
/// Returns only positions where quantity > 0 AND the stock is present in
/// `stocks_by_id`. Weight is normalized over the kept positions so the
/// returned weights always sum to 1.0 (or 0.0 if total_cost is zero).
pub fn compute_positions(
    transactions: &[Transaction],
    stocks_by_id: &HashMap<i32, Stock>,
) -> Vec<Position> {
    let mut aggregates: HashMap<i32, (f64, f64, String)> = HashMap::new();

    for tx in transactions {
        let stock_id = match tx.stock_id {
            Some(id) => id,
            None => continue,
        };

        let entry = aggregates
            .entry(stock_id)
            .or_insert((0.0, 0.0, tx.currency.clone()));

        match tx.transaction_type {
            TransactionType::Buy => {
                let qty = tx.quantity.unwrap_or(0.0);
                let price = tx.unit_price.unwrap_or(0.0);
                let cost = qty * price * tx.exchange_rate + tx.fees * tx.exchange_rate;
                entry.0 += qty;
                entry.1 += cost;
            }
            TransactionType::Sell => {
                let qty = tx.quantity.unwrap_or(0.0);
                if entry.0 > 0.0 {
                    let avg = entry.1 / entry.0;
                    entry.0 -= qty;
                    entry.1 = entry.0 * avg;
                    if entry.0 < 0.0 {
                        entry.0 = 0.0;
                        entry.1 = 0.0;
                    }
                }
            }
            TransactionType::Split => {
                let from = tx.split_from.unwrap_or(1) as f64;
                let to = tx.split_to.unwrap_or(1) as f64;
                if from > 0.0 {
                    let ratio = to / from;
                    entry.0 *= ratio;
                }
            }
            _ => {}
        }
    }

    let kept: Vec<(i32, f64, f64, String)> = aggregates
        .into_iter()
        .filter(|(_, (qty, _, _))| *qty > 0.0)
        .filter(|(stock_id, _)| stocks_by_id.contains_key(stock_id))
        .map(|(stock_id, (qty, cost, currency))| (stock_id, qty, cost, currency))
        .collect();

    let total_cost_sum: f64 = kept.iter().map(|(_, _, cost, _)| *cost).sum();

    let mut positions: Vec<Position> = kept
        .into_iter()
        .map(|(stock_id, quantity, total_cost, currency)| {
            let stock = stocks_by_id[&stock_id].clone();
            let average_cost = if quantity > 0.0 { total_cost / quantity } else { 0.0 };
            let weight = if total_cost_sum > 0.0 {
                total_cost / total_cost_sum
            } else {
                0.0
            };
            Position {
                stock,
                quantity,
                average_cost,
                total_cost,
                currency,
                weight,
            }
        })
        .collect();

    positions.sort_by(|a, b| a.stock.id.cmp(&b.stock.id));
    positions
}
