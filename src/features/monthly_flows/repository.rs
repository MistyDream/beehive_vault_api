use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::{
    database::Database,
    features::transactions::domain::TransactionNature,
    types::{CategoryId, HouseholdId},
};

use super::domain::MonthlyFlowGroup;

#[derive(Clone)]
pub struct MonthlyFlowRepository {
    database: Database,
}

impl MonthlyFlowRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn groups(
        &self,
        household_id: HouseholdId,
        date_from: NaiveDate,
        date_until: NaiveDate,
    ) -> Result<Vec<MonthlyFlowGroup>, sqlx::Error> {
        let rows = sqlx::query_as::<_, MonthlyFlowGroupRow>(
            "SELECT t.nature, t.category_id, c.name AS category_name, \
             SUM(CASE \
                 WHEN t.nature = 'income' THEN \
                     CASE WHEN a.kind IN ('credit_card', 'loan', 'other_liability') \
                         THEN -t.amount ELSE t.amount END \
                 ELSE \
                     CASE WHEN a.kind IN ('credit_card', 'loan', 'other_liability') \
                         THEN t.amount ELSE -t.amount END \
             END) AS amount, COUNT(*) AS transaction_count \
             FROM transactions t \
             JOIN accounts a ON a.household_id = t.household_id AND a.id = t.account_id \
             LEFT JOIN categories c \
                 ON c.household_id = t.household_id AND c.id = t.category_id \
             WHERE t.household_id = $1 AND t.deleted_at IS NULL \
               AND t.booking_date >= $2 AND t.booking_date < $3 \
               AND t.nature IN ('income', 'expense') \
             GROUP BY t.nature, t.category_id, c.name",
        )
        .bind(household_id)
        .bind(date_from)
        .bind(date_until)
        .fetch_all(self.database.pool())
        .await?;

        rows.into_iter().map(MonthlyFlowGroup::try_from).collect()
    }
}

#[derive(sqlx::FromRow)]
struct MonthlyFlowGroupRow {
    nature: String,
    category_id: Option<CategoryId>,
    category_name: Option<String>,
    amount: Decimal,
    transaction_count: i64,
}

impl TryFrom<MonthlyFlowGroupRow> for MonthlyFlowGroup {
    type Error = sqlx::Error;

    fn try_from(row: MonthlyFlowGroupRow) -> Result<Self, Self::Error> {
        Ok(Self {
            nature: TransactionNature::try_from(row.nature.as_str())
                .map_err(|error| sqlx::Error::Decode(Box::new(error)))?,
            category_id: row.category_id,
            category_name: row.category_name,
            amount: row.amount,
            transaction_count: row.transaction_count,
        })
    }
}
