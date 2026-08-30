use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgConnection;

use crate::{
    database::Database,
    types::{
        AccountBalance, AccountId, BalanceSnapshotId, CurrencyCode, HouseholdId, InstitutionId,
        TimeZoneId,
    },
};

use super::domain::{
    Account, AccountKind, BalanceSnapshot, BalanceSource, NewAccount, NewBalanceSnapshot,
};

macro_rules! account_select {
    ($suffix:literal) => {
        concat!(
            "SELECT a.id, a.household_id, a.institution_id, a.name, a.kind, a.currency, ",
            "latest.amount AS latest_balance, latest.balance_date, ",
            "COALESCE(latest.amount, 0::numeric) + COALESCE(activity.amount, 0::numeric) ",
            "AS calculated_balance, a.archived_at, a.created_at, a.updated_at ",
            "FROM accounts a ",
            "LEFT JOIN LATERAL (",
            "SELECT amount, balance_date FROM account_balance_snapshots ",
            "WHERE account_id = a.id ORDER BY balance_date DESC LIMIT 1",
            ") latest ON true ",
            "LEFT JOIN LATERAL (",
            "SELECT sum(amount) AS amount FROM transactions ",
            "WHERE account_id = a.id AND deleted_at IS NULL ",
            "AND (latest.balance_date IS NULL OR booking_date > latest.balance_date)",
            ") activity ON true ",
            $suffix
        )
    };
}

#[derive(Clone)]
pub struct AccountRepository {
    database: Database,
}

pub enum CreateBalanceOutcome {
    Created(BalanceSnapshot),
    NotAfterLatest,
}

impl AccountRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

#[derive(sqlx::FromRow)]
struct AccountRow {
    id: AccountId,
    household_id: HouseholdId,
    institution_id: Option<InstitutionId>,
    name: String,
    kind: String,
    currency: CurrencyCode,
    latest_balance: Option<AccountBalance>,
    balance_date: Option<NaiveDate>,
    calculated_balance: AccountBalance,
    archived_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<AccountRow> for Account {
    type Error = sqlx::Error;

    fn try_from(row: AccountRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            household_id: row.household_id,
            institution_id: row.institution_id,
            name: row.name,
            kind: AccountKind::try_from(row.kind.as_str())
                .map_err(|error| sqlx::Error::Decode(Box::new(error)))?,
            currency: row.currency,
            latest_balance: row.latest_balance,
            balance_date: row.balance_date,
            calculated_balance: row.calculated_balance,
            archived_at: row.archived_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct BalanceRow {
    id: BalanceSnapshotId,
    account_id: AccountId,
    amount: AccountBalance,
    balance_date: NaiveDate,
    source: String,
    created_at: DateTime<Utc>,
}

impl TryFrom<BalanceRow> for BalanceSnapshot {
    type Error = sqlx::Error;

    fn try_from(row: BalanceRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            account_id: row.account_id,
            amount: row.amount,
            balance_date: row.balance_date,
            source: BalanceSource::try_from(row.source.as_str())
                .map_err(|error| sqlx::Error::Decode(Box::new(error)))?,
            created_at: row.created_at,
        })
    }
}

impl AccountRepository {
    pub async fn household_currency(
        &self,
        household_id: HouseholdId,
    ) -> Result<Option<CurrencyCode>, sqlx::Error> {
        sqlx::query_scalar("SELECT base_currency FROM households WHERE id = $1")
            .bind(household_id)
            .fetch_optional(self.database.pool())
            .await
    }

    pub async fn household_timezone(
        &self,
        household_id: HouseholdId,
    ) -> Result<Option<TimeZoneId>, sqlx::Error> {
        let timezone =
            sqlx::query_scalar::<_, String>("SELECT timezone FROM households WHERE id = $1")
                .bind(household_id)
                .fetch_optional(self.database.pool())
                .await?;
        timezone
            .map(TimeZoneId::new)
            .transpose()
            .map_err(|error| sqlx::Error::Decode(Box::new(error)))
    }

