mod repository;

pub use repository::{DemoError, DemoRepository};

pub const DEMO_HOUSEHOLD_ID: &str = "019d9000-0000-7000-8000-000000000001";
pub const DEMO_CHECKING_ACCOUNT_ID: &str = "019d9000-0000-7000-8000-000000000015";
pub const DEMO_STATEMENT: &str = include_str!("../../../demo/statement.csv");
