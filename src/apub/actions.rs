pub mod actors;

pub use actors::*;

use reqwest::Client;
use crate::errors::{ActionResult, ActionError};

// Name your user agent after your app?
static APP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
);

pub fn get_client() -> ActionResult<Client> {
    reqwest::ClientBuilder::new()
        .user_agent(APP_USER_AGENT)
        .danger_accept_invalid_certs(true)
        .build()
        .or(Err(ActionError::FetchError))
}