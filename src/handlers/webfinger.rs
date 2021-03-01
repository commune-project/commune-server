use crate::state::AppState;
use crate::errors;
use crate::apub::webfinger;

use tokio;
use warp;
use warp::Reply;
use std::sync::Arc;
use serde::Deserialize;

pub fn map_content_type_webfinger<T: warp::reply::Reply>(reply: T) -> warp::reply::WithHeader<T> {
    warp::reply::with_header(reply, "Content-Type", "application/jrd+json; charset=utf-8")
}

#[derive(Deserialize)]
pub struct WebfingerResource {
    resource: String,
}

pub async fn get_webfinger(
    app_state: Arc<AppState>,
    _domain: String,
    webfinger_resource: WebfingerResource,
) -> Result<impl warp::Reply, warp::Rejection> {
    let resource = webfinger_resource.resource;
    let conn = app_state
        .db
        .get()
        .expect("couldn't get db connection from pool");

    let result =
        tokio::task::spawn_blocking(move || webfinger::get_webfinger(&conn, resource.as_str()))
            .await
            .or(Err(errors::ActionError::InternalError));

    match result {
        Ok(Ok(value)) => Ok(warp::reply::json(&value).into_response()),
        Ok(Err(err)) => Ok(err.into_response()),
        Err(err) => Ok(err.into_response()),
    }
}
