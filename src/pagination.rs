use serde::{Deserialize, Deserializer, de::Error as _};

const DEFAULT_LIMIT: i64 = 50;
const MAXIMUM_LIMIT: i64 = 200;
const DEFAULT_PAGE: i64 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pagination {
    limit: i64,
    page: i64,
    offset: i64,
}

impl Pagination {
    pub fn new(limit: Option<i64>, page: Option<i64>) -> Result<Self, PaginationError> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT);
        if !(1..=MAXIMUM_LIMIT).contains(&limit) {
            return Err(PaginationError::InvalidLimit);
        }

        let page = page.unwrap_or(DEFAULT_PAGE);
        if page < 1 {
            return Err(PaginationError::InvalidPage);
        }
        let offset = page
            .checked_sub(1)
            .and_then(|page_index| page_index.checked_mul(limit))
            .ok_or(PaginationError::PageTooLarge)?;

        Ok(Self {
            limit,
            page,
            offset,
        })
    }

    pub fn limit(self) -> i64 {
        self.limit
    }

    pub fn page(self) -> i64 {
        self.page
    }

    pub fn offset(self) -> i64 {
        self.offset
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PaginationError {
    #[error("limit must contain a value between 1 and 200")]
    InvalidLimit,
    #[error("page must be greater than or equal to one")]
    InvalidPage,
    #[error("page is too large")]
    PageTooLarge,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
pub struct PaginationQuery {
    #[serde(default, deserialize_with = "deserialize_optional_integer")]
    limit: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_integer")]
    page: Option<i64>,
}

impl TryFrom<PaginationQuery> for Pagination {
    type Error = PaginationError;

    fn try_from(query: PaginationQuery) -> Result<Self, Self::Error> {
        Self::new(query.limit, query.page)
    }
}

fn deserialize_optional_integer<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)?
        .map(|value| value.parse().map_err(D::Error::custom))
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_pagination_starts_on_the_first_page() {
        let pagination = Pagination::new(None, None).unwrap();

        assert_eq!(pagination.limit(), 50);
        assert_eq!(pagination.page(), 1);
        assert_eq!(pagination.offset(), 0);
    }

    #[test]
    fn page_is_converted_to_a_database_offset() {
        let pagination = Pagination::new(Some(25), Some(3)).unwrap();

        assert_eq!(pagination.offset(), 50);
    }

    #[test]
    fn invalid_pagination_is_rejected() {
        assert_eq!(
            Pagination::new(Some(201), None).unwrap_err(),
            PaginationError::InvalidLimit
        );
        assert_eq!(
            Pagination::new(None, Some(0)).unwrap_err(),
            PaginationError::InvalidPage
        );
        assert_eq!(
            Pagination::new(Some(200), Some(i64::MAX)).unwrap_err(),
            PaginationError::PageTooLarge
        );
    }
}
