pub mod actors;
//pub mod inbox;

use warp;
use crate::state::AppState;
use std::sync::Arc;

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
