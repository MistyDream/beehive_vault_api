use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::market::price::Price;
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
    /// Latest persisted close price in the stock's native currency. `None`
    /// when no market data is available (graceful degradation: the wallet
    /// keeps working when prices are missing).
    pub current_price: Option<f64>,
    /// `quantity * current_price`, expressed in the stock's native currency.
    /// `None` when the price is missing.
    pub current_value: Option<f64>,
    /// `current_value - total_cost`. `None` when the price is missing.
    ///
    /// KNOWN LIMITATION: `current_value` is in the stock's currency while
    /// `total_cost` has already been converted to the portfolio's base
    /// currency at transaction time. For a multi-currency portfolio these
    /// are different units and the subtraction is not meaningful. Tracked
    /// separately on the backend roadmap — valuation currency conversion.
    pub unrealized_pnl: Option<f64>,
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
                current_price: None,
                current_value: None,
                unrealized_pnl: None,
            }
        })
        .collect();

    positions.sort_by(|a, b| a.stock.id.cmp(&b.stock.id));
    positions
}

/// Enrich positions in place with the latest close price per stock.
/// Positions whose stock is missing from `prices_by_stock_id` keep their
/// `current_price` / `current_value` / `unrealized_pnl` at `None`.
pub fn valorize_positions(positions: &mut [Position], prices_by_stock_id: &HashMap<i32, Price>) {
    for position in positions.iter_mut() {
        let Some(price) = prices_by_stock_id.get(&position.stock.id) else {
            continue;
        };
        // `to_f64` only returns `None` for Decimal values outside f64 range,
        // which no realistic stock close can reach. The position stays unvalued
        // in that paranoid branch and `PortfolioSummary.positions_without_price`
        // will still surface it to the caller.
        let Some(close) = price.close.to_f64() else {
            continue;
        };
        let value = position.quantity * close;
        position.current_price = Some(close);
        position.current_value = Some(value);
        position.unrealized_pnl = Some(value - position.total_cost);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::market::enums::MarketRegion;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn stock(id: i32, symbol: &str) -> Stock {
        Stock {
            id,
            symbol: symbol.to_string(),
            name: format!("Stock {id}"),
            isin: format!("ISIN{id:04}"),
            currency: "EUR".to_string(),
            market_region: MarketRegion::Europe,
            market: None,
            sector: None,
            industry: None,
            country: None,
        }
    }

    fn position(stock: Stock, quantity: f64, total_cost: f64) -> Position {
        Position {
            stock,
            quantity,
            average_cost: if quantity > 0.0 { total_cost / quantity } else { 0.0 },
            total_cost,
            currency: "EUR".to_string(),
            weight: 1.0,
            current_price: None,
            current_value: None,
            unrealized_pnl: None,
        }
    }

    fn price(stock_id: i32, close: &str) -> Price {
        Price {
            id: stock_id as i64,
            stock_id,
            price_date: NaiveDate::from_ymd_opt(2026, 4, 24).unwrap(),
            close: Decimal::from_str(close).unwrap(),
            source: "yahoo".to_string(),
            fetched_at: NaiveDate::from_ymd_opt(2026, 4, 24)
                .unwrap()
                .and_hms_opt(12, 0, 0)
                .unwrap(),
        }
    }

    #[test]
    fn valorizes_all_positions_when_every_price_is_present() {
        // 10 shares @ cost 100 EUR (1000 cost basis), latest close 130 → value 1300, pnl +300.
        let mut positions = vec![position(stock(1, "AAA"), 10.0, 1000.0)];
        let mut prices = HashMap::new();
        prices.insert(1, price(1, "130.00"));

        valorize_positions(&mut positions, &prices);

        let p = &positions[0];
        assert_eq!(p.current_price, Some(130.0));
        assert_eq!(p.current_value, Some(1300.0));
        assert_eq!(p.unrealized_pnl, Some(300.0));
    }

    #[test]
    fn leaves_unpriced_positions_untouched() {
        // Two positions, only one has a price — graceful degradation.
        let mut positions = vec![
            position(stock(1, "AAA"), 10.0, 1000.0),
            position(stock(2, "BBB"), 5.0, 500.0),
        ];
        let mut prices = HashMap::new();
        prices.insert(1, price(1, "110.00"));

        valorize_positions(&mut positions, &prices);

        assert_eq!(positions[0].current_value, Some(1100.0));
        assert_eq!(positions[1].current_value, None);
        assert_eq!(positions[1].current_price, None);
        assert_eq!(positions[1].unrealized_pnl, None);
    }

    #[test]
    fn empty_price_map_leaves_all_fields_at_none() {
        let mut positions = vec![position(stock(1, "AAA"), 10.0, 1000.0)];
        valorize_positions(&mut positions, &HashMap::new());
        assert!(positions[0].current_price.is_none());
        assert!(positions[0].current_value.is_none());
        assert!(positions[0].unrealized_pnl.is_none());
    }

    #[test]
    fn computes_negative_pnl_when_close_is_below_cost() {
        // 4 shares @ 50 cost (total 200), close 40 → value 160, pnl -40.
        let mut positions = vec![position(stock(1, "AAA"), 4.0, 200.0)];
        let mut prices = HashMap::new();
        prices.insert(1, price(1, "40.00"));

        valorize_positions(&mut positions, &prices);

        assert_eq!(positions[0].current_value, Some(160.0));
        assert_eq!(positions[0].unrealized_pnl, Some(-40.0));
    }
}