    pub async fn institution_exists(
        &self,
        institution_id: InstitutionId,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM institutions WHERE id = $1)")
            .bind(institution_id)
            .fetch_one(self.database.pool())
            .await
    }

    pub async fn create(
        &self,
        account: NewAccount,
        initial_balance: NewBalanceSnapshot,
    ) -> Result<(), sqlx::Error> {
        let mut transaction = self.database.begin_transaction().await?;
        Self::insert_account(&mut transaction, account).await?;
        Self::insert_balance(&mut transaction, initial_balance).await?;
        transaction.commit().await
    }

    async fn insert_account(
        connection: &mut PgConnection,
        account: NewAccount,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO accounts (id, household_id, institution_id, name, kind, currency) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(account.id)
        .bind(account.household_id)
        .bind(account.institution_id)
        .bind(account.name)
        .bind(account.kind.as_str())
        .bind(account.currency)
        .execute(connection)
        .await?;
        Ok(())
    }

    async fn insert_balance(
        connection: &mut PgConnection,
        balance: NewBalanceSnapshot,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO account_balance_snapshots \
             (id, account_id, amount, balance_date, source) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(balance.id)
        .bind(balance.account_id)
        .bind(balance.amount)
        .bind(balance.balance_date)
        .bind(balance.source.as_str())
        .execute(connection)
        .await?;
        Ok(())
    }

