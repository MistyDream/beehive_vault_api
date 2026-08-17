use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum FieldUpdate<T> {
    #[default]
    Unchanged,
    Set(T),
    Clear,
}

impl<'de, T> Deserialize<'de> for FieldUpdate<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<T>::deserialize(deserializer).map(|value| match value {
            Some(value) => Self::Set(value),
            None => Self::Clear,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    struct UpdateRequest {
        #[serde(default)]
        note: FieldUpdate<String>,
    }

    #[test]
    fn missing_field_is_unchanged() {
        let request: UpdateRequest = serde_json::from_str("{}").unwrap();

        assert_eq!(request.note, FieldUpdate::Unchanged);
    }

    #[test]
    fn null_field_is_cleared() {
        let request: UpdateRequest = serde_json::from_str(r#"{"note":null}"#).unwrap();

        assert_eq!(request.note, FieldUpdate::Clear);
    }

    #[test]
    fn provided_field_is_set() {
        let request: UpdateRequest = serde_json::from_str(r#"{"note":"Updated"}"#).unwrap();

        assert_eq!(request.note, FieldUpdate::Set("Updated".to_owned()));
    }
}
