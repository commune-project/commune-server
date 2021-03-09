pub mod actors;
pub mod inbox;
pub mod auth;

use warp;
use warp::Filter;
use warp::Rejection;
use crate::state::AppState;
use crate::errors::ActionError;
use std::sync::Arc;
use serde::de::DeserializeOwned;
use bytes::{Bytes, Buf};

pub async fn set_domain(
    app_state: Arc<AppState>,
    host_authority: Option<warp::filters::host::Authority>,
) -> Result<String, warp::Rejection> {
    if let Some(authority) = host_authority {
        let domain = String::from(authority.host());
        if (&app_state.local_domains).contains(&domain) {
            Ok(domain)
        } else {
            Err(warp::reject::not_found())
        }
    } else {
        Err(warp::reject::not_found())
    }
}

pub fn map_content_type_ap<T: warp::reply::Reply>(reply: T) -> warp::reply::WithHeader<T> {
    warp::reply::with_header(reply, "Content-Type", "application/activity+json")
}

pub fn activity_json<T: DeserializeOwned + Send>() -> impl Filter<Extract = (T,), Error = Rejection> + Copy {
    warp::filters::body::bytes()
    .and_then(|buf: Bytes| async move {
        let mut buf = buf;
        serde_json::from_slice(&buf.copy_to_bytes(buf.remaining())).map_err(|e| {
            eprintln!("invalid form: {}", e);
            warp::reject::custom(ActionError::InvalidForm)
        })
    })
}