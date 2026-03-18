use serde::Serialize;

#[derive(Serialize)]
pub struct ImportGurufocusResponse {
    pub imported: usize,
}
