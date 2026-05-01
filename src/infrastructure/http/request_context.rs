tokio::task_local! {
    pub static REQUEST_PATH: String;
}

pub fn current_path() -> Option<String> {
    REQUEST_PATH.try_with(|p| p.clone()).ok()
}
