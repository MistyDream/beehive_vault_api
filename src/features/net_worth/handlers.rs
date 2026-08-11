use axum::{
    Json,
    extract::{Path, State},
};
use serde::Serialize;

use crate::{
    error::ApiError,
    types::{AccountBalance, CurrencyCode, HouseholdId},
};

use super::NetWorthModule;

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub(super) struct NetWorthResponse {
    currency: CurrencyCode,
    assets: AccountBalance,
    liabilities: AccountBalance,
    net_worth: AccountBalance,
}

#[derive(sqlx::FromRow)]
struct NetWorthAmounts {
    assets: AccountBalance,
    liabilities: AccountBalance,
    net_worth: AccountBalance,
}

pub(super) async fn summary(
    State(module): State<NetWorthModule>,
    Path(household_id): Path<HouseholdId>,
) -> Result<Json<NetWorthResponse>, ApiError> {
    let currency: CurrencyCode =
        sqlx::query_scalar("SELECT base_currency FROM households WHERE id = $1")
            .bind(household_id)
            .fetch_optional(module.database.pool())
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
    .fetch_one(module.database.pool())
    .await?;

    Ok(Json(NetWorthResponse {
        currency,
        assets: amounts.assets,
        liabilities: amounts.liabilities,
        net_worth: amounts.net_worth,
    }))
}
