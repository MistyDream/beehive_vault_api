use crate::types::{AccountBalance, CurrencyCode};

pub struct NetWorthSummary {
    pub currency: CurrencyCode,
    pub assets: AccountBalance,
    pub liabilities: AccountBalance,
    pub net_worth: AccountBalance,
}
