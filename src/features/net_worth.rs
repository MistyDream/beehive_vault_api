use axum::{
    Json,
    extract::{Path, State},
};
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::{AppState, error::ApiError};

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct NetWorthResponse {
    currency: String,
    assets: Decimal,
    liabilities: Decimal,
    net_worth: Decimal,
}

#[derive(sqlx::FromRow)]
struct NetWorthAmounts {
    assets: Decimal,
    liabilities: Decimal,
    net_worth: Decimal,
}

pub async fn summary(
    State(state): State<AppState>,
    Path(household_id): Path<Uuid>,
) -> Result<Json<NetWorthResponse>, ApiError> {
    let currency: String = sqlx::query_scalar("SELECT base_currency FROM households WHERE id = $1")
        .bind(household_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Household not found".to_owned()))?;

    let amounts = sqlx::query_as::<_, NetWorthAmounts>(
        "WITH latest_balances AS ( \
            SELECT a.kind, COALESCE(balance.amount, 0::numeric) AS amount \
            FROM accounts a \
            LEFT JOIN LATERAL ( \
                SELECT amount FROM account_balance_snapshots \
                WHERE account_id = a.id ORDER BY balance_date DESC LIMIT 1 \
            ) balance ON true \
            WHERE a.household_id = $1 AND a.archived_at IS NULL \
        ), contributions AS ( \
            SELECT CASE WHEN kind IN ('credit_card', 'loan', 'other_liability') \
                THEN -amount ELSE amount END AS amount FROM latest_balances \
        ) SELECT \
            COALESCE(sum(greatest(amount, 0::numeric)), 0::numeric) AS assets, \
            COALESCE(sum(greatest(-amount, 0::numeric)), 0::numeric) AS liabilities, \
            COALESCE(sum(amount), 0::numeric) AS net_worth \
        FROM contributions",
    )
    .bind(household_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(NetWorthResponse {
        currency,
        assets: amounts.assets,
        liabilities: amounts.liabilities,
        net_worth: amounts.net_worth,
    }))
}
