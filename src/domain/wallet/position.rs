use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::wallet::enums::TransactionType;
use crate::domain::wallet::transaction::Transaction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub stock_id: i32,
    pub quantity: f64,
    pub average_cost: f64,
    pub total_cost: f64,
    pub currency: String,
}

/// Compute open positions from a chronologically ordered list of transactions.
///
/// Uses weighted average cost method. Splits adjust quantity and average cost.
/// Returns only positions where quantity > 0.
pub fn compute_positions(transactions: &[Transaction]) -> Vec<Position> {
    let mut positions: HashMap<i32, (f64, f64, String)> = HashMap::new();
    // (quantity, total_cost_basis, currency)

    for tx in transactions {
        let stock_id = match tx.stock_id {
            Some(id) => id,
            None => continue,
        };

        let entry = positions
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
                    // Reduce cost basis proportionally (average cost method)
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
                    // Total cost stays the same, average cost adjusts
                }
            }
            _ => {}
        }
    }

    positions
        .into_iter()
        .filter(|(_, (qty, _, _))| *qty > 0.0)
        .map(|(stock_id, (quantity, total_cost, currency))| {
            let average_cost = if quantity > 0.0 {
                total_cost / quantity
            } else {
                0.0
            };
            Position {
                stock_id,
                quantity,
                average_cost,
                total_cost,
                currency,
            }
        })
        .collect()
}
