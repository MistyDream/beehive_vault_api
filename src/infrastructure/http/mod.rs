pub mod controllers;
pub mod dto;
pub mod error;
pub mod etag;
pub mod request_context;
pub mod routes;
pub mod server;
pub mod state;

pub const PRIVATE_SHORT_CACHE: &str = "private, max-age=30";