    pub async fn list(&self, household_id: HouseholdId) -> Result<Vec<Account>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AccountRow>(account_select!(
            "WHERE a.household_id = $1 AND a.archived_at IS NULL ORDER BY lower(a.name)"
        ))
        .bind(household_id)
        .fetch_all(self.database.pool())
        .await?;
        rows.into_iter().map(Account::try_from).collect()
    }

    pub async fn find(
        &self,
        household_id: HouseholdId,
        account_id: AccountId,
    ) -> Result<Option<Account>, sqlx::Error> {
        let row = sqlx::query_as::<_, AccountRow>(account_select!(
            "WHERE a.household_id = $1 AND a.id = $2 AND a.archived_at IS NULL"
        ))
        .bind(household_id)
        .bind(account_id)
        .fetch_optional(self.database.pool())
        .await?;
        row.map(Account::try_from).transpose()
    }

    pub async fn find_including_archived(
        &self,
        household_id: HouseholdId,
        account_id: AccountId,
    ) -> Result<Option<Account>, sqlx::Error> {
        let row = sqlx::query_as::<_, AccountRow>(account_select!(
            "WHERE a.household_id = $1 AND a.id = $2"
        ))
        .bind(household_id)
        .bind(account_id)
        .fetch_optional(self.database.pool())
        .await?;
        row.map(Account::try_from).transpose()
    }

    pub async fn has_transactions(
        &self,
        household_id: HouseholdId,
        account_id: AccountId,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM transactions \
             WHERE household_id = $1 AND account_id = $2)",
        )
        .bind(household_id)
        .bind(account_id)
        .fetch_one(self.database.pool())
        .await
    }

    pub async fn update(
        &self,
        household_id: HouseholdId,
        account_id: AccountId,
        name: Option<&str>,
        kind: Option<AccountKind>,
        institution_id: Option<InstitutionId>,
        remove_institution: bool,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE accounts SET \
         name = COALESCE($3, name), kind = COALESCE($4, kind), \
         institution_id = CASE WHEN $6 THEN NULL ELSE COALESCE($5, institution_id) END, \
         updated_at = now() WHERE household_id = $1 AND id = $2 AND archived_at IS NULL",
        )
        .bind(household_id)
        .bind(account_id)
        .bind(name)
        .bind(kind.map(AccountKind::as_str))
        .bind(institution_id)
        .bind(remove_institution)
        .execute(self.database.pool())
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn archive(
        &self,
        household_id: HouseholdId,
        account_id: AccountId,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE accounts SET archived_at = now(), updated_at = now() \
         WHERE household_id = $1 AND id = $2 AND archived_at IS NULL",
        )
        .bind(household_id)
        .bind(account_id)
        .execute(self.database.pool())
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn create_balance(
        &self,
        snapshot_id: BalanceSnapshotId,
        account_id: AccountId,
        amount: AccountBalance,
        balance_date: NaiveDate,
        source: BalanceSource,
    ) -> Result<CreateBalanceOutcome, sqlx::Error> {
        let mut transaction = self.database.begin_transaction().await?;
        sqlx::query_scalar::<_, AccountId>("SELECT id FROM accounts WHERE id = $1 FOR UPDATE")
            .bind(account_id)
            .fetch_one(&mut *transaction)
            .await?;
        let latest_balance_date = sqlx::query_scalar::<_, Option<NaiveDate>>(
            "SELECT max(balance_date) FROM account_balance_snapshots WHERE account_id = $1",
        )
        .bind(account_id)
        .fetch_one(&mut *transaction)
        .await?;
        if latest_balance_date.is_some_and(|latest| balance_date <= latest) {
            transaction.rollback().await?;
            return Ok(CreateBalanceOutcome::NotAfterLatest);
        }

        let row = sqlx::query_as::<_, BalanceRow>(
            "INSERT INTO account_balance_snapshots (id, account_id, amount, balance_date, source) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING id, account_id, amount, balance_date, source, created_at",
        )
        .bind(snapshot_id)
        .bind(account_id)
        .bind(amount)
        .bind(balance_date)
        .bind(source.as_str())
        .fetch_one(&mut *transaction)
        .await?;
        let balance = BalanceSnapshot::try_from(row)?;
        transaction.commit().await?;
        Ok(CreateBalanceOutcome::Created(balance))
    }

    pub async fn list_balances(
        &self,
        account_id: AccountId,
    ) -> Result<Vec<BalanceSnapshot>, sqlx::Error> {
        let rows = sqlx::query_as::<_, BalanceRow>(
            "SELECT id, account_id, amount, balance_date, source, created_at \
         FROM account_balance_snapshots WHERE account_id = $1 \
         ORDER BY balance_date DESC",
        )
        .bind(account_id)
        .fetch_all(self.database.pool())
        .await?;
        rows.into_iter().map(BalanceSnapshot::try_from).collect()
    }

    pub async fn update_balance(
        &self,
        household_id: HouseholdId,
        account_id: AccountId,
        balance_id: BalanceSnapshotId,
        amount: Option<AccountBalance>,
        balance_date: Option<NaiveDate>,
    ) -> Result<Option<BalanceSnapshot>, sqlx::Error> {
        let mut transaction = self.database.begin_transaction().await?;
        let account_exists = sqlx::query_scalar::<_, AccountId>(
            "SELECT id FROM accounts \
             WHERE household_id = $1 AND id = $2 AND archived_at IS NULL FOR UPDATE",
        )
        .bind(household_id)
        .bind(account_id)
        .fetch_optional(&mut *transaction)
        .await?
        .is_some();
        if !account_exists {
            transaction.rollback().await?;
            return Ok(None);
        }

        let row = sqlx::query_as::<_, BalanceRow>(
            "UPDATE account_balance_snapshots AS balance SET \
             amount = COALESCE($4, balance.amount), \
             balance_date = COALESCE($5, balance.balance_date) \
             FROM accounts AS account \
             WHERE balance.id = $3 AND balance.account_id = account.id \
               AND account.id = $2 AND account.household_id = $1 \
               AND account.archived_at IS NULL \
             RETURNING balance.id, balance.account_id, balance.amount, \
                       balance.balance_date, balance.source, balance.created_at",
        )
        .bind(household_id)
        .bind(account_id)
        .bind(balance_id)
        .bind(amount)
        .bind(balance_date)
        .fetch_optional(&mut *transaction)
        .await?;
        let balance = row.map(BalanceSnapshot::try_from).transpose()?;
        transaction.commit().await?;
        Ok(balance)
    }
}
