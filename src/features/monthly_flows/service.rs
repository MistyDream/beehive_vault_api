use crate::{
    error::{ApiError, ProblemKind},
    features::{households::HouseholdRepository, transactions::domain::TransactionNature},
    types::HouseholdId,
};

use super::{
    domain::{Month, MonthlyFlowReport, MonthlyFlowSection},
    repository::MonthlyFlowRepository,
};

#[derive(Clone)]
pub struct MonthlyFlowService {
    repository: MonthlyFlowRepository,
    household_repository: HouseholdRepository,
}

impl MonthlyFlowService {
    pub fn new(
        repository: MonthlyFlowRepository,
        household_repository: HouseholdRepository,
    ) -> Self {
        Self {
            repository,
            household_repository,
        }
    }

    pub async fn get(
        &self,
        household_id: HouseholdId,
        month: Month,
    ) -> Result<MonthlyFlowReport, ApiError> {
        let household = self
            .household_repository
            .find(household_id)
            .await?
            .ok_or_else(|| ApiError::new(ProblemKind::HouseholdNotFound))?;
        let groups = self
            .repository
            .groups(
                household_id,
                month.first_day(),
                month.next_month_first_day(),
            )
            .await?;
        let income = MonthlyFlowSection::from_groups(
            groups
                .iter()
                .filter(|group| group.nature == TransactionNature::Income)
                .cloned(),
        );
        let expenses = MonthlyFlowSection::from_groups(
            groups
                .iter()
                .filter(|group| group.nature == TransactionNature::Expense)
                .cloned(),
        );
        let net_flow = income.total - expenses.total;

        Ok(MonthlyFlowReport {
            month,
            currency: household.base_currency,
            income,
            expenses,
            net_flow,
        })
    }
}
