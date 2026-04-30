use garde::Validate;
use serde::Deserialize;

#[derive(Debug, Deserialize, Validate)]
#[garde(context(()))]
pub struct StockSearchQuery {
    /// Optional: a missing `q` produces a 400 from the controller; a present
    /// but invalid `q` produces a 422 from this validator.
    #[garde(inner(length(chars, max = 50), custom(non_blank_min_2)))]
    pub q: Option<String>,
}

fn non_blank_min_2(value: &String, _: &()) -> garde::Result {
    if value.trim().chars().count() < 2 {
        return Err(garde::Error::new(
            "must contain at least 2 non-whitespace characters",
        ));
    }
    Ok(())
}
